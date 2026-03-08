# cc-session

Fast CLI tool for finding and resuming Claude Code sessions.

![cc-session demo](docs/demo.gif)

## Why this matters

When working with Claude Code across many projects, you accumulate hundreds of conversation sessions. Each lives as a JSONL file buried under `~/.claude/projects/`, identified by UUID. Resuming the right session requires manually navigating to the correct project directory and guessing the session ID.

cc-session fixes this by scanning all your sessions, presenting them in a searchable list, and generating the exact resume command with one keypress. Built in Rust for instant startup, it handles 2,000+ sessions in under 500ms.

## Features

- **Interactive TUI** with single-line session display (prompt text left, project + time right-aligned)
- **Seamless search**: just start typing to filter across project name, git branch, and message text. No mode switch needed. After a short debounce, a background deep search automatically scans full conversation content too.
- **Conversation viewer** with full session replay, syntax-highlighted code blocks, markdown tables, clickable URLs, and styled headings
- **In-conversation search**: press `/` to search within a conversation, navigate matches with `n`/`N`
- **Theme-aware rendering**: auto-detects dark/light terminal background, with `--dark`/`--light` overrides
- **Time filters** (`--since 7d`, `--last 50`) to scope results
- **Cross-platform clipboard** (macOS, Linux X11/Wayland) with stdout fallback
- **Markup stripping** removes Claude Code internal tags for clean display

## Install

### macOS (Homebrew)

```bash
brew install rhuss/tap/cc-session
```

### Linux / macOS (install script)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/rhuss/cc-session/releases/latest/download/cc-session-installer.sh | sh
```

### Build from source

```bash
cargo install --git https://github.com/rhuss/cc-session
```

Requires a [Rust toolchain](https://rustup.rs/).

## Quick start

```bash
# Browse all sessions interactively
cc-session

# Only show sessions from the last week
cc-session --since 7d

# Limit to the 20 most recent sessions
cc-session --last 20

# Force light theme
cc-session --light
```

## Usage

Run `cc-session` to open the interactive session browser. Sessions are displayed one per line with the prompt text on the left and the project name + relative time on the right (dimmed).

### Seamless search

Just start typing to filter sessions. No mode switch needed. The filter matches case-insensitive substrings across project names, git branches, and prompt text. The full query (including spaces) is matched as a literal substring. The list updates in real-time.

Press Escape once to clear the filter, twice to quit. Press Enter to open the selected session.

After a short debounce (300ms), a background deep search automatically scans full conversation content for your query. Sessions matching inside their conversation are merged into the results.

### Conversation viewer

Press Enter on a session to open the conversation viewer. It shows the full session with all user and assistant messages, visually distinguished by role (cyan for You, yellow for Claude), separated by horizontal lines showing project name, branch, and timestamp. Content is capped at 120 characters wide and centered for comfortable reading.

Features:
- **Syntax highlighting**: Code blocks with language detection via syntect (Rust, Python, TypeScript, and many more)
- **Markdown tables**: Pipe-delimited tables rendered with box-drawing borders and word-wrapped cells
- **Styled headings**: `# headings` with subtle background tints spanning the full width
- **Inline markdown**: `**bold**`, `*italic*`, `` `inline code` `` rendered with proper terminal styling
- **Clickable URLs**: Links rendered with underline and color, auto-clickable in Ghostty/iTerm2
- **Word wrapping**: Text wraps at word boundaries, never mid-word
- **Message merging**: Consecutive messages from the same role are combined into a single entry
- **In-view search**: Press `/` to search within the conversation. Matches are highlighted, current match emphasized. Press `n`/`N` to jump between matches. Match counter shown as `"query" 1/6`.
- **Auto-scroll**: When entering from a search, the viewer auto-scrolls to center the first match on screen
- **Theme-aware**: Colors adapt to dark or light terminal backgrounds

Navigation: `Space`/`PageDown` page down, `b`/`PageUp` page up, `g` top, `G` bottom, arrows for line-by-line scrolling. Press `Enter` to copy the resume command and exit, `Esc` to return to the session list.

### Time filters

```bash
cc-session --since 7d     # last 7 days
cc-session --since 2w     # last 2 weeks
cc-session --since 1m     # last 30 days
cc-session --last 20      # most recent 20 sessions
cc-session --since 7d --last 10  # both constraints
```

## Key bindings

### Session list

| Key | Action |
|-----|--------|
| `Down` / `Up` | Move cursor down / up |
| Type any character | Start filtering (seamless search) |
| `Backspace` | Delete last filter character |
| `Left` / `Right` | Move cursor within filter text |
| `Enter` | Open conversation viewer |
| `Esc` | Clear filter (first), quit (second) |
| `Ctrl-C` | Quit |

### Conversation viewer

| Key | Action |
|-----|--------|
| `Space` / `PageDown` | Scroll down one page |
| `b` / `PageUp` | Scroll up one page |
| `g` | Scroll to top |
| `G` | Scroll to bottom |
| `Down` / `Up` | Scroll down / up one line |
| `/` | Search within conversation |
| `n` / `N` | Jump to next / previous match |
| `Enter` | Copy resume command to clipboard and exit |
| `Esc` | Clear search (first), back to list (second) |

## How it works

1. **Discovery**: Scans `~/.claude/projects/` for session JSONL files using parallel I/O (rayon). Each `.jsonl` file is a session, identified by UUID filename.

2. **Parsing**: Reads the first few lines of each session file to find the first real user message (skipping `file-history-snapshot` entries and internal markup). Extracts project path, git branch, timestamp, and cleaned prompt text.

3. **Display**: Single-line format with the prompt text left-aligned and project + time right-aligned. Seamless search filters in real-time as you type.

4. **Deep search**: After a debounce, searches full conversation content in parallel using rayon. Matches are merged into the filtered results with a pre-built session index for O(1) lookups.

5. **Conversation viewer**: Loads all user and assistant messages from the session JSONL file. Merges consecutive same-role entries. Renders syntax-highlighted code blocks (syntect), markdown tables with box-drawing borders, styled headings, and clickable URLs. Pre-wraps text at word boundaries to 120 characters max. Centers content on wide terminals.

6. **Resume command**: Generates `cd '<project-path>' && claude -r <session-id>` with properly quoted paths. Copied to clipboard via arboard (cross-platform).

## Files

| Path | Purpose |
|------|---------|
| `~/.claude/projects/` | Session data (read-only) |

## License

MIT
