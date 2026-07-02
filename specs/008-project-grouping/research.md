# Research: Project Grouping View

## R1: DisplayEntry Extension Strategy

**Decision**: Add a `ProjectHeader` variant to a new `DisplayItem` enum that wraps both project headers and session entries. The existing `DisplayEntry` struct stays unchanged for session entries.

**Rationale**: The current `DisplayEntry` is a struct with `match_type`, `source`, and `timestamp`. Rather than converting it to an enum (which would break all existing code that accesses fields directly), we introduce a `DisplayItem` enum with `Session(DisplayEntry)` and `Header(ProjectGroup)` variants. The `display_entries` field on `App` changes from `Vec<DisplayEntry>` to `Vec<DisplayItem>`.

**Alternatives considered**:
- Converting `DisplayEntry` to an enum: Requires rewriting every field access across view.rs, input.rs, mod.rs. Too invasive.
- Adding an `Option<ProjectGroup>` field to `DisplayEntry`: Semantically muddled, wastes space on every session entry.

## R2: Collapse State Management

**Decision**: Track expanded projects in a `HashSet<String>` of project names on `App`. A project is expanded if its name is in the set; collapsed otherwise.

**Rationale**: Simple, O(1) lookup. When filter is active, the auto-expand logic bypasses the set entirely (all matching groups are shown expanded). When filter clears, the set is cleared too (all groups return to collapsed per FR-008).

**Alternatives considered**:
- Boolean on each project header entry: Requires mutating display entries on toggle, which conflicts with the rebuild pattern.
- Separate `HashMap<String, bool>`: Unnecessary, a HashSet is sufficient since collapsed is the default.

## R3: Grouping Algorithm in rebuild_display_entries

**Decision**: When grouped mode is active, `rebuild_display_entries` first collects all visible sessions (from metadata filter + content results, same as today), groups them by `project_name`, sorts groups by most recent session timestamp, then interleaves `Header` and `Session` items. Collapsed groups emit only their header; expanded groups emit header + sessions.

**Rationale**: Keeps all grouping logic in one place. The existing filter and search code remains untouched. The grouping is purely a display-layer concern.

## R4: Mouse Support Feasibility

**Decision**: Defer mouse click support (FR-005 is SHOULD, not MUST). Crossterm supports mouse events but the codebase currently only handles `Event::Key`. Adding mouse support requires enabling mouse capture (`EnableMouseCapture`/`DisableMouseCapture`), handling `Event::Mouse` in the event loop, and mapping click coordinates to display rows. This is doable but adds complexity to the event loop.

**Rationale**: The spec marks FR-005 as SHOULD/deferrable. Keyboard navigation (Enter to toggle) covers the core use case. Mouse support can be added in a follow-up.

## R5: Ctrl-T Keybinding

**Decision**: Handle `KeyCode::Char('t')` with `KeyModifiers::CONTROL` in `handle_browsing`. This toggles a `grouped_mode: bool` field on `App` and triggers `rebuild_display_entries`.

**Rationale**: Ctrl-T is not used by any existing keybinding. The browsing input handler already checks for Ctrl-C, so the pattern is established.
