# Tasks: Release Pipeline, Documentation & Demo

**Input**: Design documents from `/specs/002-release-docs-demo/`
**Prerequisites**: plan.md (required), spec.md (required), research.md

**Tests**: Not applicable (infrastructure/documentation feature).

**Organization**: Tasks grouped by user story for independent delivery.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Prerequisites and foundational files

- [ ] T001 (cc-session-bzs.1) Add MIT LICENSE file at `LICENSE`
- [ ] T002 (cc-session-bzs.2) Add description, repository, and license fields to `Cargo.toml` metadata

**Checkpoint**: Cargo.toml has complete package metadata, LICENSE exists.

---

## Phase 2: User Story 1 - Install via Homebrew (Priority: P1) MVP

**Goal**: Automated release pipeline producing binaries for 4 targets + Homebrew formula update.

**Independent Test**: Push a tag, verify GitHub Actions builds binaries and updates the Homebrew tap formula.

### Implementation for User Story 1

- [ ] T003 (cc-session-61r.1) [US1] Run `cargo dist init` with targets x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu. This generates `.github/workflows/release.yml` and adds `[workspace.metadata.dist]` to `Cargo.toml`.
- [ ] T004 (cc-session-61r.2) [US1] Add Homebrew tap configuration to `Cargo.toml`: set `tap = "rhuss/homebrew-tap"` and `publish-jobs = ["homebrew"]` in the `[workspace.metadata.dist]` section.
- [ ] T005 (cc-session-61r.3) [US1] Verify the generated `.github/workflows/release.yml` triggers on tag push (pattern `v*`), builds all 4 targets, and publishes to GitHub Releases.
- [ ] T006 (cc-session-61r.4) [US1] Test the release pipeline by running `cargo dist plan` locally to verify configuration is valid.

**Checkpoint**: `cargo dist plan` succeeds. `.github/workflows/release.yml` exists with correct targets.

---

## Phase 3: User Story 2 - README Documentation (Priority: P1) MVP

**Goal**: Comprehensive README with GIF demo, matching cc-setup's structure.

**Independent Test**: View README on GitHub, verify all sections present, GIF renders, install instructions work.

### Implementation for User Story 2

- [ ] T007 (cc-session-c9y.1) [US2] Write `README.md` with structure: GIF demo placeholder, "Why this matters" section explaining the problem (finding Claude sessions across projects is tedious), features list (TUI, filter, detail view, scriptable mode, deep search, quick mode, shell integration).
- [ ] T008 (cc-session-c9y.2) [US2] Add installation section to `README.md` covering three methods: Homebrew (`brew install rhuss/tap/cc-session`), install script (curl one-liner from cargo-dist), and building from source (`cargo install --git`).
- [ ] T009 (cc-session-c9y.3) [US2] Add usage section to `README.md` documenting: default TUI mode, filter mode (`/`), detail view (Enter), scriptable mode (`-s`), deep search (`-g`), quick mode (`-q`), time filters (`--since`, `--last`), shell integration (`--shell-setup --install`). Include example commands for each.
- [ ] T010 (cc-session-c9y.4) [US2] Add key bindings table to `README.md` covering all TUI interactions: j/k (navigate), / (filter), Enter (detail/select), Esc (back/quit), q (quit), Ctrl-G (deep search), Tab (switch buttons in detail view).
- [ ] T011 (cc-session-c9y.5) [US2] Add "How it works" section to `README.md` explaining: session discovery from `~/.claude/projects/`, parallel JSONL scanning, fuzzy matching, clipboard integration, markup stripping.

**Checkpoint**: README renders correctly with all sections. GIF placeholder present (replaced after Phase 4).

---

## Phase 4: User Story 3 - Reproducible Demo (Priority: P2)

**Goal**: Scripted demo recording with fake data, exported as GIF.

**Independent Test**: Run demo scripts, verify GIF is generated without errors.

### Implementation for User Story 3

- [ ] T012 (cc-session-2hs.1) [US3] Create `demo/create-fixtures.sh` that generates fake session JSONL data in a temp directory. Include ~15 sessions across 5 projects (api-gateway, auth-service, dashboard, data-pipeline, docs-site) with varied git branches, timestamps spanning 2 weeks, and realistic prompt texts. Output structure must match `~/.claude/projects/<encoded-path>/<uuid>.jsonl`.
- [ ] T013 (cc-session-2hs.2) [US3] Create `demo/record-demo.sh` that: (1) runs create-fixtures.sh, (2) builds cc-session in release mode, (3) starts a tmux session with fixed dimensions (100x30), (4) launches asciinema recording, (5) runs cc-session with CLAUDE_HOME pointing to fixtures, (6) uses tmux send-keys to automate: wait, scroll down 3 times, press /, type "auth", wait, press Enter on a match, wait on detail view, press Enter to copy, (7) stops recording. Produces `demo/demo.cast`.
- [ ] T014 (cc-session-2hs.3) [US3] Create `demo/export-gif.sh` that converts `demo/demo.cast` to `demo/demo.gif` using agg with appropriate settings (font size, theme, speed). Copy to `docs/demo.gif` for README embedding.
- [ ] T015 (cc-session-2hs.4) [US3] Update `README.md` to replace the GIF placeholder with the actual demo GIF path: `docs/demo.gif`.
- [ ] T016 (cc-session-2hs.5) [US3] Add prerequisite check to demo scripts: verify asciinema, agg, and tmux are installed, print install instructions if missing.

**Checkpoint**: `demo/record-demo.sh && demo/export-gif.sh` produces `docs/demo.gif`. README displays the GIF.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and validation

- [ ] T017 (cc-session-nvb.1) Add `demo/` directory to `.gitignore` for generated files (demo.cast, but keep scripts tracked). Add `docs/` to track the GIF.
- [ ] T018 (cc-session-nvb.2) Run `cargo dist plan` to verify the full release configuration is valid.
- [ ] T019 (cc-session-nvb.3) Verify README renders correctly on GitHub (check GIF, code blocks, tables, links).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies, start immediately
- **US1 Release (Phase 2)**: Depends on Setup (needs Cargo.toml metadata)
- **US2 README (Phase 3)**: Can start in parallel with US1 (independent files)
- **US3 Demo (Phase 4)**: Depends on a working binary (from Phase 2 or existing build). Depends on README existing (Phase 3) for GIF insertion.
- **Polish (Phase 5)**: Depends on all prior phases

### Parallel Opportunities

- T007, T008, T009, T010, T011 (README sections, same file but logically independent writes)
- Phase 2 (release) and Phase 3 (README) can run in parallel
- T012, T013, T014 (demo scripts, different files)

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (LICENSE, Cargo.toml metadata)
2. Complete Phase 2: US1 Release pipeline
3. Complete Phase 3: US2 README with GIF placeholder
4. **STOP and VALIDATE**: Push tag, verify release works, verify README renders
5. This is the shippable MVP

### Incremental Delivery

1. Setup + Release pipeline = can ship binaries
2. Add README = users can discover and install
3. Add Demo = compelling visual, drives adoption

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- The demo scripts should be idempotent (safe to re-run)
- Generated files (demo.cast, demo.gif) should be gitignored except the final docs/demo.gif

## Beads Task Management

This project uses beads (`bd`) for persistent task tracking across sessions:
- Run `/sdd:beads-task-sync` to create bd issues from this file
- `bd ready --json` returns unblocked tasks (dependencies resolved)
- `bd close <id>` marks a task complete (use `-r "reason"` for close reason)
- `bd sync` persists state to git
