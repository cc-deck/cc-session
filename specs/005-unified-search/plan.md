# Implementation Plan: Unified Incremental Search

**Branch**: `005-unified-search` | **Date**: 2026-03-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/005-unified-search/spec.md`

## Summary

Replace the two-mode search UX (metadata filter via `/` + deep content search via `Ctrl-G`) with a single unified search that combines instant metadata filtering with debounced background content search. The approach extends the existing `Filtering` mode with debounce timing, background thread cancellation, and result merging, while removing the `DeepSearchInput` and `DeepSearching` modes entirely.

## Technical Context

**Language/Version**: Rust stable, 2021 edition, MSRV 1.80.0
**Primary Dependencies**: ratatui 0.30, crossterm 0.29, clap 4.5, regex 1, rayon 1.11, serde/serde_json 1.0, chrono 0.4, dirs 6.0, arboard 3.6
**Storage**: Read-only access to `~/.claude/projects/**/*.jsonl`
**Testing**: cargo test, cargo clippy
**Target Platform**: macOS, Linux (terminal)
**Project Type**: CLI/TUI application
**Performance Goals**: Metadata filter <10ms/keystroke, full results (metadata + content) <1s after typing stops
**Constraints**: No new runtime dependencies required. Debounce uses existing 100ms poll loop.
**Scale/Scope**: 2000-2500 sessions, ~750MB JSONL data

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution is a template with no defined principles. No gates to check. **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/005-unified-search/
├── spec.md
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs              # CLI entry point (no changes)
├── session.rs           # Session data types (no changes)
├── discovery.rs         # Session scanning (no changes)
├── filter.rs            # Metadata filtering (no changes)
├── search.rs            # Deep search (add cancellation flag support)
├── scriptable.rs        # CLI -s mode (no changes)
└── tui/
    ├── mod.rs           # App state, modes, event loop (major changes)
    ├── input.rs         # Key handlers (major changes)
    └── view.rs          # Rendering (moderate changes)

tests/
└── (existing tests, may need updates for removed modes)
```

**Structure Decision**: Single project, flat module layout. No new files needed. All changes are modifications to existing files in `src/tui/` and `src/search.rs`.

## Design Decisions

### D1: Debounce via Poll Loop

Use `Instant`-based tracking in the existing 100ms poll loop rather than a separate timer thread. Track `last_keystroke: Option<Instant>` in App state. On each poll iteration without a key event, check if 300ms has elapsed. Worst case: content search triggers at ~400ms (one poll cycle late), imperceptible to users. See [research.md](research.md#r1-debounce-timer-in-crossterm-event-loop).

### D2: Two-Layer Result Model

Maintain `filtered_indices` for instant metadata matches and a separate `content_results: Vec<ContentMatch>` for content-only matches. Compute a merged display list on demand, sorted by timestamp, with match type tracked per entry. This keeps metadata filtering O(n) per keystroke without touching content results. See [research.md](research.md#r2-result-merging-strategy).

### D3: Selection Stability by File Path

Track the selected session by its `file_path` (PathBuf) before a merge, then find its new index after merge. The file path is already a unique key used in the session index HashMap. See [research.md](research.md#r3-selection-stability-during-merge).

### D4: AtomicBool Cancellation

Share an `Arc<AtomicBool>` between the main thread and the content search thread. The search thread checks the flag periodically (per file scanned) and exits early if set. Main thread sets the flag on new keystrokes. See [research.md](research.md#r4-cancellation-of-in-progress-content-search).

### D5: Mode Consolidation

Remove `Mode::DeepSearchInput` and `Mode::DeepSearching`. Extend `Mode::Filtering` with a `ContentSearchState` enum (Idle, Debouncing, Searching, Complete) to track the content search sub-phase. This reduces modes from 6 to 4. See [research.md](research.md#r6-mode-consolidation).

### D6: Conversation Auto-Scroll

Reuse the existing `initial_search_terms` mechanism in `enter_conversation()` to pass the unified search query to the conversation viewer. Add auto-scroll to the first match occurrence after pre-rendering. See [research.md](research.md#r5-conversation-navigation-from-content-match).

## Implementation Approach

### Phase 1: Core Infrastructure

1. Add new state fields to App struct (`last_keystroke`, `content_search_state`, `content_results`, `cancel_flag`)
2. Add `ContentSearchState`, `ContentMatch`, `MatchType`, and `DisplayEntry` types
3. Remove `Mode::DeepSearchInput` and `Mode::DeepSearching` variants
4. Add cancellation support to `deep_search_indexed()` in search.rs

### Phase 2: Event Loop & Input

5. Extend the poll loop with debounce checking logic
6. Refactor `handle_filter()` to handle debounce, cancellation, and merged results
7. Remove `handle_deep_search_input()` and related Ctrl-G handling
8. Update conversation entry to auto-scroll for content matches

### Phase 3: Rendering

9. Update session list rendering with content-match indicators
10. Update status bar to show search progress phases
11. Remove deep search input UI rendering
12. Update help overlay to remove Ctrl-G reference

### Phase 4: Testing & Polish

13. Update existing tests for removed modes
14. Add tests for debounce, cancellation, and result merging
15. Performance validation against benchmarks

## Complexity Tracking

No constitution violations to justify. Design follows simplicity principles: no new dependencies, no new files, no new architectural patterns.
