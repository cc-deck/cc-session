# Research: Release Pipeline, Documentation & Demo

**Date**: 2026-02-26
**Feature**: 002-release-docs-demo

## Decision 1: Release Tool

**Decision**: cargo-dist 0.31.0
**Rationale**: Rust-native release tool by Axo. Generates GitHub Actions workflows, builds cross-platform binaries, creates install scripts, and supports Homebrew tap publishing. The Go equivalent (GoReleaser) is already used by cc-setup, making cargo-dist the natural Rust counterpart.
**Alternatives considered**:
- GoReleaser: Supports Rust builds but not idiomatic, requires Go-centric config
- Custom GitHub Actions + cross-rs: More maintenance, no install script generation
- trust (cross-ci): Abandoned, cargo-dist is its successor

## Decision 2: Homebrew Tap Integration

**Decision**: Publish to existing `rhuss/homebrew-tap` via cargo-dist's built-in Homebrew support
**Rationale**: cargo-dist natively supports `tap = "rhuss/homebrew-tap"` in Cargo.toml metadata. It generates the formula, runs `brew style --fix`, and commits to the tap repo. Same tap used by cc-setup for a unified `brew install rhuss/tap/cc-session` experience.
**Configuration**:
```toml
[workspace.metadata.dist]
tap = "rhuss/homebrew-tap"
publish-jobs = ["homebrew"]
```
**Alternatives considered**:
- Separate tap per tool: Unnecessary fragmentation
- Manual formula: More maintenance, error-prone

## Decision 3: Target Platforms

**Decision**: macOS arm64/amd64 + Linux arm64/amd64 (4 targets, no Windows)
**Rationale**: Claude Code primarily targets macOS and Linux developers. Windows support can be added later. These 4 targets cover the core audience.
**Targets**:
- x86_64-apple-darwin
- aarch64-apple-darwin
- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu

## Decision 4: Demo Recording Tool

**Decision**: asciinema + agg (already installed)
**Rationale**: asciinema records terminal sessions as .cast files (text-based, editable). agg converts .cast to GIF. Both already installed on the developer's machine. The recording will be driven by a shell script using tmux send-keys for reproducible automation.
**Alternatives considered**:
- VHS (by Charmbracelet): Declarative tape files, but not installed and adds a dependency
- ttygif: Older, less flexible output
- Manual recording: Not reproducible

## Decision 5: Demo Automation

**Decision**: tmux send-keys with sleep delays
**Rationale**: Simple, no external dependencies beyond tmux (standard on macOS/Linux). The script creates a tmux session, sends keystrokes with timed delays, while asciinema records the output. Produces consistent results across runs.
**Alternative**: expect/pexpect scripts (more complex, no benefit for this use case)

## Decision 6: Install Script

**Decision**: Use cargo-dist's auto-generated installer.sh
**Rationale**: cargo-dist generates a shell installer script that is published as part of each GitHub release. It handles platform detection, checksum verification, and installs to `~/.cargo/bin/`. No need to write a custom install.sh.
**Alternative**: Custom install.sh like cc-setup has (unnecessary duplication since cargo-dist handles this)
