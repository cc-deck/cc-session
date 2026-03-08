// Substring-based filtering for sessions

use crate::session::Session;

/// Filter sessions by requiring the query to appear as a case-insensitive
/// substring in "{project_name} {git_branch} {first_message}".
///
/// The full query (including spaces) is matched literally.
/// Returns matching indices in original order.
pub fn filter_sessions(sessions: &[Session], query: &str) -> Vec<usize> {
    let query_trimmed = query.trim();
    if query_trimmed.is_empty() {
        return (0..sessions.len()).collect();
    }

    let query_lower = query_trimmed.to_lowercase();

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

            if haystack.contains(&query_lower) {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}
