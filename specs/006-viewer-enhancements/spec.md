# Feature Specification: Conversation Viewer Enhancements

**Feature Branch**: `006-viewer-enhancements`
**Created**: 2026-03-08
**Status**: Draft
**Input**: Brainstorm at `specs/006-viewer-enhancements/brainstorm.md`. Improve the conversation viewer with system content stripping, syntax highlighting, markdown table rendering, visual role separation, and adaptive light/dark themes.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Clean Conversation Content (Priority: P1)

As a user, I want the conversation viewer to show only the actual conversation content (my prompts and Claude's responses) without system-injected noise like "Caveat: The messages below were generated..." text, tool output listings, or hook results.

**Why this priority**: This is the most impactful fix. Currently, system-injected content dominates many user messages, burying the actual user prompt under pages of tool output and system warnings. This makes the viewer nearly unusable for reviewing what was actually discussed.

**Independent Test**: Open a conversation that contains local command caveats and tool output. Verify that only the user's actual typed prompts and Claude's responses are visible, with system-injected blocks fully removed.

**Acceptance Scenarios**:

1. **Given** a conversation where user messages contain `<local-command-caveat>` blocks, **When** viewing the conversation, **Then** the caveat text and its content are not displayed.
2. **Given** a conversation where user messages contain `<local-command-stdout>` tool output, **When** viewing the conversation, **Then** the tool output is not displayed, only the user's actual prompt text remains.
3. **Given** a user message that consists entirely of system-injected content (no actual user text), **When** viewing the conversation, **Then** the message is skipped entirely (not shown as an empty entry).
4. **Given** a user message with both system-injected content and actual user text, **When** viewing the conversation, **Then** only the actual user text is displayed.

---

### User Story 2 - Visual Role Separation (Priority: P1)

As a user, I want clear visual boundaries between "You" and "Claude" message blocks so I can quickly scan the conversation flow and identify who said what.

**Why this priority**: The current small colored text labels ("▶ You", "◀ Claude") are easy to miss when scrolling. Full-width role headers with background colors and subtle message body tinting create an immediately scannable conversation layout, similar to a chat application.

**Independent Test**: Open any conversation and verify that role transitions are immediately visible through full-width colored header bars and that user messages have a distinct background tint.

**Acceptance Scenarios**:

1. **Given** a conversation is displayed, **When** a user message block begins, **Then** a full-width header bar with the "You" label is shown with a distinctive background color.
2. **Given** a conversation is displayed, **When** a Claude message block begins, **Then** a full-width header bar with the "Claude" label is shown with a different distinctive background color.
3. **Given** user message body lines are displayed, **When** viewing the content, **Then** they have a subtle background tint distinguishing them from Claude's messages (which use the default terminal background).
4. **Given** the role header bar is displayed, **When** the bar renders, **Then** the timestamp is integrated into the header bar (right-aligned), replacing the separate separator line.

---

### User Story 3 - Adaptive Color Theme (Priority: P2)

As a user, I want the viewer to automatically detect whether my terminal has a light or dark background and adapt its colors accordingly, with the option to override manually.

**Why this priority**: The current hardcoded dark-theme colors produce poor contrast on light terminals. Theme detection is also a prerequisite for syntax highlighting themes (US5). Without it, users with light terminals see washed-out or invisible text.

**Independent Test**: Launch the viewer in a light-background terminal and verify that all text is readable with appropriate contrast. Then use the manual override flag and verify it forces the opposite theme.

**Acceptance Scenarios**:

1. **Given** a terminal with a dark background, **When** the viewer launches without flags, **Then** the dark color theme is automatically selected.
2. **Given** a terminal with a light background, **When** the viewer launches without flags, **Then** the light color theme is automatically selected with appropriate contrast.
3. **Given** any terminal, **When** the user passes a `--light` or `--dark` flag, **Then** the specified theme is used regardless of auto-detection.
4. **Given** a terminal where auto-detection fails (no COLORFGBG, no OSC 11 support), **When** the viewer launches, **Then** the dark theme is used as the default fallback.

---

### User Story 4 - Markdown Table Rendering (Priority: P2)

As a user, I want markdown tables in conversations to be rendered with proper alignment and grid lines so they are easy to read, instead of showing raw pipe-separated text.

**Why this priority**: Tables are common in Claude's responses (comparison tables, API docs, data summaries). Raw markdown tables with pipe characters are hard to parse visually, especially with varying column widths. Proper rendering significantly improves readability.

**Independent Test**: Open a conversation containing a markdown table and verify it renders with grid lines, aligned columns, and bold headers.

**Acceptance Scenarios**:

1. **Given** a conversation containing a markdown table, **When** viewing the message, **Then** the table is rendered with Unicode box-drawing characters (grid lines) instead of raw pipe characters.
2. **Given** a table with varying column content widths, **When** rendering the table, **Then** columns are auto-sized to fit their content, with consistent alignment.
3. **Given** a table with a header row (first row followed by `|---|` separator), **When** rendering, **Then** the header row is displayed with bold styling and a distinct separator line below it.
4. **Given** a table that would exceed the content area width, **When** rendering, **Then** the table gracefully handles overflow (either truncating cells or wrapping within cells).

---

### User Story 5 - Syntax Highlighted Code Blocks (Priority: P3)

As a user, I want code blocks in conversations to be syntax-highlighted based on the specified language, so I can quickly read and understand code snippets in Claude's responses.

**Why this priority**: Code blocks are the most common content type in Claude Code conversations. While the current single-color rendering is functional, proper syntax highlighting (keywords, strings, comments in different colors) dramatically improves code readability. This is the largest change and depends on the theme system (US3).

**Independent Test**: Open a conversation containing a code block with a language tag (e.g., ` ```rust `) and verify that keywords, strings, comments, and other tokens are rendered in distinct colors matching the active theme.

**Acceptance Scenarios**:

1. **Given** a code block with a language tag (e.g., ` ```python `), **When** rendering the block, **Then** syntax tokens (keywords, strings, comments, operators) are displayed in distinct colors.
2. **Given** a code block without a language tag (` ``` ` only), **When** rendering the block, **Then** the code is displayed with the current single-color style (no highlighting attempted).
3. **Given** a code block with an unrecognized language tag, **When** rendering, **Then** the code falls back to the single-color style gracefully.
4. **Given** the active theme is "light", **When** rendering syntax-highlighted code, **Then** the highlight colors are appropriate for a light background (different palette than dark theme).
5. **Given** a code block within the content width, **When** rendering, **Then** the code block has a subtle background color distinguishing it from surrounding prose.

---

### Edge Cases

- What happens when a user message contains only system-injected content after stripping? The message should be skipped entirely, not shown as an empty block.
- What happens when a markdown table has misaligned pipes or missing cells? The table should fall back to raw text rendering rather than crashing or producing garbled output.
- What happens when OSC 11 theme detection times out? Default to dark theme within 100ms to avoid blocking the UI.
- What happens with very wide tables (more columns than terminal width)? Truncate cell content with ellipsis rather than wrapping to avoid unreadable multi-line cells.
- What happens with nested code blocks (code block inside a table cell)? Render the inner content as plain text; do not attempt nested syntax highlighting.
- What happens when a code fence language tag contains extra tokens (e.g., ` ```rust,ignore `)? Extract the first word as the language identifier.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST strip entire content blocks (tag and content) for known system-injected XML tags, not just the tag markers.
- **FR-002**: System MUST skip user messages that become empty after system content stripping.
- **FR-003**: System MUST render role transitions with full-width colored header bars containing the role label and right-aligned timestamp.
- **FR-004**: System MUST apply a subtle background tint to user message body lines, distinct from the default background used for assistant messages.
- **FR-005**: System MUST auto-detect the terminal's background color (light vs dark) and select an appropriate color theme.
- **FR-006**: System MUST provide `--light` and `--dark` CLI flags to manually override theme auto-detection.
- **FR-007**: System MUST default to the dark theme when auto-detection fails or is unsupported.
- **FR-008**: System MUST render markdown tables with Unicode box-drawing grid characters, auto-sized columns, and bold header rows.
- **FR-009**: System MUST fall back to raw text rendering for malformed or unparseable markdown tables.
- **FR-010**: System MUST apply syntax highlighting to code blocks when a recognized language tag is present on the opening fence.
- **FR-011**: System MUST fall back to the existing single-color code style when no language tag is specified or the language is unrecognized.
- **FR-012**: System MUST provide syntax highlighting color palettes appropriate for both light and dark themes.
- **FR-013**: System MUST apply a distinct background color to code blocks to visually separate them from surrounding prose.

### Key Entities

- **Theme**: A named collection of colors for all visual elements (role headers, message backgrounds, code, separators, headings, search highlights). Two built-in themes: dark and light.
- **System Tag**: An XML-like tag injected by the Claude Code infrastructure (not the user or model) whose content should be stripped from display. Includes `system-reminder`, `local-command-caveat`, `local-command-stdout`, `command-name`, `command-message`, `command-args`, and others.
- **Code Block**: A fenced code section (` ``` `) with an optional language tag. Contains source code that may be syntax-highlighted.
- **Table Block**: A sequence of consecutive lines starting with `|` characters, representing a markdown table with optional header separator.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: System-injected content (caveats, tool output, hook results) is never visible in the conversation viewer for any session.
- **SC-002**: Users can visually identify role transitions (You vs Claude) within 0.5 seconds of scanning any conversation page.
- **SC-003**: Code blocks with recognized language tags display at least 3 distinct token colors (keywords, strings, comments) matching the active theme.
- **SC-004**: Markdown tables render with aligned columns and grid lines, readable without mental parsing of raw pipe syntax.
- **SC-005**: The viewer is readable (all text has sufficient contrast) on both light and dark terminal backgrounds, whether auto-detected or manually overridden.
- **SC-006**: Theme detection completes within 100ms and does not delay the viewer's initial render.

## Assumptions

- The set of system-injected XML tags is finite and known. New tags can be added to the strip list as they are discovered.
- Terminal color detection via COLORFGBG environment variable is available in most modern terminals (iTerm2, Terminal.app, Alacritty, Kitty, WezTerm). OSC 11 is a fallback for others.
- The code fence language tag is the first word after the triple backticks (e.g., `rust` from ` ```rust `).
- Syntax highlighting color palettes are chosen for readability, not exact IDE reproduction.
- Table rendering does not need to support merged cells, colspan, or other extended table features.
- The existing `is_meta_message` function correctly identifies messages that should be skipped even without system tag stripping.

## Dependencies

- **005-unified-search**: The content tag stripping in search.rs (completed in v0.4.0) provides a pattern for the viewer's tag stripping, but the viewer needs a more comprehensive solution that strips content blocks (not just tags).
