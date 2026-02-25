use std::path::PathBuf;

use cc_session::discovery::{apply_filters, discover_sessions};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn discover_sessions_from_fixtures() {
    let sessions = discover_sessions(&fixture_dir());
    assert_eq!(sessions.len(), 3, "should find 3 session files");
}

#[test]
fn sessions_sorted_newest_first() {
    let sessions = discover_sessions(&fixture_dir());
    for i in 1..sessions.len() {
        assert!(
            sessions[i - 1].timestamp >= sessions[i].timestamp,
            "sessions should be sorted newest first"
        );
    }
}

#[test]
fn session_id_from_filename() {
    let sessions = discover_sessions(&fixture_dir());
    let ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"11111111-1111-1111-1111-111111111111"));
    assert!(ids.contains(&"22222222-2222-2222-2222-222222222222"));
    assert!(ids.contains(&"33333333-3333-3333-3333-333333333333"));
}

#[test]
fn first_user_message_extracted() {
    let sessions = discover_sessions(&fixture_dir());
    let s = sessions
        .iter()
        .find(|s| s.id == "11111111-1111-1111-1111-111111111111")
        .unwrap();
    assert_eq!(s.first_message, "Help me implement the list endpoints feature");
    assert_eq!(s.git_branch.as_deref(), Some("feat-endpoints"));
}

#[test]
fn array_content_extracted() {
    let sessions = discover_sessions(&fixture_dir());
    let s = sessions
        .iter()
        .find(|s| s.id == "22222222-2222-2222-2222-222222222222")
        .unwrap();
    assert_eq!(s.first_message, "Add OAuth2 support to the client");
}

#[test]
fn apply_last_filter() {
    let sessions = discover_sessions(&fixture_dir());
    let filtered = apply_filters(sessions, None, Some(2));
    assert_eq!(filtered.len(), 2);
}

#[test]
fn empty_directory_returns_empty() {
    let sessions = discover_sessions(&PathBuf::from("/nonexistent/path"));
    assert!(sessions.is_empty());
}
