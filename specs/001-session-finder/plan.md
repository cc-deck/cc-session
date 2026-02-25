# Implementation Plan: cc-session - Claude Code Session Finder

**Branch**: `001-session-finder` | **Date**: 2026-02-25 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-session-finder/spec.md`

## Summary

Build a fast Rust CLI tool that discovers Claude Code sessions from `~/.claude/`, presents them in an interactive TUI with fuzzy filtering and deep search, and generates resume commands for clipboard or stdout. Uses ratatui for TUI, nucleo for fuzzy matching, rayon for parallel I/O, and arboard for cross-platform clipboard.

## Technical Context

**Language/Version**: Rust (stable, 2021 edition, MSRV 1.80.0)
**Primary Dependencies**: ratatui 0.30, crossterm 0.29, clap 4.5, nucleo 0.5, arboard 3.6, rayon 1.11, serde/serde_json 1.0, chrono 0.4, dirs 6.0
**Storage**: Read-only access to `~/.claude/history.jsonl` and `~/.claude/projects/**/*.jsonl` (no database)
**Testing**: cargo test (unit tests for parsing/filtering, integration tests with fixture data)
**Target Platform**: macOS, Linux (X11/Wayland), Windows (cross-platform single binary)
**Project Type**: CLI tool with interactive TUI
**Performance Goals**: <500ms startup for 2,000 sessions, <50ms filter latency, <5s deep search for 1GB data
**Constraints**: Read-only, single static binary, no runtime dependencies
**Scale/Scope**: Up to 2,000+ sessions, ~750MB of JSONL data

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

No project-specific constitution defined (template only). No gates to evaluate. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/001-session-finder/
├── plan.md              # This file
├── research.md          # Phase 0: technology decisions
├── data-model.md        # Phase 1: data structures
├── quickstart.md        # Phase 1: getting started guide
├── contracts/           # Phase 1: CLI interface contract
│   └── cli.md
└── tasks.md             # Phase 2: task breakdown
```

### Source Code (repository root)

```text
Cargo.toml               # Project manifest with dependencies
src/
├── main.rs              # Entry point, CLI parsing, mode dispatch
├── session.rs           # Session/HistoryEntry data types and parsing
├── discovery.rs         # Session discovery: history.jsonl + JSONL enrichment
├── search.rs            # Deep search: parallel JSONL content scanning
├── filter.rs            # Fuzzy filtering with nucleo
├── clipboard.rs         # Cross-platform clipboard with fallback
├── tui/
│   ├── mod.rs           # TUI app state, event loop (Elm architecture)
│   ├── view.rs          # Rendering: two-line session display, status bar
│   └── input.rs         # Key event handling, mode switching
└── scriptable.rs        # -s mode: slim selection menu, stdout output

tests/
├── fixtures/            # Sample history.jsonl and session JSONL files
│   ├── history.jsonl
│   └── sessions/
│       ├── -project-a/
│       │   └── test-uuid-1.jsonl
│       └── -project-b/
│           └── test-uuid-2.jsonl
├── discovery_test.rs    # Session discovery and enrichment tests
├── filter_test.rs       # Fuzzy filter tests
├── search_test.rs       # Deep search tests
├── session_test.rs      # JSONL parsing tests
└── scriptable_test.rs   # Scriptable mode output tests
```

**Structure Decision**: Single-project Rust binary. The `src/tui/` subdirectory separates TUI concerns (state, view, input) from core logic (discovery, search, filtering). This keeps the TUI replaceable and core logic independently testable.
