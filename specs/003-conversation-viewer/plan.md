# Implementation Plan: Conversation Viewer

**Branch**: `003-conversation-viewer` | **Date**: 2026-03-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-conversation-viewer/spec.md`

## Summary

Replace the existing detail view (20 truncated user prompts in a bordered box with buttons) with a full conversation viewer. The viewer shows all user and assistant messages in a scrollable paged view with role indicators, horizontal separators, timestamps, code fence highlighting, and incremental in-view search with n/N navigation.

## Technical Context

**Language/Version**: Rust (stable, 2021 edition, MSRV 1.80.0)
**Primary Dependencies**: ratatui 0.30, crossterm 0.29, serde/serde_json 1.0, chrono 0.4
**Storage**: Read-only access to `~/.claude/projects/**/*.jsonl`
**Testing**: cargo test
**Target Platform**: macOS, Linux (terminal applications)
**Project Type**: CLI/TUI application
**Performance Goals**: Viewer opens within 1s for 500-message sessions, 60fps scrolling
**Constraints**: No new crate dependencies (use existing ratatui/crossterm)
**Scale/Scope**: Single-user CLI tool, sessions typically 10-500 messages

## Constitution Check

*No project constitution has been defined (template only). Proceeding without gates.*

## Project Structure

### Documentation (this feature)

```text
specs/003-conversation-viewer/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── session.rs           # Add ConversationMessage type
├── discovery.rs         # Add load_conversation() function
├── tui/
│   ├── mod.rs           # Add ConversationViewer mode, replace Detail mode
│   ├── input.rs         # Add conversation viewer key handlers
│   └── view.rs          # Add conversation rendering, search highlighting
tests/
└── conversation_test.rs # Tests for message loading and formatting
```

**Structure Decision**: Existing single-project layout. New functionality fits into existing modules with minimal additions.
