# Data Model: Search Context Preview

## New Entities

### MatchSnippet

Represents a context snippet extracted from a content search match.

| Field | Type | Description |
|-------|------|-------------|
| text | String | The raw snippet text (up to 300 chars), centered on the best keyword cluster |
| keyword_ranges | Vec<(usize, usize)> | Character offset ranges of each keyword occurrence within `text` (start, end) |
| has_more | bool | True if additional matches exist in the session beyond this snippet |

**Invariants**:
- `text` is cleaned (system blocks and tags stripped, skill expansion compressed)
- All `keyword_ranges` entries fall within `0..text.len()` (char boundaries)
- `keyword_ranges` is sorted by start offset
- `text` does not contain newlines (collapsed to single-line for list display)

### SearchResult

Pairs a Session with its optional snippet from content search.

| Field | Type | Description |
|-------|------|-------------|
| session | Session | The matched session metadata |
| snippet | Option<MatchSnippet> | Present when the session was found via content search |

**Lifecycle**: Created by `deep_search_indexed`, sent through the `mpsc` channel, stored in `App::content_results`, consumed by `view::render_session_list`.

## Modified Entities

### filter_sessions (filter.rs)

**Current**: Matches against `"{project_name} {git_branch} {first_message}"`.

**Modified**: Matches only against `first_message`.

### file_matches_all (search.rs)

**Current signature**: `fn file_matches_all(path: &Path, regexes: &[Regex]) -> bool`

**New signature**: `fn file_matches_all(path: &Path, regexes: &[Regex]) -> Option<MatchSnippet>`

Returns `None` when no match found, `Some(snippet)` when matched.

### deep_search_indexed (search.rs)

**Current return type**: `Vec<Session>`

**New return type**: `Vec<SearchResult>`

### App::content_results (tui/mod.rs)

**Current type**: `Vec<Session>`

**New type**: `Vec<SearchResult>`

### App::search_receiver (tui/mod.rs)

**Current type**: `Option<mpsc::Receiver<Vec<Session>>>`

**New type**: `Option<mpsc::Receiver<Vec<SearchResult>>>`

## Data Flow

```
User types → filter_query updated → filter_sessions(sessions, query) [first_message only]
                                  → 300ms debounce → deep_search_indexed()
                                                          ↓
                                              file_matches_all() returns Option<MatchSnippet>
                                                          ↓
                                              SearchResult { session, snippet }
                                                          ↓
                                              mpsc::channel → App::content_results
                                                          ↓
                                              rebuild_display_items() merges results
                                                          ↓
                                              render_session_list() checks for snippet
                                                          ↓
                                              Content match? → show snippet (dimmed context, normal keyword)
                                              Metadata only? → show first_message (existing behavior)
```
