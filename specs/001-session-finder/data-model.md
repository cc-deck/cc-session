# Data Model: cc-session

**Date**: 2026-02-25
**Feature**: 001-session-finder

## Entities

### Session

Represents a single Claude Code conversation that can be resumed.

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `id` | UUID (string) | Filename of `.jsonl` file | Unique session identifier |
| `project_path` | Absolute path (string) | Decoded from project directory name | Original project directory |
| `project_name` | String | Last segment of `project_path` | Display name (e.g., "antwort") |
| `git_branch` | Option<String> | First user entry `gitBranch` field | Git branch at session start |
| `timestamp` | DateTime<Utc> | First user entry `timestamp` field | Session start time (ISO 8601) |
| `first_message` | String | First user entry `message.content` | First user prompt text |
| `cwd` | Absolute path (string) | First user entry `cwd` field | Working directory at session start |
| `claude_version` | Option<String> | First user entry `version` field | Claude Code version |
| `project_exists` | bool | Checked at discovery time | Whether `project_path` still exists on disk |

**Derivation rules**:
- `project_name`: Extract last path component from `project_path` (e.g., `/Users/rhuss/Development/ai/antwort` becomes `antwort`)
- `first_message`: If `message.content` is a string, use directly. If array of content blocks, concatenate all `text` fields. Truncate to 200 characters for storage.
- `project_exists`: `std::path::Path::exists(project_path)`

### HistoryEntry

Lightweight record from `~/.claude/history.jsonl`. Used optionally for cross-referencing the `display` field.

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `display` | String | `display` field | User-friendly prompt summary |
| `timestamp` | i64 | `timestamp` field | Unix milliseconds timestamp |
| `project` | Absolute path (string) | `project` field | Project directory path |
| `pasted_contents` | HashMap<String, String> | `pastedContents` field | Pasted file contents (usually empty) |

**Note**: history.jsonl does NOT contain `sessionId`. It cannot be used as the primary session index.

### SessionFileEntry

A single line from a session JSONL file.

| Field | Type | Values | Description |
|-------|------|--------|-------------|
| `type` | String | `"user"`, `"assistant"`, `"file-history-snapshot"` | Entry type discriminator |
| `session_id` | Option<String> | UUID | Present on user/assistant entries |
| `cwd` | Option<String> | Absolute path | Present on user/assistant entries |
| `git_branch` | Option<String> | Branch name | Present on user/assistant entries |
| `timestamp` | Option<String> | ISO 8601 | Present on all entries |
| `version` | Option<String> | Semver string | Claude Code version |
| `message` | Option<MessageContent> | See below | The message payload |

### MessageContent

| Field | Type | Description |
|-------|------|-------------|
| `role` | String | `"user"` or `"assistant"` |
| `content` | StringOrArray | Plain string or array of content blocks |

**StringOrArray** is a custom deserializer:
- If JSON string: use directly as text
- If JSON array: iterate elements, collect all `{type: "text", text: "..."}` blocks, concatenate text fields

## Relationships

```
~/.claude/projects/
  └── <encoded-project-path>/          # 1 Project : N Sessions
        └── <session-id>.jsonl         # 1 Session : N SessionFileEntries
              ├── file-history-snapshot (0+)
              ├── user message (1st = Session metadata)
              ├── assistant message
              └── ... (alternating user/assistant)

~/.claude/history.jsonl                # Flat list of HistoryEntries
                                       # Cross-references by project path + timestamp
```

## State Transitions

### TUI App State

```
┌──────────┐   /    ┌──────────┐  Ctrl-G  ┌─────────────┐
│ Browsing ├───────→│ Filtering├─────────→│ Deep Search  │
└────┬─────┘   Esc  └────┬─────┘   Esc    └──────┬──────┘
     │         ←─────────┘        ←───────────────┘
     │ Enter                       Enter
     ▼                             ▼
┌──────────────┐             ┌──────────────┐
│ Copy to      │             │ Copy to      │
│ Clipboard    │             │ Clipboard    │
└──────────────┘             └──────────────┘
```

States:
- **Browsing**: Default. Navigate with j/k/arrows. Enter copies command.
- **Filtering**: Activated by `/`. Fuzzy filter across display fields. Esc returns to Browsing.
- **Deep Search**: Activated by Ctrl-G from Filtering. Searches full JSONL content. Esc returns to Browsing.

## Directory Encoding/Decoding

**Encoding** (path to directory name):
- Replace all `/` with `-`
- Result starts with `-` (from the leading `/`)
- Example: `/Users/rhuss/Development/ai/antwort` becomes `-Users-rhuss-Development-ai-antwort`

**Decoding** (directory name to path):
- Replace leading `-` with `/`
- Replace all remaining `-` with `/`
- **Caveat**: This is lossy if directory names contain hyphens. Verify by checking `cwd` from the first session entry.
- Recommended approach: Use directory name for initial decode, then override with `cwd` from the session JSONL first user entry (authoritative source).
