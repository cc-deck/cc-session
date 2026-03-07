// Substring-based filtering for sessions

use crate::session::Session;

/// Filter sessions by requiring all space-separated terms to appear as
/// case-insensitive substrings in "{project_name} {git_branch} {first_message}".
///
/// Returns matching indices (in original order, since all matches are equal rank).
pub fn filter_sessions(sessions: &[Session], query: &str) -> Vec<usize> {
    let query_lower = query.to_lowercase();
    let terms: Vec<&str> = query_lower.split_whitespace().collect();

    if terms.is_empty() {
        return (0..sessions.len()).collect();
    }

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

            if terms.iter().all(|term| haystack.contains(term)) {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}
