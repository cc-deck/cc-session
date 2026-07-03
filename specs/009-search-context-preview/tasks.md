# Tasks: Search Context Preview

**Input**: Design documents from `specs/009-search-context-preview/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new data types needed across the feature

- [x] T001 Add `MatchSnippet` struct (text, keyword_ranges, has_more) and `SearchResult` struct (session, snippet) in `src/search.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core search infrastructure changes that all user stories depend on

**Note**: No user story work can begin until this phase is complete

- [x] T002 Redesign `file_matches_all` in `src/search.rs` to return `Option<MatchSnippet>` instead of `bool`. Add snippet extraction with cluster scoring: for each cleaned message where all regexes match, record keyword match positions using `Regex::find_iter`, compute the minimum character span containing at least one match of every keyword. Track the best (smallest span) cluster across messages. Tie-breaking: prefer user messages over assistant messages at equal span, then prefer earlier conversation position. Extract a 300-char window centered on the best cluster using `is_char_boundary`-safe slicing. Set `has_more = true` when total messages with matches > 1. Include a unit test that creates a temporary JSONL file with messages containing keywords at different densities and asserts the returned snippet is centered on the densest cluster.
- [x] T003 Update `deep_search_indexed` in `src/search.rs` to return `Vec<SearchResult>`. In the `par_iter` closure: call `file_matches_all` (now returns `Option<MatchSnippet>`), then look up the `Session` from `session_index` (fast path) or call `search_file_with_metadata` (fallback, returns `Option<Session>`, no signature change needed since the snippet already came from `file_matches_all`). Pair them into `SearchResult { session, snippet: Some(snippet) }`. Update the sort and return type.
- [x] T004 Update `App::content_results` type from `Vec<Session>` to `Vec<SearchResult>` in `src/tui/mod.rs`. Update `search_receiver` channel type to `mpsc::Receiver<Vec<SearchResult>>`. Update `display_session_from_entry` to access `result.session` for Content-sourced entries.
- [x] T005 Add a method `snippet_for_entry(&self, entry: &DisplayEntry) -> Option<&MatchSnippet>` to `App` in `src/tui/mod.rs` that returns the snippet for a content-matched display entry (by looking up the content_results index from DisplaySource::Content).

**Checkpoint**: Search returns snippets alongside sessions, TUI state can access them

**Interfaces**:
- `MatchSnippet { text: String, keyword_ranges: Vec<(usize, usize)>, has_more: bool }` (from T001)
- `SearchResult { session: Session, snippet: Option<MatchSnippet> }` (from T001)
- `file_matches_all(path: &Path, regexes: &[Regex]) -> Option<MatchSnippet>` (from T002)
- `deep_search_indexed(...) -> Vec<SearchResult>` (from T003)
- `App::content_results: Vec<SearchResult>` (from T004)
- `App::snippet_for_entry(&self, entry: &DisplayEntry) -> Option<&MatchSnippet>` (from T005)

---

## Phase 3: User Story 1 - Content search shows matching context (Priority: P1) MVP

**Goal**: When content search finds matches, the session row shows a snippet centered on the keyword instead of first_message, with dimmed surrounding context.

**Independent Test**: Search for a term that appears deep in a session but not in its first message. The session row should show the matching snippet with the keyword in normal color and context dimmed.

- [x] T006 [US1] In `render_session_list` in `src/tui/view.rs`, when rendering a DisplayItem::Session with MatchType::Content or MatchType::Both, check for a snippet via `app.snippet_for_entry`. If present, replace `session.first_message` with the snippet text truncated to `max_msg_len`.
- [x] T007 [US1] Add snippet-aware rendering in `src/tui/view.rs`: split the snippet text into keyword spans and context spans using `keyword_ranges`. Render keyword spans with the normal `msg_style` and context spans with `app.theme.text_dim`. Append `+` indicator (in dim style) when `has_more` is true. Include a unit test confirming `has_more` renders as `+` and absent `has_more` renders no indicator.

---

## Phase 4: User Story 3 - Metadata filter matches only visible text (Priority: P2)

**Goal**: The metadata filter matches only against `first_message`, not `project_name` or `git_branch`.

**Independent Test**: Type a project name in the filter. Sessions from that project should not appear unless the project name is also in their first_message.

- [x] T008 [P] [US3] Modify `filter_sessions` in `src/filter.rs` to match only against `session.first_message` instead of `"{project_name} {git_branch} {first_message}"`. Remove the `branch` variable and `format!` concatenation. Update existing tests to reflect the new matching behavior: sessions should not match on project_name or git_branch. Add a test case confirming project_name-only matches no longer return results.

---

## Phase 5: Polish & Cross-Cutting

**Purpose**: UTF-8 safety, edge cases, final validation

- [x] T009 Add UTF-8 boundary safety tests in `src/search.rs`: create a JSONL file with multi-byte UTF-8 content (emoji, CJK characters) near snippet boundaries and verify no panics during snippet extraction. Also add a test confirming `has_more` is `true` when keywords match in multiple messages and `false` when matched in only one.
- [x] T010 Run `cargo clippy` and `cargo test` to verify all changes compile cleanly and pass. Run a manual performance check: time a content search across the full session corpus (~2170 sessions) to confirm it completes under 1 second (SC-003).

## Dependencies

```text
T001 → T002 → T003 → T004 → T005 (sequential: type changes cascade)
T005 → T006 → T007 (rendering depends on snippet access)
T008 is independent (can run in parallel with US1 work, touches filter.rs only)
T009, T010 run after all other tasks
```

## Parallel Execution Opportunities

- **T008** (US3: filter simplification) can run in parallel with **T006 + T007** (US1: snippet rendering) since they touch different files (`filter.rs` vs `view.rs`)

## Implementation Strategy

**MVP**: Phase 1-3 (T001-T007). Content search shows snippets with cluster scoring, dimmed context, and `+` indicator. This delivers the core value including multi-keyword support.

**Increment 1**: Phase 4 (T008). Metadata filter simplification. Can be done independently.

**Increment 2**: Phase 5 (T009-T010). Polish, UTF-8 edge cases, performance validation.
