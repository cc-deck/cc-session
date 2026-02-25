# Feature Specification: cc-session - Claude Code Session Finder

**Feature Branch**: `001-session-finder`
**Created**: 2026-02-25
**Status**: Draft
**Input**: User description: "A fast Rust CLI tool for finding and resuming Claude Code sessions with interactive TUI and deep search"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse and Resume Recent Sessions (Priority: P1)

A developer has been working across multiple projects with Claude Code over the past week. They want to quickly find and resume a specific conversation. They launch `cc-session` and see a chronologically sorted list of all their sessions, each showing the project name, git branch, relative timestamp, and first message. They scroll through, spot the session they need, press Enter, and the resume command is copied to their clipboard ready to paste into the terminal.

**Why this priority**: This is the core value proposition. Without session browsing and resume command generation, the tool has no purpose.

**Independent Test**: Can be fully tested by running `cc-session` in a terminal with existing Claude sessions in `~/.claude/` and verifying the TUI displays sessions and copies the correct command on selection.

**Acceptance Scenarios**:

1. **Given** the user has Claude sessions stored in `~/.claude/projects/`, **When** they run `cc-session`, **Then** they see all sessions listed with project name, git branch, relative time, and first user message, sorted most-recent-first.
2. **Given** the TUI is displaying sessions, **When** the user navigates to a session and presses Enter, **Then** the command `cd <project-path> && claude -r <session-id>` is copied to their system clipboard.
3. **Given** the TUI is displaying sessions, **When** the user navigates using j/k keys or arrow keys, **Then** the cursor moves between session entries.
4. **Given** the TUI is active, **When** the user presses q or Esc, **Then** the application exits cleanly.

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

A developer remembers discussing a specific error message or code pattern in a Claude session but can't recall which project or when. They use `cc-session -g "ConnectionRefused"` to search through all conversation content. The tool scans the full JSONL session files in parallel and presents matching sessions. Alternatively, in the TUI, they press Ctrl-G to switch from quick filter to deep search mode.

**Why this priority**: Addresses the "I know I discussed this somewhere" use case. Powerful but requires scanning large amounts of data, so it's a secondary mode.

**Independent Test**: Can be tested by running `cc-session -g <known-phrase>` where the phrase exists in a session's conversation content but not in the first message.

**Acceptance Scenarios**:

1. **Given** a pattern exists within a session's conversation content, **When** the user runs `cc-session -g <pattern>`, **Then** sessions containing that pattern are listed with context.
2. **Given** the TUI is in normal filter mode, **When** the user presses Ctrl-G, **Then** the search mode switches to deep search, scanning full session content.
3. **Given** the `-g` flag is combined with `-s`, **When** matches are found, **Then** the scriptable selection behavior applies (single match prints command, multiple matches show menu).

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

### Edge Cases

- What happens when `~/.claude/history.jsonl` does not exist? The tool displays a clear message: "No Claude sessions found. Is Claude Code installed?"
- What happens when a session JSONL file referenced in history.jsonl has been deleted? The session is silently skipped.
- What happens when the clipboard is unavailable (headless server, SSH)? The tool falls back to printing the command to stdout with a note that clipboard is unavailable.
- What happens when a session's project directory no longer exists? The session is still listed but visually marked (e.g., dimmed or with a warning indicator). The resume command is still generated.
- What happens when history.jsonl is malformed or contains invalid JSON lines? Invalid lines are skipped with a count of skipped entries shown if any exist.
- What happens when the terminal is too narrow for the two-line display? The display gracefully truncates text with ellipsis, never wrapping or breaking layout.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The tool MUST discover all Claude Code sessions by scanning `.jsonl` files in `~/.claude/projects/` subdirectories. It MAY use `~/.claude/history.jsonl` as a supplementary data source for display text.
- **FR-002**: The tool MUST enrich session data by reading the first line of each session's JSONL file (located in `~/.claude/projects/<encoded-path>/<session-id>.jsonl`) to extract git branch and working directory.
- **FR-003**: The tool MUST display sessions in a two-line format: line 1 shows project name (last path segment), git branch, and relative timestamp; line 2 shows the first user message (truncated to terminal width, indented).
- **FR-004**: The tool MUST sort sessions by timestamp in descending order (most recent first) by default.
- **FR-005**: The tool MUST support interactive navigation via j/k keys and arrow keys.
- **FR-006**: The tool MUST support fuzzy filtering via the `/` key, matching across project name, git branch, and first message text.
- **FR-007**: The tool MUST copy the command `cd <project-path> && claude -r <session-id>` to the system clipboard when a session is selected with Enter.
- **FR-008**: The tool MUST support a `-s <query>` flag for scriptable mode. With a single match, the command is printed to stdout. With 2-10 matches, a slim selection menu is shown. With more than 10 matches, the top 10 are shown with a "more results" indicator.
- **FR-009**: The tool MUST support a `-g <pattern>` flag for deep search that scans full session JSONL file content using parallel file reading.
- **FR-010**: The tool MUST support Ctrl-G in the TUI to switch from quick filter to deep search mode.
- **FR-011**: The tool MUST support `--since <duration>` to filter sessions by age (e.g., 7d, 2w, 1m).
- **FR-012**: The tool MUST support `--last <n>` to limit the number of displayed sessions.
- **FR-013**: The tool MUST operate in read-only mode, never modifying any session data or Claude configuration files.
- **FR-014**: The tool MUST support cross-platform clipboard operations (macOS, Linux X11/Wayland, Windows).
- **FR-015**: The tool MUST be distributable as a single static binary with no runtime dependencies.
- **FR-016**: The tool MUST fall back to printing the command to stdout when clipboard access is unavailable.
- **FR-017**: The tool MUST handle the `-g` flag in combination with `-s` for scriptable deep search.

### Key Entities

- **Session**: Represents a Claude Code conversation instance. Key attributes: session ID (UUID), project path, git branch, first user message, timestamp, working directory.
- **Project**: A directory that has been used with Claude Code. Identified by its absolute path, encoded as a hyphen-separated directory name under `~/.claude/projects/`.
- **History Entry**: A lightweight index record from `history.jsonl` containing display text, timestamp, project path, and session ID.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The tool displays a full session list within 500ms of launch for session histories containing up to 2,000 sessions.
- **SC-002**: Fuzzy filter results update within 50ms of each keystroke.
- **SC-003**: Deep search completes within 5 seconds for up to 1 GB of session data.
- **SC-004**: The resume command copied to clipboard is correct and functional 100% of the time (correct path, correct session ID).
- **SC-005**: The tool launches and exits cleanly without residual terminal state corruption.
- **SC-006**: Users can locate and resume a target session in under 15 seconds from tool launch.

## Assumptions

- Claude Code stores session data in the `~/.claude/` directory structure as documented (history.jsonl, projects/ with path-encoded directories, JSONL session files).
- The first user message entry in each session JSONL file (may not be the first line, as `file-history-snapshot` entries can precede it) contains `cwd`, `gitBranch`, `timestamp`, and `message.content` fields.
- The history.jsonl file contains entries with `display`, `timestamp`, and `project` fields (notably, it does NOT contain `sessionId`; sessions must be discovered from the project directories).
- User message content (`message.content`) can be either a plain string or an array of content blocks with `{type, text}` structure; both must be handled.
- The `claude -r <session-id>` command is the correct way to resume a Claude Code session.
- Users have a standard terminal emulator that supports ANSI escape sequences and alternate screen buffer.

## Out of Scope

- Modifying, deleting, or archiving sessions.
- Displaying full conversation content (only first message preview is shown).
- Remote session discovery (only local `~/.claude/` is scanned).
- Integration with IDE extensions or other Claude Code interfaces.
- Session tagging, bookmarking, or annotation.
