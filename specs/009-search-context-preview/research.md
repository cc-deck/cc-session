# Research: Search Context Preview

## R1: Snippet extraction within file_matches_all

**Decision**: Redesign `file_matches_all` to return `Option<MatchSnippet>` instead of `bool`. The function already reads and parses every JSONL line, extracts text blocks, and runs regex matching. Adding snippet extraction is a small addition to the existing loop.

**Rationale**: The function processes each message's cleaned text and checks regex matches. At the point where a match is confirmed, we already have the cleaned text and the regex match positions. Capturing a window around the match position requires only a character-boundary-safe slice operation, which the codebase already has patterns for (see `extract_entry_type`'s `is_char_boundary` usage).

**Alternatives considered**:
- Second pass over the file after `file_matches_all` returns true: doubles I/O for every matched file.
- Storing full message text in the index: memory-prohibitive for 2000+ sessions.

## R2: Multi-keyword cluster scoring

**Decision**: Use a sliding-window approach. For each message, find all keyword match positions (character offsets). For messages containing all keywords, compute the span (distance between first and last keyword match). Pick the message with the smallest span. For messages missing some keywords, fall back to the first match of any keyword.

**Rationale**: A full cross-message density analysis would require storing match positions from all messages and comparing across them, significantly complicating the single-pass architecture. Per-message scoring is simpler and sufficient since most meaningful matches have keywords close together within a single conversational turn.

**Alternatives considered**:
- Cross-message clustering: complex, requires buffering all message texts. Not worth it since each message typically represents a complete thought.
- First-match-wins: simple but often shows irrelevant context when the first occurrence is in an unrelated part of the conversation.

## R3: Snippet context window sizing

**Decision**: Extract a generous raw snippet (300 chars) during search. The TUI rendering layer truncates to the available column width at render time. This decouples search from terminal geometry.

**Rationale**: The search runs in a background thread with no access to terminal dimensions. Extracting a large window and truncating at render time is the cleanest separation of concerns. 300 chars covers approximately 3-4 lines of typical terminal width (80-120 chars), providing enough context for the densest cluster to be visible after centering and truncation.

**Alternatives considered**:
- Passing terminal width to the search thread: couples search to display state and requires re-extraction on terminal resize.
- Fixed snippet size matching typical terminal width: breaks on narrow or wide terminals.

## R4: Dimmed text styling in ratatui

**Decision**: Use `Style::default().fg(Color::DarkGray)` for surrounding context and the theme's normal text color for keywords. In light theme, use `Color::Gray` (a lighter gray) for the dimmed text to maintain contrast.

**Rationale**: ratatui's `Color::DarkGray` maps to the terminal's ANSI color 8, which is the standard "bright black" / dim text. This works across all terminal emulators. The existing theme system already has `text` and `text_dim` colors that handle dark/light variants.

**Alternatives considered**:
- Using the `Modifier::DIM` style modifier: Not reliably supported across all terminal emulators (some ignore it entirely).
- Background tint on keywords: Looks chunky in terminal cells and fights with row selection highlighting.

## R5: Return type change propagation

**Decision**: Introduce a `SearchResult` struct in `search.rs`. Change `deep_search_indexed` to return `Vec<SearchResult>`. Update the `mpsc` channel type in `App`, update `content_results` to `Vec<SearchResult>`, and update `display_session_from_entry` and `rebuild_display_items` to handle the new type.

**Rationale**: The type change follows the existing data flow: search -> channel -> App state -> display. Each step in the chain handles the new type. The `DisplayEntry` already has a `match_type` field that distinguishes content matches, so the rendering layer can check for a snippet when `match_type` is `Content` or `Both`.

**Alternatives considered**:
- Storing snippets in a separate `HashMap<String, MatchSnippet>` keyed by session ID: adds a parallel data structure that must be kept in sync with `content_results`.
- Adding an `Option<MatchSnippet>` field to `Session`: pollutes the shared session type with search-specific data.
