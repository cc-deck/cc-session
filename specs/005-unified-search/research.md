# Research: Unified Incremental Search

**Feature**: 005-unified-search
**Date**: 2026-03-07

## R1: Debounce Timer in crossterm Event Loop

**Decision**: Use `Instant`-based debounce tracking within the existing 100ms poll loop.

**Rationale**: The current event loop polls every 100ms (`event::poll(Duration::from_millis(100))`). Rather than adding a dedicated timer thread, track `last_keystroke: Option<Instant>` in the App state. On each poll iteration (when no key event arrives), check if 300ms has elapsed since the last keystroke. If so, trigger the content search. This approach requires zero new dependencies and integrates naturally with the existing loop structure.

**Alternatives considered**:
- Dedicated timer thread with mpsc: Adds complexity, unnecessary given the 100ms poll granularity (worst case: content search triggers at 400ms instead of 300ms, imperceptible to users).
- tokio/async runtime: Massive dependency addition for a simple timer. Rejected.

## R2: Result Merging Strategy

**Decision**: Maintain two separate result sets (metadata-filtered indices and content-search results) and compute a merged view on demand.

**Rationale**: The current architecture uses `filtered_indices: Vec<usize>` for metadata matches (indices into `sessions`). Content search returns `Vec<Session>` from background thread. To merge:
1. Content results arrive as `Vec<Session>`, each with a `file_path`.
2. Identify which content matches are NOT already in the metadata-filtered set (by comparing file paths).
3. Build a merged display list that interleaves by timestamp, tracking match type per entry.

This avoids mutating the original `sessions` vector and keeps metadata filtering instant.

**Alternatives considered**:
- Replacing `sessions` entirely with merged results: Breaks the instant metadata filter (would need to re-scan on every keystroke).
- Appending content results to `sessions`: Mutates shared state, complicates cleanup when query changes.

## R3: Selection Stability During Merge

**Decision**: Track the selected session by its file path, not by index position.

**Rationale**: When content results merge in and the display list changes, the index of the currently-selected session may shift. By storing the file path of the selected session before merge and finding its new index after merge, selection follows the item. The `Session.file_path` field (PathBuf) provides a unique, stable identifier.

**Alternatives considered**:
- Storing session ID: The file path already serves as a unique key (used in the session index HashMap).
- Clamping to nearest index: Would cause the user to land on a different session.

## R4: Cancellation of In-Progress Content Search

**Decision**: Use an `Arc<AtomicBool>` cancellation flag shared between the main thread and the search thread.

**Rationale**: The current deep search spawns a `std::thread` with an `Arc` clone of the session index. Adding an `Arc<AtomicBool>` that the search thread checks periodically (e.g., per file) allows the main thread to signal cancellation on new keystrokes. The search thread checks the flag and exits early. This is lightweight and requires no new dependencies.

**Alternatives considered**:
- Dropping the mpsc receiver: Does not actually stop the thread, just ignores results. Wastes CPU.
- Thread kill/abort: Not safely possible in Rust without `unsafe`.

## R5: Conversation Navigation from Content Match

**Decision**: Reuse the existing conversation search infrastructure to scroll to the first match.

**Rationale**: The conversation viewer already supports `ConversationSearch` mode with search highlighting (`Rgb(100, 80, 0)` background) and `n`/`N` navigation. When opening a conversation from a content-only match, set `initial_search_terms` from the unified search query (this path already exists for deep search results at `src/tui/mod.rs:160-189`). Then auto-scroll to the first match occurrence in the pre-rendered lines.

The existing `enter_conversation()` already preserves `deep_search_query` as `initial_search_terms`. This behavior needs to be extended to the unified search query.

**Alternatives considered**:
- Building a separate scroll-to-match system: Duplicates existing functionality.
- Not scrolling: Poor UX for content-only matches where the match may be deep in conversation.

## R6: Mode Consolidation

**Decision**: Remove `Mode::DeepSearchInput` and `Mode::DeepSearching`. Extend `Mode::Filtering` to handle the unified search lifecycle.

**Rationale**: The current 6 modes become 4:
- `Browsing` (unchanged)
- `Filtering` (extended: now includes debounced content search, progress indicator, merged results)
- `Conversation` (unchanged)
- `ConversationSearch` (unchanged)

The `Filtering` mode gains new state fields for tracking content search phase (idle, searching, complete). This simplifies the mode transitions and removes the `Ctrl-G` entry point.

**Alternatives considered**:
- Adding a new `UnifiedSearch` mode: Increases mode count, duplicates filtering logic.
- Keeping DeepSearching as a sub-state: Confusing, since the user sees no mode change.

## R7: Visual Distinction for Content-Only Matches

**Decision**: Use a small icon/prefix character in the session list entry to indicate content-only matches.

**Rationale**: Content-only matches should be subtly distinguished without being distracting. A dimmed dot or magnifying glass icon before the session entry communicates "matched in content" vs metadata matches which have no indicator. This aligns with the brainstorm document's suggestion of a "dimmed dot."

**Alternatives considered**:
- Different background color for content rows: Too visually heavy, distracting during scanning.
- Separate sections (metadata first, then content): Breaks timestamp ordering decided in clarifications.
