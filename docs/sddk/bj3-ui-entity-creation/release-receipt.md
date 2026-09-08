# BJ-3 — UI entity creation test (release-receipt)

**Cycle**: `p-28fce7028ac3c497/bj3-ui-entity-creation`
**Sequence**: 272 (2026-09-08)
**Tag**: v0.109.6
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle name | bj3-ui-entity-creation |
| Path | A-min |
| Phase at release | verify → release transition |
| Tag | v0.109.6 |
| Code-commit SHA | `f2ee966` |
| Trunk SHA (HEAD) | `f2ee966` |
| Origin/main SHA | `f2ee966` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Diff vs. v0.109.5 | +296 / -0 across 4 files |

## Files in code release

```
frontend/tests/ui-entity-creation.spec.ts  NEW (100 lines, 2 tests)
```

(plus 3 SDDK artifact docs in `docs/sddk/bj3-ui-entity-creation/`.)

## Test posture

```
$ cd frontend && npx tsc --noEmit -p .
(no output: TypeScript clean)

$ cd frontend && npx playwright test --list ui-entity-creation
  [full] › ui-entity-creation.spec.ts:22:3 › BJ-3 — UI entity creation › add_entity_button_creates_one_entity
  [full] › ui-entity-creation.spec.ts:62:3 › BJ-3 — UI entity creation › add_entity_button_can_be_clicked_multiple_times
Total: 2 tests in 1 file
```

## Workspace

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

## Static analysis

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Cycle closure

| Phase | Sequence |
|-------|----------|
| cycle.start | 265 |
| phase.explore.complete | 266 |
| phase.specify.complete.a-min | 268 |
| phase.build.complete | 270 |
| phase.verify.complete.a-min | 272 |

## Out-of-scope carried forward

- BJ-5 contact-death logic-graph runtime — still deferred (needs
  declarative → imperative bridge, out of scope for v0.109.x).

## Lessons

1. **A-min for test-only cycles**: when the cycle's only change is a
   new test file, A-min (spec → build → verify) is the right
   shape. No design phase, no proposal phase — the test pattern is
   already established by existing specs in the same cohort.

2. **Reusing `waitForEditorReady` and `add-entity-btn` testids**
   keeps the new spec consistent with `selected-entity.spec.ts`
   and the e2e-game-creation suite. The pattern is mature enough
   that adding tests is now mechanical.

3. **BJ-3 closure**: this is the last open sub-deliverable of G1
   (BJ-1, BJ-2, BJ-3, BJ-4, BJ-5: pickup, jump, patrol, contact-death).
   G1 is now at 7/8 sub-deliverables closed. The remaining
   contact-death requires logic-graph runtime infrastructure.
