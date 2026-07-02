# Feature Specification: Project Grouping View

**Feature Branch**: `008-project-grouping`
**Created**: 2026-07-02
**Status**: Draft
**Input**: User description: "Add toggle-able project grouping view to session list"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Toggle Grouped View (Priority: P1)

A user with many sessions across different projects wants to see their sessions organized by project. They press Ctrl-T to switch from the default flat chronological list to a grouped view. Project headers appear as collapsible rows with chevron indicators. All groups start collapsed, showing a compact overview of all projects sorted by most recent activity. The user presses Ctrl-T again to return to the familiar flat list.

**Why this priority**: This is the core feature. Without the toggle and grouped display, nothing else works.

**Independent Test**: Can be fully tested by launching cc-session, pressing Ctrl-T, verifying project headers appear collapsed, pressing Ctrl-T again to return to flat view.

**Acceptance Scenarios**:

1. **Given** the session list is in flat mode (default), **When** the user presses Ctrl-T, **Then** the view switches to grouped mode with all projects collapsed, showing chevron headers sorted by most recent session timestamp.
2. **Given** the session list is in grouped mode, **When** the user presses Ctrl-T, **Then** the view switches back to flat chronological mode with the same session selected (or nearest).
3. **Given** the session list is in grouped mode, **When** viewing project headers, **Then** each header shows a right-pointing chevron (collapsed), the project name, and the relative time of the most recent session in that project.

---

### User Story 2 - Expand and Collapse Groups (Priority: P1)

A user in grouped mode sees collapsed project headers. They navigate to a project header and press Enter to expand it, revealing the sessions underneath indented below the header. The chevron changes from right-pointing to down-pointing. They press Enter on the header again to collapse it. Mouse click on a header also toggles expand/collapse.

**Why this priority**: Expand/collapse is essential for the grouped view to be usable. Without it, collapsed groups are a dead end.

**Independent Test**: Can be tested by entering grouped mode, navigating to a header, pressing Enter to expand, verifying sessions appear indented, pressing Enter again to collapse.

**Acceptance Scenarios**:

1. **Given** a collapsed project header is selected, **When** the user presses Enter, **Then** the group expands showing its sessions indented below the header, and the chevron changes to down-pointing.
2. **Given** an expanded project header is selected, **When** the user presses Enter, **Then** the group collapses hiding its sessions, and the chevron changes to right-pointing.
3. **Given** a session row (not a header) is selected in grouped mode, **When** the user presses Enter, **Then** the conversation opens (same behavior as flat mode).
4. **Given** a project header is visible, **When** the user clicks on it with the mouse, **Then** the group toggles between expanded and collapsed.

---

### User Story 3 - Filter and Search in Grouped Mode (Priority: P2)

A user in grouped mode types a filter query. Groups containing matching sessions auto-expand to show those sessions. Groups with no matching sessions are hidden entirely. When the filter is cleared, groups return to their collapsed state.

**Why this priority**: Search/filter is the primary way users find sessions. It must work seamlessly in grouped mode to avoid forcing users to toggle back to flat mode.

**Independent Test**: Can be tested by entering grouped mode, typing a filter term, verifying matching groups expand and non-matching groups disappear, then clearing the filter and verifying groups re-collapse.

**Acceptance Scenarios**:

1. **Given** grouped mode with all groups collapsed, **When** the user types a metadata filter query, **Then** groups containing matching sessions auto-expand showing only matching sessions, and groups with no matches are hidden.
2. **Given** grouped mode with a filter active, **When** the content search completes (after debounce), **Then** content-matched sessions also appear under their project groups, which auto-expand.
3. **Given** grouped mode with a filter active, **When** the user clears the filter (Backspace or Escape), **Then** all groups return to collapsed state and all projects are visible again.
4. **Given** grouped mode with manually expanded groups, **When** the user types a filter, **Then** the filter results take precedence (auto-expand matching, hide empty). When the filter is cleared, previously manually expanded groups return to collapsed.

---

### User Story 4 - Border Title in Grouped Mode (Priority: P3)

When in grouped mode, the border title reflects the grouping context. It shows the number of projects alongside the session counts already shown in flat mode.

**Why this priority**: Nice-to-have visual indicator that grouped mode is active and how many projects exist.

**Independent Test**: Can be tested by toggling to grouped mode and checking the border title shows project count.

**Acceptance Scenarios**:

1. **Given** the session list is in grouped mode with no filter, **Then** the border title shows the total number of projects and total sessions (e.g., "cc-session (15 projects, 852)").
2. **Given** the session list is in grouped mode with a filter active, **Then** the border title shows filtered projects and filtered sessions (e.g., "cc-session (3/15 projects, 12/852)").

---

### Edge Cases

- What happens when all sessions belong to a single project? One group header is shown; it functions the same as multiple groups.
- What happens when a project has only one session? The group header is shown with one session underneath when expanded.
- What happens when the user navigates with arrow keys past a collapsed group? The cursor moves to the next group header, skipping the hidden sessions.
- What happens when Ctrl-T is pressed during an active content search? The view mode toggles and search results are re-rendered in the new mode. The search is not cancelled.
- What happens when the session list is empty? Grouped mode shows nothing (same as flat mode).
- What happens when the user scrolls in grouped mode? The scrollbar reflects the actual number of visible rows (headers + expanded sessions), not the total session count.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a Ctrl-T keyboard shortcut to toggle between flat (default) and grouped view modes.
- **FR-002**: In grouped mode, system MUST display project headers as rows in the session list, each showing a chevron indicator (`▶` when collapsed, `▼` when expanded), the project name, and the relative timestamp of the most recent session in that project.
- **FR-003**: In grouped mode with no active filter, all project groups MUST be collapsed by default.
- **FR-004**: System MUST expand a group when the user presses Enter on a project header row, and collapse it when Enter is pressed again on the same header.
- **FR-005**: System SHOULD support mouse click on project headers to toggle expand/collapse. This requires adding mouse event handling to the input layer (not currently present in the codebase). If mouse support proves too complex, this requirement may be deferred without blocking the feature.
- **FR-006**: When a session row is selected and Enter is pressed, the conversation MUST open regardless of view mode (behavior unchanged from flat mode).
- **FR-007**: When a filter or search is active in grouped mode, groups containing matching sessions MUST auto-expand and groups with no matches MUST be hidden.
- **FR-008**: When the filter is cleared in grouped mode, all groups MUST return to collapsed state. This intentionally discards any manual expand/collapse state the user set before filtering, providing a clean reset.
- **FR-009**: Project groups MUST be ordered by the timestamp of their most recent session (newest first). Sessions within a group MUST be ordered newest-first.
- **FR-010**: The data model MUST represent project headers as distinct display entries distinguishable from session entries. The `rebuild_display_entries` function MUST handle insertion of headers and grouping of sessions when grouped mode is active.
- **FR-011**: Sessions under an expanded group MUST be visually indented relative to the project header.
- **FR-012**: The border title MUST show project count when in grouped mode.
- **FR-013**: Flat mode MUST remain the default when cc-session starts. The preference does not persist across restarts.
- **FR-014**: Content search (deep search) MUST work identically in grouped mode, with results appearing under their project groups.

### Key Entities

- **ProjectHeader**: A display entry representing a project group header. Contains the project name, most recent session timestamp, and expanded/collapsed state.
- **DisplayEntry / DisplaySource**: Extended to distinguish project headers from session entries. The existing `DisplaySource` enum (with `Sessions` and `Content` variants) and `DisplayEntry` struct need a mechanism to represent header rows. The exact approach (new enum variant, wrapper enum, or flag) is an implementation decision for the plan phase.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can toggle between flat and grouped view in under 0.5 seconds (single keypress, instant visual feedback).
- **SC-002**: Users can locate a specific project in grouped mode within 3 seconds when viewing up to 50 projects (scan collapsed headers).
- **SC-003**: Searching for sessions in grouped mode produces the same result set as searching in flat mode (no false positives or missed results).
- **SC-004**: All existing keyboard shortcuts and navigation patterns continue to work in both modes with no regressions.

## Assumptions

- The grouped/flat preference does not need to persist across restarts. The application always starts in flat mode.
- Left/Right arrow keys are kept free for future use and are not used for expand/collapse.
- The project name used for grouping is the `project_name` field already present on each Session (derived from the working directory).
- Mouse click support requires adding crossterm mouse event handling, which is not currently present in the codebase. This is new work scoped under FR-005.
