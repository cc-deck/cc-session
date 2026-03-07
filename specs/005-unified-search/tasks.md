# Tasks: Unified Incremental Search

**Input**: Design documents from `/specs/005-unified-search/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md

**Tests**: Not explicitly requested. Test tasks are omitted. Validation via `cargo test` and `cargo clippy` is part of polish.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new types and state fields needed by all user stories

- [ ] T001 (cc-session-onp.1) Add `ContentSearchState`, `MatchType`, `ContentMatch`, and `DisplayEntry` types to `src/tui/mod.rs`
- [ ] T002 (cc-session-onp.2) Add unified search state fields to App struct (`last_keystroke`, `content_search_state`, `content_results`, `cancel_flag`) in `src/tui/mod.rs`
- [ ] T003 (cc-session-onp.3) Remove `Mode::DeepSearchInput` and `Mode::DeepSearching` variants from Mode enum in `src/tui/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure changes that MUST be complete before user stories

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 (cc-session-jkf.1) Add cancellation flag parameter (`Arc<AtomicBool>`) to `deep_search_indexed()` in `src/search.rs` with per-file check and early exit
- [ ] T005 (cc-session-jkf.2) Remove `handle_deep_search_input()` function and `Ctrl-G` transition from `handle_filter()` in `src/tui/input.rs`
- [ ] T006 (cc-session-jkf.3) Remove deep search input UI rendering (DeepSearchInput/DeepSearching match arms) in `src/tui/view.rs`
- [ ] T007 (cc-session-jkf.4) Add merged display list builder method to App that combines `filtered_indices` and `content_results` sorted by timestamp, returning `Vec<DisplayEntry>` in `src/tui/mod.rs`
- [ ] T008 (cc-session-jkf.5) Fix all compilation errors from mode removal and signature changes across `src/tui/mod.rs`, `src/tui/input.rs`, `src/tui/view.rs`

**Checkpoint**: Code compiles with new types and removed modes. Metadata filter via `/` works as before. No content search yet.

---

## Phase 3: User Story 1 - Single Search Entry Point (Priority: P1) MVP

**Goal**: Press `/`, type a query, get both metadata and content results automatically

**Independent Test**: Press `/`, type "auth", verify metadata matches appear instantly, then content matches merge in after a brief pause

### Implementation for User Story 1

- [ ] T009 (cc-session-8um.1) [US1] Add debounce logic to the main event loop poll iteration: check `last_keystroke` age against 300ms threshold, spawn content search thread when elapsed, in `src/tui/mod.rs`
- [ ] T010 (cc-session-8um.2) [US1] Extend `handle_filter()` to set `last_keystroke` on each keystroke, cancel any running content search via `cancel_flag`, and reset `content_search_state` to Debouncing in `src/tui/input.rs`
- [ ] T011 (cc-session-8um.3) [US1] Implement content search result receiving: replace existing `poll_search()` with unified version that merges content results into `content_results` and sets `content_search_state` to Complete in `src/tui/mod.rs`
- [ ] T012 (cc-session-8um.4) [US1] Update session list rendering to use the merged display list (from T007) instead of `filtered_indices` alone when content results are present in `src/tui/view.rs`
- [ ] T013 (cc-session-8um.5) [US1] Implement selection stability: before merge, record selected session's file path; after merge, find and restore selection to the same session in `src/tui/mod.rs`
- [ ] T014 (cc-session-8um.6) [US1] Update Escape handling in Filtering mode to cancel content search, clear `content_results`, and restore full session list in `src/tui/input.rs`
- [ ] T015 (cc-session-8um.7) [US1] Remove `original_sessions` and `deep_search_query` fields from App struct, update `enter_conversation()` to use `filter_query` as search terms for conversation viewer in `src/tui/mod.rs`

**Checkpoint**: Unified search works end-to-end. Typing triggers metadata filter instantly, content search after 300ms pause. Results merge by timestamp. Ctrl-G no longer exists.

---

## Phase 4: User Story 2 - Progressive Search Feedback (Priority: P1)

**Goal**: Status bar shows match count, search progress indicator, and final total

**Independent Test**: Search for a common term, observe status bar transitions: "5 matches" then "5 matches (searching content...)" then "17 matches"

### Implementation for User Story 2

- [ ] T016 (cc-session-6gs.1) [US2] Update status bar rendering to show match count from merged display list size in `src/tui/view.rs`
- [ ] T017 (cc-session-6gs.2) [US2] Add "searching content..." progress indicator with spinner to status bar when `content_search_state` is Searching, reusing existing `spinner_char()` in `src/tui/view.rs`
- [ ] T018 (cc-session-6gs.3) [US2] Update status bar to show final total count when `content_search_state` is Complete in `src/tui/view.rs`

**Checkpoint**: Status bar provides clear feedback throughout the search lifecycle.

---

## Phase 5: User Story 3 - Visual Distinction Between Match Types (Priority: P2)

**Goal**: Content-only matches show a subtle visual indicator in the session list

**Independent Test**: Search for a term matching both metadata and content in different sessions, verify content-only matches have a distinct indicator

### Implementation for User Story 3

- [ ] T019 (cc-session-kzx.1) [US3] Add content-match indicator (dimmed dot or icon prefix) to session list entry rendering for `MatchType::Content` entries in `src/tui/view.rs`
- [ ] T020 (cc-session-kzx.2) [US3] Ensure sessions matching both metadata and content show as `MatchType::Both` (displayed as metadata, no indicator) in the merged display list builder in `src/tui/mod.rs`

**Checkpoint**: Content-only matches are visually distinguishable from metadata matches.

---

## Phase 6: User Story 4 - Responsive Typing During Background Search (Priority: P2)

**Goal**: Typing remains instant during content search; new keystrokes cancel and restart the search cycle

**Independent Test**: Type rapidly during a content search, verify metadata results update per-keystroke and content search restarts after pausing

### Implementation for User Story 4

- [ ] T021 (cc-session-emg.1) [US4] Verify cancellation flag is set atomically on keystroke and content search thread exits promptly (may need to reduce check interval in `deep_search_indexed`) in `src/search.rs`
- [ ] T022 (cc-session-emg.2) [US4] Ensure `content_results` is cleared on new keystroke so stale content results don't flash in the list, in `src/tui/input.rs`
- [ ] T023 (cc-session-emg.3) [US4] Verify empty query (all characters deleted) cancels content search and restores full session list correctly in `src/tui/input.rs`

**Checkpoint**: Typing is never blocked by background search. Cancellation and restart cycle works reliably.

---

## Phase 7: Conversation Auto-Scroll (Cross-cutting: FR-011)

**Purpose**: Content-only matches auto-scroll to the first occurrence in conversation viewer

- [ ] T024 (cc-session-hul.1) Update `enter_conversation()` to detect content-only match type and auto-scroll to first occurrence of search term in pre-rendered lines in `src/tui/mod.rs`
- [ ] T025 (cc-session-hul.2) Ensure `n`/`N` navigation works from the auto-scrolled position for subsequent matches in `src/tui/input.rs`

**Checkpoint**: Opening a content-only match scrolls to and highlights the first occurrence. Navigation to next/previous match works.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Cleanup, validation, and final refinements

- [ ] T026 (cc-session-07c.1) Update help overlay text to remove `Ctrl-G` reference and describe unified search behavior in `src/tui/view.rs`
- [ ] T027 (cc-session-07c.2) Run `cargo clippy` and fix all warnings across modified files
- [ ] T028 (cc-session-07c.3) Run `cargo test` and fix any broken tests from mode removal
- [ ] T029 (cc-session-07c.4) Manual performance validation: metadata filter <10ms/keystroke, full results <1s on 2000+ session dataset
- [ ] T030 (cc-session-07c.5) Run quickstart.md testing checklist for end-to-end validation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies, start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion, BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 completion, MVP target
- **US2 (Phase 4)**: Depends on Phase 3 (needs merged display list and search states)
- **US3 (Phase 5)**: Depends on Phase 3 (needs MatchType in display entries)
- **US4 (Phase 6)**: Depends on Phase 3 (needs cancellation infrastructure)
- **Auto-Scroll (Phase 7)**: Depends on Phase 3 (needs unified search query in conversation)
- **Polish (Phase 8)**: Depends on all previous phases

### User Story Dependencies

- **User Story 1 (P1)**: Foundational only. MVP, no other story dependencies.
- **User Story 2 (P1)**: Depends on US1 (needs search states and merged list to display counts).
- **User Story 3 (P2)**: Depends on US1 (needs MatchType tracking in display entries).
- **User Story 4 (P2)**: Depends on US1 (needs cancellation flag infrastructure).

### Within Each User Story

- Core state/logic before rendering
- Rendering before polish

### Parallel Opportunities

- T001, T002, T003 can partially overlap (same file, but independent sections)
- T005 and T006 can run in parallel (different files)
- US3 (Phase 5) and US4 (Phase 6) can run in parallel after US1
- T024 and T025 can run in parallel (different files)
- T026, T027, T028 can run in parallel

---

## Parallel Example: After Phase 3 (US1) Completes

```bash
# These can run in parallel since they modify different aspects:
Task: "T019 [US3] Add content-match indicator in src/tui/view.rs"
Task: "T021 [US4] Verify cancellation flag in src/search.rs"
Task: "T024 Auto-scroll in src/tui/mod.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T008)
3. Complete Phase 3: User Story 1 (T009-T015)
4. **STOP and VALIDATE**: Unified search works end-to-end
5. This delivers the core value: single search replacing two modes

### Incremental Delivery

1. Setup + Foundational -> Code compiles, old modes removed
2. Add US1 -> Unified search works (MVP!)
3. Add US2 -> Status bar feedback
4. Add US3 + US4 in parallel -> Visual distinction + responsive cancellation
5. Add auto-scroll + polish -> Complete feature

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- No new files are created; all changes modify existing files
- No new dependencies in Cargo.toml
- Commit after each phase checkpoint
- Stop at any checkpoint to validate independently

<!-- SDD-TRAIT:beads -->
## Beads Task Management

This project uses beads (`bd`) for persistent task tracking across sessions:
- Run `/sdd:beads-task-sync` to create bd issues from this file
- `bd ready --json` returns unblocked tasks (dependencies resolved)
- `bd close <id>` marks a task complete (use `-r "reason"` for close reason, NOT `--comment`)
- `bd comments add <id> "text"` adds a detailed comment to an issue
- `bd sync` persists state to git
- `bd create "DISCOVERED: [short title]" --labels discovered` tracks new work
  - Keep titles crisp (under 80 chars); add details via `bd comments add <id> "details"`
- Run `/sdd:beads-task-sync --reverse` to update checkboxes from bd state
- **Always use `jq` to parse bd JSON output, NEVER inline Python one-liners**
