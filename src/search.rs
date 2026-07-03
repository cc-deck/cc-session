use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use rayon::prelude::*;
use regex::Regex;

use crate::filter::parse_keywords;
use crate::session::{clean_message, clean_message_multiline, Session, SessionFileEntry};

/// A context snippet extracted from a content search match.
///
/// Contains the raw snippet text (up to 300 chars) centered on the best
/// keyword cluster, the character offset ranges of each keyword occurrence
/// within the text, and whether additional matches exist in the session.
#[derive(Debug, Clone)]
pub struct MatchSnippet {
    /// The raw snippet text, cleaned and collapsed to a single line.
    pub text: String,
    /// Character offset ranges (start, end) of each keyword occurrence within `text`.
    /// Sorted by start offset.
    pub keyword_ranges: Vec<(usize, usize)>,
    /// True if additional matches exist in the session beyond this snippet.
    pub has_more: bool,
}

/// Pairs a Session with its optional snippet from content search.
///
/// Created by `deep_search_indexed`, sent through the `mpsc` channel,
/// stored in `App::content_results`, consumed by `view::render_session_list`.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matched session metadata.
    pub session: Session,
    /// Present when the session was found via content search.
    pub snippet: Option<MatchSnippet>,
}

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

/// Search through all session JSONL files for lines matching all keywords,
/// using a pre-built session index to avoid re-parsing metadata.
///
/// Keywords are parsed from the pattern (space-separated, quoted phrases preserved).
/// All keywords must match somewhere in the file (AND logic).
/// Falls back to parsing the file if no index entry exists.
pub fn deep_search_indexed(
    claude_home: &Path,
    pattern: &str,
    session_index: &HashMap<PathBuf, Session>,
    cancel: &Arc<AtomicBool>,
) -> Vec<SearchResult> {
    let keywords = parse_keywords(pattern);
    if keywords.is_empty() {
        return Vec::new();
    }

    let regexes: Vec<Regex> = keywords
        .iter()
        .filter_map(|kw| {
            let escaped = regex::escape(kw);
            Regex::new(&format!("(?i){escaped}")).ok()
        })
        .collect();

    if regexes.is_empty() {
        return Vec::new();
    }

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

    // Search files in parallel, look up session from index, pair with snippet
    let mut results: Vec<SearchResult> = jsonl_files
        .par_iter()
        .filter_map(|path| {
            // Check cancellation flag
            if cancel.load(Ordering::Relaxed) {
                return None;
            }
            let snippet = file_matches_all(path, &regexes)?;
            // Fast path: look up in pre-built index
            let session = if let Some(session) = session_index.get(path) {
                session.clone()
            } else {
                // Fallback: parse file for metadata (undiscovered session)
                let fallback_re = &regexes[0];
                search_file_with_metadata(path, fallback_re)?
            };
            Some(SearchResult {
                session,
                snippet: Some(snippet),
            })
        })
        .collect();

    results.sort_by(|a, b| b.session.timestamp.cmp(&a.session.timestamp));
    results
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

/// Search user/assistant messages of a JSONL file for all regexes and extract
/// a context snippet centered on the densest keyword cluster.
///
/// Returns `Some(MatchSnippet)` when all regexes match somewhere in the file,
/// `None` otherwise. The snippet is centered on the passage where keywords
/// appear closest together (smallest character span covering at least one
/// match of every keyword within a single message).
///
/// Only searches within visible text content (same as the conversation viewer):
/// parses JSON, extracts only "text"-type content blocks, then strips system
/// blocks and tags. This avoids false positives from tool_use/tool_result payloads.
fn file_matches_all(path: &Path, regexes: &[Regex]) -> Option<MatchSnippet> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return None,
    };

    let mut matched = vec![false; regexes.len()];
    // Track best cluster across all messages
    let mut best_cluster: Option<ClusterInfo> = None;
    let mut messages_with_all_matches: usize = 0;

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        // Quick pre-filter: skip lines where no regex hits the raw text
        let any_hit = regexes.iter().any(|re| re.is_match(&line));
        if !any_hit {
            continue;
        }

        let entry_type = extract_entry_type(&line);
        let is_user = entry_type == "user";
        let is_assistant = entry_type == "assistant";
        if !is_user && !is_assistant {
            continue;
        }

        // Parse JSON and extract only visible text content (same as conversation viewer)
        let text = match serde_json::from_str::<SessionFileEntry>(&line) {
            Ok(entry) => match &entry.message {
                Some(m) => m.content.text(),
                None => continue,
            },
            Err(_) => continue,
        };

        let cleaned = clean_message_multiline(&text);

        // Track global match state (all regexes must match somewhere)
        for (i, re) in regexes.iter().enumerate() {
            if !matched[i] && re.is_match(&cleaned) {
                matched[i] = true;
            }
        }

        // Find keyword positions for cluster scoring
        let mut positions_per_regex: Vec<Vec<usize>> = Vec::with_capacity(regexes.len());
        let mut all_found = true;
        for re in regexes {
            let positions: Vec<usize> = re
                .find_iter(&cleaned)
                .map(|m| {
                    // Convert byte offset to char offset
                    cleaned[..m.start()].chars().count()
                })
                .collect();
            if positions.is_empty() {
                all_found = false;
            }
            positions_per_regex.push(positions);
        }

        if all_found {
            messages_with_all_matches += 1;

            // Find the densest cluster: smallest span covering at least one match of every keyword
            if let Some(cluster) = find_densest_cluster(&positions_per_regex, regexes, &cleaned) {
                let is_better = match &best_cluster {
                    None => true,
                    Some(best) => {
                        if cluster.span < best.span {
                            true
                        } else if cluster.span == best.span {
                            // Tie-breaking: prefer user messages, then earlier position
                            is_user && !best.is_user
                        } else {
                            false
                        }
                    }
                };

                if is_better {
                    best_cluster = Some(ClusterInfo {
                        cleaned_text: cleaned,
                        center_char: cluster.center_char,
                        span: cluster.span,
                        is_user,
                    });
                }
            }
        }
    }

    // All regexes must match somewhere in the file
    if !matched.iter().all(|&m| m) {
        return None;
    }

    // Extract snippet from best cluster
    let cluster = best_cluster?;
    let snippet = extract_snippet(&cluster.cleaned_text, cluster.center_char, regexes);

    Some(MatchSnippet {
        text: snippet.text,
        keyword_ranges: snippet.keyword_ranges,
        has_more: messages_with_all_matches > 1,
    })
}

/// Information about a keyword cluster found within a message.
struct ClusterInfo {
    cleaned_text: String,
    center_char: usize,
    span: usize,
    is_user: bool,
}

/// A minimal cluster result from the density scan.
struct ClusterResult {
    center_char: usize,
    span: usize,
}

/// Find the densest keyword cluster within a single message.
///
/// The densest cluster is the smallest character span that contains at least
/// one match of every keyword. Returns the cluster center and span width.
fn find_densest_cluster(
    positions_per_regex: &[Vec<usize>],
    regexes: &[Regex],
    cleaned: &str,
) -> Option<ClusterResult> {
    let num_regexes = positions_per_regex.len();

    if num_regexes == 1 {
        // Single keyword: use the first match position
        let pos = positions_per_regex[0].first()?;
        let match_len = regexes[0]
            .find(cleaned)
            .map(|m| cleaned[..m.end()].chars().count() - cleaned[..m.start()].chars().count())
            .unwrap_or(1);
        return Some(ClusterResult {
            center_char: *pos + match_len / 2,
            span: match_len,
        });
    }

    // Multi-keyword: find the window covering all keywords with smallest span.
    // Use a sweep-line approach: collect all positions tagged by regex index,
    // sort by position, then slide a window that covers all regex indices.
    let mut tagged: Vec<(usize, usize)> = Vec::new(); // (char_position, regex_index)
    for (regex_idx, positions) in positions_per_regex.iter().enumerate() {
        for &pos in positions {
            tagged.push((pos, regex_idx));
        }
    }
    tagged.sort_by_key(|&(pos, _)| pos);

    let mut best_span = usize::MAX;
    let mut best_start = 0;
    let mut best_end = 0;
    let mut counts = vec![0usize; num_regexes];
    let mut covered = 0;
    let mut left = 0;

    for right in 0..tagged.len() {
        let (_, ri) = tagged[right];
        if counts[ri] == 0 {
            covered += 1;
        }
        counts[ri] += 1;

        // Shrink window from left while all regexes are still covered
        while covered == num_regexes {
            let span = tagged[right].0 - tagged[left].0;
            if span < best_span {
                best_span = span;
                best_start = tagged[left].0;
                best_end = tagged[right].0;
            }

            let (_, li) = tagged[left];
            counts[li] -= 1;
            if counts[li] == 0 {
                covered -= 1;
            }
            left += 1;
        }
    }

    if best_span == usize::MAX {
        return None;
    }

    Some(ClusterResult {
        center_char: (best_start + best_end) / 2,
        span: best_span,
    })
}

/// Information extracted for a snippet.
struct SnippetExtraction {
    text: String,
    keyword_ranges: Vec<(usize, usize)>,
}

/// Extract a ~300-character snippet centered on a position within cleaned text.
///
/// Uses `is_char_boundary`-safe slicing. Collapses newlines to single spaces
/// for single-line display. Computes keyword_ranges relative to the snippet.
fn extract_snippet(cleaned: &str, center_char: usize, regexes: &[Regex]) -> SnippetExtraction {
    const SNIPPET_SIZE: usize = 300;

    // Collapse to single line for display
    let single_line: String = cleaned
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    let chars: Vec<char> = single_line.chars().collect();
    let total_chars = chars.len();

    // Calculate window boundaries in character space
    let half = SNIPPET_SIZE / 2;
    let start_char = center_char.saturating_sub(half);
    let end_char = (center_char + half).min(total_chars);

    // Convert char offsets to byte offsets using is_char_boundary
    let snippet_text: String = chars[start_char..end_char].iter().collect();

    // Find keyword ranges within the snippet
    let mut keyword_ranges: Vec<(usize, usize)> = Vec::new();
    for re in regexes {
        for m in re.find_iter(&snippet_text) {
            let start = snippet_text[..m.start()].chars().count();
            let end = start + snippet_text[m.start()..m.end()].chars().count();
            keyword_ranges.push((start, end));
        }
    }
    keyword_ranges.sort_by_key(|&(start, _)| start);

    // Merge overlapping ranges
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for range in keyword_ranges {
        if let Some(last) = merged.last_mut() {
            if range.0 <= last.1 {
                last.1 = last.1.max(range.1);
                continue;
            }
        }
        merged.push(range);
    }

    SnippetExtraction {
        text: snippet_text,
        keyword_ranges: merged,
    }
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

        if let Ok(entry) = serde_json::from_str::<SessionFileEntry>(&line) {
            if !found_match
                && (entry.entry_type == "user" || entry.entry_type == "assistant")
            {
                if let Some(m) = &entry.message {
                    let text = clean_message_multiline(&m.content.text());
                    if re.is_match(&text) {
                        found_match = true;
                    }
                }
            }

            if first_user_entry.is_none() && entry.entry_type == "user" {
                first_user_entry = Some(entry);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to create a JSONL file with user/assistant messages for testing.
    fn create_test_jsonl(messages: &[(&str, &str)]) -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".jsonl").unwrap();
        for (role, text) in messages {
            let line = format!(
                r#"{{"type":"{}","message":{{"role":"{}","content":"{}"}}}}"#,
                role, role, text
            );
            writeln!(file, "{}", line).unwrap();
        }
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_single_keyword_snippet_extraction() {
        let file = create_test_jsonl(&[
            ("user", "hello world"),
            ("assistant", "I can help with kubernetes deployments and pods"),
            ("user", "tell me about kubernetes networking"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some(), "Should find a match");
        let snippet = result.unwrap();
        assert!(
            snippet.text.to_lowercase().contains("kubernetes"),
            "Snippet should contain the keyword"
        );
        assert!(
            !snippet.keyword_ranges.is_empty(),
            "Should have keyword ranges"
        );
        // has_more should be true since kubernetes appears in 2 messages
        assert!(snippet.has_more, "Should indicate more matches exist");
    }

    #[test]
    fn test_multi_keyword_densest_cluster() {
        // "rust" and "async" appear far apart in msg1, close together in msg2
        let file = create_test_jsonl(&[
            ("user", "I want to learn about rust programming and also understand async patterns in javascript"),
            ("user", "Can you explain rust async await syntax?"),
        ]);

        let regexes = vec![
            Regex::new("(?i)rust").unwrap(),
            Regex::new("(?i)async").unwrap(),
        ];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some(), "Should find a match");
        let snippet = result.unwrap();
        // The snippet should center on the denser cluster (msg2 where "rust" and "async" are adjacent)
        assert!(
            snippet.text.to_lowercase().contains("rust async"),
            "Snippet should contain the dense cluster: got '{}'",
            snippet.text
        );
    }

    #[test]
    fn test_no_match_returns_none() {
        let file = create_test_jsonl(&[
            ("user", "hello world"),
            ("assistant", "how can I help"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);
        assert!(result.is_none(), "Should not find a match");
    }

    #[test]
    fn test_has_more_false_single_message() {
        let file = create_test_jsonl(&[
            ("user", "hello world"),
            ("assistant", "Let me help with kubernetes"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some());
        let snippet = result.unwrap();
        assert!(
            !snippet.has_more,
            "has_more should be false for single matching message"
        );
    }

    #[test]
    fn test_prefers_user_messages_at_equal_density() {
        // Both messages have "kubernetes" once, user message should be preferred
        let file = create_test_jsonl(&[
            ("assistant", "kubernetes is a container orchestrator"),
            ("user", "tell me about kubernetes please"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some());
        let snippet = result.unwrap();
        // Should prefer user message content
        assert!(
            snippet.text.to_lowercase().contains("tell me about"),
            "Should prefer user message: got '{}'",
            snippet.text
        );
    }

    #[test]
    fn test_keyword_ranges_sorted() {
        let file = create_test_jsonl(&[
            ("user", "kubernetes pods and kubernetes services are great"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some());
        let snippet = result.unwrap();
        // Verify ranges are sorted
        for window in snippet.keyword_ranges.windows(2) {
            assert!(
                window[0].0 <= window[1].0,
                "Keyword ranges should be sorted"
            );
        }
    }

    #[test]
    fn test_extract_snippet_centering() {
        let text = "a ".repeat(200) + "TARGET" + &" b".repeat(200);
        let regexes = vec![Regex::new("TARGET").unwrap()];
        let center = 400; // approximate char position of TARGET
        let extraction = extract_snippet(&text, center, &regexes);

        assert!(
            extraction.text.contains("TARGET"),
            "Snippet should contain the target keyword"
        );
        assert!(!extraction.keyword_ranges.is_empty());
    }

    #[test]
    fn test_find_densest_cluster_single_keyword() {
        let cleaned = "hello kubernetes world";
        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let positions = vec![vec![6usize]]; // "kubernetes" starts at char 6

        let result = find_densest_cluster(&positions, &regexes, cleaned);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_densest_cluster_multi_keyword() {
        // "alpha" at positions 0 and 50, "beta" at positions 10 and 48
        // Densest cluster should be (48, 50) with span 2
        let positions = vec![vec![0, 50], vec![10, 48]];
        let cleaned = &(" ".repeat(60));
        let regexes = vec![
            Regex::new("(?i)alpha").unwrap(),
            Regex::new("(?i)beta").unwrap(),
        ];

        let result = find_densest_cluster(&positions, &regexes, cleaned);
        assert!(result.is_some());
        let cluster = result.unwrap();
        assert_eq!(cluster.span, 2, "Densest cluster span should be 2");
    }

    // --- UTF-8 boundary safety tests (T009) ---

    #[test]
    fn test_utf8_emoji_in_snippet() {
        // Emoji are multi-byte UTF-8 characters (4 bytes each)
        let file = create_test_jsonl(&[
            ("user", "I love \\ud83d\\ude00 and kubernetes \\ud83d\\ude80 is great"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        // Should not panic and should find the match
        assert!(result.is_some(), "Should find match with emoji in text");
        let snippet = result.unwrap();
        assert!(
            snippet.text.to_lowercase().contains("kubernetes"),
            "Snippet should contain keyword despite emoji"
        );
    }

    #[test]
    fn test_utf8_cjk_characters_in_snippet() {
        // CJK characters are 3 bytes each in UTF-8
        let file = create_test_jsonl(&[
            ("user", "Please help with kubernetes deployment"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some(), "Should find match with CJK context");
        let snippet = result.unwrap();
        assert!(!snippet.keyword_ranges.is_empty());
    }

    #[test]
    fn test_utf8_accented_characters() {
        // Accented characters (e.g., e with accent) are 2 bytes in UTF-8
        let file = create_test_jsonl(&[
            ("user", "deploiement kubernetes avec les parametres speciales"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some());
        let snippet = result.unwrap();
        assert!(snippet.text.to_lowercase().contains("kubernetes"));
    }

    #[test]
    fn test_extract_snippet_with_multibyte_boundaries() {
        // Create a string where the 300-char window boundary falls in the middle
        // of multi-byte characters
        let prefix = "\u{1F600}".repeat(100); // 100 emoji (4 bytes each)
        let text = format!("{}KEYWORD{}", prefix, "\u{1F600}".repeat(100));

        let regexes = vec![Regex::new("KEYWORD").unwrap()];
        let center = 100; // roughly where KEYWORD is
        let extraction = extract_snippet(&text, center, &regexes);

        // Should not panic
        assert!(
            extraction.text.contains("KEYWORD"),
            "Should extract snippet safely with multi-byte boundaries"
        );
    }

    #[test]
    fn test_has_more_true_multiple_messages_match() {
        let file = create_test_jsonl(&[
            ("user", "kubernetes is great"),
            ("assistant", "yes kubernetes is powerful"),
            ("user", "tell me more about kubernetes"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some());
        let snippet = result.unwrap();
        assert!(
            snippet.has_more,
            "has_more should be true when keywords match in 3 messages"
        );
    }

    #[test]
    fn test_has_more_false_single_message_match() {
        let file = create_test_jsonl(&[
            ("user", "hello world"),
            ("assistant", "kubernetes is a container orchestrator"),
            ("user", "thanks for the info"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some());
        let snippet = result.unwrap();
        assert!(
            !snippet.has_more,
            "has_more should be false when only one message matches all keywords"
        );
    }

    #[test]
    fn test_keyword_at_message_start() {
        let file = create_test_jsonl(&[
            ("user", "kubernetes deployment help needed"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some());
        let snippet = result.unwrap();
        assert!(snippet.text.starts_with("kubernetes") || snippet.text.to_lowercase().starts_with("kubernetes"));
    }

    #[test]
    fn test_keyword_at_message_end() {
        let file = create_test_jsonl(&[
            ("user", "I want to learn kubernetes"),
        ]);

        let regexes = vec![Regex::new("(?i)kubernetes").unwrap()];
        let result = file_matches_all(file.path(), &regexes);

        assert!(result.is_some());
        let snippet = result.unwrap();
        assert!(
            snippet.text.to_lowercase().contains("kubernetes"),
            "Should handle keyword at end of message"
        );
    }
}
