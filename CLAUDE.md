# cc-session Development Guidelines

## Tech Stack
- Rust (stable, 2021 edition, MSRV 1.80.0)
- ratatui 0.30, crossterm 0.29, clap 4.5, syntect 5.3, termbg 0.6
- rayon 1.11, regex 1, serde/serde_json 1.0, chrono 0.4, arboard 3.6
- Read-only access to `~/.claude/projects/**/*.jsonl`

## Commands

```bash
cargo test        # Run all tests
cargo clippy      # Lint check
cargo build       # Dev build
```

## Code Style

Follow standard Rust conventions. Run `cargo clippy` before committing.

## Release Process

Releases use cargo-dist via GitHub Actions, triggered by pushing annotated tags.

### Steps

1. Collect changes since last release: `git log <last-tag>..HEAD --oneline`
2. Create release notes in `CHANGELOG.md` (prepend new section for the new version)
3. Bump version in `Cargo.toml`
4. Commit version bump and changelog together
5. Create annotated tag: `git tag -a v<version> -m "<one-line summary>"`
6. Push: `git push origin main v<version>`
7. cargo-dist CI creates the GitHub release and reads CHANGELOG.md automatically
8. Verify: `gh release view v<version>`

### Changelog Format

```markdown
## What's New

- **Feature name**: Brief description of what changed and why
- **Bug fix**: What was broken and how it was fixed

## Bug Fixes

- Description of fix
```

Keep entries concise. Group by: "What's New" for features/improvements, "Bug Fixes" for fixes. No need to list every commit, summarize related changes into single entries.

### Important

- Do NOT create GH releases manually (cargo-dist CI creates them)
- Homebrew tap at cc-deck/tap updates automatically
- GPG signing can timeout; use `-c commit.gpgsign=false` or `-c tag.gpgsign=false` if needed
