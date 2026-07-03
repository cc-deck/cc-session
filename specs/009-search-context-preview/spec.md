# Feature Specification: Search Context Preview

**Feature Branch**: `009-search-context-preview`
**Created**: 2026-07-03
**Status**: Draft
**Input**: Brainstorm document `brainstorm/02-search-context-preview.md`

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Content search shows matching context (Priority: P1)

A user types a search term in the session list to find sessions where a specific topic was discussed. When results arrive from the background content search, each matching session's title column displays a snippet of the matched conversation text centered on the search keyword, instead of the session's first message. The keyword appears in normal text color while the surrounding context is dimmed, making the match reason immediately visible.

**Why this priority**: This is the core feature. Without it, content search matches show no indication of why a session matched, forcing the user to open each one to find the relevant content.

**Independent Test**: Type a search term that matches deep conversation content (not the first message). Verify the session row shows a snippet containing the keyword, with dimmed surrounding text, instead of the first message.

**Acceptance Scenarios**:

1. **Given** a session where the keyword "kubernetes" appears in the 10th message but not in the first message, **When** the user searches for "kubernetes", **Then** the session's title column shows a snippet from the 10th message with "kubernetes" in normal text color and surrounding context dimmed.
2. **Given** a content search is active, **When** results arrive, **Then** all content-matched sessions display context snippets, not their first messages.
3. **Given** a session that matches both the metadata filter (keyword in first_message) and content search, **When** the user views the session list, **Then** the session shows a content snippet (not the first message), since the content match provides richer context.

---

### User Story 2 - Multi-keyword search shows densest cluster (Priority: P1)

A user searches for multiple keywords (e.g., "rust async"). The snippet displayed for each matching session centers on the passage where the search keywords appear closest together, giving the most relevant preview of the match.

**Why this priority**: Multi-keyword search is a primary use case. Showing a random keyword occurrence instead of the densest cluster would produce misleading or irrelevant previews.

**Independent Test**: Search for two keywords that appear together in one passage and separately in others. Verify the snippet shows the passage where they appear closest.

**Acceptance Scenarios**:

1. **Given** a session where "rust" and "async" appear 50 lines apart in one place but 2 lines apart in another, **When** the user searches "rust async", **Then** the snippet centers on the passage where they are 2 lines apart.
2. **Given** a multi-keyword search, **When** multiple keywords appear within the snippet's visible width, **Then** all visible keywords are shown in normal text color (not dimmed).

---

### User Story 3 - Metadata filter matches only visible text (Priority: P2)

The metadata filter (instant, as-you-type filtering) matches only against the session's first message text. It no longer matches against project name or git branch, which previously caused sessions to appear in results with no visible reason for the match.

**Why this priority**: This simplification ensures the filter only matches what the user can see in the title column. Project-level filtering is handled by the grouped view toggle.

**Note**: This is a breaking change from the current behavior where `filter_sessions` matches against `"{project_name} {git_branch} {first_message}"`. Users who relied on typing a project name to filter sessions will need to use the grouped view toggle instead.

**Independent Test**: Type a project name (e.g., "cc-session") in the filter. Verify that only sessions whose first message contains that text appear, not all sessions from that project.

**Acceptance Scenarios**:

1. **Given** a session from project "cc-session" whose first message is "fix the login bug", **When** the user types "cc-session" in the filter, **Then** the session does not appear (the filter no longer matches project name).
2. **Given** a session with git branch "feature/auth", **When** the user types "auth", **Then** the session only appears if "auth" is in the first message, not because of the branch name.
3. **Given** the filter is active with a term that appears in a session's first message, **When** the user views results, **Then** the matching keyword is highlighted in the first message text (existing behavior preserved).

---

### User Story 4 - More-matches indicator (Priority: P3)

When a session has multiple content matches beyond the displayed snippet, a minimal indicator (such as `+`) appears at the end of the snippet to signal that more matches exist within the session.

**Why this priority**: Nice-to-have that improves information density without adding clutter. The user can prioritize sessions with more matches.

**Independent Test**: Search for a term that appears many times in one session. Verify the snippet shows a `+` indicator at the end.

**Acceptance Scenarios**:

1. **Given** a session where the keyword appears in 5 different messages, **When** the snippet displays the best match, **Then** a `+` indicator appears after the snippet text.
2. **Given** a session where the keyword appears only once, **When** the snippet displays the match, **Then** no `+` indicator appears.

---

### Edge Cases

- What happens when the keyword is at the very beginning or end of a message? The snippet shows as much context as available on the other side, without padding.
- What happens when the keyword spans a message boundary (split across two messages)? Each message is searched independently; cross-message spans are not matched.
- What happens when the snippet is shorter than the available column width? Left-align the snippet (consistent with current first_message display).
- What happens when no content matches are found but the metadata filter matches? The session displays its first message as usual (no snippet to show).
- What happens when the terminal is very narrow? The snippet adapts to available width, truncating with `...` if the context does not fit.
- What happens with multi-byte UTF-8 keywords? Snippet extraction must use character boundaries, not byte boundaries, to avoid panics.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The search system MUST extract a context snippet for each content-matched session during the same file-reading pass used for matching (no additional I/O). This applies to both the primary indexed path (`file_matches_all` + index lookup) and the fallback metadata-parsing path (`search_file_with_metadata`). The `file_matches_all` function currently returns `bool` and must be redesigned to also return snippet data when a match is found.
- **FR-002**: The context snippet MUST be centered on the keyword (or keyword cluster for multi-keyword searches).
- **FR-003**: For multi-keyword searches, the system MUST select the passage where search keywords appear closest together (densest cluster). Distance is measured in characters within a single cleaned message text. Cross-message clustering is not performed; each message is scored independently.
- **FR-004**: When a session has multiple matching passages, the system MUST display the best match (densest cluster = smallest character span covering all keywords), not the first occurrence. Ties are broken by preferring user messages (per FR-011), then by earlier position in the conversation.
- **FR-005**: The snippet display MUST show keyword text in the normal text color and surrounding context text in a dimmed/gray color.
- **FR-006**: When additional matches exist beyond the displayed snippet, the system MUST show a minimal `+` indicator after the snippet text.
- **FR-007**: The snippet MUST fill the available title column width, truncated with `...` if the context exceeds available space. Snippet extraction in the search layer produces a generously-sized snippet (e.g., 300 characters). The TUI rendering layer truncates to the actual available column width at draw time. This decouples search from UI geometry.
- **FR-008**: The metadata filter MUST match only against `first_message`. It MUST NOT match against `project_name` or `git_branch`.
- **FR-009**: The search system MUST return a new result type containing both the session metadata and the optional match snippet. In the TUI layer, `App::content_results` (currently `Vec<Session>`) MUST change to `Vec<SearchResult>`. The `mpsc` channel and `search_receiver` must carry the new type. Display rendering in `view.rs` must check for a snippet and render it instead of `first_message` when present.
- **FR-010**: Snippet extraction MUST use character boundaries (not byte boundaries) for all string slicing to prevent UTF-8 panics.
- **FR-011**: The system MUST prefer user messages over assistant messages when multiple passages have equal keyword density, since user messages provide better context for session identification. When multiple user messages also tie on density, prefer the earlier message in the conversation (first occurrence wins).

### Key Entities

- **SearchResult**: A pairing of a Session with an optional MatchSnippet. Replaces `Session` as the content search return type.
- **MatchSnippet**: Contains the snippet text, the character ranges of keywords within that text, and a flag indicating whether more matches exist in the session.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can identify why a content search matched a session without opening the session, for every content-matched result in the list.
- **SC-002**: The snippet displayed for multi-keyword searches shows the passage with the smallest character span covering all keywords within a single message. When all keywords co-occur in at least one message, the snippet MUST show that co-occurring passage (deterministic, testable with known input data).
- **SC-003**: Content search with snippet extraction completes within the same performance envelope as the current search (under 1 second for common terms across 2000+ sessions).
- **SC-004**: No metadata filter results appear with invisible match reasons (no matches on hidden project_name or git_branch fields).

## Assumptions

- The snippet context window adapts to the available terminal column width for the title area. No fixed character budget is hardcoded.
- Snippet extraction adds minimal overhead to the existing search pass since it piggybacks on already-parsed and cleaned message text.
- The dimmed text style uses the terminal's built-in dim attribute or a gray foreground color, respecting both dark and light theme variants.
- The `+` indicator is a single character appended after the truncated snippet, not consuming significant column space.
