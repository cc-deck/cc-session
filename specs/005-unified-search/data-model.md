# Data Model: Unified Incremental Search

**Feature**: 005-unified-search
**Date**: 2026-03-07

## Entity Changes

### Modified: App (src/tui/mod.rs)

Existing fields retained. New fields added for unified search state:

| Field | Type | Purpose |
|-------|------|---------|
| `last_keystroke` | `Option<Instant>` | Tracks when the last filter keystroke occurred, for debounce timing |
| `content_search_state` | `ContentSearchState` | Phase of the background content search lifecycle |
| `content_results` | `Vec<ContentMatch>` | Content-only matches from the most recent background search |
| `cancel_flag` | `Arc<AtomicBool>` | Shared flag to cancel in-progress background content search |

### Removed Fields (from App)

| Field | Reason |
|-------|--------|
| `deep_search_query` | Replaced by `filter_query` serving dual purpose |
| `original_sessions` | No longer needed; sessions list is never replaced, only augmented |

### New: ContentSearchState (enum)

Represents the phase of the background content search within filtering mode.

| Variant | Meaning |
|---------|---------|
| `Idle` | No content search running or needed (query empty or too short) |
| `Debouncing` | Keystroke received, waiting for 300ms pause before searching |
| `Searching` | Background thread is scanning content |
| `Complete` | Content search finished, results merged into display |

### New: ContentMatch (struct)

A session that matched the search query in conversation content but not in metadata.

| Field | Type | Purpose |
|-------|------|---------|
| `session` | `Session` | The matched session data |
| `file_path` | `PathBuf` | Unique key for deduplication against metadata matches |

### New: DisplayEntry (struct or enum)

Represents a single entry in the merged results list, tracking its match source.

| Field | Type | Purpose |
|-------|------|---------|
| `session_index` | `Option<usize>` | Index into `sessions` vec (for metadata matches) |
| `content_match_index` | `Option<usize>` | Index into `content_results` vec (for content-only matches) |
| `match_type` | `MatchType` | Whether this entry matched by metadata, content, or both |
| `timestamp` | `DateTime` | For sorting by recency |

### New: MatchType (enum)

| Variant | Meaning |
|---------|---------|
| `Metadata` | Session matched by project name, branch, or first message |
| `Content` | Session matched only by conversation content |
| `Both` | Session matched in both metadata and content (displayed as metadata) |

## Mode Changes

### Removed Modes

| Mode | Reason |
|------|--------|
| `DeepSearchInput` | Consolidated into `Filtering` with debounced content search |
| `DeepSearching` | Consolidated into `Filtering` with `ContentSearchState::Searching` |

### Modified: Mode::Filtering

Now encompasses the full unified search lifecycle. The mode remains `Filtering` throughout metadata filtering, debouncing, content searching, and result display. The `ContentSearchState` field tracks the sub-phase.

## State Transitions

```
Browsing --[press /]--> Filtering (ContentSearchState::Idle)

Filtering:
  keystroke --> apply metadata filter instantly
                set last_keystroke = now
                cancel any running content search
                set ContentSearchState::Debouncing

  poll (300ms elapsed, no keystroke) --> spawn content search thread
                                         set ContentSearchState::Searching

  content search complete --> merge results
                              set ContentSearchState::Complete

  Esc --> cancel content search if running
          restore full session list
          return to Browsing

  Enter --> enter Conversation for selected session
            (scroll to match if content-only match)

Conversation --[Esc/q]--> Filtering (query preserved, results preserved)
```

## Relationship to Existing Entities

- **Session** (src/session.rs): Unchanged. Used as-is in both metadata and content matches.
- **session_index** (`Arc<HashMap<PathBuf, Session>>`): Unchanged. Reused for background content search.
- **filtered_indices** (`Vec<usize>`): Still used for instant metadata filtering. Content results are tracked separately and merged for display.
