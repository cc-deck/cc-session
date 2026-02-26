# cc-session

Fast CLI tool for finding and resuming Claude Code sessions.

<!-- TODO: Replace with actual demo GIF -->
![cc-session demo](docs/demo.gif)

## Why this matters

When working with Claude Code across many projects, you accumulate hundreds of conversation sessions. Each lives as a JSONL file buried under `~/.claude/projects/`, identified by UUID. Resuming the right session requires manually navigating to the correct project directory and guessing the session ID.

cc-session fixes this by scanning all your sessions, presenting them in a searchable list, and generating the exact resume command with one keypress. Built in Rust for instant startup, it handles 2,000+ sessions in under 500ms.

## Features

- **Interactive TUI** with single-line session display (prompt text left, project + time right-aligned)
- **Fuzzy filtering** via `/` key, matching across project name, git branch, and message text
- **Detail view** showing the last 20 user prompts for session confirmation before resuming
- **Scriptable mode** (`-s`) with slim selection menu for shell scripting
- **Deep search** (`-g`) scanning full conversation content across all sessions in parallel
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

# Find sessions matching "auth" with fuzzy search
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

Press Enter on a session to open the **detail view**, showing the last 20 user prompts in a bordered box. From there, press Enter to copy the resume command to clipboard and exit, or Tab to switch to the "Back" button.

### Filter mode

Press `/` to enter filter mode. Type to fuzzy-search across project names, git branches, and prompt text. The list updates in real-time. Press Esc to clear the filter, Enter to open the selected session's detail view.

Press `Ctrl-G` while in filter mode to trigger a **deep search** that scans full conversation content (not just the first message).

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
| `Enter` | Open detail view |
| `q` / `Esc` | Quit |
| `Ctrl-C` | Quit |

### Filter mode

| Key | Action |
|-----|--------|
| Type | Fuzzy search |
| `Backspace` | Delete character |
| `Ctrl-G` | Deep search with current query |
| `Enter` | Open selected session's detail view |
| `Esc` | Clear filter, return to list |

### Detail view

| Key | Action |
|-----|--------|
| `Tab` | Switch between "Copy & Exit" and "Back" buttons |
| `Enter` | Activate focused button |
| `Esc` / `q` | Back to session list |

## How it works

1. **Discovery**: Scans `~/.claude/projects/` for session JSONL files using parallel I/O (rayon). Each `.jsonl` file is a session, identified by UUID filename.

2. **Parsing**: Reads the first few lines of each session file to find the first real user message (skipping `file-history-snapshot` entries and internal markup). Extracts project path, git branch, timestamp, and cleaned prompt text.

3. **Display**: Single-line format with the prompt text left-aligned and project + time right-aligned. Fuzzy matching via nucleo (same algorithm as fzf).

4. **Resume command**: Generates `cd '<project-path>' && claude -r <session-id>` with properly quoted paths. Copied to clipboard via arboard (cross-platform).

5. **Deep search**: For `-g` mode, uses regex + rayon to scan all JSONL content in parallel. Matches in any message (user or assistant) trigger inclusion.

## Files

| Path | Purpose |
|------|---------|
| `~/.claude/projects/` | Session data (read-only) |
| `~/.zshrc` or `~/.bashrc` | Shell functions (if `--shell-setup --install` used) |

## License

MIT
