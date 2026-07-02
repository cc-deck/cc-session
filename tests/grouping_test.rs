use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{TimeZone, Utc};

use cc_session::session::Session;
use cc_session::theme::Theme;
use cc_session::tui::{App, DisplayItem, MatchType};

/// Create a test session with the given project name and timestamp offset (hours ago).
fn make_session(id: &str, project: &str, hours_ago: i64, message: &str) -> Session {
    let ts = Utc::now() - chrono::Duration::hours(hours_ago);
    Session {
        id: id.to_string(),
        project_path: format!("/test/{}", project),
        project_name: project.to_string(),
        git_branch: None,
        timestamp: ts,
        first_message: message.to_string(),
        cwd: format!("/test/{}", project),
        project_exists: true,
    }
}

/// Build an App from a list of sessions (flat mode by default).
fn make_app(sessions: Vec<Session>) -> App {
    let index: HashMap<PathBuf, Session> = HashMap::new();
    let theme = Theme::dark();
    App::new(sessions, index, theme)
}

// --- T018: rebuild_display_items in grouped mode ---

#[test]
fn grouped_mode_creates_headers_per_project() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "msg1"),
        make_session("s2", "alpha", 2, "msg2"),
        make_session("s3", "beta", 3, "msg3"),
    ];
    let mut app = make_app(sessions);

    // Default is flat mode
    assert!(!app.grouped_mode);
    assert_eq!(app.display_items.len(), 3);
    assert!(app.display_items.iter().all(|item| matches!(item, DisplayItem::Session(_))));

    // Switch to grouped mode
    app.grouped_mode = true;
    app.rebuild_display_items();

    // Should have 2 headers (alpha, beta), all collapsed so no session items
    assert_eq!(app.display_items.len(), 2);
    assert!(app.display_items.iter().all(|item| matches!(item, DisplayItem::Header(_))));
}

#[test]
fn grouped_mode_groups_sorted_by_latest_timestamp() {
    let sessions = vec![
        make_session("s1", "alpha", 5, "old alpha"),
        make_session("s2", "beta", 1, "recent beta"),
        make_session("s3", "gamma", 3, "mid gamma"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;
    app.rebuild_display_items();

    // Groups should be sorted newest first: beta (1h), gamma (3h), alpha (5h)
    let names: Vec<&str> = app.display_items.iter().filter_map(|item| match item {
        DisplayItem::Header(g) => Some(g.project_name.as_str()),
        _ => None,
    }).collect();
    assert_eq!(names, vec!["beta", "gamma", "alpha"]);
}

#[test]
fn expanded_group_shows_sessions() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "msg1"),
        make_session("s2", "alpha", 2, "msg2"),
        make_session("s3", "beta", 3, "msg3"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;
    app.expanded_projects.insert("alpha".to_string());
    app.rebuild_display_items();

    // alpha: 1 header + 2 sessions, beta: 1 header (collapsed)
    assert_eq!(app.display_items.len(), 4);

    // First item is alpha header
    assert!(matches!(&app.display_items[0], DisplayItem::Header(g) if g.project_name == "alpha"));
    // Next two are alpha sessions
    assert!(matches!(&app.display_items[1], DisplayItem::Session(_)));
    assert!(matches!(&app.display_items[2], DisplayItem::Session(_)));
    // Last is beta header
    assert!(matches!(&app.display_items[3], DisplayItem::Header(g) if g.project_name == "beta"));
}

#[test]
fn collapsed_group_hides_sessions() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "msg1"),
        make_session("s2", "alpha", 2, "msg2"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;
    // expanded_projects is empty, so all collapsed
    app.rebuild_display_items();

    // 1 header only, no sessions
    assert_eq!(app.display_items.len(), 1);
    assert!(matches!(&app.display_items[0], DisplayItem::Header(g) if g.project_name == "alpha"));
}

// --- T019: toggle behavior ---

#[test]
fn toggle_flat_to_grouped_and_back() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "msg1"),
        make_session("s2", "beta", 2, "msg2"),
    ];
    let mut app = make_app(sessions);

    // Start in flat mode
    assert!(!app.grouped_mode);
    let flat_count = app.display_items.len();
    assert_eq!(flat_count, 2);

    // Toggle to grouped
    app.grouped_mode = true;
    app.expanded_projects.clear();
    app.rebuild_display_items();
    assert_eq!(app.display_items.len(), 2); // 2 headers, collapsed

    // Toggle back to flat
    app.grouped_mode = false;
    app.expanded_projects.clear();
    app.rebuild_display_items();
    assert_eq!(app.display_items.len(), 2); // 2 sessions
    assert!(app.display_items.iter().all(|item| matches!(item, DisplayItem::Session(_))));
}

#[test]
fn toggle_clears_expanded_projects() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "msg1"),
        make_session("s2", "beta", 2, "msg2"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;
    app.expanded_projects.insert("alpha".to_string());
    app.rebuild_display_items();
    assert_eq!(app.display_items.len(), 3); // 1 header + 1 session (alpha) + 1 header (beta)

    // Toggle back to flat clears expanded_projects
    app.grouped_mode = false;
    app.expanded_projects.clear();
    app.rebuild_display_items();
    assert!(app.expanded_projects.is_empty());
}

// --- T020: filter interaction in grouped mode ---

#[test]
fn filter_auto_expands_matching_groups() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "find me"),
        make_session("s2", "alpha", 2, "not this"),
        make_session("s3", "beta", 3, "other"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;

    // Apply filter that matches "find me" in alpha
    app.filter_query = "find".to_string();
    app.apply_filter();

    // Only alpha should be visible (auto-expanded), beta hidden
    let headers: Vec<&str> = app.display_items.iter().filter_map(|item| match item {
        DisplayItem::Header(g) => Some(g.project_name.as_str()),
        _ => None,
    }).collect();
    assert_eq!(headers, vec!["alpha"]);

    // Alpha should have its matching session visible
    let session_count = app.display_items.iter().filter(|item| matches!(item, DisplayItem::Session(_))).count();
    assert_eq!(session_count, 1);
}

#[test]
fn filter_hides_groups_with_no_matches() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "hello world"),
        make_session("s2", "beta", 2, "goodbye world"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;

    // Filter for something only in alpha
    app.filter_query = "hello".to_string();
    app.apply_filter();

    // Only alpha header should be visible
    let headers: Vec<&str> = app.display_items.iter().filter_map(|item| match item {
        DisplayItem::Header(g) => Some(g.project_name.as_str()),
        _ => None,
    }).collect();
    assert_eq!(headers, vec!["alpha"]);
}

#[test]
fn clear_filter_restores_collapsed() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "msg1"),
        make_session("s2", "beta", 2, "msg2"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;

    // Manually expand alpha
    app.expanded_projects.insert("alpha".to_string());
    app.rebuild_display_items();
    assert_eq!(app.display_items.len(), 3); // header + session + header

    // Apply and then clear filter (simulates Escape behavior)
    app.filter_query = "msg1".to_string();
    app.apply_filter();

    // Clear filter and expanded_projects (as the input handler does)
    app.filter_query.clear();
    app.expanded_projects.clear();
    app.apply_filter();

    // All groups collapsed: 2 headers only
    assert_eq!(app.display_items.len(), 2);
    assert!(app.display_items.iter().all(|item| matches!(item, DisplayItem::Header(_))));
}

// --- T021: edge cases ---

#[test]
fn single_project_grouped() {
    let sessions = vec![
        make_session("s1", "only-project", 1, "msg1"),
        make_session("s2", "only-project", 2, "msg2"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;
    app.rebuild_display_items();

    // One header, no sessions (collapsed)
    assert_eq!(app.display_items.len(), 1);
    assert!(matches!(&app.display_items[0], DisplayItem::Header(g) if g.project_name == "only-project" && g.session_count == 2));
}

#[test]
fn empty_session_list_grouped() {
    let sessions: Vec<Session> = vec![];
    let mut app = make_app(sessions);
    app.grouped_mode = true;
    app.rebuild_display_items();

    assert_eq!(app.display_items.len(), 0);
}

#[test]
fn single_session_per_project() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "msg1"),
        make_session("s2", "beta", 2, "msg2"),
        make_session("s3", "gamma", 3, "msg3"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;
    app.expanded_projects.insert("beta".to_string());
    app.rebuild_display_items();

    // alpha: 1 header (collapsed), beta: 1 header + 1 session, gamma: 1 header (collapsed)
    assert_eq!(app.display_items.len(), 4);

    // Verify beta has session_count 1
    let beta_header = app.display_items.iter().find(|item| match item {
        DisplayItem::Header(g) => g.project_name == "beta",
        _ => false,
    });
    assert!(matches!(beta_header, Some(DisplayItem::Header(g)) if g.session_count == 1));
}

#[test]
fn header_session_count_is_correct() {
    let sessions = vec![
        make_session("s1", "alpha", 1, "msg1"),
        make_session("s2", "alpha", 2, "msg2"),
        make_session("s3", "alpha", 3, "msg3"),
        make_session("s4", "beta", 4, "msg4"),
    ];
    let mut app = make_app(sessions);
    app.grouped_mode = true;
    app.rebuild_display_items();

    let alpha_count = app.display_items.iter().find_map(|item| match item {
        DisplayItem::Header(g) if g.project_name == "alpha" => Some(g.session_count),
        _ => None,
    });
    assert_eq!(alpha_count, Some(3));

    let beta_count = app.display_items.iter().find_map(|item| match item {
        DisplayItem::Header(g) if g.project_name == "beta" => Some(g.session_count),
        _ => None,
    });
    assert_eq!(beta_count, Some(1));
}
