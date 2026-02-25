use cc_session::session::{SessionFileEntry, StringOrArray};

#[test]
fn parse_user_entry_with_string_content() {
    let json = r#"{"type":"user","sessionId":"abc","cwd":"/tmp","gitBranch":"main","timestamp":"2025-02-20T10:00:00.000Z","version":"2.1.50","message":{"role":"user","content":"hello world"},"uuid":"msg-1"}"#;
    let entry: SessionFileEntry = serde_json::from_str(json).unwrap();
    assert_eq!(entry.entry_type, "user");
    assert_eq!(entry.cwd.as_deref(), Some("/tmp"));
    assert_eq!(entry.git_branch.as_deref(), Some("main"));
    let msg = entry.message.unwrap();
    assert_eq!(msg.content.text(), "hello world");
}

#[test]
fn parse_user_entry_with_array_content() {
    let json = r#"{"type":"user","sessionId":"abc","cwd":"/tmp","timestamp":"2025-02-20T10:00:00.000Z","message":{"role":"user","content":[{"type":"text","text":"first"},{"type":"thinking","thinking":"..."},{"type":"text","text":" second"}]},"uuid":"msg-2"}"#;
    let entry: SessionFileEntry = serde_json::from_str(json).unwrap();
    let msg = entry.message.unwrap();
    assert_eq!(msg.content.text(), "first second");
}

#[test]
fn parse_file_history_snapshot() {
    let json = r#"{"type":"file-history-snapshot","messageId":"snap-1","snapshot":{"messageId":"snap-1","trackedFileBackups":{},"timestamp":"2025-02-20T10:00:00.000Z"},"isSnapshotUpdate":false}"#;
    let entry: SessionFileEntry = serde_json::from_str(json).unwrap();
    assert_eq!(entry.entry_type, "file-history-snapshot");
    assert!(entry.message.is_none());
}

#[test]
fn parse_assistant_entry() {
    let json = r#"{"type":"assistant","sessionId":"abc","cwd":"/tmp","gitBranch":"main","timestamp":"2025-02-20T10:00:05.000Z","message":{"role":"assistant","content":[{"type":"text","text":"I can help with that."}]},"uuid":"msg-3"}"#;
    let entry: SessionFileEntry = serde_json::from_str(json).unwrap();
    assert_eq!(entry.entry_type, "assistant");
}

#[test]
fn malformed_json_line_fails_gracefully() {
    let json = r#"{"type":"user", broken json}"#;
    let result = serde_json::from_str::<SessionFileEntry>(json);
    assert!(result.is_err());
}

#[test]
fn string_or_array_text_extraction() {
    let s = StringOrArray::Str("plain text".to_string());
    assert_eq!(s.text(), "plain text");
}
