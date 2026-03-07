use cc_session::discovery::discover_sessions;
use cc_session::filter::filter_sessions;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn filter_by_project_name() {
    let sessions = discover_sessions(&fixture_dir());
    let matches = filter_sessions(&sessions, "project-b");
    assert!(!matches.is_empty(), "should match project-b sessions");
}

#[test]
fn filter_by_message_content() {
    let sessions = discover_sessions(&fixture_dir());
    let matches = filter_sessions(&sessions, "OAuth2");
    assert!(!matches.is_empty(), "should match session with OAuth2 message");
}

#[test]
fn empty_query_returns_all() {
    let sessions = discover_sessions(&fixture_dir());
    let matches = filter_sessions(&sessions, "");
    assert_eq!(matches.len(), sessions.len(), "empty query should return all sessions");
}

#[test]
fn nonmatching_query_returns_empty() {
    let sessions = discover_sessions(&fixture_dir());
    let matches = filter_sessions(&sessions, "xyzzynonexistent12345");
    assert!(matches.is_empty());
}
