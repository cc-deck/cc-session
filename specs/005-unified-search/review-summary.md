# Review Summary: Unified Incremental Search

**Feature**: 005-unified-search
**Date**: 2026-03-07
**Spec Score**: 93/100 (PASS)
**Plan Score**: PASS (100% coverage, 0 critical red flags)

## Feature Overview

Replaces the two-mode search UX (metadata filter via `/` + deep content search via `Ctrl-G`) with a single unified search. Press `/`, type a query, and both metadata and content results appear automatically with progressive feedback.

## Spec Review Highlights

- 4 user stories (2x P1, 2x P2), 11 functional requirements, 5 success criteria
- 3 clarifications resolved: selection merge behavior, conversation auto-scroll, no result cap
- No implementation details leak into specification
- All requirements testable and unambiguous

## Plan Review Highlights

- **Coverage**: 11/11 FRs and 5/5 SCs mapped to specific tasks
- **30 tasks** across 8 phases (3 setup, 5 foundational, 7 US1/MVP, 3 US2, 2 US3, 3 US4, 2 auto-scroll, 5 polish)
- **No new files** created; all changes modify existing `src/tui/` and `src/search.rs`
- **No new dependencies** in Cargo.toml
- **Key design decisions**: debounce via existing poll loop, two-layer result model, AtomicBool cancellation, mode consolidation (6 modes to 4)

## Key Decisions for Reviewers

1. **Debounce approach**: Uses `Instant`-based tracking in the existing 100ms poll loop rather than a separate timer thread. Worst case: 400ms trigger instead of 300ms (imperceptible).

2. **Result merging**: Maintains separate metadata and content result sets, computes merged display on demand. Keeps metadata filtering instant without touching content results.

3. **Mode removal**: `DeepSearchInput` and `DeepSearching` modes are fully removed. `Filtering` mode is extended with `ContentSearchState` sub-phases.

4. **Selection stability**: Tracks selected session by file path (unique key) during merge, finds new index after.

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Merge flicker during result interleaving | Low | Selection stability by file path (D3) |
| 10ms metadata ceiling too tight | Low | Current filter is already instant; validate in T029 |
| Thread cancellation not prompt enough | Low | Per-file AtomicBool check; reduce interval if needed (T021) |

## Suggested Review Focus

- **plan.md**: Design decisions D1-D6 and their rationale
- **data-model.md**: New types (ContentSearchState, DisplayEntry, MatchType) and mode reduction
- **tasks.md**: Phase 3 (US1/MVP) as the critical path; parallel opportunities after US1

## Beads Sync

- 8 phases created, 30 tasks synced, 22 dependencies mapped
- Use `bd ready --json` to get next actionable tasks

## Next Steps

1. Review spec + plan artifacts
2. Approve or request changes
3. Begin implementation with `/speckit.implement` or `/sdd:beads-execute`
