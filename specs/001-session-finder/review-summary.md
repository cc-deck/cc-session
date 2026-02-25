# Review Summary: cc-session - Claude Code Session Finder

**Spec:** specs/001-session-finder/spec.md | **Plan:** specs/001-session-finder/plan.md
**Generated:** 2026-02-25

---

## Executive Summary

When working with Claude Code across many projects, developers accumulate hundreds of conversation sessions. Each session lives as a JSONL file buried under `~/.claude/projects/`, identified by UUID. Resuming the right session currently requires manually navigating to the correct project directory and remembering (or guessing) the session ID, then running `claude -r <uuid>`. This friction slows down the workflow every time a developer switches context.

cc-session solves this by providing a fast, interactive terminal tool that discovers all Claude Code sessions, presents them in a searchable list, and generates the exact resume command with one keypress. The tool scans `~/.claude/projects/` to find all session files, reads the first user message from each for context, and displays them in a two-line format showing the project name, git branch, relative timestamp, and message preview.

The tool operates in two modes. The default interactive TUI lets users browse and filter sessions with fuzzy search (like fzf), then copies `cd <path> && claude -r <id>` to the clipboard on Enter. A scriptable mode (`-s <query>`) prints the command to stdout for shell integration, with a slim selection menu when multiple sessions match. A deep search mode (`-g <pattern>`) scans full conversation content across all session files using parallel I/O for when users need to find a session by something mentioned mid-conversation rather than the first message.

Built in Rust for startup speed and single-binary distribution, the tool targets sub-500ms launch times even with 2,000+ sessions. Key dependencies are ratatui for the TUI, nucleo for fuzzy matching (same algorithm as fzf), arboard for cross-platform clipboard, and rayon for parallel file scanning.

## PR Contents

This spec PR includes the following artifacts:

| Artifact | Description |
|----------|-------------|
| `spec.md` | Feature specification with 5 user stories, 17 functional requirements, 6 edge cases, and 6 success criteria |
| `plan.md` | Implementation plan: Rust project structure, tech stack, dependency decisions |
| `research.md` | 8 technology decisions with rationale and alternatives considered |
| `data-model.md` | 4 data entities (Session, HistoryEntry, SessionFileEntry, MessageContent) with relationships and state transitions |
| `contracts/cli.md` | CLI interface contract: flags, exit codes, output formats, key bindings |
| `quickstart.md` | Build and usage instructions |
| `tasks.md` | 35 tasks across 8 phases |
| `checklists/requirements.md` | Spec quality validation checklist (all items passing) |
| `review-summary.md` | This file |

## Technical Decisions

### Session Discovery via Directory Scanning (not history.jsonl)

- **Chosen approach:** Scan `~/.claude/projects/` subdirectories for `.jsonl` files, read the first user message entry from each
- **Alternatives considered:**
  - Use `history.jsonl` as primary index: Rejected because history.jsonl does not contain `sessionId`, making it insufficient as a session discovery source
  - Build a SQLite cache: Rejected because rayon parallel scanning is fast enough for the data sizes involved (~2000 files), and a cache adds write operations to an otherwise read-only tool
- **Trade-off:** Slightly slower startup (must list directories + read first lines) but zero dependency on a single index file and no cache invalidation complexity
- **Reviewer question:** Should the tool also support a `CLAUDE_HOME` override for non-standard installations?

### nucleo for Fuzzy Matching (not grep crate)

- **Chosen approach:** nucleo 0.5 (same engine as fzf, used by Helix editor) for fuzzy matching; rayon + serde_json + regex for deep search
- **Alternatives considered:**
  - ripgrep's grep crate: Rejected because documentation is sparse and it has no JSONL awareness; we need to search within structured JSON fields
  - fuzzy-matcher/skim: Rejected because nucleo is ~6x faster and has full Unicode support
- **Trade-off:** Two separate search engines (nucleo for fuzzy filter, regex for deep search) adds code but each is optimal for its use case

### Elm Architecture for TUI (matching cc-setup's pattern)

- **Chosen approach:** Single App state struct with update/view separation, using ratatui + crossterm
- **Alternatives considered:**
  - Component architecture (tui-realm): Rejected as over-engineered for a focused single-screen tool
  - Minimal crossterm-only: Rejected because it requires building all widgets from scratch
- **Trade-off:** Proven pattern from cc-setup (Go/BubbleTea), translates cleanly to ratatui

## Critical References

| Reference | Why it needs attention |
|-----------|----------------------|
| `spec.md` FR-008: Scriptable mode behavior | Defines the multi-match selection menu UX. The 1/2-10/10+ thresholds and menu-on-stderr/command-on-stdout split need careful review for usability. |
| `data-model.md` Directory Encoding/Decoding | Path encoding is lossy (hyphens in real directory names cause ambiguity). The mitigation (override with `cwd` from session entry) works but needs validation with edge cases. |
| `data-model.md` StringOrArray deserializer | Custom serde logic for `message.content` that handles both string and array formats. Incorrect handling would cause silent data loss. |
| `plan.md` Performance Goals section | 500ms startup, 50ms filter, 5s deep search targets need validation with real data (750MB of sessions). |

## Reviewer Checklist

### Verify
- [ ] Session JSONL structure assumptions match actual Claude Code behavior (field names, entry types, content format)
- [ ] The `claude -r <session-id>` resume command syntax is correct for current Claude Code versions
- [ ] Directory encoding/decoding handles all edge cases (paths with hyphens, spaces, Unicode)
- [ ] Two-line display format fits comfortably in standard 80-column terminals

### Question
- [ ] Should the scriptable mode (`-s`) copy to clipboard instead of stdout, or is stdout the right choice for pipe-ability?
- [ ] Is the 10-session threshold for the slim menu appropriate, or should it be configurable?
- [ ] Should deep search (`-g`) support regex by default, or should there be separate flags for regex vs literal search?

### Watch out for
- [ ] Large session files (up to 48MB observed) could slow down deep search; may need streaming or size limits
- [ ] The `file-history-snapshot` entries at the start of session files could cause the tool to skip sessions if not handled correctly
- [ ] Clipboard access in headless environments (CI, SSH) may behave unexpectedly across platforms

## Scope Boundaries
- **In scope:** Session discovery, interactive TUI, fuzzy filter, deep search, scriptable mode, time filters, clipboard integration
- **Out of scope:** Session modification/deletion, full conversation display, remote sessions, IDE integration, tagging/bookmarks
- **Why these boundaries:** The tool is a focused session finder, not a session manager. Read-only operation eliminates data safety concerns and keeps the tool simple.

## Naming & Schema Decisions

| Item | Name | Context |
|------|------|---------|
| Binary name | `cc-session` | Consistent with `cc-setup` naming convention |
| Short flag for scriptable mode | `-s` / `--select` | Mnemonic for "select and print" |
| Short flag for deep search | `-g` / `--grep` | Mnemonic matching ripgrep/grep convention |
| Session display | Two-line format | Project + branch + time on line 1, message on line 2 |
| Resume command | `cd <path> && claude -r <id>` | Uses `cwd` from session (not decoded directory path) |

## Risk Areas

| Risk | Impact | Mitigation |
|------|--------|------------|
| Claude Code changes session storage format | High | Session parsing is isolated in `session.rs`; custom deserializer handles format variations |
| Directory encoding lossy for hyphenated paths | Medium | Always use `cwd` from session JSONL as authoritative path, not decoded directory name |
| Deep search slow on large session histories | Medium | rayon parallelism + raw text pre-filter before JSON parsing; `--since` flag to scope |
| Clipboard unavailable in headless environments | Low | Automatic fallback to stdout with user notification |
| nucleo API changes between versions | Low | Pin to nucleo 0.5.x in Cargo.toml |

---
*Share this with reviewers. Full context in linked spec and plan.*
