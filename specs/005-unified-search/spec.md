# Feature Specification: Unified Incremental Search

**Feature Branch**: `005-unified-search`
**Created**: 2026-03-07
**Status**: Draft
**Input**: User description: "Unified incremental search replacing two-mode search with a single search that combines metadata filtering and deep content search"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Single Search Entry Point (Priority: P1)

As a user, I want to press `/` and type a search term to find sessions matching by both metadata (project name, branch, first message) and conversation content, without needing to know which search mode to use.

**Why this priority**: This is the core value proposition. The current two-mode UX (filter via `/` plus deep search via `Ctrl-G`) forces users to decide where their search term might appear. A single search entry point removes that cognitive burden entirely.

**Independent Test**: Can be fully tested by pressing `/`, typing a term like "auth", and verifying that both metadata matches and content matches appear in the results list.

**Acceptance Scenarios**:

1. **Given** the application is in browsing mode, **When** I press `/` and type a search term, **Then** metadata matches appear immediately (within one keystroke).
2. **Given** I have typed a search term and paused typing, **When** the debounce period elapses, **Then** a background content search begins automatically.
3. **Given** a background content search completes, **When** additional content-only matches are found, **Then** they merge into the visible results list without disrupting my current selection.

---

### User Story 2 - Progressive Search Feedback (Priority: P1)

As a user, I want to see clear feedback about search progress so I know whether the search is still running or complete.

**Why this priority**: Without progress feedback, users cannot tell whether the results are final or more are coming. This is essential for the unified search to feel reliable rather than confusing.

**Independent Test**: Can be tested by searching for a common term and observing the status bar transition from match count to "searching content..." to the final count.

**Acceptance Scenarios**:

1. **Given** I am typing a search term, **When** only metadata matches are shown, **Then** the status bar displays the current match count (e.g., "5 matches").
2. **Given** the debounce period has elapsed, **When** the background content search is running, **Then** the status bar shows a progress indicator (e.g., "5 matches (searching content...)").
3. **Given** the content search completes, **When** new content matches are merged, **Then** the status bar updates to show the final total count (e.g., "17 matches").

---

### User Story 3 - Visual Distinction Between Match Types (Priority: P2)

As a user, I want to visually distinguish whether a session matched by metadata or by conversation content, so I can gauge the relevance of each result.

**Why this priority**: Knowing the match source helps users prioritize which sessions to open. A session whose title matches "auth" is likely more relevant than one where "auth" appeared deep in conversation. However, the search still works without this distinction.

**Independent Test**: Can be tested by searching for a term that matches both metadata and content across different sessions, and verifying that content-only matches have a distinct visual indicator.

**Acceptance Scenarios**:

1. **Given** search results contain both metadata matches and content-only matches, **When** I view the results list, **Then** content-only matches display a subtle visual indicator distinguishing them from metadata matches.
2. **Given** a session matches by both metadata and content, **When** it appears in results, **Then** it is shown as a metadata match (no content indicator needed).

---

### User Story 4 - Responsive Typing During Background Search (Priority: P2)

As a user, I want my typing to remain responsive even while a content search is running in the background, and I want new keystrokes to restart the search cycle.

**Why this priority**: If typing feels sluggish during content search, the unified experience would be worse than the current two-mode approach. Responsiveness is critical for the search to feel seamless.

**Independent Test**: Can be tested by typing rapidly while a content search is in progress and verifying that each keystroke updates metadata results instantly while cancelling and restarting the content search debounce.

**Acceptance Scenarios**:

1. **Given** a background content search is running, **When** I type an additional character, **Then** the metadata results update immediately and the running content search is cancelled.
2. **Given** I typed additional characters that cancelled a content search, **When** I pause typing for the debounce period, **Then** a new content search starts with the updated query.
3. **Given** I clear the search input entirely, **When** the input becomes empty, **Then** any running content search is cancelled and the full session list is restored.

---

### Edge Cases

- What happens when the user types and deletes characters rapidly? Content search should only trigger after the debounce period with no new keystrokes.
- What happens when content search returns zero additional results? The status bar should update to show the final count without any visual glitch or flicker.
- What happens when the user presses Escape during a content search? The search should be cancelled and the session list restored to the unfiltered state.
- What happens when the session index is very large (2000+ sessions)? Metadata filtering must remain instant; content search should complete within the established performance budget.
- What happens when the search term matches thousands of sessions in content? All matching results are shown with no cap, sorted by recency. The existing scrollable list handles large result sets.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a single search entry point via the `/` key that searches both metadata and conversation content.
- **FR-002**: System MUST display metadata matches immediately (within one keystroke latency) as the user types.
- **FR-003**: System MUST initiate a background content search after a debounce period of 300ms following the last keystroke.
- **FR-004**: System MUST cancel any in-progress content search when the user types additional characters.
- **FR-005**: System MUST merge content-only matches into the results list while keeping the user's selection on the currently-selected session (the selection follows the item, not the index position).
- **FR-006**: System MUST display a progress indicator in the status bar while content search is running.
- **FR-007**: System MUST visually distinguish content-only matches from metadata matches in the results list.
- **FR-008**: System MUST remove the separate deep search mode (`Ctrl-G` entry point) and consolidate all search into the unified `/` search.
- **FR-009**: System MUST preserve the user's search query when navigating into a conversation and returning to the results list.
- **FR-010**: System MUST restore the full unfiltered session list when the user clears the search input or presses Escape.
- **FR-011**: When the user opens a conversation from a content-only match, the system MUST scroll to and highlight the first occurrence of the search term in the conversation.

### Key Entities

- **Search Query**: The text entered by the user in the search input, used for both metadata filtering and content searching.
- **Match Type**: Classification of how a session matched the query, either by metadata (project name, branch, first message) or by conversation content.
- **Search State**: The current phase of the search lifecycle: idle, metadata-only results, content search in progress, or search complete.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can find any session (whether matching by title or content) using a single search interaction, without needing to switch search modes.
- **SC-002**: Metadata results appear within 1 keystroke latency (under 10ms per keystroke).
- **SC-003**: Full search results (metadata plus content) are available within 1 second of the user stopping typing, for datasets up to 2500 sessions.
- **SC-004**: Typing responsiveness is never degraded by background content search; each keystroke updates metadata results within 10ms regardless of content search state.
- **SC-005**: The number of interaction steps to perform a content search is reduced from 3 (press Ctrl-G, type query, press Enter) to 1 (type query after pressing `/`).

## Assumptions

- The pre-built session index from feature 004 (fast deep search) is available and provides content search performance under 1 second for common terms.
- The existing metadata filtering logic can be reused without modification for the immediate filtering phase.
- A 300ms debounce period provides a good balance between responsiveness and avoiding unnecessary content scans during fast typing.
- Results sorting follows the current timestamp-based ordering, with metadata matches and content matches interleaved by recency rather than grouped by match type.
- The `Ctrl-G` keybinding and associated deep search input/searching modes will be fully removed, not just hidden.

## Clarifications

### Session 2026-03-07

- Q: When new content-match results merge into the list, what happens to the user's selection? → A: Selection follows the currently-selected session (cursor stays on same item even if its position shifts).
- Q: When opening a conversation from a content-only match, should the viewer navigate to matching content? → A: Scroll to and highlight the first occurrence of the search term in the conversation.
- Q: Should results be capped when the search term matches thousands of sessions? → A: No cap. Show all matching results sorted by recency; the scrollable list handles large result sets.

## Dependencies

- **004-fast-deep-search**: The pre-built session index (completed in v0.3.1) is a hard prerequisite. Without sub-second content search, the debounced unified search UX would be unacceptably slow.
