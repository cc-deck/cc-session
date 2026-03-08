# Data Model: Conversation Viewer Enhancements

**Feature**: 006-viewer-enhancements
**Date**: 2026-03-08

## New Entities

### Theme (struct)

Centralized color configuration for all visual elements. Two built-in instances (dark, light).

| Field | Type | Purpose |
|-------|------|---------|
| `name` | String | "dark" or "light" |
| `user_header_bg` | Color | Full-width background for "You" header bar |
| `user_header_fg` | Color | Text color for "You" header bar |
| `assistant_header_bg` | Color | Full-width background for "Claude" header bar |
| `assistant_header_fg` | Color | Text color for "Claude" header bar |
| `user_message_bg` | Color | Subtle background tint for user message body lines |
| `code_block_bg` | Color | Background for code fence blocks |
| `text` | Color | Default text color |
| `text_dim` | Color | Dimmed text (separators, metadata) |
| `heading` | Color | Markdown heading color |
| `separator` | Color | Line separator color |
| `table_border` | Color | Box-drawing grid character color |
| `table_header` | Style | Header row styling (bold + color) |
| `search_highlight_bg` | Color | Background for search term highlights |
| `syntect_theme` | String | Name of the syntect theme to use for code blocks |

### SystemTag (constant list)

List of XML tag names whose entire content blocks should be stripped from display.

| Tag Name | Source |
|----------|--------|
| `system-reminder` | Claude Code infrastructure |
| `local-command-caveat` | Local command execution |
| `local-command-stdout` | Command output |
| `command-name` | Slash command name |
| `command-message` | Slash command content |
| `command-args` | Command arguments |
| `user-prompt-submit-hook` | Hook output |
| `available-deferred-tools` | Tool discovery |
| `rosa-auth` | ROSA plugin auth context |
| `rosa-profile` | ROSA plugin profile |
| `sdd-context` | SDD plugin context |
| `skill-enforcement` | Skill enforcement rules |
| `fast_mode_info` | Fast mode metadata |
| `task-notification` | Task system notifications |
| `antml_thinking` | Model thinking blocks |
| `antml_function_calls` | Model function calls |

### TableBlock (struct)

Parsed markdown table ready for grid rendering.

| Field | Type | Purpose |
|-------|------|---------|
| `headers` | Vec<String> | Header row cell contents |
| `rows` | Vec<Vec<String>> | Data row cell contents |
| `column_widths` | Vec<usize> | Computed max width per column |
| `has_header` | bool | Whether a separator row was detected |

## Modified Entities

### ConversationState (in App)

No structural changes. The `lines: Vec<Line<'static>>` field now contains richer styled content (syntax-highlighted spans, table grid lines, background-tinted lines).

### App (in mod.rs)

| New Field | Type | Purpose |
|-----------|------|---------|
| `theme` | Theme | Active color theme, determined at startup |

## Processing Pipeline Changes

### Message Content Processing (load_conversation)

Current pipeline:
```
raw JSONL → parse JSON → extract message text → strip_tags → clean_message_multiline → ConversationMessage
```

New pipeline:
```
raw JSONL → parse JSON → extract message text → strip_system_blocks → clean_remaining_tags → clean_message_multiline → ConversationMessage
```

Where `strip_system_blocks` removes entire content blocks for known system tags, and `clean_remaining_tags` handles any remaining unknown tags (strip delimiters only, preserve content).

### Pre-render Pipeline (pre_render_conversation)

Current pipeline:
```
ConversationMessage → detect code fences → wrap lines → render_markdown_inline → Vec<Line>
```

New pipeline:
```
ConversationMessage → role header bar → detect table blocks → detect code fences →
  if table: parse_table → render_table_grid
  if code fence: syntax_highlight (syntect) → styled spans
  else: render_markdown_inline → styled spans
→ apply message background tint → Vec<Line>
```
