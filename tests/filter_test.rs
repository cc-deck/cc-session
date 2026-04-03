use cc_session::discovery::discover_sessions;
use cc_session::filter::{filter_sessions, parse_keywords};
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

// --- parse_keywords tests ---

#[test]
fn parse_keywords_simple_words() {
    assert_eq!(parse_keywords("foo bar baz"), vec!["foo", "bar", "baz"]);
}

#[test]
fn parse_keywords_quoted_phrase() {
    assert_eq!(parse_keywords(r#"foo "hello world" bar"#), vec!["foo", "hello world", "bar"]);
}

#[test]
fn parse_keywords_only_quoted() {
    assert_eq!(parse_keywords(r#""hello world""#), vec!["hello world"]);
}

#[test]
fn parse_keywords_empty() {
    let empty: Vec<String> = Vec::new();
    assert_eq!(parse_keywords(""), empty);
    assert_eq!(parse_keywords("   "), empty);
}

#[test]
fn parse_keywords_extra_spaces() {
    assert_eq!(parse_keywords("  foo   bar  "), vec!["foo", "bar"]);
}

#[test]
fn parse_keywords_mixed() {
    assert_eq!(parse_keywords(r#"rust "cargo build" test"#), vec!["rust", "cargo build", "test"]);
}

#[test]
fn filter_keyword_and_logic() {
    let sessions = discover_sessions(&fixture_dir());
    // "project-b" matches, adding a nonexistent keyword should return empty
    let matches = filter_sessions(&sessions, "project-b xyzzy99");
    assert!(matches.is_empty(), "AND logic: both keywords must match");
}

#[test]
fn filter_multiple_keywords_match() {
    let sessions = discover_sessions(&fixture_dir());
    // "project-b" and "build" both appear in the same session's metadata
    let matches = filter_sessions(&sessions, "project-b build");
    assert!(!matches.is_empty(), "both keywords present in session metadata");
}
