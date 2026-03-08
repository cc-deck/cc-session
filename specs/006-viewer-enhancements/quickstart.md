# Quickstart: Conversation Viewer Enhancements

**Feature**: 006-viewer-enhancements

## What Changed

The conversation viewer gets five major improvements: clean content (system noise removed), visual role separation (full-width header bars), adaptive light/dark themes, markdown table grid rendering, and syntax-highlighted code blocks.

## New Dependencies

Add to `Cargo.toml`:
```toml
syntect = "5.3"
syntect-tui = "3.0"
termbg = "0.6"
```

## New CLI Flags

```
--light    Force light color theme
--dark     Force dark color theme
```

## Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` | Add syntect, syntect-tui, termbg dependencies |
| `src/session.rs` | Add `strip_system_blocks()` function, update message processing |
| `src/discovery.rs` | Use `strip_system_blocks` in `load_conversation` |
| `src/tui/mod.rs` | Add Theme struct and field to App, detect theme at startup |
| `src/tui/view.rs` | Major changes: role headers, table rendering, syntax highlighting, theme-aware colors |
| `src/main.rs` | Add `--light`/`--dark` CLI flags |

## New Files

| File | Purpose |
|------|---------|
| `src/theme.rs` | Theme struct, dark/light theme definitions, detection logic |
| `src/tui/table.rs` | Markdown table parsing and box-drawing grid rendering |
| `src/tui/syntax.rs` | Syntect integration for code block highlighting |

## Build & Test

```bash
cargo build
cargo test
cargo clippy
```

## Testing Checklist

- [ ] System caveat text no longer visible in conversations
- [ ] Tool output (directory listings, file contents) stripped from user messages
- [ ] Pure-system messages skipped entirely (no empty blocks)
- [ ] Full-width role header bars with colored backgrounds
- [ ] Timestamp integrated into role header bar
- [ ] User message body has subtle background tint
- [ ] `--light` flag produces readable output on light terminal
- [ ] `--dark` flag forces dark theme
- [ ] Auto-detection selects correct theme (test on both light/dark terminals)
- [ ] Markdown tables render with box-drawing grid
- [ ] Table columns auto-sized
- [ ] Table headers rendered bold
- [ ] Code blocks with language tag show syntax highlighting
- [ ] Code blocks without language tag use single-color fallback
- [ ] Code blocks have distinct background color
- [ ] Performance: viewer renders within 500ms for conversations with 100+ messages
