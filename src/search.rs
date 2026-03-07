use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rayon::prelude::*;
use regex::Regex;

use crate::session::{clean_message, Session, SessionFileEntry};

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

/// Check if any line in a JSONL file matches the regex (content scan only).
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
        if re.is_match(&line) {
            return true;
        }
    }
    false
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
