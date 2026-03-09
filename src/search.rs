use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use rayon::prelude::*;
use regex::Regex;

use crate::session::{clean_message, Session, SessionFileEntry, strip_system_blocks, strip_tags};

/// Build a file-path-to-session index from discovered sessions.
///
/// Maps each session's JSONL file path to a clone of the Session.
/// Used by `deep_search_indexed` to avoid re-parsing files for metadata.
pub fn build_session_index(claude_home: &Path, sessions: &[Session]) -> HashMap<PathBuf, Session> {
    let projects_dir = claude_home.join("projects");
    let mut index = HashMap::with_capacity(sessions.len());

    for session in sessions {
        let encoded_dir = session.project_path.replace('/', "-");
        let file_path = projects_dir
            .join(&encoded_dir)
            .join(format!("{}.jsonl", session.id));
        index.insert(file_path, session.clone());
    }

    index
}

/// Search through all session JSONL files for lines matching `pattern`,
/// using a pre-built session index to avoid re-parsing metadata.
///
/// Falls back to parsing the file if no index entry exists.
pub fn deep_search_indexed(
    claude_home: &Path,
    pattern: &str,
    session_index: &HashMap<PathBuf, Session>,
    cancel: &Arc<AtomicBool>,
) -> Vec<Session> {
    let ci_pattern = if pattern.starts_with("(?") {
        pattern.to_string()
    } else {
        format!("(?i){}", pattern)
    };
    let re = match Regex::new(&ci_pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Invalid search pattern: {e}");
            return Vec::new();
        }
    };

    let projects_dir = claude_home.join("projects");
    if !projects_dir.is_dir() {
        return Vec::new();
    }

    // Collect all .jsonl file paths
    let mut jsonl_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(files) = fs::read_dir(&path) {
                    for file in files.flatten() {
                        let fpath = file.path();
                        if fpath.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                            jsonl_files.push(fpath);
                        }
                    }
                }
            }
        }
    }

    // Search files in parallel, look up session from index
    let mut sessions: Vec<Session> = jsonl_files
        .par_iter()
        .filter_map(|path| {
            // Check cancellation flag
            if cancel.load(Ordering::Relaxed) {
                return None;
            }
            if !file_matches(path, &re) {
                return None;
            }
            // Fast path: look up in pre-built index
            if let Some(session) = session_index.get(path) {
                return Some(session.clone());
            }
            // Fallback: parse file for metadata (undiscovered session)
            search_file_with_metadata(path, &re)
        })
        .collect();

    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sessions
}

/// Deep search without index. Used by tests and as a standalone entry point.
#[allow(dead_code)]
pub fn deep_search(claude_home: &Path, pattern: &str) -> Vec<Session> {
    let ci_pattern = if pattern.starts_with("(?") {
        pattern.to_string()
    } else {
        format!("(?i){}", pattern)
    };
    let re = match Regex::new(&ci_pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Invalid search pattern: {e}");
            return Vec::new();
        }
    };

    let projects_dir = claude_home.join("projects");
    if !projects_dir.is_dir() {
        return Vec::new();
    }

    let mut jsonl_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(files) = fs::read_dir(&path) {
                    for file in files.flatten() {
                        let fpath = file.path();
                        if fpath.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                            jsonl_files.push(fpath);
                        }
                    }
                }
            }
        }
    }

    let mut sessions: Vec<Session> = jsonl_files
        .par_iter()
        .filter_map(|path| search_file_with_metadata(path, &re))
        .collect();

    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sessions
}

/// Check if any user/assistant message in a JSONL file matches the regex.
///
/// Only searches within user and assistant entries, and strips XML-like
/// tags (system-reminder, local-command-caveat, etc.) before matching
/// to avoid false positives from system-injected content.
fn file_matches(path: &Path, re: &Regex) -> bool {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        // Quick check: does the raw line match at all?
        if !re.is_match(&line) {
            continue;
        }
        // Parse the entry type properly (simple string check can false-match
        // against nested JSON content like type=progress containing "type":"user")
        let entry_type = extract_entry_type(&line);
        if entry_type != "user" && entry_type != "assistant" {
            continue;
        }
        // Strip system blocks then tags (same pipeline as conversation viewer)
        let system_stripped = strip_system_blocks(&line);
        let cleaned = strip_tags(&system_stripped);
        if re.is_match(&cleaned) {
            return true;
        }
    }
    false
}

/// Extract the top-level "type" field from a JSONL line without full parsing.
/// Searches for the exact key `"type":"value"` (not `"userType"` etc.)
/// within the first 500 chars to handle newer Claude Code JSON formats
/// that include additional metadata fields before the type.
fn extract_entry_type(line: &str) -> &str {
    // Find a safe UTF-8 boundary near 500 bytes
    let max_len = line.len().min(500);
    let safe_end = (0..=max_len).rev().find(|&i| line.is_char_boundary(i)).unwrap_or(0);
    let prefix = &line[..safe_end];
    let needle = "\"type\":\"";
    let mut search_from = 0;
    while let Some(pos) = prefix[search_from..].find(needle) {
        let abs_pos = search_from + pos;
        // Ensure this is the "type" key, not e.g. "userType"
        // The char before the quote must be { or , or whitespace (start of key)
        let is_standalone = if abs_pos == 0 {
            true
        } else {
            let prev = prefix.as_bytes()[abs_pos - 1];
            prev == b'{' || prev == b',' || prev == b' ' || prev == b'\t'
        };
        if is_standalone {
            let start = abs_pos + needle.len();
            if let Some(end) = prefix[start..].find('"') {
                return &prefix[start..start + end];
            }
        }
        search_from = abs_pos + 1;
    }
    ""
}

/// Search a single JSONL file for the pattern and extract session metadata.
/// Used as fallback when the session is not in the pre-built index.
fn search_file_with_metadata(path: &Path, re: &Regex) -> Option<Session> {
    let session_id = path.file_stem()?.to_str()?.to_string();

    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut found_match = false;
    let mut first_user_entry: Option<SessionFileEntry> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }

        if !found_match && re.is_match(&line) {
            found_match = true;
        }

        if first_user_entry.is_none() {
            if let Ok(entry) = serde_json::from_str::<SessionFileEntry>(&line) {
                if entry.entry_type == "user" {
                    first_user_entry = Some(entry);
                }
            }
        }

        if found_match && first_user_entry.is_some() {
            break;
        }
    }

    if !found_match {
        return None;
    }

    let entry = first_user_entry?;
    let cwd = entry.cwd.unwrap_or_default();
    let timestamp: DateTime<Utc> = entry
        .timestamp
        .and_then(|t| t.parse().ok())
        .unwrap_or_else(Utc::now);

    let first_message = entry
        .message
        .map(|m| {
            let raw = m.content.text();
            clean_message(&raw)
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(200)
                .collect::<String>()
        })
        .unwrap_or_default();

    let project_name = Path::new(&cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let project_exists = Path::new(&cwd).exists();

    Some(Session {
        id: session_id,
        project_path: cwd.clone(),
        project_name,
        git_branch: entry.git_branch,
        timestamp,
        first_message,
        cwd,
        project_exists,
    })
}
