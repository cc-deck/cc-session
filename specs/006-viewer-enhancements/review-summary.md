# Review Summary: Conversation Viewer Enhancements

**Feature**: 006-viewer-enhancements
**Date**: 2026-03-08
**Spec Score**: 91/100 (PASS)

## Feature Overview

Five enhancements to the conversation viewer: strip system-injected content, full-width role header bars with message tinting, adaptive light/dark themes, markdown table grid rendering, and syntax-highlighted code blocks.

## Spec Highlights

- 5 user stories (2x P1, 2x P2, 1x P3), 13 functional requirements, 6 success criteria
- MVP is US1 (clean content), deliverable independently
- US3 (theme) is prerequisite for US2, US4, US5

## Plan Highlights

- **43 tasks** across 8 phases
- **3 new files**: `src/theme.rs`, `src/tui/table.rs`, `src/tui/syntax.rs`
- **3 new dependencies**: syntect 5.3, syntect-tui 3.0, termbg 0.6
- Binary size increase: ~3MB (syntect syntax definitions)
- **Key design**: Theme struct with dark/light variants, syntect for code highlighting, string-based system tag stripping (no new regex)

## Key Decisions for Reviewers

1. **System tag stripping**: String find/drain loop targeting known tag list, not regex. Strips tag + content for system tags only.
2. **Syntax highlighting**: syntect (same engine as bat) with syntect-tui bridge. Language detected from code fence tag.
3. **Theme detection**: termbg crate with 100ms timeout, CLI override flags.
4. **Table rendering**: Manual parser with Unicode box-drawing grid. No external markdown parser.
5. **Inspiration**: OpenCode's semantic syntax colors and visual role separation adapted for terminal TUI.

## Task Breakdown

| Phase | User Story | Tasks | Key Files |
|-------|-----------|-------|-----------|
| Setup | - | 4 | Cargo.toml, new modules |
| Foundational | - | 5 | theme.rs, mod.rs, view.rs |
| US1 (P1 MVP) | Clean content | 5 | session.rs, discovery.rs |
| US3 (P2) | Theme system | 7 | theme.rs, main.rs, view.rs |
| US2 (P1) | Visual headers | 4 | view.rs |
| US4 (P2) | Tables | 6 | table.rs, view.rs |
| US5 (P3) | Syntax highlight | 7 | syntax.rs, view.rs, mod.rs |
| Polish | - | 5 | All files |

## Suggested Review Focus

- **spec.md**: User stories and acceptance criteria
- **research.md**: Technology decisions (syntect, termbg, table approach)
- **data-model.md**: Theme struct design and processing pipeline changes
- **tasks.md**: Phase 3 (US1/MVP) as critical path, Phase 4 (US3/theme) as foundation for visual changes

## Next Steps

1. Review spec + plan artifacts
2. Begin implementation with `/speckit.implement` or `/sdd:beads-execute`
3. MVP deliverable after Phase 3 (US1: clean content)
