// Keyword-based filtering for sessions

use crate::session::Session;

/// Parse a search query into keywords.
/// Words separated by spaces are individual keywords (ANDed during search).
/// Quoted phrases (`"..."`) are treated as single keywords preserving spaces.
pub fn parse_keywords(query: &str) -> Vec<String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut keywords = Vec::new();
    let mut chars = trimmed.chars().peekable();
    let mut current = String::new();

    while let Some(&c) = chars.peek() {
        if c == '"' {
            chars.next(); // consume opening quote
            let mut quoted = String::new();
            while let Some(&qc) = chars.peek() {
                if qc == '"' {
                    chars.next(); // consume closing quote
                    break;
                }
                quoted.push(qc);
                chars.next();
            }
            if !quoted.is_empty() {
                if !current.is_empty() {
                    keywords.push(current.clone());
                    current.clear();
                }
                keywords.push(quoted);
            }
        } else if c == ' ' {
            if !current.is_empty() {
                keywords.push(current.clone());
                current.clear();
            }
            chars.next();
        } else {
            current.push(c);
            chars.next();
        }
    }

    if !current.is_empty() {
        keywords.push(current);
    }

    keywords
}

/// Filter sessions by requiring all keywords to appear as case-insensitive
/// substrings in "{project_name} {git_branch} {first_message}".
///
/// Keywords are split on whitespace; quoted phrases are kept as single terms.
/// All keywords must match (AND logic).
/// Returns matching indices in original order.
pub fn filter_sessions(sessions: &[Session], query: &str) -> Vec<usize> {
    let keywords = parse_keywords(query);
    if keywords.is_empty() {
        return (0..sessions.len()).collect();
    }

    let keywords_lower: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();

    sessions
        .iter()
        .enumerate()
        .filter_map(|(idx, session)| {
            let branch = session.git_branch.as_deref().unwrap_or("");
            let haystack = format!(
                "{} {} {}",
                session.project_name, branch, session.first_message
            )
            .to_lowercase();

            if keywords_lower.iter().all(|kw| haystack.contains(kw.as_str())) {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}
