use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rayon::prelude::*;
use regex::Regex;

use crate::session::{Session, SessionFileEntry};

/// Search through all session JSONL files for lines matching `pattern`.
///
/// Uses parallel file scanning via rayon. For each matching file, extracts
/// session metadata from the first user entry. Returns sessions sorted by
/// timestamp descending.
pub fn deep_search(claude_home: &Path, pattern: &str) -> Vec<Session> {
    let re = match Regex::new(pattern) {
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

    // Search files in parallel
    let mut sessions: Vec<Session> = jsonl_files
        .par_iter()
        .filter_map(|path| search_file(path, &re))
        .collect();

    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sessions
}

/// Search a single JSONL file for the pattern. If any line matches,
/// extract session metadata from the first user entry.
fn search_file(path: &Path, re: &Regex) -> Option<Session> {
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

        // Check for pattern match in raw text first (fast path)
        if !found_match && re.is_match(&line) {
            found_match = true;
        }

        // Parse first user entry for metadata
        if first_user_entry.is_none() {
            if let Ok(entry) = serde_json::from_str::<SessionFileEntry>(&line) {
                if entry.entry_type == "user" {
                    first_user_entry = Some(entry);
                }
            }
        }

        // Early exit if we have both
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
            let full = m.content.text();
            full.lines()
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
