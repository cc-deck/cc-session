# Feature Specification: Fast Deep Search

**Feature Branch**: `004-fast-deep-search`
**Created**: 2026-03-07
**Status**: Draft
**Input**: Optimize deep search by reusing pre-loaded session metadata instead of re-parsing files for each match, reducing search time from 16s to under 1s for common terms.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fast Deep Search for Common Terms (Priority: P1)

A developer searches for a common term like "function" that appears in hundreds of sessions. Previously this took 16 seconds because the tool re-parsed each matching file to extract session metadata. Now the search completes in under 1 second because session metadata is already loaded at startup and matched by file path.

**Why this priority**: This is the core performance fix. Common search terms are the worst case and the most noticeable to users.

**Independent Test**: Launch cc-session, trigger a deep search (Ctrl-G) for a common term like "function". Verify results appear in under 1 second.

**Acceptance Scenarios**:

1. **Given** the session list is loaded (2000+ sessions), **When** the user deep-searches for a common term matching 500+ sessions, **Then** results appear in under 1 second.
2. **Given** a deep search completes, **When** the user examines the results, **Then** the same sessions are found as with the previous implementation (no results lost).
3. **Given** a deep search is performed, **When** a session file has no corresponding pre-loaded session (e.g., filtered out by --since or --last), **Then** that session is still found and displayed correctly.

---

### User Story 2 - Consistent Performance Regardless of Match Count (Priority: P1)

A developer performs deep searches throughout the day. Whether the search matches 5 sessions or 500, the response time is consistently fast because the bottleneck (session metadata construction) has been eliminated.

**Why this priority**: Predictable performance builds user trust. The previous 0.3s-to-16s range depending on match count was confusing.

**Independent Test**: Benchmark deep search with terms matching varying numbers of sessions (1, 10, 100, 500+). Verify all complete in under 1 second.

**Acceptance Scenarios**:

1. **Given** a search term matches 1 session, **When** deep search runs, **Then** it completes in under 0.5 seconds.
2. **Given** a search term matches 500+ sessions, **When** deep search runs, **Then** it completes in under 1 second.
3. **Given** a search term matches 0 sessions, **When** deep search runs, **Then** it completes in under 0.5 seconds.

---

### User Story 3 - Deep Search via CLI Flag (Priority: P2)

A developer uses `cc-session -g "pattern"` from the command line. The same performance improvement applies to the CLI deep search mode.

**Why this priority**: CLI mode uses the same search function. The fix should benefit both entry points.

**Independent Test**: Run `cc-session -g "function" -q` and verify it completes in under 1 second.

**Acceptance Scenarios**:

1. **Given** the user runs `cc-session -g "common_term" -q`, **When** the search completes, **Then** the result is returned in under 1 second.

---

### Edge Cases

- What happens when a session file exists on disk but wasn't discovered at startup (e.g., created after launch)? The deep search should still find it by constructing a Session on the fly as a fallback.
- What happens when the file-to-session mapping has stale entries (session file deleted between startup and search)? Skip gracefully, do not crash.
- What happens when --since or --last filters are active? Deep search should search ALL session files on disk, not just the filtered subset, but the file-to-session lookup uses the full (unfiltered) session list.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Deep search MUST reuse session metadata from the pre-loaded session list instead of re-parsing JSONL files to extract metadata for each match.
- **FR-002**: A file-path-to-session index MUST be built at startup from the discovered sessions, mapping each JSONL file path to its corresponding Session.
- **FR-003**: When a matching file has no entry in the index (undiscovered session), the search MUST fall back to constructing a Session by parsing the file (preserving current behavior for edge cases).
- **FR-004**: The search results MUST be identical to the previous implementation (same sessions found, same metadata displayed).
- **FR-005**: The optimization MUST apply to both TUI deep search (Ctrl-G) and CLI deep search (`-g` flag).
- **FR-006**: Deep search MUST remain case-insensitive.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Deep search for a term matching 500+ sessions completes in under 1 second (previously 16+ seconds).
- **SC-002**: Deep search for a term matching 0-10 sessions completes in under 0.5 seconds (no regression from current 0.3-0.5s).
- **SC-003**: Memory overhead of the file-path index is under 1 MB for 2000+ sessions.
- **SC-004**: Search results are identical to the previous implementation for all test cases.

## Assumptions

- The session list loaded at startup contains the vast majority of session files. The fallback path (re-parsing) handles rare edge cases.
- Session file paths can be reliably reconstructed from Session metadata (project_path + session id).
- The file-to-session index can be built as a HashMap during session discovery with negligible startup cost.

## Out of Scope

- Unified incremental search (merging metadata filter with deep search into a single mode). This is planned as a future enhancement building on top of this optimization. See `brainstorm-unified-search.md` in this directory.
- Caching or indexing of file contents.
- Background pre-scanning of session files.

## Changelog

- 2026-03-07: Initial specification.
