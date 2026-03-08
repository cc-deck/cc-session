# Tasks: Conversation Viewer Enhancements

**Input**: Design documents from `/specs/006-viewer-enhancements/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md

**Tests**: Not explicitly requested. Validation via `cargo test` and `cargo clippy` is part of polish.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependencies and create new module files

- [ ] T001 (cc-session-21b.1) Add `syntect = "5.3"`, `syntect-tui = "3.0"`, and `termbg = "0.6"` dependencies to `Cargo.toml`
- [ ] T002 (cc-session-21b.2) [P] Create `src/theme.rs` module with empty Theme struct skeleton and register in `src/main.rs` as `mod theme`
- [ ] T003 (cc-session-21b.3) [P] Create `src/tui/table.rs` module with empty function signatures and register in `src/tui/mod.rs`
- [ ] T004 (cc-session-21b.4) [P] Create `src/tui/syntax.rs` module with empty function signatures and register in `src/tui/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user stories

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 (cc-session-dyq.1) Define the `Theme` struct with all color fields (role headers, message backgrounds, code, tables, text, search highlight, syntect theme name) in `src/theme.rs`
- [ ] T006 (cc-session-dyq.2) Implement `Theme::dark()` constructor with dark theme color values in `src/theme.rs`
- [ ] T007 (cc-session-dyq.3) Implement `Theme::light()` constructor with light theme color values in `src/theme.rs`
- [ ] T008 (cc-session-dyq.4) Add `theme: Theme` field to the `App` struct and initialize with `Theme::dark()` default in `src/tui/mod.rs`
- [ ] T009 (cc-session-dyq.5) Pass `&app.theme` to `view::render()` and propagate to all rendering functions in `src/tui/view.rs`

**Checkpoint**: Code compiles with Theme struct and dark/light constructors. Theme is accessible during rendering but not yet used for colors.

---

## Phase 3: User Story 1 - Clean Conversation Content (Priority: P1) MVP

**Goal**: Strip system-injected content blocks so only actual user prompts and Claude responses are visible

**Independent Test**: Open a conversation with local command caveats and tool output. Verify only actual user text appears.

### Implementation for User Story 1

- [ ] T010 (cc-session-jh1.1) [US1] Add the `SYSTEM_TAGS` constant list (all known system tag names) and implement `strip_system_blocks()` function using string find/drain loop in `src/session.rs`
- [ ] T011 (cc-session-jh1.2) [US1] Update `load_conversation()` in `src/discovery.rs` to call `strip_system_blocks()` before `clean_message_multiline()` in the message processing pipeline
- [ ] T012 (cc-session-jh1.3) [US1] Update `is_meta_message()` in `src/session.rs` to also check if the message is empty after `strip_system_blocks()` (skip pure-system messages)
- [ ] T013 (cc-session-jh1.4) [US1] Update `strip_system_blocks` to handle tags with attributes (e.g., `<rosa-profile name="aaet">`) by matching on tag name prefix before `>` or space in `src/session.rs`
- [ ] T014 (cc-session-jh1.5) [US1] Verify that the existing `strip_tags` in search.rs content matching still works correctly with the new `strip_system_blocks` in `src/search.rs`

**Checkpoint**: Conversations no longer show "Caveat:" text, tool output, or system reminders. Pure-system messages are skipped.

---

## Phase 4: User Story 3 - Adaptive Color Theme (Priority: P2, but prerequisite for US2/US5)

**Goal**: Auto-detect terminal background and adapt colors, with --light/--dark override

**Independent Test**: Launch with `--light` flag and verify all text is readable on a light terminal. Launch without flags and verify auto-detection.

### Implementation for User Story 3

- [ ] T015 (cc-session-95e.1) [US3] Implement `Theme::detect(timeout)` using `termbg::theme()` with fallback to dark in `src/theme.rs`
- [ ] T016 (cc-session-95e.2) [US3] Add `--light` and `--dark` CLI flags to the Cli struct in `src/main.rs`
- [ ] T017 (cc-session-95e.3) [US3] Wire theme selection logic in `src/main.rs`: CLI flag overrides auto-detection, pass result to `tui::run()`
- [ ] T018 (cc-session-95e.4) [US3] Update `tui::run()` signature to accept a `Theme` parameter and store in `App` in `src/tui/mod.rs`
- [ ] T019 (cc-session-95e.5) [US3] Replace all hardcoded RGB color values in `render_status_bar()` with `app.theme` field references in `src/tui/view.rs`
- [ ] T020 (cc-session-95e.6) [US3] Replace all hardcoded RGB color values in `render_session_list()` with `app.theme` field references in `src/tui/view.rs`
- [ ] T021 (cc-session-95e.7) [US3] Replace hardcoded `Rgb(100,80,0)` search highlight background with `app.theme.search_highlight_bg` in `highlight_terms()` in `src/tui/view.rs`

**Checkpoint**: `--light` produces readable output on light terminals. `--dark` forces dark theme. Auto-detection works. All colors come from Theme struct.

---

## Phase 5: User Story 2 - Visual Role Separation (Priority: P1)

**Goal**: Full-width role header bars with colored backgrounds, user message body tinting

**Independent Test**: Open any conversation and verify role transitions are immediately visible through colored bars and user messages have distinct background.

### Implementation for User Story 2

- [ ] T022 (cc-session-5c9.1) [US2] Refactor role header rendering in `pre_render_conversation()`: replace "▶ You" / "◀ Claude" text spans with full-width `Line` objects using `theme.user_header_bg`/`theme.assistant_header_bg` background colors in `src/tui/view.rs`
- [ ] T023 (cc-session-5c9.2) [US2] Integrate timestamp into the role header bar as right-aligned text, removing the separate separator line above each message in `src/tui/view.rs`
- [ ] T024 (cc-session-5c9.3) [US2] Apply `theme.user_message_bg` background tint to all body lines within user message blocks in `pre_render_conversation()` in `src/tui/view.rs`
- [ ] T025 (cc-session-5c9.4) [US2] Ensure the background tint spans the full content width (pad lines with spaces to content_width) in `src/tui/view.rs`

**Checkpoint**: Role headers are full-width colored bars. User messages have a subtle background tint. Timestamps are in the header bar.

---

## Phase 6: User Story 4 - Markdown Table Rendering (Priority: P2)

**Goal**: Render markdown tables with Unicode box-drawing grid, auto-sized columns, bold headers

**Independent Test**: Open a conversation containing a markdown table and verify grid rendering with aligned columns.

### Implementation for User Story 4

- [ ] T026 (cc-session-3gb.1) [US4] Implement `detect_table_block()` function that identifies consecutive pipe-starting lines in `src/tui/table.rs`
- [ ] T027 (cc-session-3gb.2) [US4] Implement `parse_table()` that splits cells, detects separator rows, and computes column widths in `src/tui/table.rs`
- [ ] T028 (cc-session-3gb.3) [US4] Implement `render_table()` that produces `Vec<Line<'static>>` with box-drawing characters (┌─┬─┐, │, ├─┼─┤, └─┴─┘) and bold headers in `src/tui/table.rs`
- [ ] T029 (cc-session-3gb.4) [US4] Handle table overflow: truncate cell content with "..." when total table width exceeds content area width in `src/tui/table.rs`
- [ ] T030 (cc-session-3gb.5) [US4] Integrate table detection into `pre_render_conversation()`: collect consecutive pipe lines, call `parse_table` + `render_table`, fall back to plain text on parse failure in `src/tui/view.rs`
- [ ] T031 (cc-session-3gb.6) [US4] Apply `theme.table_border` color to box-drawing characters and `theme.table_header` style to header row in `src/tui/table.rs`

**Checkpoint**: Markdown tables display with grid lines, auto-sized columns, and bold headers. Malformed tables fall back to plain text.

---

## Phase 7: User Story 5 - Syntax Highlighted Code Blocks (Priority: P3)

**Goal**: Code blocks with language tags display full syntax highlighting; code blocks have distinct background

**Independent Test**: Open a conversation with a ```rust code block and verify keywords, strings, comments have distinct colors.

### Implementation for User Story 5

- [ ] T032 (cc-session-1qw.1) [US5] Implement `SyntaxHighlighter` struct that lazily loads `SyntaxSet` and `ThemeSet` from syntect defaults in `src/tui/syntax.rs`
- [ ] T033 (cc-session-1qw.2) [US5] Implement `highlight_code()` method that takes code lines + language tag + theme name and returns `Vec<Line<'static>>` using `syntect-tui::into_span()` in `src/tui/syntax.rs`
- [ ] T034 (cc-session-1qw.3) [US5] Extract language tag from code fence opening line (first word after ```) and handle extra tokens like `rust,ignore` in `src/tui/syntax.rs`
- [ ] T035 (cc-session-1qw.4) [US5] Integrate syntax highlighter into `pre_render_conversation()`: when inside a code fence with a recognized language, use `highlight_code()` instead of single-color style in `src/tui/view.rs`
- [ ] T036 (cc-session-1qw.5) [US5] Apply `theme.code_block_bg` background color to all code block lines (both highlighted and fallback) in `src/tui/view.rs`
- [ ] T037 (cc-session-1qw.6) [US5] Select syntect theme based on `theme.syntect_theme` field (dark: "base16-ocean.dark", light: "InspiredGitHub") in `src/tui/syntax.rs`
- [ ] T038 (cc-session-1qw.7) [US5] Add `SyntaxHighlighter` to `App` struct (initialized once at startup) and pass to rendering functions in `src/tui/mod.rs`

**Checkpoint**: Code blocks with language tags show multi-colored syntax highlighting. Unrecognized languages fall back to single color. Code blocks have distinct background.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Cleanup, validation, and final refinements

- [ ] T039 (cc-session-9uy.1) Run `cargo clippy` and fix all warnings across modified and new files
- [ ] T040 (cc-session-9uy.2) Run `cargo test` and fix any broken tests
- [ ] T041 (cc-session-9uy.3) [P] Update conversation status bar help text to reflect any changed keybindings in `src/tui/view.rs`
- [ ] T042 (cc-session-9uy.4) Run quickstart.md testing checklist for end-to-end validation
- [ ] T043 (cc-session-9uy.5) Performance validation: verify viewer renders within 500ms for 100+ message conversations

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies, start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion, BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2, MVP target. No dependency on theme.
- **US3 (Phase 4)**: Depends on Phase 2. Prerequisite for US2 and US5 (they need theme colors).
- **US2 (Phase 5)**: Depends on US3 (needs theme colors for header bars and message tinting)
- **US4 (Phase 6)**: Depends on US3 (needs theme colors for table borders)
- **US5 (Phase 7)**: Depends on US3 (needs theme for syntect theme selection)
- **Polish (Phase 8)**: Depends on all previous phases

### User Story Dependencies

- **User Story 1 (P1)**: Foundational only. MVP, independent of theme.
- **User Story 3 (P2)**: Foundational only. Prerequisite for US2, US4, US5.
- **User Story 2 (P1)**: Depends on US3 (needs theme colors).
- **User Story 4 (P2)**: Depends on US3 (needs theme colors). Can parallel with US2 and US5.
- **User Story 5 (P3)**: Depends on US3 (needs theme for syntect). Can parallel with US2 and US4.

### Parallel Opportunities

- T002, T003, T004 can run in parallel (different new files)
- T006 and T007 can run in parallel (same file but independent constructors)
- US4 (Phase 6), US2 (Phase 5), and US5 (Phase 7) can run in parallel after US3
- T039, T040, T041 can run in parallel

---

## Parallel Example: After Phase 4 (US3 Theme) Completes

```bash
# These can run in parallel since they modify different files/aspects:
Task: "T022 [US2] Refactor role header rendering in src/tui/view.rs"
Task: "T026 [US4] Implement detect_table_block() in src/tui/table.rs"
Task: "T032 [US5] Implement SyntaxHighlighter in src/tui/syntax.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T009)
3. Complete Phase 3: User Story 1 (T010-T014)
4. **STOP and VALIDATE**: System content stripped, conversations are clean
5. This delivers the highest-impact fix: no more "Caveat:" noise

### Incremental Delivery

1. Setup + Foundational -> New modules, theme struct, dependencies
2. Add US1 -> Clean conversations (MVP!)
3. Add US3 -> Theme system with auto-detection
4. Add US2 + US4 + US5 in parallel -> Visual headers, tables, syntax highlighting
5. Polish -> Final validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- 3 new files created: `src/theme.rs`, `src/tui/table.rs`, `src/tui/syntax.rs`
- 3 new dependencies: syntect, syntect-tui, termbg
- US3 (theme) is executed before US2 (visual) despite both being specified, because US2 needs theme colors
- Commit after each phase checkpoint

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
