# Brainstorm: Project Grouping View

**Date:** 2026-07-02
**Status:** active
**Issue:** https://github.com/cc-deck/cc-session/issues/5

## Problem Framing

With 850+ sessions, the flat chronological list makes it hard to find sessions for a specific project. PR #2 implemented project grouping but bundled it with archiving, titles, forking, and move-to-project. We want to extract just the grouping concept, redesigned to fit the existing minimalist UI: off by default, toggle-able, and non-disruptive to the current search/filter workflow.

## Approaches Considered

### A: Virtual row model (chosen)

Insert `ProjectHeader` entries into the existing flat `display_entries` Vec. Headers are first-class display entries alongside session entries. Grouping is a view-layer concern; filter/search logic stays untouched. Navigation, scrolling, and click targeting work naturally because header rows have concrete positions in the list.

- Pros: Minimal change to existing architecture. Filter/search code unchanged. Easy to toggle since it just rebuilds display entries.
- Cons: Arrow key navigation must skip/stop at headers. Scroll offset calculations need to account for header rows.

### B: Separate view state

Two independent display entry lists (flat and grouped), toggled between. Each with its own rendering and navigation logic.

- Pros: Clean separation, each mode is self-contained.
- Cons: Duplicates rendering and navigation logic. Filter changes need to update both models. Code drift risk.

### C: Render-time grouping only

No data model changes. Detect project boundaries during rendering and insert header lines on the fly. Collapse state in a separate `HashSet<String>`.

- Pros: Zero data model changes. Purely cosmetic.
- Cons: Cursor position doesn't map 1:1 to display rows. Mouse click targeting and scroll calculations become complex.

## Decision

Approach A: Virtual row model. Keeps the existing architecture intact while making headers first-class entries.

## Key Requirements

- **Toggle**: Ctrl-T switches between flat (default) and grouped view
- **Header style**: Chevron + project name + relative time of most recent session
  - `▶ cc-session  2 hours ago` (collapsed)
  - `▼ cc-session  2 hours ago` (expanded, sessions indented below)
- **Default state**: All groups collapsed when no filter/search is active (compact project overview)
- **Filter/search active**: Matching groups auto-expand, empty groups hidden
- **Expand/collapse**: Enter or mouse click on a project header toggles. Enter on a session row opens the conversation (unchanged)
- **Content search**: Works identically in both modes. Results appear under their project groups, auto-expanding those groups
- **Ordering**: Projects ordered by most recent session timestamp. Sessions within a project ordered newest-first
- **Data model**: `DisplayEntry` gets a `ProjectHeader` variant. `rebuild_display_entries` handles insertion/grouping

## Open Questions

- Should the grouped/flat preference persist across restarts (e.g., in a config file), or always start flat?
- Should Left/Right arrow keys also collapse/expand groups (in addition to Enter), or keep them free for future use?
- How should the session count be shown in the border title? Currently shows `(filtered/total)`. In grouped mode, should it also show project count?
