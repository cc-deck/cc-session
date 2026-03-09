# Changelog

## v0.7.3

### What's New

- **Scrollbar in session list**: Vertical scrollbar (same style as conversation viewer) appears when the list exceeds the viewport
- **Match counter in border title**: Search match position (e.g. 3/6) now shown in the conversation viewer border title instead of the status bar
- **Skill expansion compression**: Bare `{Skill: name}` invocations (without arguments) are now compressed to `/name` in both session list and conversation viewer

### Bug Fixes

- **Fix panic on multi-byte UTF-8 characters**: The deep search type extraction could panic when slicing at byte 500 landed inside a multi-byte character (e.g. `→`). Now uses safe char boundary detection.

## v0.7.2

### What's New

- **Conversation viewer border**: Added border with "cc-session" title around the conversation viewer for visual consistency with the session list
- **Vertical scrollbar**: Conversation viewer now shows a scrollbar on the right edge tracking scroll position
- **Skill expansion compression**: Expanded slash commands (e.g. `/sdd:brainstorm args`) are compressed back to their original short form instead of showing the full skill template
- **Duplicate message suppression**: Identical consecutive messages from the same role are no longer shown twice

## v0.7.1

### Bug Fixes

- **Fix deep search missing results**: Newer Claude Code versions include additional metadata fields before the `type` field, pushing it past the 200-character search window. Increased to 500 characters with proper key matching to avoid false positives from fields like `userType`.

## v0.7.0

### What's New

- **Interactive-only mode**: Removed non-interactive CLI modes (`-s`, `-q`, `-g`, `--shell-setup`) to focus on the interactive TUI experience
- **Updated demo**: New demo recording showcasing syntax highlighting, markdown tables, seamless search, and in-conversation search
- **README refresh**: Documentation updated to reflect the simplified interface

## v0.6.0

### What's New

- **Seamless search**: Just start typing to filter sessions, no mode switch needed
- **Syntax-highlighted code blocks**: Language detection via syntect with dark/light theme support
- **Markdown tables**: Pipe-delimited tables rendered with box-drawing borders and word-wrapped cells
- **Clickable URLs**: Links rendered with underline, auto-clickable in Ghostty/iTerm2
- **Theme-aware rendering**: Auto-detects terminal background with `--dark`/`--light` overrides
- **In-conversation search**: Press `/` to search within conversations with match navigation
- **Styled headings**: Subtle background tints spanning the full width
- **Session list border**: Visual border with session count and arrowhead cursor
