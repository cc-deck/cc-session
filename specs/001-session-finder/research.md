# Research: cc-session

**Date**: 2026-02-25
**Feature**: 001-session-finder

## Decision 1: TUI Framework

**Decision**: ratatui 0.30 + crossterm 0.29
**Rationale**: ratatui is the de-facto Rust TUI library with 18.6k GitHub stars, 18.1M+ downloads, and a mature widget ecosystem. It provides built-in List, Scrollbar, and Paragraph widgets. crossterm 0.29 is the default backend, providing cross-platform terminal handling.
**Alternatives considered**:
- cursive: Higher-level API but less flexible for custom layouts and less actively maintained
- crossterm-only: Maximum control but requires building all widgets from scratch
- tui-realm: React-like component architecture, adds complexity without clear benefit for a focused tool

**App Architecture**: Elm Architecture (TEA) pattern. Single state struct, update function for events, view function for rendering. This matches cc-setup's BubbleTea pattern in Go.

## Decision 2: Fuzzy Matching Engine

**Decision**: nucleo 0.5
**Rationale**: Same scoring algorithm as fzf, ~6x faster than fuzzy-matcher/skim. Full Unicode support. Built-in parallel matching with lock-free item streaming. Used in production by Helix editor.
**Alternatives considered**:
- fuzzy-matcher (skim): Older, slower, ASCII-only for case folding and bonuses
- sublime_fuzzy: Less mature, fewer features
- Custom implementation: Unnecessary when nucleo provides exactly what's needed

## Decision 3: CLI Argument Parsing

**Decision**: clap 4.5 with derive API
**Rationale**: Industry standard for Rust CLI tools. Derive API provides compile-time validation and self-documenting code. Produces --help output automatically.
**Alternatives considered**:
- argh: Google's parser, simpler but less feature-rich
- structopt: Merged into clap 4's derive API
- Manual parsing: Error-prone, no --help generation

## Decision 4: Clipboard

**Decision**: arboard 3.6 with wayland-data-control feature
**Rationale**: Maintained by 1Password, 16.7M+ downloads. Supports macOS (NSPasteboard), Linux X11, Linux Wayland, and Windows. Reliable and well-tested.
**Alternatives considered**:
- cli-clipboard: Less maintained, fewer platform features
- copypasta: Older, less active development
- Shell out to pbcopy/xclip: Platform-specific, external dependency

## Decision 5: Deep Search Engine

**Decision**: rayon + serde_json + regex (not ripgrep's grep crate)
**Rationale**: The grep crate from ripgrep has sparse documentation and doesn't provide JSONL-aware parsing. Using rayon for parallel file I/O + serde_json for structured JSONL parsing + regex for pattern matching gives us better control over the search behavior and cleaner code. We can search within specific JSON fields rather than raw text.
**Alternatives considered**:
- grep crate (ripgrep): Poorly documented, no JSONL awareness, adds complexity without benefit
- Shell out to rg: External dependency, harder to parse output for our two-line display
- memmap + manual parsing: Premature optimization, BufReader is sufficient for our data sizes

## Decision 6: Session Discovery Strategy

**Decision**: Scan project directories for JSONL files, not history.jsonl
**Rationale**: Research revealed that `history.jsonl` does NOT contain `sessionId`. It only has `display`, `pastedContents`, `timestamp`, and `project`. Session IDs must be discovered by scanning `.jsonl` files in `~/.claude/projects/`. The first entry in session files can be a `file-history-snapshot` (not a user message), so we must scan for the first `type: "user"` entry.
**Data flow**:
1. Scan `~/.claude/projects/` for all project directories
2. For each project directory, list all `.jsonl` files (each is a session)
3. Extract session ID from filename (strip `.jsonl`)
4. Decode project path from directory name (replace leading `-` and internal `-` with `/`)
5. Read first few lines of each session JSONL to find the first user message entry (skip `file-history-snapshot` entries)
6. Extract `cwd`, `gitBranch`, `timestamp`, `message.content` from the first user message
7. Optionally cross-reference with `history.jsonl` for the `display` field (cleaner first-message text)
**Alternatives considered**:
- history.jsonl as primary index: Missing sessionId field makes this insufficient as sole source
- SQLite cache: Adds complexity, the parallel scan is fast enough

## Decision 7: Relative Timestamp Formatting

**Decision**: chrono 0.4 + chrono-humanize 0.2
**Rationale**: chrono is the dominant datetime crate in Rust. chrono-humanize provides clean "2 hours ago" formatting with both rough and precise modes.
**Alternatives considered**:
- jiff: Newer, better timezone handling, but lacks a humanize companion
- time crate: Leaner but no direct "time ago" support
- DIY implementation: Unnecessary when chrono-humanize exists

## Decision 8: Home Directory Resolution

**Decision**: dirs 6.0
**Rationale**: 149.8M+ downloads, handles platform differences correctly ($HOME on Linux/macOS, FOLDERID on Windows). Simple API, no overhead.
**Alternatives considered**:
- std::env::var("HOME"): Not cross-platform (fails on Windows)
- directories crate: More features than needed (we just need home_dir)

## Key Finding: Data Format Corrections

During research, we discovered several corrections to our initial assumptions:

1. **history.jsonl lacks sessionId**: The `sessionId` field does not exist in history entries. Sessions must be discovered from the project directories.
2. **First JSONL entry may not be a user message**: `file-history-snapshot` entries can precede the first user message. We must iterate until we find `type: "user"`.
3. **message.content can be string OR array**: User message content is sometimes a plain string, sometimes an array of `{type, text}` content blocks. We must handle both.
4. **Resume command**: `claude -r <session-id>` is confirmed correct (also `claude --resume <session-id>`).
5. **Directory encoding**: Path encoding replaces `/` with `-` and prepends a leading `-`. Example: `/Users/rhuss/project` becomes `-Users-rhuss-project`.
