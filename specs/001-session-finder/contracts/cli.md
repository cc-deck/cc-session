# CLI Contract: cc-session

**Date**: 2026-02-25
**Feature**: 001-session-finder

## Command Synopsis

```
cc-session [OPTIONS]
cc-session -s <QUERY>
cc-session -g <PATTERN> [-s]
```

## Arguments and Options

| Flag | Long | Argument | Description |
|------|------|----------|-------------|
| | (none) | (none) | Launch interactive TUI (default mode) |
| `-s` | `--select` | `<QUERY>` | Scriptable mode: search and print resume command |
| `-g` | `--grep` | `<PATTERN>` | Deep search through full session content |
| | `--since` | `<DURATION>` | Filter sessions newer than duration (e.g., 7d, 2w, 1m) |
| | `--last` | `<N>` | Show only the N most recent sessions |
| `-h` | `--help` | | Show help message |
| `-V` | `--version` | | Show version |

### Duration Format for --since

| Suffix | Meaning | Example |
|--------|---------|---------|
| `d` | Days | `7d` = last 7 days |
| `w` | Weeks | `2w` = last 2 weeks |
| `m` | Months (30 days) | `1m` = last 30 days |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (command copied or printed) |
| 1 | No matching sessions found |
| 2 | Error (missing ~/.claude, parse failure, etc.) |

## Output Contract

### Interactive TUI Mode (default)

**Alternate screen buffer**: TUI uses alternate screen, restores terminal on exit.

**Display format** (two lines per session):
```
 <project-name> · <git-branch> · <relative-time>
   "<first-message-truncated>"
```

**Clipboard output on Enter**:
```
cd <project-path> && claude -r <session-id>
```

**Fallback** (clipboard unavailable):
```
# Clipboard unavailable, command printed to stdout:
cd <project-path> && claude -r <session-id>
```

### Scriptable Mode (-s)

**Single match** (stdout):
```
cd /Users/rhuss/Development/ai/antwort && claude -r 2448f8bf-33e7-40a2-95df-75a0156569d4
```

**Multiple matches** (2-10, interactive menu on stderr, result on stdout):
```
  3 sessions match "antwort":

  1  antwort · 028-list-endpoints · 2 hours ago
     "Help me implement the list endpoints..."
  2  antwort · main · yesterday
     "Fix the build error in the API module..."
  3  antwort · feat-auth · 3 days ago
     "Add OAuth2 support to the client..."

  Select [1-3]:
```

On selection, prints the resume command to stdout.

**More than 10 matches** (menu shows top 10):
```
  15 sessions match "antwort" (showing top 10):

  1  antwort · 028-list-endpoints · 2 hours ago
     ...
  ...
  10 antwort · initial-setup · 2 weeks ago
     ...

  Select [1-10]:
```

**No matches** (stderr, exit code 1):
```
No sessions found matching "nonexistent"
```

### Deep Search Mode (-g)

Same display format as default TUI, but sessions shown are those containing the pattern in their full conversation content.

When combined with `-s`: same scriptable behavior as above.

## TUI Key Bindings

| Key | Mode | Action |
|-----|------|--------|
| `j` / `↓` | Browse | Move cursor down |
| `k` / `↑` | Browse | Move cursor up |
| `Enter` | Browse/Filter | Select session, copy command to clipboard |
| `/` | Browse | Enter filter mode |
| `Esc` | Filter/DeepSearch | Return to browse mode |
| `Ctrl-G` | Filter | Switch to deep search mode |
| `q` | Browse | Quit |
| `Esc` | Browse | Quit |

## Environment

| Variable | Default | Description |
|----------|---------|-------------|
| `HOME` | (required) | User home directory for locating `~/.claude/` |
| `CLAUDE_HOME` | `~/.claude` | Override Claude data directory (optional, for testing) |
