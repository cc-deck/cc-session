# Research: Conversation Viewer Enhancements

**Feature**: 006-viewer-enhancements
**Date**: 2026-03-08

## R1: System Tag Content Stripping

**Decision**: Use a string-based state machine that strips known system tags AND their content, replacing the current `strip_tags` which only removes tag delimiters.

**Rationale**: The current `strip_tags` function iterates characters and skips content between `<` and `>` (the tag markers), but preserves the text between opening and closing tags. For system-injected blocks like `<local-command-caveat>Caveat text...</local-command-caveat>`, the "Caveat text..." remains visible. A targeted approach that strips entire blocks for known system tags avoids false content removal (preserving legitimate `<` and `>` in code/text).

**Known system tags** (from live session analysis): `system-reminder`, `local-command-caveat`, `local-command-stdout`, `command-name`, `command-message`, `command-args`, `user-prompt-submit-hook`, `available-deferred-tools`, `rosa-auth`, `rosa-profile`, `sdd-context`, `skill-enforcement`, `fast_mode_info`, `task-notification`, `task-id`, `plugin-name`, `output-file`, `antml_thinking`, `antml_function_calls`.

**Alternatives considered**:
- Regex-based stripping: Works but creates a regex compilation dependency per call. The string `find`/`drain` approach is simpler.
- Strip ALL tags and content (current approach but extended): Would remove legitimate HTML-like content in code examples.

## R2: Syntax Highlighting Library

**Decision**: Use `syntect` (v5.3.0) with `syntect-tui` (v3.0.6) bridge for ratatui integration.

**Rationale**: syntect is the same engine used by `bat` and provides Sublime Text syntax definitions with 150+ languages built-in. The `syntect-tui` crate provides a direct `into_span()` adapter that converts syntect highlight segments to ratatui `Span` objects. This avoids manual color mapping. Binary size impact is ~2-3MB (syntax definitions are bundled as compressed binary dumps). Loading defaults takes ~23ms.

**Code pattern**:
```rust
let ps = SyntaxSet::load_defaults_newlines();
let ts = ThemeSet::load_defaults();
let syntax = ps.find_syntax_by_extension("rs").unwrap();
let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
for line in code {
    let spans: Vec<Span> = h.highlight_line(line, &ps)?
        .into_iter()
        .filter_map(|seg| into_span(seg).ok())
        .collect();
}
```

**Built-in themes**: base16-ocean.dark, base16-ocean.light, Solarized (dark/light), InspiredGitHub, base16-eighties.dark, base16-mocha.dark.

**Alternatives considered**:
- tree-sitter: Requires per-language parser binaries, much heavier dependency. Overkill for read-only display.
- Regex-based keyword highlighting: Fragile, language-specific, poor coverage.

**OpenCode inspiration**: OpenCode uses Shiki (JS equivalent of syntect) with semantic token categories: keyword, string, comment, property, type, constant, operator. We get similar categorization from syntect's scope-based highlighting.

## R3: Terminal Background Detection

**Decision**: Use the `termbg` crate (v0.6.2) for auto-detection with CLI override flags.

**Rationale**: termbg provides a three-tier detection approach: WIN32API (Windows), OSC 11 escape sequence query (Unix), and COLORFGBG environment variable fallback. It supports 19+ terminal emulators and converts detected RGB to a light/dark determination using YCbCr color space (Y > 0.5 = light). The 100ms timeout is appropriate for OSC 11 queries.

**Code pattern**:
```rust
let timeout = Duration::from_millis(100);
let theme = match termbg::theme(timeout) {
    Ok(termbg::Theme::Light) => Theme::light(),
    Ok(termbg::Theme::Dark) | Err(_) => Theme::dark(),
};
```

**Alternatives considered**:
- terminal-light: Simpler but less terminal coverage.
- Manual OSC 11 implementation with crossterm: More control but reinvents what termbg already handles.
- COLORFGBG-only: Many terminals don't set this variable.

## R4: Markdown Table Rendering

**Decision**: Manual table parsing with Unicode box-drawing grid rendering. No external dependency.

**Rationale**: Markdown tables are a simple structure (pipe-separated cells with optional header separator). A manual parser that detects table blocks, computes column widths, and renders with box-drawing characters is straightforward and avoids adding a full markdown parser dependency. Tables that exceed the content width will truncate cell content with ellipsis.

**Box-drawing characters**:
```
┌─┬─┐  Top border
│ │ │  Cell content
├─┼─┤  Header separator
│ │ │  Data rows
└─┴─┘  Bottom border
```

**Parsing approach**: Detect consecutive lines starting with `|`, parse cells by splitting on `|`, identify separator rows (all `-`/`:`), compute max column widths, render with box-drawing.

**OpenCode inspiration**: OpenCode renders tables with CSS borders (header gets stronger bottom border, data cells get subtle bottom border). For our TUI, the box-drawing grid provides equivalent visual structure.

**Alternatives considered**:
- comrak/markdown-rs: Full GFM parser, overkill for just tables.
- Simplified rendering (bold header + aligned spaces): User explicitly chose full grid rendering.

## R5: Theme System Design

**Decision**: Define a `Theme` struct with named color slots, two built-in themes (dark/light), selected at startup.

**Rationale**: A centralized theme struct avoids scattered hardcoded RGB values throughout the rendering code. OpenCode's approach of semantic color variables (`--syntax-keyword`, `--text-strong`, `--border-weak`) is directly applicable. We'll define equivalent Rust struct fields.

**Theme color slots** (inspired by OpenCode):
- Role headers: `user_header_bg`, `user_header_fg`, `assistant_header_bg`, `assistant_header_fg`
- Message bodies: `user_message_bg` (subtle tint), default bg for assistant
- Code: `code_block_bg`, syntax colors via syntect theme selection
- Tables: `table_border`, `table_header_fg`
- General: `text`, `text_dim`, `heading`, `separator`, `search_highlight_bg`

**Dark theme colors** (current palette, refined):
- User header: Cyan bg, dark text
- Assistant header: Yellow bg, dark text
- User message bg: Rgb(25, 30, 40) (slightly lighter than terminal bg)
- Code block bg: Rgb(20, 25, 35)
- Syntect theme: base16-ocean.dark

**Light theme colors**:
- User header: Cyan bg, white text
- Assistant header: Yellow bg, dark text
- User message bg: Rgb(235, 240, 248)
- Code block bg: Rgb(245, 247, 250)
- Syntect theme: InspiredGitHub

## R6: New Dependencies Summary

| Crate | Version | Purpose | Binary Impact |
|-------|---------|---------|--------------|
| syntect | 5.3.0 | Syntax highlighting engine | ~2-3MB |
| syntect-tui | 3.0.6 | syntect-to-ratatui bridge | minimal |
| termbg | 0.6.2 | Terminal background detection | minimal |

Total new binary size impact: ~3MB (primarily syntect's bundled syntax definitions).
