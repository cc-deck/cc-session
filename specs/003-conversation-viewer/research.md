# Research: Conversation Viewer

## R1: JSONL Message Structure for Assistant Entries

**Decision**: Assistant entries in session JSONL files use the same `SessionFileEntry` structure with `type: "assistant"` and `message.content` as `StringOrArray`. We can reuse the existing parser.

**Rationale**: Examined `search.rs` and `discovery.rs`, which already parse both user and assistant entries. The `SessionFileEntry` struct handles both roles.

**Alternatives considered**: Creating a separate parser for assistant entries. Rejected because the format is identical.

## R2: Conversation Loading Strategy

**Decision**: Load all messages (user + assistant) from the session JSONL file into a `Vec<ConversationMessage>` in chronological order. Skip `file-history-snapshot` entries, system entries, and meta-messages. Do NOT load all content into the widget lines at once; instead pre-render all lines into a `Vec<Line>` on entry and scroll through it.

**Rationale**: Sessions typically have 10-500 messages. Pre-rendering all lines is fast enough and simplifies scrolling (just offset into the vec). No need for lazy rendering at this scale.

**Alternatives considered**:
- Lazy line rendering (only render visible lines). Rejected as premature optimization for typical session sizes.
- Streaming/chunked loading. Rejected for same reason.

## R3: Word Wrapping Strategy

**Decision**: Use ratatui's `Paragraph` widget with `Wrap { trim: false }` for rendering conversation content. This handles word wrapping at terminal width automatically.

**Rationale**: ratatui's built-in wrapping is sufficient. No need for manual line-splitting logic.

**Alternatives considered**: Manual word wrapping before building spans. Rejected because ratatui handles this natively.

**Update**: After further analysis, `Paragraph::wrap` makes it difficult to know the actual rendered line count (needed for page-up/page-down and search-to-line-offset). Better approach: pre-wrap lines manually during the rendering preparation phase, producing a flat `Vec<Line>` where each entry is one terminal row. This gives us exact control over scroll offsets and search hit positions.

## R4: Search Implementation

**Decision**: Implement incremental search within the pre-rendered line buffer. On each keystroke, scan all lines for case-insensitive matches and record positions. Highlight matches using dark yellow underline. Track a "current match index" for n/N navigation.

**Rationale**: With pre-rendered lines, search is a simple string scan. Match positions map directly to scroll offsets.

**Alternatives considered**: Using a separate search index. Rejected as unnecessary for the data sizes involved.

## R5: Code Fence Highlighting

**Decision**: Track "inside code fence" state while pre-rendering lines. Lines within ` ``` ` blocks get a different foreground color (e.g., `Color::Rgb(130, 170, 200)`, a muted blue). The fence delimiter line itself also gets this color.

**Rationale**: Simple state machine (toggle on ` ``` `). No parsing of language identifiers needed for v1.

**Alternatives considered**: Full syntax highlighting with `syntect`. Deferred to a future iteration as it adds a significant dependency.

## R6: Replacing Detail Mode vs Adding New Mode

**Decision**: Replace the existing `Mode::Detail` and `DetailState` with a new `Mode::Conversation` and `ConversationState`. Remove the old detail view code (bordered box with buttons).

**Rationale**: The conversation viewer supersedes the detail view entirely. Keeping both would add complexity without benefit.

**Alternatives considered**: Adding a separate mode alongside Detail. Rejected because the spec explicitly replaces the detail view.
