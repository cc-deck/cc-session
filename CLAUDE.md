# cc-session Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-25

## Active Technologies
- Rust (stable, 2021 edition, MSRV 1.80.0) + Shell scripts (bash) + cargo-dist 0.31.0, asciinema 3.1.0, agg 1.7.0, tmux (002-release-docs-demo)
- Rust (stable, 2021 edition, MSRV 1.80.0) + ratatui 0.30, crossterm 0.29, serde/serde_json 1.0, chrono 0.4 (003-conversation-viewer)
- Read-only access to `~/.claude/projects/**/*.jsonl` (003-conversation-viewer)
- Rust stable, 2021 edition, MSRV 1.80.0 + ratatui 0.30, crossterm 0.29, clap 4.5, regex 1, rayon 1.11, serde/serde_json 1.0, chrono 0.4, dirs 6.0, arboard 3.6 (005-unified-search)
- Read-only access to `~/.claude/projects/**/*.jsonl` (005-unified-search)

- Rust (stable, 2021 edition, MSRV 1.80.0) + ratatui 0.30, crossterm 0.29, clap 4.5, nucleo 0.5, arboard 3.6, rayon 1.11, serde/serde_json 1.0, chrono 0.4, dirs 6.0 (001-session-finder)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust (stable, 2021 edition, MSRV 1.80.0): Follow standard conventions

## Recent Changes
- 005-unified-search: Added Rust stable, 2021 edition, MSRV 1.80.0 + ratatui 0.30, crossterm 0.29, clap 4.5, regex 1, rayon 1.11, serde/serde_json 1.0, chrono 0.4, dirs 6.0, arboard 3.6
- 003-conversation-viewer: Added Rust (stable, 2021 edition, MSRV 1.80.0) + ratatui 0.30, crossterm 0.29, serde/serde_json 1.0, chrono 0.4
- 002-release-docs-demo: Added Rust (stable, 2021 edition, MSRV 1.80.0) + Shell scripts (bash) + cargo-dist 0.31.0, asciinema 3.1.0, agg 1.7.0, tmux


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
