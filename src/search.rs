// Deep search: parallel JSONL content scanning

use std::path::Path;

use crate::session::Session;

/// Search through session JSONL files for a pattern.
///
/// Returns matching sessions (currently a placeholder that returns an empty vec).
pub fn deep_search(_claude_home: &Path, _pattern: &str) -> Vec<Session> {
    Vec::new()
}
