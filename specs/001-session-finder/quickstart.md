# Quickstart: cc-session

## Prerequisites

- Rust toolchain (rustup, cargo)
- Claude Code installed with existing sessions in `~/.claude/`

## Build

```bash
cargo build --release
```

The binary is at `target/release/cc-session`.

## Install

```bash
cargo install --path .
```

## Usage

### Browse all sessions

```bash
cc-session
```

Navigate with j/k, filter with /, select with Enter.

### Find a session from a project

```bash
cc-session -s antwort
```

Prints the resume command or shows a selection menu.

### Search conversation content

```bash
cc-session -g "ConnectionRefused"
```

Deep searches through all session JSONL files for the pattern.

### Recent sessions only

```bash
cc-session --since 7d
cc-session --last 20
```

## Development

```bash
# Run in debug mode
cargo run

# Run tests
cargo test

# Run with a specific Claude data directory
CLAUDE_HOME=/path/to/test/.claude cargo run
```
