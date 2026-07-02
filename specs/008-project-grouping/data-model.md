# Data Model: Project Grouping View

## Entities

### DisplayItem (new)

Wraps both session entries and project headers in a single enum for the display list.

- **Variant Session**: Contains a `DisplayEntry` (existing struct, unchanged)
- **Variant Header**: Contains a `ProjectGroup`

### ProjectGroup (new)

Represents a project header row in the grouped view.

- **project_name** (String): The project name, derived from the session's `project_name` field
- **latest_timestamp** (DateTime\<Utc\>): Timestamp of the most recent session in this group, used for ordering groups and displaying relative time
- **session_count** (usize): Number of sessions in this group (used for border title counts)

### App (modified)

Existing application state struct, extended with grouping fields.

- **grouped_mode** (bool): Whether the grouped view is active. Default: `false`
- **expanded_projects** (HashSet\<String\>): Set of project names whose groups are currently expanded. Empty by default (all collapsed)
- **display_items** (Vec\<DisplayItem\>): Replaces `display_entries: Vec<DisplayEntry>`. Contains interleaved headers and session entries when in grouped mode, or session-only entries when in flat mode

### DisplayEntry (unchanged)

Existing struct. No modifications needed. Contains `match_type`, `source`, `timestamp`.

### Session (unchanged)

Existing struct. The `project_name` field already present is used as the grouping key.

## Relationships

```
App 1──* DisplayItem
DisplayItem::Session 1──1 DisplayEntry
DisplayItem::Header 1──1 ProjectGroup
ProjectGroup *──1 project_name (groups Session entities by project_name)
```

## State Transitions

### View Mode Toggle (Ctrl-T)

```
flat_mode <──Ctrl-T──> grouped_mode
```

On transition to grouped: rebuild display items with headers inserted, all groups collapsed.
On transition to flat: rebuild display items without headers (existing behavior).

### Group Expand/Collapse (Enter on header)

```
collapsed ──Enter──> expanded (project_name added to expanded_projects set)
expanded ──Enter──> collapsed (project_name removed from expanded_projects set)
```

### Filter Interaction

```
no_filter + grouped ──type query──> filter_active + grouped (matching groups auto-expand, empty groups hidden)
filter_active + grouped ──clear filter──> no_filter + grouped (expanded_projects cleared, all groups collapsed)
```
