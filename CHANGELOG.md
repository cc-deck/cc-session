# Changelog

## v0.7.4

### What's New

- **Repository transfer**: Moved from `rhuss/cc-session` to `cc-deck/cc-session`. All URLs, Homebrew tap references, and release pipeline updated accordingly.

## v0.7.3

### What's New

- **Scrollbar in session list**: Vertical scrollbar appears when the list exceeds the viewport
- **Match counter in border title**: Search match position (e.g. 3/6) shown in conversation border title instead of status bar
- **Skill expansion compression**: Bare `{Skill: name}` invocations (without arguments) compressed to `/name`

### Bug Fixes

- **Fix panic on multi-byte UTF-8 characters**: Deep search type extraction could panic when slicing at byte 500 landed inside a multi-byte character

## v0.7.2

### What's New

- **Conversation viewer border**: Border with "cc-session" title for visual consistency with session list
- **Vertical scrollbar**: Conversation viewer shows a scrollbar tracking scroll position
- **Skill expansion compression**: Expanded slash commands compressed back to their short form (e.g. `/sdd:brainstorm args`)
- **Duplicate message suppression**: Identical consecutive messages from the same role no longer shown twice

## v0.7.1

### Bug Fixes

- **Fix deep search missing results**: Newer Claude Code versions include additional metadata fields before `type`, pushing it past the 200-char search window. Increased to 500 chars with proper key matching.

## v0.7.0

### What's New

- **Interactive-only mode**: Removed non-interactive CLI modes (`-s`, `-q`, `-g`, `--shell-setup`) to focus on the interactive TUI
- **Updated demo**: New recording showcasing syntax highlighting, markdown tables, seamless search, and in-conversation search
- **README refresh**: Documentation updated for the simplified interface

## v0.6.0

### What's New

- **Seamless search**: Just start typing to filter sessions, no mode switch needed
- **Syntax-highlighted code blocks**: Language detection via syntect with dark/light theme support
- **Markdown tables**: Pipe-delimited tables rendered with box-drawing borders and word-wrapped cells
- **Clickable URLs**: Links with underline, auto-clickable in Ghostty/iTerm2
- **Theme-aware rendering**: Auto-detects terminal background with `--dark`/`--light` overrides
- **In-conversation search**: Press `/` to search with match navigation (`n`/`N`)
- **Styled headings**: Subtle background tints spanning full width
- **Session list border**: Border with session count and arrowhead cursor

## v0.5.1

### What's New

- **Clickable URLs**: URLs rendered with underline and link color
- **Heading backgrounds**: Subtle background tint on markdown headings
- **Improved tables**: Word-wrapped cells, row separators, inline markdown formatting
- **Better syntax theme**: Switched to base16-eighties.dark for dark terminals
- **Cleaner code blocks**: Removed language label above code blocks

## v0.5.0

### What's New

- **Syntax highlighting**: Code blocks with language detection via syntect
- **Theme support**: Auto-detect dark/light terminal background via termbg
- **System content stripping**: Remove Claude Code internal tags for clean display
- **Role headers**: Distinct colored headers for user and assistant messages

## v0.4.1

### What's New

- **Version info**: `--version` now shows git hash and build date

## v0.4.0

### What's New

- **Unified incremental search**: Single search that filters metadata instantly and searches conversation content in the background (after 300ms debounce)
- **False positive fix**: Strip system tags before content matching

## v0.3.1

### What's New

- **30x faster deep search**: Pre-built session index with O(1) HashMap lookups instead of re-parsing files

## v0.3.0

### What's New

- **Conversation viewer**: Full session replay with all user and assistant messages
- **Markdown rendering**: Bold, italic, inline code, and headings
- **Word wrapping**: Text wraps at word boundaries, max 120 chars wide, centered
- **In-view search**: Press `/` to search within conversations with highlighting
- **Message merging**: Consecutive same-role messages combined into single entries

## v0.2.0

### What's New

- **Substring filtering**: Replace fuzzy matching with exact substring search
- **Demo recording**: Automated asciinema demo with tmux scripting
- **Shell integration**: `ccs` and `ccf` helper functions

## v0.1.0

### What's New

- **Initial release**: Interactive TUI for browsing Claude Code sessions
- **Session discovery**: Parallel scanning of `~/.claude/projects/` JSONL files
- **Filtering**: Search across project names, git branches, and prompt text
- **Scriptable mode**: Non-interactive selection for shell scripting
- **Clipboard integration**: Copy resume command with one keypress
- **Deep search**: Search full conversation content across all sessions
