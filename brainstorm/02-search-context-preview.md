# Brainstorm: Search Context Preview

**Date:** 2026-07-03
**Status:** active

## Problem Framing

When content search finds sessions matching a keyword, the session list still displays `first_message` (the first user message in the session) as the title. This means the displayed text often has no visible connection to why the session matched. The user has to open each session to discover the matching content. This is especially disorienting for content-only matches where the keyword doesn't appear anywhere in the visible metadata.

Additionally, the metadata filter currently searches across `project_name`, `git_branch`, and `first_message` combined. Matches on `git_branch` (not shown anywhere in the UI) or `project_name` (redundant with grouped view) create invisible match reasons. The filter should only match what the user can see.

## Approaches Considered

### A: Snippet extraction during search (chosen)

Modify the existing search pass (`file_matches_all` / `deep_search_indexed`) to also extract a context snippet while already reading and parsing the file. Return a new `SearchResult` struct containing both the `Session` and an optional `MatchSnippet`. Rendering replaces `first_message` with the snippet for content matches.

- Pros: No extra file I/O (piggybacks on the existing read pass). Snippets arrive instantly with search results. Clean data flow from search to rendering.
- Cons: Makes the search function slightly heavier. New `SearchResult` type ripples through the channel and App state.

### B: Lazy snippet extraction on render

Keep search returning `Vec<Session>`. Add a per-session snippet cache. Extract snippets on-demand for visible rows only, via background reads triggered during rendering.

- Pros: Zero impact on search speed. Only processes visible rows.
- Cons: Visible flicker as snippets load after results appear. Redundant file I/O (files already read during search). Complex per-row async state.

### C: Pre-indexed content

Extend the startup index to cache full message text for all sessions, enabling instant snippet extraction at query time without file I/O.

- Pros: Fastest snippet extraction.
- Cons: Massive memory increase (full message text for 2000+ sessions). Changes startup/indexing characteristics fundamentally.

## Decision

Approach A: Snippet extraction during search. The search pass already reads, parses, and cleans every matching message. Adding snippet scoring is incremental cost with zero extra I/O.

## Key Requirements

- **Metadata filter simplification**: Remove `project_name` and `git_branch` from the metadata filter. Match only against `first_message`. The filter shows what you see.
- **Content match snippets**: For sessions matching via content search, replace `first_message` in the title column with a context snippet from the matched conversation.
- **Snippet centering**: Center the snippet on the keyword. For multi-keyword searches, find the passage where keywords appear closest together (densest cluster).
- **Best match selection**: When a session has multiple matches, show the snippet from the best match (closest keyword cluster), not the first occurrence.
- **Highlight style**: Keyword text stays in normal color; surrounding context text is dimmed/gray. The keyword stands out by contrast.
- **More matches indicator**: If additional matches exist beyond the shown snippet, append a minimal indicator (e.g., `+`). Not a full count.
- **Data model**: New `SearchResult { session: Session, snippet: Option<MatchSnippet> }` and `MatchSnippet { text: String, keyword_ranges: Vec<(usize, usize)>, has_more: bool }`.
- **Snippet width**: Fills the available title column space, truncated with `...` if needed.

## Open Questions

- What is the optimal context window size around the keyword in chars? Should it adapt to available terminal width or use a fixed budget?
- Should user messages be preferred over assistant messages when scoring "best match", or treat all message roles equally?
- When the snippet is shorter than the available column width, should it be left-aligned or centered in the column?
