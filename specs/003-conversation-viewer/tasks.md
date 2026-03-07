# Tasks: Conversation Viewer

**Feature Branch**: `003-conversation-viewer`
**Generated**: 2026-03-07

## Task Dependency Graph

```
T1 (data types) ──┬──> T3 (conversation loading)
                   │
T2 (remove detail) ┤
                   │
                   ├──> T4 (pre-rendering + word wrap)
                   │
                   ├──> T5 (conversation view rendering)
                   │
                   ├──> T6 (input handling + scrolling)
                   │
                   └──> T7 (in-view search)
                        │
                        └──> T8 (auto-scroll to search hit)
```

## Tasks

### T1: Add ConversationMessage and ConversationState types
- [ ] **File**: `src/session.rs`
- **Depends on**: Nothing
- **Description**: Add `MessageRole` enum (User, Assistant), `ConversationMessage` struct (role, text, timestamp). These are the data types for loading conversation content.
- **Acceptance**: Types compile, can be constructed from test data.

### T2: Remove old Detail mode and replace with Conversation mode
- [ ] **File**: `src/tui/mod.rs`, `src/tui/input.rs`, `src/tui/view.rs`
- **Depends on**: T1
- **Description**: Replace `Mode::Detail` with `Mode::Conversation`. Replace `DetailState` with `ConversationState` (session_idx, lines as `Vec<Line<'static>>`, scroll_offset, search state fields, initial_search_terms). Replace `DetailButton` removal. Replace `Action::EnterDetail` with `Action::EnterConversation`. Update `App` struct: replace `detail` field with `conversation` field. Remove `render_detail` function. Remove `handle_detail` function. Update `enter_detail`/`leave_detail` to `enter_conversation`/`leave_conversation`.
- **Acceptance**: Code compiles with new mode. Old detail view code is removed. Enter from list opens (empty) conversation mode.

### T3: Add load_conversation() to discovery module
- [ ] **File**: `src/discovery.rs`
- **Depends on**: T1
- **Description**: Add `load_conversation(claude_home, session) -> Vec<ConversationMessage>` that reads ALL user and assistant entries from a session JSONL file. Skip `file-history-snapshot`, system entries, and meta-messages (using existing `is_meta_message`). Return messages in chronological order. Clean text with `clean_message()` but do NOT truncate.
- **Acceptance**: Loading a test session returns all user and assistant messages with correct roles, full text, and timestamps.

### T4: Pre-render conversation lines with word wrapping
- [ ] **File**: `src/tui/view.rs` (or new `src/tui/conversation.rs`)
- **Depends on**: T2, T3
- **Description**: Implement `pre_render_conversation(messages: &[ConversationMessage], width: usize, search_terms: &[&str]) -> Vec<Line<'static>>`. For each message:
  1. Emit a separator line: `─` chars filling width, timestamp right-aligned, DarkGray color.
  2. Emit a role header line: `▶ You` (Cyan) or `◀ Claude` (Green).
  3. Emit message body lines, word-wrapped to width. Track code fence state (toggle on lines starting with ` ``` `). Code fence content gets muted blue color (`Color::Rgb(130, 170, 200)`). Regular text gets `Color::Reset`.
  4. Apply search term highlighting (underline) across all text lines using the existing `highlight_terms` function.
- **Acceptance**: Given test messages, produces correct line count with proper wrapping, separators, role headers, and code fence coloring.

### T5: Render conversation view in TUI
- [ ] **File**: `src/tui/view.rs`
- **Depends on**: T2, T4
- **Description**: Add `render_conversation(frame, app, area)` function. Layout: main content area + 1-line status bar at bottom. Render the pre-rendered lines from `ConversationState.lines` starting at `scroll_offset`. Status bar shows: ` SESSION VIEWER  Space/b scroll  g/G top/bottom  / search  n/N next/prev  Enter copy & exit  Esc back`. When search is active, show search input in status bar instead. Update `render()` to call `render_conversation` when in `Mode::Conversation`.
- **Acceptance**: Conversation view renders with messages, separators, role headers, and status bar. Scrolling shows correct content window.

### T6: Input handling for conversation viewer (scrolling + actions)
- [ ] **File**: `src/tui/input.rs`
- **Depends on**: T2
- **Description**: Add `handle_conversation(app, key) -> Action` handler:
  - `Space`: scroll down by page height
  - `b`: scroll up by page height
  - `g`: scroll to top (offset = 0)
  - `G`: scroll to bottom (offset = max)
  - `j` / `Down`: scroll down by 1 line
  - `k` / `Up`: scroll up by 1 line
  - `Enter`: copy resume command & exit (`Action::CopyCommand`)
  - `Esc` / `q`: back to list (`Action::BackToList`)
  - `/`: enter search input mode
  - Page height should be calculated from frame area height minus status bar.
- **Acceptance**: All navigation keys work correctly. Enter copies command. Esc returns to list.

### T7: In-view incremental search with n/N
- [ ] **File**: `src/tui/input.rs`, `src/tui/view.rs`
- **Depends on**: T4, T5, T6
- **Description**: When `/` is pressed in conversation mode, switch to search input sub-mode. Typing updates `search_query` and re-highlights lines incrementally. Build `match_positions` (line indices with matches). Highlight matches in dark yellow (`Color::Rgb(180, 140, 0)`) with underline. `Enter` confirms search, scrolls to first match, sets `search_confirmed = true`. `n` jumps to next match, `N` to previous. `Esc` cancels search input. When search is confirmed, n/N cycle through `match_positions` wrapping around.
- **Acceptance**: Typing in search mode highlights matches incrementally. n/N navigate between matches. Match count shown in status bar.

### T8: Auto-scroll to search hit on entry
- [ ] **File**: `src/tui/mod.rs`
- **Depends on**: T7
- **Description**: When entering conversation mode from filter or deep search results, populate `initial_search_terms` from `app.filter_query` or `app.deep_search_query`. During pre-rendering, these terms are highlighted. After pre-rendering, find the first line containing a match and set `scroll_offset` to that line. Also populate `match_positions` so n/N immediately work.
- **Acceptance**: Opening a session from deep search results scrolls to and highlights the first match. n/N navigate from there.
