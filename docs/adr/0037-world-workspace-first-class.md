# ADR-0037: World Workspace Is a First-Class Product Context

## Status

Accepted + Implemented — 2026-08-18

## Implementation

- **Version**: v0.95.0
- **Implementation date**: 2026-08-18
- **Branch**: `feat/world-workspace-slice-5-recipes-ldtk` (merged to `origin/main` via stacked PRs)
- **Commits**: 11 commits across 5 slices (6531d54 → 646f503)

### Slice Structure

| Slice | Name | Commits |
|-------|------|---------|
| 1 | Model + Persistence | 6531d54 |
| 2 | Capability + Commands | 99d78c3, 00098af |
| 3 | WASM + Frontend Gateway | b68610d, fcc2dbf |
| 4 | UI | b9015c7 |
| 5 | Recipes + LDtk Bridge | a0a16dd, e566f29, d671b8b, 01cacdd, 6048f2a, 646f503 |

### Verification

- **Spec scenarios passed**: 12/12
- **Tasks complete**: 41/41
- **Tests**: 606/607 pass (1 pre-existing failure: `validation_center_tests::wasm_validation_cycle_in_active_graph`)
- **Architecture invariants upheld**: ADR-0037 line 14, ADR-0031, ADR-0032, ADR-0033, ADR-0040, ADR-0046

### Known Deviations

- **D1**: World WASM exports live in `editor-bevy/src/lib.rs` per design decision (WorldApi re-uses RuntimeApi's direct-export pattern); `editor-wasm` typed-backend wrappers intentionally skipped
- **D2**: Menu "View → World Workspace" falls through to `todo()` — ModeContextBar provides functional workaround
- **D3**: 4 prettier format issues on world-related files (trivial whitespace)


## Context

2D games frequently need relationships among levels/rooms that are awkward to express through a flat asset list. LDtk-style world authoring is highly productive for platformers, metroidvanias, RPGs and room-based games.

## Decision

Add a `WorldDocument`/World Workspace that **references existing Level Scene Assets**. It does not create a second level-content model.

A world owns:

- level placement/layout metadata;
- neighbour/portal relationships;
- world-space position/dimensions;
- entrances/exits/spawn links;
- streaming/load policy metadata;
- validation of topology and references.

Supported layout policies begin with `Free`, `Grid`, `Horizontal`, `Vertical`; future policies are additive.

## Consequences

The editor gains a differentiated 2D production surface while keeping Level Scene Asset as the unit of level content.
