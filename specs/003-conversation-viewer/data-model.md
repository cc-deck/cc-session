# Data Model: Conversation Viewer

## New Types

### ConversationMessage

Represents a single message in a conversation, used for loading from JSONL.

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `role` | `MessageRole` | `entry.message.role` | User or Assistant |
| `text` | `String` | `entry.message.content.text()` + `clean_message()` | Cleaned message content (full text, not truncated) |
| `timestamp` | `DateTime<Utc>` | `entry.timestamp` | When the message was sent |

### MessageRole

```rust
enum MessageRole {
    User,
    Assistant,
}
```

### ConversationState

Replaces `DetailState`. Holds all state for the conversation viewer mode.

| Field | Type | Description |
|-------|------|-------------|
| `session_idx` | `usize` | Index into `app.sessions` |
| `lines` | `Vec<RenderedLine>` | Pre-rendered, pre-wrapped terminal lines |
| `scroll_offset` | `usize` | First visible line index |
| `search_query` | `String` | Current search input |
| `search_active` | `bool` | Whether search input is active (typing) |
| `search_confirmed` | `bool` | Whether search has been confirmed (Enter pressed) |
| `match_positions` | `Vec<usize>` | Line indices containing matches |
| `current_match` | `usize` | Index into `match_positions` for n/N navigation |
| `initial_search_terms` | `Vec<String>` | Pre-populated from filter/deep search query |

### RenderedLine

A pre-rendered terminal line ready for display.

| Field | Type | Description |
|-------|------|-------------|
| `spans` | `Vec<(String, LinePartKind)>` | Text segments with their kind |

### LinePartKind

```rust
enum LinePartKind {
    Separator,      // Horizontal rule + timestamp (DarkGray)
    RoleHeader,     // "▶ You" or "◀ Claude"
    Text,           // Regular message text
    CodeFence,      // Text inside ``` blocks (muted blue)
    FenceDelimiter, // The ``` line itself
}
```

## Relationships

```
Session --(load_conversation)--> Vec<ConversationMessage>
Vec<ConversationMessage> --(pre_render)--> Vec<RenderedLine>  (wrapped to terminal width)
ConversationState.lines: Vec<RenderedLine>
ConversationState referenced by App when Mode::Conversation
```

## Removed Types

| Type | Reason |
|------|--------|
| `DetailState` | Replaced by `ConversationState` |
| `DetailButton` | No longer needed (actions are key-driven, no button widgets) |

## Modified Types

| Type | Change |
|------|--------|
| `Mode` | Replace `Detail` with `Conversation` |
| `App` | Replace `detail: Option<DetailState>` with `conversation: Option<ConversationState>` |
| `Action` | Replace `EnterDetail(usize)` with `EnterConversation(usize)`, remove `BackToList` (reuse or rename) |
