# Quickstart: Project Grouping View

## What This Feature Does

Adds a toggle-able grouped view to cc-session's session list. Press Ctrl-T to switch between the default flat chronological list and a grouped view where sessions are organized under collapsible project headers.

## Key Files to Modify

| File | What Changes |
|------|-------------|
| `src/tui/mod.rs` | New `DisplayItem` enum, `ProjectGroup` struct, grouping fields on `App`, `rebuild_display_items` logic |
| `src/tui/input.rs` | Ctrl-T handler, Enter on header toggle |
| `src/tui/view.rs` | Header row rendering, session indentation, border title with project count |
| `tests/grouping_test.rs` | New test file for grouping behavior |

## Build & Test

```bash
cargo build          # Build
cargo test           # Run all tests (existing + new grouping tests)
cargo clippy         # Lint
cargo run            # Run locally, press Ctrl-T to test grouping
```

## Implementation Order

1. Data model changes in `mod.rs` (DisplayItem, ProjectGroup, App fields)
2. Update all existing code that references `display_entries` to use `display_items`
3. Add Ctrl-T and Enter-on-header input handling
4. Add header rendering and session indentation in view
5. Write tests
