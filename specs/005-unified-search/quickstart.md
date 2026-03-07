# Quickstart: Unified Incremental Search

**Feature**: 005-unified-search

## What Changed

The two-mode search (metadata filter via `/` + deep content search via `Ctrl-G`) is replaced by a single unified search. Press `/`, type your query, and results from both metadata and conversation content appear automatically.

## Key Interactions

1. **Press `/`** to start searching
2. **Type your query** - metadata matches appear instantly
3. **Pause typing** - after 300ms, content search runs in background
4. **Watch the status bar** - shows match count and search progress
5. **Content-only matches** appear with a visual indicator
6. **Press Enter** on any result to open the conversation
   - Content-only matches auto-scroll to the first occurrence

## Files to Modify

| File | Change |
|------|--------|
| `src/tui/mod.rs` | Add unified search state fields, remove DeepSearchInput/DeepSearching modes, add debounce logic to poll loop |
| `src/tui/input.rs` | Remove `handle_deep_search_input`, extend `handle_filter` with debounce and cancellation, update conversation entry for auto-scroll |
| `src/tui/view.rs` | Add content-match indicator rendering, update status bar for search progress, remove deep search input UI |
| `src/search.rs` | Add cancellation flag support to `deep_search_indexed()` |
| `src/filter.rs` | No changes needed (metadata filtering stays as-is) |

## Build & Test

```bash
cargo build
cargo test
cargo clippy
```

## Testing Checklist

- [ ] `/` starts unified search, metadata matches appear per-keystroke
- [ ] Content search triggers after 300ms pause
- [ ] Status bar shows "N matches (searching content...)" then final count
- [ ] Typing during content search cancels and restarts
- [ ] Content-only matches have visual indicator
- [ ] Enter on content-only match scrolls to first occurrence
- [ ] Esc restores full session list
- [ ] `Ctrl-G` no longer does anything (removed)
- [ ] Performance: metadata <10ms, full results <1s
