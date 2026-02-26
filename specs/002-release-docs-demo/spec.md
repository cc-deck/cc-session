# Feature Specification: Release Pipeline, Documentation & Demo

**Feature Branch**: `002-release-docs-demo`
**Created**: 2026-02-26
**Status**: Draft
**Input**: User description: "Release pipeline, README documentation, and asciinema demo for cc-session"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install via Homebrew (Priority: P1)

A developer hears about cc-session and wants to install it. They run `brew install rhuss/tap/cc-session` and the tool is available immediately. The formula pulls a pre-built binary for their platform (macOS or Linux, arm64 or amd64) from a GitHub release.

**Why this priority**: Without a release mechanism, the tool can't be distributed. Homebrew is the primary install channel for the target audience (developers on macOS/Linux).

**Independent Test**: Run `brew install rhuss/tap/cc-session` on a clean machine and verify `cc-session --version` works.

**Acceptance Scenarios**:

1. **Given** a developer on macOS (arm64 or amd64), **When** they run `brew install rhuss/tap/cc-session`, **Then** the tool is installed and `cc-session --version` prints the current version.
2. **Given** a developer on Linux (amd64 or arm64), **When** they run `brew install rhuss/tap/cc-session`, **Then** the tool is installed and functional.
3. **Given** a new version is tagged in the repository, **When** the release pipeline runs, **Then** pre-built binaries for all 4 platform targets are published to a GitHub release and the Homebrew formula is updated automatically.
4. **Given** a developer without Homebrew, **When** they run the install script via curl, **Then** the binary is downloaded, checksum-verified, and installed to a local directory.

---

### User Story 2 - Discover and Understand the Tool (Priority: P1)

A developer lands on the cc-session GitHub page. They see a GIF demo showing the tool in action, understand what it does within 30 seconds, and find clear installation instructions. The README follows the same structure as cc-setup for a consistent experience across the cc-* tool family.

**Why this priority**: Without documentation, even a great tool won't get adopted. The README is the primary discovery and onboarding surface.

**Independent Test**: Show the README to someone unfamiliar with cc-session and verify they can understand what it does, install it, and use the basic features within 5 minutes.

**Acceptance Scenarios**:

1. **Given** a developer visits the GitHub repo, **When** they view the README, **Then** they see a GIF demo at the top showing the tool in action.
2. **Given** a developer reads the README, **When** they look for installation, **Then** they find instructions for Homebrew, an install script, and building from source.
3. **Given** a developer reads the README, **When** they look for usage, **Then** they find documentation for all modes (TUI, filter, detail view, scriptable, deep search, quick, time filters, shell integration) with examples.
4. **Given** a developer reads the README, **When** they look for keyboard shortcuts, **Then** they find a key bindings table covering all TUI interactions.

---

### User Story 3 - Reproducible Demo Recording (Priority: P2)

A maintainer wants to update the demo GIF after a UI change. They run the demo script, which generates fake session data, launches the tool against it, automates the interaction, records it with asciinema, and exports a GIF. The entire process is reproducible with no manual steps and no exposure of private data.

**Why this priority**: A compelling demo GIF drives adoption. Making it reproducible ensures it can be updated as the tool evolves.

**Independent Test**: Run the demo script on a fresh checkout and verify it produces a GIF file without errors.

**Acceptance Scenarios**:

1. **Given** a developer runs the demo fixture script, **Then** a temporary directory is created with realistic but fake session JSONL data (multiple projects, branches, timestamps, varied prompts).
2. **Given** fake session data exists, **When** the recording script runs, **Then** it launches cc-session against the fake data, automates browsing, filtering, entering the detail view, and copying.
3. **Given** an asciinema recording file exists, **When** the export script runs, **Then** a GIF is produced suitable for embedding in the README.
4. **Given** the demo scripts are run on different machines, **Then** the output is visually consistent (same data, same terminal dimensions).

---

### Edge Cases

- What happens when the release pipeline fails mid-way? Partial releases should be cleaned up. GitHub releases can be re-triggered by re-pushing the tag.
- What happens when the Homebrew formula is out of sync with the latest release? The formula update is part of the automated pipeline, so it stays in sync.
- What happens when the demo is run without asciinema/agg installed? The script checks for prerequisites and prints install instructions if missing.
- What happens when the install script is run on an unsupported platform? It prints a clear error and suggests building from source.

## Requirements *(mandatory)*

### Functional Requirements

#### Release Pipeline

- **FR-001**: The project MUST have an automated release pipeline that builds binaries for macOS (arm64, amd64) and Linux (arm64, amd64) when a version tag is pushed.
- **FR-002**: Each release MUST publish pre-built binaries and checksums to a GitHub release.
- **FR-003**: Each release MUST update the Homebrew formula in the `rhuss/homebrew-tap` repository automatically.
- **FR-004**: The project MUST include an install script that downloads the correct binary for the user's platform, verifies its checksum, and installs it to a local directory.
- **FR-005**: The project MUST be installable via `cargo install cc-session` from crates.io or from source.

#### Documentation

- **FR-006**: The project MUST have a README with a GIF demo at the top, followed by: problem statement, features, installation, quick start, usage (all modes), shell integration, key bindings, and architecture overview.
- **FR-007**: The README structure MUST follow the same style as the cc-setup project for consistency across the cc-* tool family.
- **FR-008**: Installation instructions MUST cover three methods: Homebrew (`brew install rhuss/tap/cc-session`), install script (curl one-liner), and building from source.
- **FR-009**: The README MUST document all CLI flags, TUI key bindings, and modes with usage examples.

#### Demo

- **FR-010**: The project MUST include a demo directory with scripts to generate fake session data, automate a TUI recording, and export a GIF.
- **FR-011**: The demo MUST use fake data only (no real session content), be fully automated (no manual interaction), and produce consistent output across runs.
- **FR-012**: The demo scenario MUST show the full flow: launch, browse sessions, filter with `/`, enter detail view, copy and exit.

### Key Entities

- **Release**: A versioned distribution of the cc-session binary for multiple platforms, published to GitHub Releases and Homebrew.
- **Formula**: A Homebrew package definition in `rhuss/homebrew-tap` that points to the release binary URLs and checksums.
- **Demo Fixture**: A set of fake session JSONL files mimicking the `~/.claude/` directory structure with realistic but synthetic data.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A new user can install cc-session via Homebrew in under 60 seconds and run `cc-session --version` successfully.
- **SC-002**: A new user can understand what cc-session does, install it, and use it within 5 minutes of reading the README.
- **SC-003**: The demo GIF can be regenerated from scratch (on a machine with asciinema and agg) in under 2 minutes with a single command.
- **SC-004**: The release pipeline completes within 10 minutes of a tag push, producing binaries for all 4 targets and an updated Homebrew formula.
- **SC-005**: The install script works on macOS (arm64/amd64) and Linux (amd64/arm64) without requiring any pre-installed tools beyond curl and a shell.

## Assumptions

- The GitHub repository `rhuss/cc-session` will be created for the public release.
- The existing `rhuss/homebrew-tap` repository accepts formula additions for new tools.
- The developer has `asciinema` and `agg` installed for demo GIF generation (not required for end users).
- The project uses semver tags (e.g., `v0.1.0`) to trigger releases.
- `cargo install` requires users to have a Rust toolchain installed.

## Out of Scope

- Windows release binaries (Linux and macOS only for initial release).
- Publishing to crates.io (can be added later, `cargo install --git` is sufficient).
- Continuous integration for PRs (only release pipeline for tags).
- Auto-updating mechanism (users update via `brew upgrade` or re-running the install script).
- Video tutorials or documentation beyond the README.
