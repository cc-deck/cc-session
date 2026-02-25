use std::path::PathBuf;

use cc_session::search::deep_search;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn pattern_in_assistant_response() {
    let sessions = deep_search(&fixture_dir(), "ConnectionRefused");
    assert_eq!(sessions.len(), 1, "should find session with ConnectionRefused in assistant response");
    assert_eq!(sessions[0].id, "33333333-3333-3333-3333-333333333333");
}

#[test]
fn pattern_in_user_message() {
    let sessions = deep_search(&fixture_dir(), "OAuth2");
    assert!(!sessions.is_empty(), "should find session with OAuth2 in user message");
}

#[test]
fn pattern_not_found() {
    let sessions = deep_search(&fixture_dir(), "xyzzynonexistent12345");
    assert!(sessions.is_empty());
}

#[test]
fn regex_pattern_works() {
    let sessions = deep_search(&fixture_dir(), "Connection[A-Z]");
    assert!(!sessions.is_empty(), "regex pattern should match");
}

#[test]
fn invalid_regex_returns_empty() {
    let sessions = deep_search(&fixture_dir(), "[invalid(regex");
    assert!(sessions.is_empty());
}
