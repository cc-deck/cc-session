# Feature Specification: Conversation Viewer

**Feature Branch**: `003-conversation-viewer`
**Created**: 2026-03-07
**Status**: Implemented
**Input**: Replace the detail view with a full conversation viewer showing all user and assistant messages in a scrollable, searchable paged view.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Full Conversation from Session List (Priority: P1)

A developer selects a session from the list and presses Enter to open the full conversation viewer. Instead of the previous detail view (showing only 20 truncated user prompts), they see the complete conversation with both user and assistant messages. Messages are visually distinguished by role, separated by horizontal lines with right-aligned timestamps. The developer scrolls through with Space/b and uses g/G to jump to top/bottom. Content is centered and capped at 120 characters wide for comfortable reading.

**Why this priority**: This is the core feature, replacing the limited detail view with a rich conversation browser.

**Independent Test**: Launch cc-session, select a session, press Enter. Verify the full conversation is shown with role indicators, timestamps, and horizontal separators. Verify Space/b/g/G navigation works.

**Acceptance Scenarios**:

1. **Given** the session list is displayed, **When** the user presses Enter on a session, **Then** a full conversation view opens showing all user and assistant messages from the session JSONL file.
2. **Given** the conversation view is open, **When** the user presses Space or PageDown, **Then** the view scrolls down by one page.
3. **Given** the conversation view is open, **When** the user presses `b` or PageUp, **Then** the view scrolls up by one page.
4. **Given** the conversation view is open, **When** the user presses `g`, **Then** the view scrolls to the top of the conversation.
5. **Given** the conversation view is open, **When** the user presses `G`, **Then** the view scrolls to the bottom of the conversation.
6. **Given** the conversation view is open, **When** the user presses Enter, **Then** the resume command is copied to clipboard and the TUI exits.
7. **Given** the conversation view is open, **When** the user presses Esc or `q`, **Then** the view returns to the session list.

---

### User Story 2 - Search Within Conversation (Priority: P1)

A developer opens a conversation and presses `/` to search for a specific term. As they type, matching terms are highlighted incrementally with a yellow background. They press Enter to confirm the search, then use `n` to jump to the next match and `N` to jump to the previous match. The match is centered on screen.

**Why this priority**: Conversations can be very long. Search is essential for finding specific content within a session.

**Independent Test**: Open a conversation view, press `/`, type a term that appears multiple times, verify incremental highlighting, press Enter, then n/N to navigate between matches.

**Acceptance Scenarios**:

1. **Given** the conversation view is open, **When** the user presses `/` and types a search term, **Then** all matching text is highlighted incrementally as each character is typed.
2. **Given** a search is active, **When** the user presses Enter, **Then** the search is confirmed and the view scrolls to center the first match on screen.
3. **Given** a search is confirmed, **When** the user presses `n`, **Then** the view scrolls to center the next match.
4. **Given** a search is confirmed, **When** the user presses `N`, **Then** the view scrolls to center the previous match.
5. **Given** the user entered the conversation from a filter or deep search, **Then** the search terms from the filter/deep search are pre-highlighted in the conversation.
6. **Given** the user presses Esc during search input, **Then** the search is cancelled and highlights are cleared.

---

### User Story 3 - Auto-scroll to Search Hit (Priority: P2)

A developer performs a deep search for "ConnectionRefused", selects a matching session, and the conversation view opens with the first occurrence of "ConnectionRefused" centered on screen and highlighted.

**Why this priority**: When coming from a search, the user expects to land on the relevant content, not the top of a long conversation.

**Independent Test**: Perform a deep search, select a result, verify the conversation opens centered on the first match.

**Acceptance Scenarios**:

1. **Given** the user selects a session from deep search results, **When** the conversation view opens, **Then** it is scrolled to center the first occurrence of the deep search query on screen.
2. **Given** the user selects a session from filter results, **When** the conversation view opens, **Then** it is scrolled to center the first occurrence of the filter query (if found in conversation content).

---

### Edge Cases

- What happens when a session has no messages? Display "No messages found in this session." in the viewer.
- What happens when assistant messages contain very long code blocks? The text wraps at word boundaries within the content width (no horizontal scrolling).
- What happens when JSONL parsing fails for some lines? Skip invalid lines silently, show parseable messages.
- What happens when the terminal is resized? The view re-renders with the new dimensions (lines are re-wrapped to new width).
- What happens when a search has no matches? "No matches" is shown in the status bar and the search input remains active so the user can refine the query.

## Clarifications

### Session 2026-03-07

- Q: How does Enter behave when search input is active vs normal mode? → A: Enter confirms search when typing; copies & exits otherwise.
- Q: What happens when a search has no matches? → A: Show "No matches" in status bar, keep search input active for refinement.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Pressing Enter on a session in the list MUST open the conversation viewer, replacing the previous detail view.
- **FR-002**: The viewer MUST display all user and assistant messages from the session's JSONL file, in chronological order. Consecutive messages from the same role MUST be merged into a single entry with paragraphs separated by blank lines.
- **FR-003**: User messages MUST be visually distinct from assistant messages (different role indicator and color). User: Cyan "▶ You", Assistant: Yellow "◀ Claude".
- **FR-004**: Messages MUST be separated by a horizontal line with the timestamp right-aligned on the separator line. The separator and timestamp use DarkGray color. Content is capped at 120 characters wide and centered horizontally.
- **FR-005**: Code fence blocks (` ``` `) within messages MUST be visually distinguished from regular text (muted blue foreground color).
- **FR-006**: The viewer MUST support paged scrolling: Space/PageDown (page down), b/PageUp (page up), g (top), G (bottom), as well as arrow keys and j/k for line-by-line scrolling.
- **FR-007**: The viewer MUST support incremental search via `/`: matches highlighted as the user types, Enter to confirm, n/N for next/previous match. Matches are centered on screen when navigating.
- **FR-008**: Search matches MUST be highlighted with a yellow background (visible but not intrusive).
- **FR-009**: When entering the viewer from filter or deep search results, the search terms MUST be pre-highlighted and the view MUST scroll to center the first match on screen.
- **FR-010**: Pressing Enter MUST copy the resume command to clipboard and exit the TUI, UNLESS search input is active (in which case Enter confirms the search per FR-007).
- **FR-011**: Pressing Esc or `q` MUST return to the session list.
- **FR-012**: A status bar at the bottom MUST show available key bindings and search match count (N of M) during search.
- **FR-013**: Internal markup tags MUST be stripped from displayed message content. Line breaks MUST be preserved for proper markdown structure.
- **FR-014**: Tool-use content blocks (`tool_use`, `tool_result`) and system entries MUST be omitted from the conversation display.
- **FR-015**: Inline markdown MUST be rendered: `**bold**` as bold, `*italic*` as italic, `` `code` `` in muted blue, `# headings` in green bold with horizontal bar prefix.
- **FR-016**: Text MUST wrap at word boundaries. Lines MUST NOT exceed 120 characters. Content MUST be centered when the terminal is wider than 120 columns.

### Key Entities

- **ConversationMessage**: A single message in the conversation (after merging consecutive same-role entries). Attributes: role (user/assistant), text content (cleaned, multiline), timestamp.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The conversation viewer opens within 1 second for sessions with up to 500 messages.
- **SC-002**: Search highlighting updates within 100ms of each keystroke.
- **SC-003**: Page scrolling (Space/b) completes within 16ms (60fps rendering).
- **SC-004**: The resume command copied to clipboard is correct (same guarantee as the previous detail view).

## Assumptions

- Session JSONL files contain interleaved user and assistant entries with `type` field ("user" or "assistant").
- Assistant message content follows the same `message.content` structure as user messages (string or array of content blocks).
- Code fence blocks in messages use standard markdown triple-backtick syntax.
- Deep search is case-insensitive (regex patterns automatically get `(?i)` prefix).

## Out of Scope

- Full syntax highlighting within code blocks (planned for a future iteration with `syntect`).
- Horizontal scrolling for long lines.
- Editing or modifying conversation content.
- Exporting conversations to files.

## Changelog

- 2026-03-07: Initial specification based on brainstorm session.
- 2026-03-07: Evolved spec to match implementation:
  - FR-002: Added consecutive message merging
  - FR-003: Specified role indicator colors (Cyan for You, Yellow for Claude)
  - FR-004: Updated separator to content width (120 max, centered)
  - FR-006: Added PageUp/PageDown support
  - FR-007: Added match centering on navigation
  - FR-008: Changed from "dark yellow" to "yellow background"
  - FR-009: Changed "scroll to first match" to "center first match on screen"
  - FR-013: Added line break preservation for markdown structure
  - FR-015: New requirement for inline markdown rendering
  - FR-016: New requirement for word-boundary wrapping and max width
  - Added deep search case-insensitivity to Assumptions
  - Updated Out of Scope: basic markdown in scope, full syntect still out
  - Status changed from Draft to Implemented
