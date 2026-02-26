# Feature Specification: cc-session - Claude Code Session Finder

**Feature Branch**: `001-session-finder`
**Created**: 2026-02-25
**Status**: Implemented
**Input**: User description: "A fast Rust CLI tool for finding and resuming Claude Code sessions with interactive TUI and deep search"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse and Resume Recent Sessions (Priority: P1)

A developer has been working across multiple projects with Claude Code over the past week. They want to quickly find and resume a specific conversation. They launch `cc-session` and see a chronologically sorted list of all their sessions. Each line shows the first user prompt (left-aligned) with the project name and relative timestamp right-aligned and dimmed. They press Enter on a session to open a detail view showing the last 20 prompts for confirmation, then press Enter again to copy the resume command to clipboard and exit.

**Why this priority**: This is the core value proposition. Without session browsing and resume command generation, the tool has no purpose.

**Independent Test**: Can be fully tested by running `cc-session` in a terminal with existing Claude sessions in `~/.claude/` and verifying the TUI displays sessions, the detail view shows prompts, and the resume command is copied on confirmation.

**Acceptance Scenarios**:

1. **Given** the user has Claude sessions stored in `~/.claude/projects/`, **When** they run `cc-session`, **Then** they see all sessions listed in a single-line format (prompt text left, project + time right), sorted most-recent-first.
2. **Given** the TUI is displaying sessions, **When** the user presses Enter, **Then** a detail view opens showing the last 20 user prompts for that session in a bordered box with action buttons below.
3. **Given** the detail view is showing, **When** the user presses Enter (on "Copy to clipboard & Exit"), **Then** the command `cd '<project-path>' && claude -r <session-id>` is copied to clipboard and the TUI exits.
4. **Given** the detail view is showing, **When** the user presses Tab, **Then** focus switches between the "Copy to clipboard & Exit" and "Back" buttons.
5. **Given** the TUI is displaying sessions, **When** the user navigates using j/k keys or arrow keys, **Then** the cursor moves between session entries.
6. **Given** the TUI is active, **When** the user presses q or Esc, **Then** the application exits cleanly.

---

### User Story 2 - Filter Sessions by Keyword (Priority: P1)

A developer remembers working on something related to "authentication" but doesn't recall which project or when. They launch `cc-session`, press `/`, type "auth", and the list narrows to sessions whose project name, git branch, or first message contains "auth". They select the matching session.

**Why this priority**: With hundreds of sessions, browsing alone is insufficient. Filtering is essential for practical use.

**Independent Test**: Can be tested by launching `cc-session`, pressing `/`, typing a known keyword, and verifying only matching sessions appear.

**Acceptance Scenarios**:

1. **Given** the TUI is displaying sessions, **When** the user presses `/` and types a search term, **Then** the list filters in real-time to show only sessions matching the term across project name, git branch, and first message text.
2. **Given** the user is in filter mode, **When** they press Esc, **Then** the filter is cleared and all sessions are shown again.
3. **Given** the user has typed a filter, **When** no sessions match, **Then** an empty state message is displayed.

---

### User Story 3 - Scriptable Session Lookup (Priority: P2)

A developer wants to quickly resume a session from a specific project without the full TUI. They run `cc-session -s antwort`. If exactly one session matches, the resume command is printed to stdout. If multiple sessions match, a slim numbered selection menu appears showing the top 10 results. They pick one and the command is printed.

**Why this priority**: Enables shell scripting, aliases, and power-user workflows. Not needed for basic usage but multiplies the tool's value for advanced users.

**Independent Test**: Can be tested by running `cc-session -s <known-project>` and verifying correct stdout output or selection menu behavior.

**Acceptance Scenarios**:

1. **Given** a search query matches exactly one session, **When** the user runs `cc-session -s <query>`, **Then** the resume command is printed to stdout.
2. **Given** a search query matches 2-10 sessions, **When** the user runs `cc-session -s <query>`, **Then** a slim numbered selection menu is displayed with matching sessions. On selection, the resume command is printed to stdout.
3. **Given** a search query matches more than 10 sessions, **When** the user runs `cc-session -s <query>`, **Then** the top 10 matches are shown in the selection menu with an indication that more results exist.
4. **Given** a search query matches zero sessions, **When** the user runs `cc-session -s <query>`, **Then** a "no sessions found" message is displayed and the tool exits with a non-zero exit code.

---

### User Story 4 - Deep Search Through Conversation Content (Priority: P3)

A developer remembers discussing a specific error message or code pattern in a Claude session but can't recall which project or when. They use `cc-session -g "ConnectionRefused"` to search through all conversation content. The tool scans the full JSONL session files in parallel and shows matching sessions in the same slim selection menu as `-s`. Alternatively, in the TUI, they press Ctrl-G from filter mode to trigger a deep search.

**Why this priority**: Addresses the "I know I discussed this somewhere" use case. Powerful but requires scanning large amounts of data, so it's a secondary mode.

**Independent Test**: Can be tested by running `cc-session -g <known-phrase>` where the phrase exists in a session's conversation content but not in the first message.

**Acceptance Scenarios**:

1. **Given** a pattern exists within a session's conversation content, **When** the user runs `cc-session -g <pattern>`, **Then** matching sessions are shown in a slim selection menu (same behavior as `-s`).
2. **Given** the TUI is in filter mode, **When** the user presses Ctrl-G, **Then** a deep search is triggered using the current filter query, and matching sessions replace the list.

---

### User Story 5 - Time-Scoped Session Listing (Priority: P3)

A developer wants to see only sessions from the past week, not the full history. They run `cc-session --since 7d` or `cc-session --last 50` to limit the results.

**Why this priority**: Quality-of-life improvement that reduces noise. The fuzzy filter handles most narrowing needs, but time-scoping is useful for very large session histories.

**Independent Test**: Can be tested by running `cc-session --since 1d` and verifying only sessions from the last 24 hours appear.

**Acceptance Scenarios**:

1. **Given** the user runs `cc-session --since 7d`, **Then** only sessions with timestamps within the last 7 days are displayed.
2. **Given** the user runs `cc-session --last 50`, **Then** at most 50 sessions are displayed, ordered most-recent-first.
3. **Given** both flags are used, **Then** both constraints are applied (most restrictive wins).

---

### User Story 6 - Quick Non-Interactive Mode (Priority: P3)

A developer wants to script session resumption without any interactive prompts. They run `cc-session -s myproject -q` or `cc-session -g pattern -q` to get the top match printed directly to stdout, ready for `eval`.

**Why this priority**: Enables zero-interaction scripting and shell aliases.

**Independent Test**: Run `cc-session -s <project> -q` and verify exactly one line of output (the resume command) with no menu.

**Acceptance Scenarios**:

1. **Given** `-q` is combined with `-s` or `-g`, **When** matches exist, **Then** the resume command for the top match is printed to stdout with no menu or clipboard interaction.
2. **Given** `-q` is used and no matches exist, **Then** the tool exits with code 1.

---

### User Story 7 - Shell Integration Setup (Priority: P3)

A developer wants to install shell helper functions for quick session resumption. They run `cc-session --shell-setup` to preview the functions, or `cc-session --shell-setup --install` to append them to their shell rc file.

**Why this priority**: One-time setup that enables the `ccs` and `ccf` convenience functions.

**Independent Test**: Run `cc-session --shell-setup` and verify function definitions are printed. Run with `--install` and verify the rc file is updated.

**Acceptance Scenarios**:

1. **Given** the user runs `cc-session --shell-setup`, **Then** shell function definitions for `ccs` (deep search + resume) and `ccf` (fuzzy search + resume) are printed to stdout.
2. **Given** the user runs `cc-session --shell-setup --install`, **Then** the functions are appended to the appropriate rc file (~/.zshrc or ~/.bashrc) with idempotent markers.

---

### Edge Cases

- What happens when `~/.claude/projects/` does not exist? The tool displays "No Claude projects directory found" to stderr and exits with code 2.
- What happens when a session JSONL file fails to parse? The session is silently skipped.
- What happens when the clipboard is unavailable (headless server, SSH)? The tool falls back to printing the command to stdout after exiting the TUI.
- What happens when a session's project directory no longer exists? The session is still listed normally. The resume command is still generated.
- What happens when session JSONL contains malformed JSON lines? Invalid lines are skipped silently.
- What happens when the terminal is too narrow? The display truncates text with ellipsis, never wrapping or breaking layout.
- What happens when message content contains Claude Code internal markup (e.g., `<command-message>`, `<system-reminder>`)? The markup tags are stripped. If the cleaned message is empty, the tool scans further entries for a real user prompt.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The tool MUST discover all Claude Code sessions by scanning `.jsonl` files in `~/.claude/projects/` subdirectories.
- **FR-002**: The tool MUST enrich session data by reading user entries from each session's JSONL file to extract git branch, working directory, and first user message. It MUST skip non-user entries (e.g., `file-history-snapshot`) and internal markup messages.
- **FR-003**: The tool MUST display sessions in a single-line format: user prompt text left-aligned, project name and relative timestamp right-aligned and dimmed. Text MUST be truncated with ellipsis when it exceeds the available width.
- **FR-004**: The tool MUST sort sessions by timestamp in descending order (most recent first) by default.
- **FR-005**: The tool MUST support interactive navigation via j/k keys and arrow keys.
- **FR-006**: The tool MUST support fuzzy filtering via the `/` key, matching across project name, git branch, and first message text.
- **FR-007**: On Enter, the tool MUST open a detail view showing the last 20 user prompts in a bordered box with "Copy to clipboard & Exit" and "Back" buttons (switchable via Tab). Pressing Enter on "Copy to clipboard & Exit" MUST copy the command `cd '<project-path>' && claude -r <session-id>` (path single-quoted) to the system clipboard and exit.
- **FR-008**: The tool MUST support a `-s <query>` flag for scriptable mode. With a single match, the command is printed to stdout. With 2-10 matches, a slim selection menu is shown. With more than 10 matches, the top 10 are shown with a "more results" indicator.
- **FR-009**: The tool MUST support a `-g <pattern>` flag for deep search that scans full session JSONL file content using parallel file reading. Results are presented in the same slim selection menu as `-s`.
- **FR-010**: The tool MUST support Ctrl-G in the TUI filter mode to trigger a deep search using the current filter query.
- **FR-011**: The tool MUST support `--since <duration>` to filter sessions by age (e.g., 7d, 2w, 1m).
- **FR-012**: The tool MUST support `--last <n>` to limit the number of displayed sessions.
- **FR-013**: The tool MUST operate in read-only mode, never modifying any session data or Claude configuration files.
- **FR-014**: The tool MUST support cross-platform clipboard operations (macOS, Linux X11/Wayland, Windows).
- **FR-015**: The tool MUST be distributable as a single static binary with no runtime dependencies.
- **FR-016**: The tool MUST fall back to printing the command to stdout when clipboard access is unavailable.
- **FR-017**: The tool MUST support a `-q`/`--quick` flag that, when combined with `-s` or `-g`, prints the resume command for the top match directly to stdout without any menu or clipboard interaction.
- **FR-018**: The tool MUST support `--shell-setup` to print shell function definitions (`ccs` for deep search + resume, `ccf` for fuzzy search + resume) to stdout. When combined with `--install`, the functions MUST be appended to the user's shell rc file with idempotent markers.
- **FR-019**: The tool MUST strip Claude Code internal XML markup tags (e.g., `<command-message>`, `<command-name>`, `<system-reminder>`) from displayed message text. Messages that contain only markup MUST be skipped in favor of the next real user prompt.

### Key Entities

- **Session**: Represents a Claude Code conversation instance. Key attributes: session ID (UUID), project path, project name, git branch, first user message (cleaned of markup), timestamp, working directory.
- **Project**: A directory that has been used with Claude Code. Identified by its absolute path, encoded as a hyphen-separated directory name under `~/.claude/projects/`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The tool displays a full session list within 500ms of launch for session histories containing up to 2,000 sessions.
- **SC-002**: Fuzzy filter results update within 50ms of each keystroke.
- **SC-003**: Deep search completes within 5 seconds for up to 1 GB of session data.
- **SC-004**: The resume command copied to clipboard is correct and functional 100% of the time (correct quoted path, correct session ID).
- **SC-005**: The tool launches and exits cleanly without residual terminal state corruption.
- **SC-006**: Users can locate and resume a target session in under 15 seconds from tool launch.

## Assumptions

- Claude Code stores session data in the `~/.claude/` directory structure as documented (projects/ with path-encoded directories, JSONL session files).
- The first user message entry in each session JSONL file (may not be the first line, as `file-history-snapshot` entries can precede it) contains `cwd`, `gitBranch`, `timestamp`, and `message.content` fields.
- User message content (`message.content`) can be either a plain string or an array of content blocks with `{type, text}` structure; both must be handled.
- Claude Code injects internal XML-like tags (`<command-message>`, `<command-name>`, `<command-args>`, `<local-command-caveat>`, `<system-reminder>`) into message content. These must be stripped for display.
- The `claude -r <session-id>` command is the correct way to resume a Claude Code session.
- Users have a standard terminal emulator that supports ANSI escape sequences and alternate screen buffer.

## Out of Scope

- Modifying, deleting, or archiving sessions.
- Displaying full conversation content (detail view shows last 20 prompts only).
- Remote session discovery (only local `~/.claude/` is scanned).
- Integration with IDE extensions or other Claude Code interfaces.
- Session tagging, bookmarking, or annotation.

## Changelog

- 2026-02-26: Evolved spec to match implementation
  - FR-003: Changed from two-line to single-line display (prompt left, metadata right-aligned)
  - FR-007: Changed from direct clipboard copy to detail view with prompt history and buttons
  - FR-009: Changed `-g` from TUI mode to slim selection menu (consistent with `-s`)
  - FR-017: Changed from `-g`+`-s` combination to new `-q`/`--quick` flag for non-interactive mode
  - FR-018: Added `--shell-setup` / `--install` for shell integration
  - FR-019: Added markup tag stripping for Claude Code internal metadata
  - Added User Stories 6 (Quick Mode) and 7 (Shell Setup)
  - Removed `HistoryEntry` from Key Entities (not used, sessions discovered from project dirs)
  - Removed project_exists dimming from edge cases
  - Updated resume command format to include single-quoted paths
