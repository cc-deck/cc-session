// Fuzzy filtering with nucleo

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use crate::session::Session;

/// Run a fuzzy match of `query` against sessions, returning matching indices and scores.
///
/// Each session is matched against the string "{project_name} {git_branch} {first_message}".
/// Results are sorted by score descending (best match first).
pub fn fuzzy_filter(sessions: &[Session], query: &str) -> Vec<(usize, u32)> {
    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut buf = Vec::new();

    let mut results: Vec<(usize, u32)> = sessions
        .iter()
        .enumerate()
        .filter_map(|(idx, session)| {
            let branch = session.git_branch.as_deref().unwrap_or("");
            let haystack = format!(
                "{} {} {}",
                session.project_name, branch, session.first_message
            );
            let score = pattern.score(Utf32Str::new(&haystack, &mut buf), &mut matcher)?;
            Some((idx, score))
        })
        .collect();

    results.sort_by(|a, b| b.1.cmp(&a.1));
    results
}
