// Session discovery: scanning ~/.claude/projects/ for session JSONL files

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rayon::prelude::*;

use crate::session::{Session, SessionFileEntry, UserPrompt};

/// Return the Claude home directory.
///
/// Checks the `CLAUDE_HOME` env var first, then falls back to `~/.claude`.
pub fn get_claude_home() -> PathBuf {
    if let Ok(home) = std::env::var("CLAUDE_HOME") {
        return PathBuf::from(home);
    }
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".claude")
}

/// Discover all sessions under `claude_home/projects/`.
pub fn discover_sessions(claude_home: &Path) -> Vec<Session> {
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

    // Parse files in parallel
    let mut sessions: Vec<Session> = jsonl_files
        .par_iter()
        .filter_map(|path| parse_session_file(path))
        .collect();

    // Sort by timestamp descending (newest first)
    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sessions
}

/// Parse a single JSONL session file and extract the first user message.
fn parse_session_file(path: &Path) -> Option<Session> {
    let session_id = path.file_stem()?.to_str()?.to_string();

    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    // Read up to 20 lines looking for the first "user" entry
    for line in reader.lines().take(20) {
        let line = line.ok()?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: SessionFileEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.entry_type != "user" {
            continue;
        }

        // Found the first user entry
        let cwd = entry.cwd.unwrap_or_default();
        let git_branch = entry.git_branch;
        let timestamp_str = entry.timestamp.unwrap_or_default();
        let timestamp: DateTime<Utc> = timestamp_str
            .parse()
            .unwrap_or_else(|_| Utc::now());

        let first_message = entry
            .message
            .map(|m| m.content.text())
            .unwrap_or_default();

        // Truncate first_message to first line / reasonable length
        let first_message = first_message
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(200)
            .collect::<String>();

        let project_name = Path::new(&cwd)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let project_exists = Path::new(&cwd).exists();

        return Some(Session {
            id: session_id,
            project_path: cwd.clone(),
            project_name,
            git_branch,
            timestamp,
            first_message,
            cwd,
            project_exists,
        });
    }

    None
}

/// Load the last N user prompts from a session JSONL file.
///
/// Returns prompts in chronological order (oldest first).
pub fn load_session_prompts(claude_home: &Path, session: &Session, max: usize) -> Vec<UserPrompt> {
    let encoded_dir = session.project_path.replace('/', "-");
    let file_path = claude_home
        .join("projects")
        .join(&encoded_dir)
        .join(format!("{}.jsonl", session.id));

    let file = match fs::File::open(&file_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let mut prompts: Vec<UserPrompt> = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }
        let entry: SessionFileEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.entry_type != "user" {
            continue;
        }

        let text = entry
            .message
            .map(|m| {
                let full = m.content.text();
                // Take first line, truncate to 200 chars
                full.lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(200)
                    .collect::<String>()
            })
            .unwrap_or_default();

        // Skip meta/command messages that are empty
        if text.is_empty() {
            continue;
        }

        let timestamp: DateTime<Utc> = entry
            .timestamp
            .and_then(|t| t.parse().ok())
            .unwrap_or_else(Utc::now);

        prompts.push(UserPrompt {
            text,
            timestamp,
            uuid: entry.uuid,
        });
    }

    // Keep only the last N prompts, in chronological order (oldest first)
    let len = prompts.len();
    if len > max {
        prompts.drain(..len - max);
    }
    prompts
}

/// Apply optional time-based and count-based filters to a session list.
///
/// `since` keeps only sessions newer than `Utc::now() - since`.
/// `last` keeps only the first N sessions (already sorted newest-first).
pub fn apply_filters(
    mut sessions: Vec<Session>,
    since: Option<chrono::Duration>,
    last: Option<usize>,
) -> Vec<Session> {
    if let Some(duration) = since {
        let cutoff = Utc::now() - duration;
        sessions.retain(|s| s.timestamp >= cutoff);
    }
    if let Some(n) = last {
        sessions.truncate(n);
    }
    sessions
}
