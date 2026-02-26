# Implementation Plan: Release Pipeline, Documentation & Demo

**Branch**: `002-release-docs-demo` | **Date**: 2026-02-26 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/002-release-docs-demo/spec.md`

## Summary

Set up automated release pipeline with cargo-dist (cross-platform binaries, Homebrew tap, install script), write a comprehensive README matching cc-setup's style with a GIF demo at the top, and create a reproducible demo recording system using asciinema + agg with fake session data.

## Technical Context

**Language/Version**: Rust (stable, 2021 edition, MSRV 1.80.0) + Shell scripts (bash)
**Primary Dependencies**: cargo-dist 0.31.0, asciinema 3.1.0, agg 1.7.0, tmux
**Storage**: N/A
**Testing**: Manual verification (release pipeline, demo script, README rendering)
**Target Platform**: macOS (arm64, amd64), Linux (arm64, amd64)
**Project Type**: Release infrastructure + documentation
**Performance Goals**: Release pipeline <10 min, demo GIF generation <2 min
**Constraints**: No Windows targets, no crates.io publishing (initial release)
**Scale/Scope**: Single binary, 4 platform targets, ~15 fake demo sessions

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

No project-specific constitution defined (template only). No gates to evaluate. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/002-release-docs-demo/
├── plan.md              # This file
├── research.md          # Technology decisions
├── quickstart.md        # Getting started guide
└── tasks.md             # Task breakdown
```

### Source Code (repository root)

```text
README.md                          # Project documentation with GIF demo
LICENSE                            # MIT license
.github/
└── workflows/
    └── release.yml                # cargo-dist generated release workflow
demo/
├── create-fixtures.sh             # Generate fake session JSONL data
├── record-demo.sh                 # Automate TUI interaction recording
├── export-gif.sh                  # Convert .cast to .gif
├── demo.cast                      # Recorded terminal session (generated)
└── demo.gif                       # Final GIF for README (generated)
docs/
└── demo.gif                       # Symlink or copy for GitHub rendering
```

**Structure Decision**: No source code changes. This feature adds release config (Cargo.toml metadata + GitHub Actions), documentation (README, LICENSE), and demo scripts (demo/ directory).
