# Tasks: cc-session - Claude Code Session Finder

**Input**: Design documents from `/specs/001-session-finder/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included for core logic (parsing, discovery, filtering) but not for TUI rendering.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and Rust project structure

- [ ] T001 (cc-session-nq7.1) Initialize Rust project with `cargo init --name cc-session` and configure Cargo.toml with all dependencies: ratatui 0.30, crossterm 0.29, clap 4.5 (derive feature), nucleo 0.5, arboard 3.6 (wayland-data-control feature), rayon 1.11, serde 1.0 (derive feature), serde_json 1.0, chrono 0.4, chrono-humanize 0.2, dirs 6.0, regex 1.0. Set edition = "2021", rust-version = "1.80.0".
- [ ] T002 (cc-session-nq7.2) Create source file structure per plan: `src/main.rs`, `src/session.rs`, `src/discovery.rs`, `src/search.rs`, `src/filter.rs`, `src/clipboard.rs`, `src/scriptable.rs`, `src/tui/mod.rs`, `src/tui/view.rs`, `src/tui/input.rs`. Each file starts with module declaration only.
- [ ] T003 (cc-session-nq7.3) Create test fixture directory structure: `tests/fixtures/history.jsonl`, `tests/fixtures/sessions/-project-a/test-uuid-1.jsonl`, `tests/fixtures/sessions/-project-b/test-uuid-2.jsonl` with realistic sample data matching the Claude Code JSONL format (include file-history-snapshot entries, user entries with both string and array content formats, assistant entries).
- [ ] T004 (cc-session-nq7.4) Set up CLI argument parsing with clap derive API in `src/main.rs`. Define `Cli` struct with: `-s`/`--select <QUERY>` (Option<String>), `-g`/`--grep <PATTERN>` (Option<String>), `--since <DURATION>` (Option<String>), `--last <N>` (Option<usize>), `-h`/`--help`, `-V`/`--version`. Add mode dispatch logic: if `-s` set call scriptable mode, if `-g` set without `-s` call deep search TUI, else call interactive TUI. Wire up module declarations.

**Checkpoint**: Project compiles, `cargo run -- --help` shows usage, fixture data exists.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data types and JSONL parsing that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 (cc-session-bx6.1) [P] Implement Session and related data types in `src/session.rs`. Define structs: `Session` (id, project_path, project_name, git_branch, timestamp, first_message, cwd, project_exists), `SessionFileEntry` (type_field, session_id, cwd, git_branch, timestamp, version, message), `MessageContent` (role, content), `HistoryEntry` (display, timestamp, project). Implement custom serde deserializer for `StringOrArray` that handles both plain string and `[{type: "text", text: "..."}]` array content format. Implement `Session::resume_command()` returning `cd <cwd> && claude -r <id>`. Implement `Session::project_name()` extracting last path component.
- [ ] T006 (cc-session-bx6.2) [P] Implement session discovery in `src/discovery.rs`. Function `discover_sessions(claude_home: &Path) -> Vec<Session>` that: (1) reads `claude_home/projects/` directory, (2) for each subdirectory lists `.jsonl` files, (3) extracts session ID from filename, (4) decodes project path from directory name (replace `-` with `/`), (5) reads first few lines of each JSONL file using `BufReader` to find first `type: "user"` entry (skip `file-history-snapshot` entries), (6) extracts cwd, gitBranch, timestamp, message.content, (7) overrides decoded project path with `cwd` from the entry (authoritative), (8) checks if project path exists on disk, (9) sorts by timestamp descending. Use `rayon::par_iter` to parallelize across session files. Support `CLAUDE_HOME` environment variable override for testing.
- [ ] T007 (cc-session-bx6.3) [P] Implement clipboard module in `src/clipboard.rs`. Function `copy_to_clipboard(text: &str) -> Result<(), String>` using arboard. Function `clipboard_available() -> bool` that tests clipboard access. Fallback function `print_with_clipboard_warning(text: &str)` that prints to stdout with a "clipboard unavailable" note.
- [ ] T008 (cc-session-bx6.4) [P] Implement duration parsing for `--since` flag in `src/main.rs` or a utility module. Parse strings like "7d", "2w", "1m" into `chrono::Duration`. Support suffixes: d (days), w (weeks = 7 days), m (months = 30 days). Return error for invalid formats.
- [ ] T009 (cc-session-bx6.5) Write unit tests for session parsing in `tests/session_test.rs`. Test: parsing user entry with string content, parsing user entry with array content blocks, parsing file-history-snapshot entry (should be skipped), parsing assistant entry, handling malformed JSON lines gracefully. Use fixture files from `tests/fixtures/`.
- [ ] T010 (cc-session-bx6.6) Write unit tests for session discovery in `tests/discovery_test.rs`. Test: discovering sessions from fixture directory, extracting session ID from filename, decoding project path from directory name (including hyphenated names), sorting by timestamp descending, skipping missing JSONL files gracefully, handling empty project directories.

**Checkpoint**: `cargo test` passes. `discover_sessions()` returns correctly parsed `Vec<Session>` from fixture data.

---

## Phase 3: User Story 1 - Browse and Resume Sessions (Priority: P1) MVP

**Goal**: Interactive TUI displaying all sessions in two-line format with navigation and clipboard copy on Enter.

**Independent Test**: Run `cc-session` with real `~/.claude/` data, verify sessions appear sorted by time, navigate with j/k, press Enter to copy resume command.

### Implementation for User Story 1

- [ ] T011 (cc-session-a0o.1) [US1] Implement TUI app state and event loop in `src/tui/mod.rs`. Define `App` struct with: sessions (Vec<Session>), filtered_indices (Vec<usize>), selected (usize), scroll_offset (usize), mode (enum: Browsing, Filtering, DeepSearch), filter_query (String), status_message (Option<String>). Implement `App::new(sessions: Vec<Session>)`. Implement main event loop: initialize terminal (alternate screen, raw mode), loop on crossterm events with 250ms poll timeout, call update() then view(), restore terminal on exit. Handle panic hook to restore terminal state.
- [ ] T012 (cc-session-a0o.2) [US1] Implement two-line session rendering in `src/tui/view.rs`. Function `render_session_list(app: &App, frame: &mut Frame, area: Rect)` that: (1) calculates visible sessions based on scroll_offset and area height (each session = 2 lines), (2) renders line 1: `project_name · git_branch · relative_time` using chrono-humanize, (3) renders line 2: `  "first_message..."` (indented, truncated to terminal width with ellipsis), (4) highlights selected session with contrasting background color, (5) dims sessions where project_exists is false, (6) renders status bar at bottom showing key hints: `/ filter  Enter copy  q quit`. Handle terminal resize events.
- [ ] T013 (cc-session-a0o.3) [US1] Implement key event handling for Browse mode in `src/tui/input.rs`. Function `handle_browse_input(app: &mut App, key: KeyEvent) -> Action` that handles: j/Down (move selection down), k/Up (move selection up), Enter (copy selected session's resume_command to clipboard, set status message "Copied!"), q/Esc (quit), `/` (switch to Filtering mode). Implement scroll logic: keep selection visible by adjusting scroll_offset when selection moves beyond visible area. Enum `Action` with variants: Continue, Quit, CopyToClipboard(String).
- [ ] T014 (cc-session-a0o.4) [US1] Wire up TUI mode in `src/main.rs`. When no `-s` or `-g` flags: call `discover_sessions()`, apply `--since` and `--last` filters if provided, create `App`, run TUI event loop. Handle clipboard copy result: on success show "Copied!" in status bar for 2 seconds, on clipboard failure fall back to printing command to stdout after TUI exits.

**Checkpoint**: `cc-session` launches, shows sessions, j/k navigates, Enter copies command to clipboard.

---

## Phase 4: User Story 2 - Filter Sessions by Keyword (Priority: P1) MVP

**Goal**: Press `/` in TUI to enter filter mode with fuzzy matching across project name, git branch, and first message.

**Independent Test**: Launch `cc-session`, press `/`, type a known project name, verify list narrows to matching sessions.

### Implementation for User Story 2

- [ ] T015 (cc-session-80b.1) [US2] Implement fuzzy filter module in `src/filter.rs`. Use nucleo-matcher (low-level API from nucleo 0.5) for one-shot fuzzy matching. Function `fuzzy_filter(sessions: &[Session], query: &str) -> Vec<(usize, u32)>` that: (1) for each session, builds a filter string from `project_name + " " + git_branch + " " + first_message`, (2) scores each against the query using nucleo_matcher Pattern with CaseMatching::Smart, (3) returns Vec of (original_index, score) sorted by score descending, (4) excludes sessions with no match (score = None).
- [ ] T016 (cc-session-80b.2) [US2] Implement filter mode input handling in `src/tui/input.rs`. Function `handle_filter_input(app: &mut App, key: KeyEvent) -> Action` that handles: printable characters (append to filter_query, re-run fuzzy_filter, update filtered_indices, reset selection to 0), Backspace (remove last char from filter_query, re-filter), Esc (clear filter_query, restore full session list, switch to Browse mode), Enter (copy selected filtered session, switch to Browse mode). Render filter input at bottom of screen replacing status bar.
- [ ] T017 (cc-session-80b.3) [US2] Add filter mode rendering in `src/tui/view.rs`. When mode is Filtering: (1) render filter input line at bottom: `/ <query>|` with blinking cursor, (2) render filtered session list (only sessions in filtered_indices), (3) show match count: `N matches` or `No matches` if empty, (4) show empty state "No sessions match your query" when filtered_indices is empty.
- [ ] T018 (cc-session-80b.4) [P] [US2] Write unit tests for fuzzy filtering in `tests/filter_test.rs`. Test: filtering by project name returns matching sessions, filtering by git branch returns matching sessions, filtering by message content works, empty query returns all sessions, non-matching query returns empty vec, scoring ranks exact matches higher than partial matches.

**Checkpoint**: `/` activates filter, typing narrows list in real-time, Esc clears filter, Enter selects from filtered list.

---

## Phase 5: User Story 3 - Scriptable Session Lookup (Priority: P2)

**Goal**: `cc-session -s <query>` prints resume command or shows slim selection menu.

**Independent Test**: Run `cc-session -s <known-project>` and verify correct output behavior for 0, 1, and multiple matches.

### Implementation for User Story 3

- [ ] T019 (cc-session-f6t.1) [US3] Implement scriptable mode in `src/scriptable.rs`. Function `run_scriptable(sessions: Vec<Session>, query: &str) -> Result<(), i32>` that: (1) filters sessions using fuzzy_filter from filter.rs, (2) if 0 matches: print "No sessions found matching \"{query}\"" to stderr, return Err(1), (3) if 1 match: print resume_command() to stdout, return Ok(()), (4) if 2-10 matches: show slim numbered menu on stderr with two-line format per session, read selection from stdin, print selected resume_command to stdout, (5) if >10 matches: show top 10 in menu with "{N} sessions match \"{query}\" (showing top 10):" header, same selection behavior.
- [ ] T020 (cc-session-f6t.2) [US3] Implement slim selection menu rendering in `src/scriptable.rs`. Function `render_selection_menu(sessions: &[(usize, &Session)], query: &str, total_count: usize)` that prints to stderr: header line with match count, numbered two-line entries (same format as TUI), "Select [1-N]:" prompt. Function `read_selection(max: usize) -> Option<usize>` that reads a number from stdin, validates range.
- [ ] T021 (cc-session-f6t.3) [US3] Wire up scriptable mode in `src/main.rs`. When `-s <query>` flag is set: call `discover_sessions()`, apply `--since`/`--last` filters, call `run_scriptable()`, exit with returned code. When `-g` combined with `-s`: use deep search results instead of fuzzy filter.
- [ ] T022 (cc-session-f6t.4) [P] [US3] Write tests for scriptable mode in `tests/scriptable_test.rs`. Test: single match prints command to stdout, zero matches returns exit code 1, multiple matches formatting is correct, selection menu renders correctly for 2-10 matches, top-10 truncation works for >10 matches.

**Checkpoint**: `cc-session -s <query>` works for all match count scenarios with correct stdout/stderr separation.

---

## Phase 6: User Story 4 - Deep Search (Priority: P3)

**Goal**: `-g <pattern>` and Ctrl-G in TUI search through full JSONL conversation content.

**Independent Test**: Run `cc-session -g <phrase>` where phrase exists in conversation content but not first message, verify it finds the session.

### Implementation for User Story 4

- [ ] T023 (cc-session-ot2.1) [US4] Implement parallel deep search in `src/search.rs`. Function `deep_search(claude_home: &Path, pattern: &str) -> Vec<Session>` that: (1) compiles pattern with regex crate, (2) collects all session JSONL file paths from `claude_home/projects/`, (3) uses `rayon::par_iter` to search files in parallel, (4) for each file: read with BufReader, check each line for pattern match (in raw text first for speed, then parse matching sessions for metadata), (5) return deduplicated Vec<Session> sorted by timestamp descending. Optimize: skip files where no line matches (avoid parsing JSONL for non-matching files).
- [ ] T024 (cc-session-ot2.2) [US4] Integrate deep search into TUI via Ctrl-G in `src/tui/input.rs`. When mode is Filtering and Ctrl-G is pressed: (1) switch mode to DeepSearch, (2) take current filter_query as the search pattern, (3) spawn deep_search in a background thread (to avoid blocking TUI), (4) show "Searching..." indicator, (5) when results arrive, replace session list with search results, (6) Esc returns to Browse mode with original session list restored.
- [ ] T025 (cc-session-ot2.3) [US4] Wire up `-g` flag in `src/main.rs`. When `-g <pattern>` without `-s`: run deep_search(), launch TUI with results. When `-g <pattern>` with `-s`: run deep_search(), pass results to `run_scriptable()`.
- [ ] T026 (cc-session-ot2.4) [P] [US4] Write tests for deep search in `tests/search_test.rs`. Test: pattern found in user message content returns session, pattern found in assistant response returns session, pattern not found returns empty vec, regex patterns work, invalid regex returns error, parallel search across multiple fixture files works.

**Checkpoint**: `cc-session -g <pattern>` finds sessions by content. Ctrl-G in TUI triggers deep search.

---

## Phase 7: User Story 5 - Time-Scoped Listing (Priority: P3)

**Goal**: `--since` and `--last` flags filter sessions by time or count.

**Independent Test**: Run `cc-session --since 1d` and verify only today's sessions appear. Run `cc-session --last 5` and verify at most 5 sessions.

### Implementation for User Story 5

- [ ] T027 (cc-session-reu.1) [US5] Implement time/count filtering in `src/discovery.rs`. Function `apply_filters(sessions: Vec<Session>, since: Option<Duration>, last: Option<usize>) -> Vec<Session>` that: (1) if since is Some, filter to sessions with timestamp > (now - duration), (2) if last is Some, truncate to first N sessions (already sorted by timestamp desc), (3) if both provided, apply since first then last (most restrictive wins). Add to discovery pipeline in main.rs.
- [ ] T028 (cc-session-reu.2) [US5] Wire up --since and --last in `src/main.rs`. Parse --since string into chrono::Duration using the duration parser from T008. Apply filters before passing sessions to TUI, scriptable, or deep search modes.

**Checkpoint**: `cc-session --since 7d` and `cc-session --last 50` correctly constrain results.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, error handling, and quality improvements

- [ ] T029 (cc-session-ugu.1) [P] Handle edge case: missing `~/.claude/` directory. In `src/discovery.rs`, check if claude_home exists before scanning. Print "No Claude sessions found. Is Claude Code installed?" to stderr and exit with code 2.
- [ ] T030 (cc-session-ugu.2) [P] Handle edge case: malformed JSONL lines. In `src/session.rs` parsing, skip lines that fail JSON deserialization. Track count of skipped lines. Report count in TUI status bar if >0.
- [ ] T031 (cc-session-ugu.3) [P] Handle edge case: terminal too narrow. In `src/tui/view.rs`, truncate all text with ellipsis when content exceeds terminal width. Ensure two-line display never wraps. Set minimum terminal width (e.g., 40 chars) and show warning if below.
- [ ] T032 (cc-session-ugu.4) [P] Handle edge case: clipboard unavailable. In TUI Enter handler in `src/tui/input.rs`, catch clipboard errors and fall back to printing command to stdout after TUI exits (store command, display "Command will be printed on exit", exit TUI, then print).
- [ ] T033 (cc-session-ugu.5) Implement `CLAUDE_HOME` environment variable support in `src/discovery.rs` for testing. Default to `dirs::home_dir() + "/.claude"` when not set.
- [ ] T034 (cc-session-ugu.6) Run `cargo clippy` and `cargo fmt` to clean up all warnings and formatting. Verify `cargo build --release` produces a single static binary.
- [ ] T035 (cc-session-ugu.7) Run quickstart.md validation: build release binary, run against real `~/.claude/` data, verify all modes work (TUI browse, filter, scriptable -s, deep search -g, --since, --last).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies, can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion. BLOCKS all user stories.
- **US1 Browse (Phase 3)**: Depends on Foundational. No dependency on other stories.
- **US2 Filter (Phase 4)**: Depends on US1 (needs TUI infrastructure from Phase 3).
- **US3 Scriptable (Phase 5)**: Depends on Foundational + filter.rs from US2. Can start after US2 filter module (T015) is complete.
- **US4 Deep Search (Phase 6)**: Depends on Foundational. Can start in parallel with US2/US3 (search.rs is independent).
- **US5 Time Filters (Phase 7)**: Depends on Foundational only. Can start in parallel with any user story.
- **Polish (Phase 8)**: Depends on all desired user stories being complete.

### User Story Dependencies

```
Phase 1 (Setup)
    └── Phase 2 (Foundational)
         ├── Phase 3 (US1: Browse) ── MVP
         │    └── Phase 4 (US2: Filter)
         │         └── Phase 5 (US3: Scriptable) [needs filter.rs]
         ├── Phase 6 (US4: Deep Search) [independent]
         └── Phase 7 (US5: Time Filters) [independent]
              └── Phase 8 (Polish) [after all stories]
```

### Parallel Opportunities

- T005, T006, T007, T008 (Phase 2 core modules, different files)
- T009, T010 (Phase 2 tests, different test files)
- T018 (US2 filter test) can run in parallel with T015-T017
- T022 (US3 tests) can run in parallel with T019-T021
- T026 (US4 tests) can run in parallel with T023-T025
- Phase 6 (US4) and Phase 7 (US5) can run in parallel with Phase 4 (US2)
- T029, T030, T031, T032 (Phase 8 edge cases, different files)

---

## Parallel Example: Foundational Phase

```bash
# Launch all core modules together (different files, no dependencies):
Task: "Implement Session data types in src/session.rs"        # T005
Task: "Implement session discovery in src/discovery.rs"        # T006
Task: "Implement clipboard module in src/clipboard.rs"         # T007
Task: "Implement duration parsing in src/main.rs"              # T008
```

## Parallel Example: Independent User Stories

```bash
# After Foundational + US1 + US2 filter module complete, these can run in parallel:
Task: "Implement scriptable mode in src/scriptable.rs"         # T019 (US3)
Task: "Implement deep search in src/search.rs"                 # T023 (US4)
Task: "Implement time/count filtering in src/discovery.rs"     # T027 (US5)
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T010)
3. Complete Phase 3: US1 Browse (T011-T014)
4. Complete Phase 4: US2 Filter (T015-T018)
5. **STOP and VALIDATE**: TUI with browse + filter is fully functional
6. This is the usable MVP

### Incremental Delivery

1. Setup + Foundational = Project builds, parsing works
2. Add US1 Browse = TUI works, can copy commands (MVP core)
3. Add US2 Filter = Fuzzy search works (MVP complete)
4. Add US3 Scriptable = Power-user CLI mode
5. Add US4 Deep Search = Full content search
6. Add US5 Time Filters = Quality-of-life
7. Polish = Edge cases, cleanup

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- The `StringOrArray` custom deserializer (T005) is critical, test it thoroughly
- Directory path decoding is lossy (hyphens in real paths), always prefer `cwd` from session entry

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
