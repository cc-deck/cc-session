# Brainstorm: Unified Incremental Search (Option A)

**Status**: Future enhancement, depends on 004-fast-deep-search
**Captured**: 2026-03-07

## Idea

Replace the current two-mode search (metadata filter via `/` + deep search via `Ctrl-G`) with a single unified search that seamlessly combines both.

## How it works

1. User presses `/` and starts typing
2. **Immediately**: metadata matches appear (project name, branch, first message), same as current filter
3. **After 200-300ms debounce** (no new keystrokes): background deep search kicks off, scanning full JSONL content
4. **As results arrive**: content-only matches merge into the list, marked with a subtle indicator (e.g., dimmed dot)
5. **Status bar shows**: "42 matches (searching content...)" then "67 matches" when complete

## UX flow

```
User types "auth"
  ↓ instant
  [5 metadata matches shown]
  Status: "5 matches"

  ↓ 300ms debounce
  [background content scan starts]
  Status: "5 matches (searching content...)"

  ↓ ~0.3-0.5s later (with 004 optimization)
  [12 more content matches merge in]
  Status: "17 matches"
```

## Key decisions to make

- **Sort order**: How to interleave metadata matches and content-only matches? Options:
  - Metadata matches first, then content matches (grouped)
  - All results sorted by timestamp (mixed)
  - Metadata matches highlighted differently but all sorted by timestamp

- **Visual distinction**: Should content-only matches look different from metadata matches?
  - Option: Dim indicator showing the match is in conversation content, not the session title

- **Debounce timing**: 200ms? 300ms? Configurable?
  - 200ms feels responsive, 300ms avoids unnecessary scans during fast typing

- **Cancel behavior**: If user types more while content search is running:
  - Cancel the current search and restart debounce timer
  - Let it finish and filter results client-side (simpler)

## Prerequisites

- 004-fast-deep-search MUST be completed first (deep search <1s)
- Without fast deep search, the debounced content scan would be too slow for the unified UX

## Performance budget

With 004 optimization:
- Metadata filter: <1ms per keystroke
- Content scan: <0.5s (background, after debounce)
- Total user-perceived latency: <1s for full results

Without 004:
- Content scan for common terms: 16s (unacceptable for debounced UX)

## Complexity estimate

- Remove `Mode::DeepSearchInput` and `Mode::DeepSearching`
- Remove `Ctrl-G` key binding
- Add debounce timer to filter mode
- Add background thread spawning from filter mode
- Add result merging logic
- Moderate complexity, mostly in the TUI event loop

## Notes from brainstorm

- The two-mode UX (filter vs deep search) is more confusing than a single search that "just works"
- Users shouldn't need to know whether their search term is in metadata or content
- The spinner/progress indicator from the current deep search mode can be reused as a subtle status bar indicator
