# Implementation Plan: Conversation Viewer Enhancements

**Branch**: `006-viewer-enhancements` | **Date**: 2026-03-08 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/006-viewer-enhancements/spec.md`

## Summary

Five enhancements to the conversation viewer: (1) strip system-injected content blocks from display, (2) full-width role header bars with user message background tinting, (3) adaptive light/dark theme system with auto-detection, (4) markdown table rendering with Unicode box-drawing grid, (5) syntax highlighting for code blocks via syntect.

## Technical Context

**Language/Version**: Rust stable, 2021 edition, MSRV 1.80.0
**Primary Dependencies**: ratatui 0.30, crossterm 0.29, clap 4.5, syntect 5.3 (new), syntect-tui 3.0 (new), termbg 0.6 (new)
**Storage**: Read-only access to `~/.claude/projects/**/*.jsonl`
**Testing**: cargo test, cargo clippy
**Target Platform**: macOS, Linux (terminal)
**Project Type**: CLI/TUI application
**Performance Goals**: Viewer renders within 500ms for 100+ message conversations
**Constraints**: Binary size increase ~3MB (syntect syntax definitions). Theme detection within 100ms.

## Constitution Check

Constitution is a template with no defined principles. **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/006-viewer-enhancements/
├── brainstorm.md
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code

```text
src/
├── main.rs              # Add --light/--dark CLI flags
├── session.rs           # Add strip_system_blocks(), update message processing
├── discovery.rs         # Use strip_system_blocks in load_conversation
├── theme.rs             # NEW: Theme struct, dark/light definitions, detection
├── tui/
│   ├── mod.rs           # Add theme field to App, pass to rendering
│   ├── view.rs          # Role headers, message backgrounds, theme-aware colors
│   ├── table.rs         # NEW: Table parsing and box-drawing grid rendering
│   ├── syntax.rs        # NEW: Syntect integration for code highlighting
│   └── input.rs         # No changes
```

**Structure Decision**: Three new files (`theme.rs`, `tui/table.rs`, `tui/syntax.rs`) to keep concerns separated. Theme is at the crate root since it's used by both TUI and potentially future output modes.

## Design Decisions

### D1: Two-Phase Tag Stripping

Replace the single `strip_tags` function with a two-phase approach:
1. `strip_system_blocks(text)`: Strips entire content blocks for known system tags (tag + content between open/close). Uses string `find()` in a loop, no regex.
2. Keep existing `strip_tags` for remaining unknown tags (strips delimiters only).

The `load_conversation` pipeline becomes: raw text -> `strip_system_blocks` -> `strip_tags` -> `clean_message_multiline`.

### D2: Theme as Startup Configuration

Theme is determined once at startup (auto-detect or CLI override) and stored in `App.theme`. All rendering functions receive the theme by reference. No runtime theme switching (would require re-rendering all cached lines).

Detection order: CLI flag (`--light`/`--dark`) > `termbg::theme(100ms)` > default dark.

### D3: Syntect Lazy Initialization

`SyntaxSet` and `ThemeSet` are loaded once (lazy or at startup) and stored in a new `SyntaxHighlighter` struct. The highlighter is passed to `pre_render_conversation` and used only when a code fence has a language tag. For unrecognized languages or missing tags, fall back to the current single-color style.

### D4: Table Block Detection in Pre-render

During `pre_render_conversation`, consecutive lines starting with `|` are collected into a table block. When the block ends (non-pipe line or end of message), the collected lines are parsed as a table and rendered with box-drawing characters. If parsing fails (malformed table), the lines are rendered as plain text.

### D5: Role Header Bar Rendering

Each message block starts with a full-width `Line` where:
- The role label ("▶ You" or "◀ Claude") is left-aligned with bold colored text on a colored background
- The timestamp is right-aligned within the same line
- The background spans the full content width via ratatui's `Line` background style

User message body lines get a subtle background tint via `Line::style()`. Assistant messages keep the default terminal background.

### D6: OpenCode-Inspired Visual Design

Inspired by OpenCode's UI:
- Semantic syntax color categories (keyword, string, comment, property, type) via syntect themes
- Clear visual separation between roles through background differentiation
- Code blocks with distinct background color (slightly different from message bg)
- Table header with distinct styling (bold text, stronger border)

## Implementation Approach

### Phase 1: System Content Stripping (US1)
- Add `strip_system_blocks` to session.rs
- Update `load_conversation` in discovery.rs
- Handle pure-system messages (skip when empty after stripping)

### Phase 2: Theme System (US3, prerequisite for US2/US5)
- Create `src/theme.rs` with Theme struct and dark/light definitions
- Add `termbg` detection with CLI override
- Add `--light`/`--dark` flags to clap
- Wire theme into App struct

### Phase 3: Role Headers and Message Backgrounds (US2)
- Refactor role header rendering with full-width bars
- Integrate timestamp into header bar
- Add user message background tinting
- Use theme colors throughout

### Phase 4: Table Rendering (US4)
- Create `src/tui/table.rs` with parsing and rendering
- Detect table blocks in pre_render_conversation
- Render with box-drawing characters and auto-sized columns

### Phase 5: Syntax Highlighting (US5)
- Create `src/tui/syntax.rs` with syntect integration
- Add syntect + syntect-tui dependencies
- Integrate into code fence rendering in pre_render_conversation
- Select syntect theme based on active theme (dark/light)

## Complexity Tracking

No constitution violations to justify.
