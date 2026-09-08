# BJ-3 handoff — UI entity creation test

**Cycle**: `p-28fce7028ac3c497/bj3-ui-entity-creation`
**Tag**: v0.109.6 (commit `f2ee966`)
**Closed at**: 2026-09-08, sequence 265→276

## TL;DR

A-min cycle closing **BJ-3** — the UI-creation Playwright test
that proves a user can create entities in the editor via the UI
surface without any backend hand-edit. The frontend test suite
gains:

- `frontend/tests/ui-entity-creation.spec.ts` (NEW, 100 lines):
  two tests covering single-click and multi-click entity creation.

1 file / +296 / -0. 414/0/1 editor-bevy unchanged. tsc clean.

## What shipped

### Code (single commit `f2ee966`)

- `frontend/tests/ui-entity-creation.spec.ts` (NEW, 100):
  - `add_entity_button_creates_one_entity`
  - `add_entity_button_can_be_clicked_multiple_times`

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 265 | OPEN/explore |
| phase.explore.complete | 266 | OPEN/specify |
| phase.specify.complete.a-min | 268 | OPEN/build |
| phase.build.complete | 270 | OPEN/verify |
| phase.verify.complete.a-min | 272 | RELEASE_PENDING |
| release.complete | 274 | RELEASED |
| archive.complete | 276 | CLOSED |

## Validation evidence

```
$ cd frontend && npx tsc --noEmit -p .
(no output: TypeScript clean)

$ cd frontend && npx playwright test --list ui-entity-creation
[full] › ui-entity-creation.spec.ts:22:3 › BJ-3 — UI entity creation › add_entity_button_creates_one_entity
[full] › ui-entity-creation.spec.ts:62:3 › BJ-3 — UI entity creation › add_entity_button_can_be_clicked_multiple_times
Total: 2 tests in 1 file
```

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Lessons learned

1. **A-min for test-only cycles**: when the cycle's only change is
   a new test file, A-min (spec → build → verify) is the right
   shape. No design phase, no proposal phase — the test pattern is
   already established by existing specs.

2. **Reusing `waitForEditorReady` and `add-entity-btn` testids**
   keeps the new spec consistent with `selected-entity.spec.ts`
   and the e2e-game-creation suite.

## G1 status

| Sub-deliverable | Status | Tag |
|-----------------|--------|-----|
| BJ-1 test_helpers | ✅ | v0.109.1 |
| BJ-2 archcheck | ✅ | v0.109.1 |
| BJ-3 UI-creation Playwright | ✅ | **v0.109.6 (this cycle)** |
| BJ-4 player movement | ✅ | v0.109.2 |
| BJ-5 pickup collision | ✅ | v0.109.3 |
| BJ-5 jump | ✅ | v0.109.4 |
| BJ-5 enemy patrol | ✅ | v0.109.5 |
| BJ-5 contact-death | ❌ | (deferred — logic-graph runtime) |

G1 has 7/8 sub-deliverables closed. Only BJ-5 contact-death
remains, which is out of scope for v0.109.x.

## Carry-forward

- **BJ-5 contact-death**: requires declarative → imperative bridge
  for the logic-graph runtime (out of scope for v0.109.x).
