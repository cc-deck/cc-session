# cc-session

Fast CLI tool for finding and resuming Claude Code sessions.

<!-- TODO: Replace with actual demo GIF -->
![cc-session demo](docs/demo.gif)

## Why this matters

When working with Claude Code across many projects, you accumulate hundreds of conversation sessions. Each lives as a JSONL file buried under `~/.claude/projects/`, identified by UUID. Resuming the right session requires manually navigating to the correct project directory and guessing the session ID.

cc-session fixes this by scanning all your sessions, presenting them in a searchable list, and generating the exact resume command with one keypress. Built in Rust for instant startup, it handles 2,000+ sessions in under 500ms.

## Features

- **Interactive TUI** with single-line session display (prompt text left, project + time right-aligned)
- **Substring filtering** via `/` key, matching across project name, git branch, and message text
- **Conversation viewer** with full session replay, markdown rendering, and in-view search
- **Deep search** (`-g`) scanning full conversation content across all sessions in parallel (case-insensitive)
- **Scriptable mode** (`-s`) with slim selection menu for shell scripting
- **Quick mode** (`-q`) for non-interactive scripting (prints top match to stdout)
- **Time filters** (`--since 7d`, `--last 50`) to scope results
- **Shell integration** (`--shell-setup --install`) adding `ccs` and `ccf` helper functions
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

# Find sessions matching "auth"
cc-session -s auth

# Deep search conversation content for a specific term
cc-session -g "ConnectionRefused"

# Quick mode: print top match for scripting
cc-session -s myproject -q

# Only show sessions from the last week
cc-session --since 7d
```

### Shell integration

Install helper functions for one-command session resumption:

```bash
cc-session --shell-setup --install
```

This adds two functions to your shell:

```bash
ccs podman    # deep search + cd + resume
ccf myproject # fuzzy search + cd + resume
```

## Usage

### Interactive TUI (default)

Run `cc-session` to open the interactive session browser. Sessions are displayed one per line with the prompt text on the left and the project name + relative time on the right (dimmed).

Press Enter on a session to open the **conversation viewer**.

### Conversation viewer

The conversation viewer shows the full session with all user and assistant messages. Messages are visually distinguished by role (Cyan for You, Yellow for Claude), separated by horizontal lines with timestamps. The content is capped at 120 characters wide and centered for comfortable reading.

Features:
- **Markdown rendering**: `**bold**`, `*italic*`, `` `inline code` ``, and `# headings` are rendered with proper terminal styling
- **Code fence highlighting**: Triple-backtick blocks are shown in muted blue
- **Word wrapping**: Text wraps at word boundaries, never mid-word
- **Message merging**: Consecutive messages from the same role are combined into a single entry
- **In-view search**: Press `/` to search within the conversation. Matches are highlighted with a yellow background. Press `n`/`N` to jump between matches.
- **Auto-scroll**: When entering from a filter or deep search, the viewer auto-scrolls to center the first match on screen

Navigation: `Space`/`PageDown` page down, `b`/`PageUp` page up, `g` top, `G` bottom, `j`/`k` or arrows for line-by-line scrolling. Press `Enter` to copy the resume command and exit, `Esc` or `q` to return to the session list.

### Filter mode

Press `/` to enter filter mode. Type to search across project names, git branches, and prompt text. Each space-separated word must appear as a case-insensitive substring. The list updates in real-time. Press Esc to clear the filter, Enter to open the selected session.

Press `Ctrl-G` while in filter mode to switch to **deep search input**. Edit the query, then press Enter to scan full conversation content (all user and assistant messages). Deep search is case-insensitive and supports regex patterns. Results replace the session list. Press `Esc` to refine the query, `q` to return to the full session list.

### Scriptable mode (`-s`)

```bash
# Single match: prints command directly
cc-session -s unique-project

# Multiple matches: shows numbered menu
cc-session -s auth
#   3 sessions match "auth":
#   1  auth-service · main · 2 hours ago
#      "Add JWT token validation..."
#   2  api-gateway · feat-auth · yesterday
#      "Implement OAuth2 flow..."
#   Select [1-2]:
```

### Deep search (`-g`)

Search through all conversation content (not just the first message):

```bash
cc-session -g "ConnectionRefused"
cc-session -g "impl.*Iterator"    # regex supported
```

Deep search is case-insensitive by default. With 2,000+ sessions and 750 MB of data, searches complete in under 1 second thanks to parallel scanning with rayon and a pre-built session index.

### Quick mode (`-q`)

Non-interactive, prints the top match to stdout. Use with `eval` for one-command resumption:

```bash
eval $(cc-session -s myproject -q)
eval $(cc-session -g podman -q)
```

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
| `j` / `Down` | Move cursor down |
| `k` / `Up` | Move cursor up |
| `/` | Enter filter mode |
| `Enter` | Open conversation viewer |
| `q` / `Esc` | Quit (or return from deep search results) |
| `Ctrl-C` | Quit |

### Filter mode

| Key | Action |
|-----|--------|
| Type | Substring search (space-separated terms) |
| `Backspace` | Delete character |
| `Ctrl-G` | Switch to deep search input |
| `Enter` | Open selected session |
| `Esc` | Cancel filter, return to list |

### Deep search input

| Key | Action |
|-----|--------|
| Type | Edit search query |
| `Enter` | Execute deep search |
| `Esc` | Back to filter (or restore original list) |

### Conversation viewer

| Key | Action |
|-----|--------|
| `Space` / `PageDown` | Scroll down one page |
| `b` / `PageUp` | Scroll up one page |
| `g` | Scroll to top |
| `G` | Scroll to bottom |
| `j` / `Down` | Scroll down one line |
| `k` / `Up` | Scroll up one line |
| `/` | Search within conversation |
| `n` | Jump to next match |
| `N` | Jump to previous match |
| `Enter` | Copy resume command to clipboard and exit |
| `Esc` / `q` | Back to session list |

## How it works

1. **Discovery**: Scans `~/.claude/projects/` for session JSONL files using parallel I/O (rayon). Each `.jsonl` file is a session, identified by UUID filename. Builds a file-path-to-session index for fast deep search lookups.

2. **Parsing**: Reads the first few lines of each session file to find the first real user message (skipping `file-history-snapshot` entries and internal markup). Extracts project path, git branch, timestamp, and cleaned prompt text.

3. **Display**: Single-line format with the prompt text left-aligned and project + time right-aligned. Filtering uses case-insensitive substring matching (all space-separated terms must match).

4. **Conversation viewer**: Loads all user and assistant messages from the session JSONL file. Merges consecutive same-role entries. Renders inline markdown (bold, italic, code, headings). Pre-wraps text at word boundaries to 120 characters max. Centers content on wide terminals.

5. **Resume command**: Generates `cd '<project-path>' && claude -r <session-id>` with properly quoted paths. Copied to clipboard via arboard (cross-platform).

6. **Deep search**: Uses regex + rayon to scan all JSONL content in parallel. Looks up matching files in the pre-built session index (O(1) HashMap lookup) instead of re-parsing each file, achieving sub-second performance even for common terms matching 500+ sessions.

## Files

| Path | Purpose |
|------|---------|
| `~/.claude/projects/` | Session data (read-only) |
| `~/.zshrc` or `~/.bashrc` | Shell functions (if `--shell-setup --install` used) |

## License

MIT
