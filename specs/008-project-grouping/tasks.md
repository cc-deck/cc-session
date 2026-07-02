# Tasks: Project Grouping View

**Input**: Design documents from `specs/008-project-grouping/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: No project setup needed. Existing Rust project with all dependencies in place.

(No tasks in this phase)

---

## Phase 2: Foundational (Data Model)

**Purpose**: Core data types and App state changes that ALL user stories depend on. These tasks are split at the compilation boundary: T001 defines all new types and changes the App struct; T002 updates all consumers so the codebase compiles again.

- [x] T001 Define new types and extend App state in src/tui/mod.rs:
  - Define `ProjectGroup` struct with fields: `project_name: String`, `latest_timestamp: DateTime<Utc>`, `session_count: usize`
  - Define `DisplayItem` enum with variants: `Session(DisplayEntry)`, `Header(ProjectGroup)`
  - Add `grouped_mode: bool` (default `false`) and `expanded_projects: HashSet<String>` (default empty) to `App` struct
  - Replace `display_entries: Vec<DisplayEntry>` with `display_items: Vec<DisplayItem>` on `App`
  - Update `App::new()` to wrap entries in `DisplayItem::Session`
  - **Deliverable**: New types exist, App struct compiles with new fields, but downstream code in view.rs and input.rs will not compile yet (expected)

- [x] T002 Update all consumers of the old `display_entries` field across three files to use `display_items: Vec<DisplayItem>`:
  - src/tui/mod.rs: `display_session`, `enter_conversation`, `move_down`, `move_up`, `ensure_visible`, `rebuild_display_entries` (rename to `rebuild_display_items`), `apply_filter`, `poll_content_search` - each method must pattern-match on `DisplayItem::Session(entry)` to extract the inner `DisplayEntry` (headers are skipped in flat mode)
  - src/tui/view.rs: `render_session_list` and any code that indexes into the display list
  - src/tui/input.rs: `handle_browsing` and `handle_conversation` where they index into the display list
  - **Deliverable**: Full codebase compiles and passes `cargo test` with behavior identical to before (flat mode only, no headers emitted yet)

---

## Phase 3: User Story 1 - Toggle Grouped View (P1)

**Goal**: Ctrl-T toggles between flat (default) and grouped view with collapsed project headers.
**Independent Test**: Launch cc-session, press Ctrl-T, verify project headers appear collapsed, press Ctrl-T again to return to flat view.

- [x] T007 [US1] Implement `rebuild_display_items` method that handles both flat mode (existing behavior, wrap in DisplayItem::Session) and grouped mode (group by project_name, sort groups by latest_timestamp, emit Header + Session items respecting expanded_projects set) in src/tui/mod.rs

  **Interfaces produced** (consumed by T008, T009, T011, T014, T016):
  ```rust
  // src/tui/mod.rs - on impl App
  pub fn rebuild_display_items(&mut self)
  // Reads: self.filtered_indices, self.sessions, self.content_results,
  //        self.grouped_mode, self.expanded_projects, self.filter_query
  // Writes: self.display_items: Vec<DisplayItem>
  //
  // In flat mode: emits DisplayItem::Session entries (existing behavior).
  // In grouped mode: emits DisplayItem::Header followed by DisplayItem::Session
  //   entries for expanded groups. Collapsed groups emit Header only.
  //   Groups sorted by ProjectGroup.latest_timestamp descending.
  //   When filter_query is non-empty, bypasses expanded_projects:
  //     matching groups auto-expand, empty groups omitted.
  ```
- [x] T008 [US1] Add Ctrl-T handler in `handle_browsing` that toggles `grouped_mode`, clears `expanded_projects`, calls `rebuild_display_items`, resets selection and scroll in src/tui/input.rs
- [x] T009 [US1] Render `DisplayItem::Header` rows with chevron glyph (`▶` collapsed / `▼` expanded), project name, and relative time using distinct styling (bold) in src/tui/view.rs
- [x] T010 [US1] Indent session rows under expanded headers (prepend 4 spaces to first message column) in src/tui/view.rs

---

## Phase 4: User Story 2 - Expand and Collapse Groups (P1)

**Goal**: Enter on a project header toggles expand/collapse. Enter on a session row opens the conversation.
**Independent Test**: Enter grouped mode, navigate to a header, press Enter to expand, verify sessions appear, press Enter again to collapse.

- [x] T011 [US2] Modify Enter handler in `handle_browsing`: if selected `DisplayItem` is a `Header`, toggle project_name in `expanded_projects` and call `rebuild_display_items`; if `Session`, open conversation (existing behavior) in src/tui/input.rs
- [x] T012 [US2] Ensure arrow key navigation treats header rows as normal selectable items (cursor stops on headers, does not skip them) in src/tui/input.rs
- [x] T013 [US2] Update scrollbar state calculation to use `display_items.len()` for total row count in src/tui/view.rs

---

## Phase 5: User Story 3 - Filter and Search in Grouped Mode (P2)

**Goal**: Filter/search auto-expands matching groups, hides empty groups, clearing restores collapsed state.
**Independent Test**: Enter grouped mode, type a filter, verify matching groups expand and non-matching disappear, clear filter and verify re-collapse.

- [x] T014 [US3] Update `rebuild_display_items` grouped-mode logic: when `filter_query` is non-empty, auto-expand groups that have matching sessions and hide groups with no matches (bypass `expanded_projects` set) in src/tui/mod.rs
- [x] T015 [US3] Clear `expanded_projects` when filter is cleared (Backspace to empty or Escape) so all groups return to collapsed in src/tui/input.rs
- [x] T016 [US3] Ensure content search results (from `content_results`) are grouped under their project headers in grouped mode, with those groups auto-expanded in src/tui/mod.rs

---

## Phase 6: User Story 4 - Border Title in Grouped Mode (P3)

**Goal**: Border title shows project count when in grouped mode.
**Independent Test**: Toggle to grouped mode, verify border title shows project count.

- [x] T017 [P] [US4] Update border title rendering: in grouped mode show "(N projects, M)" with no filter, or "(X/N projects, Y/M)" with filter active, counting unique project_names in src/tui/view.rs

---

## Phase 7: Tests

**Purpose**: Comprehensive test coverage for grouping behavior.

- [x] T018 [P] Write tests for `rebuild_display_items` in grouped mode: correct grouping by project_name, ordering by latest timestamp, collapsed/expanded state in tests/grouping_test.rs
- [x] T019 [P] Write tests for toggle behavior: flat-to-grouped-and-back preserves session data, expanded_projects cleared on toggle in tests/grouping_test.rs
- [x] T020 [P] Write tests for filter interaction in grouped mode: auto-expand matching groups, hide empty, clear restores collapsed in tests/grouping_test.rs
- [x] T021 [P] Write tests for edge cases: single project, empty session list, single session per project in tests/grouping_test.rs

---

## Dependencies

```
T001 → T002              (define types, then update all consumers)
T002 → T007              (data model must compile before grouping logic)
T007 → T008, T009, T010  (US1: grouping logic enables toggle, rendering)
T007 → T011, T012, T013  (US2: grouping logic enables expand/collapse)
T007 → T014, T015, T016  (US3: grouping logic enables filter interaction)
T009 → T017              (US4: header rendering enables border title)
T007..T017 → T018..T021  (tests after implementation)
```

## Parallel Execution

- **Phase 2**: T001 → T002 sequential (types first, then consumer updates)
- **Phase 3**: T009 and T010 can run in parallel (different rendering concerns) after T007
- **Phase 4**: T012 and T013 can run in parallel after T011
- **Phase 5**: T014 and T016 are related but sequential; T015 is independent
- **Phase 6**: T017 is independent once header rendering exists
- **Phase 7**: T018-T021 are all parallel (independent test files)

## Implementation Strategy

**MVP**: Phases 1-4 (Toggle + Expand/Collapse) deliver a usable grouped view.
**Incremental**: Phase 5 (Filter interaction) adds search support. Phase 6 (Border title) is polish.
**Order**: Follow phase order. Each phase produces a testable increment.

## Summary

- **Total tasks**: 17
- **Foundational**: 2 tasks (T001-T002)
- **US1 (Toggle)**: 4 tasks (T007-T010)
- **US2 (Expand/Collapse)**: 3 tasks (T011-T013)
- **US3 (Filter/Search)**: 3 tasks (T014-T016)
- **US4 (Border Title)**: 1 task (T017)
- **Tests**: 4 tasks (T018-T021)
