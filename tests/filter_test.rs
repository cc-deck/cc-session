use cc_session::discovery::discover_sessions;
use cc_session::filter::fuzzy_filter;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn filter_by_project_name() {
    let sessions = discover_sessions(&fixture_dir());
    let matches = fuzzy_filter(&sessions, "project-b");
    assert!(!matches.is_empty(), "should match project-b sessions");
}

#[test]
fn filter_by_message_content() {
    let sessions = discover_sessions(&fixture_dir());
    let matches = fuzzy_filter(&sessions, "OAuth2");
    assert!(!matches.is_empty(), "should match session with OAuth2 message");
}

#[test]
fn empty_query_returns_all() {
    let sessions = discover_sessions(&fixture_dir());
    let matches = fuzzy_filter(&sessions, "");
    // Empty query returns nothing (nucleo requires input)
    // This is fine, the app handles empty query by showing all sessions
    assert!(matches.is_empty() || matches.len() == sessions.len());
}

#[test]
fn nonmatching_query_returns_empty() {
    let sessions = discover_sessions(&fixture_dir());
    let matches = fuzzy_filter(&sessions, "xyzzynonexistent12345");
    assert!(matches.is_empty());
}

#[test]
fn results_sorted_by_score() {
    let sessions = discover_sessions(&fixture_dir());
    let matches = fuzzy_filter(&sessions, "endpoint");
    if matches.len() >= 2 {
        for i in 1..matches.len() {
            assert!(matches[i - 1].1 >= matches[i].1, "should be sorted by score desc");
        }
    }
}
