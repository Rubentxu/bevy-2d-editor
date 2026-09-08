# BJ-3 — UI entity creation test (implementation-receipt)

**Cycle**: `p-28fce7028ac3c497/bj3-ui-entity-creation`
**Sequence**: 268 (2026-09-08)
**Path**: A-min
**Phase**: build

## Files created

```
frontend/tests/ui-entity-creation.spec.ts  NEW (100 lines, 2 tests)
```

## Implementation

`frontend/tests/ui-entity-creation.spec.ts`:
- Test 1: `add_entity_button_creates_one_entity` — load clean editor,
  assert 0 entities, click `+ Add Entity`, wait until snapshot
  shows 1 entity, assert Hierarchy panel renders
  `[data-testid="hierarchy-entity-${id}"]`.
- Test 2: `add_entity_button_can_be_clicked_multiple_times` — click
  `+ Add Entity` 3 times, wait until snapshot shows 3 entities,
  assert all 3 are rendered with distinct ids.
- `@full` cohort (matches `e2e-game-creation.spec.ts` convention).

Reuses:
- `waitForEditorReady` from `frontend/tests/helpers/`.
- `add-entity-btn` testid from `HierarchyPanel.tsx`.
- `get_scene_snapshot` bridge export from engine-bridge.ts.
- `hierarchy-entity-${id}` testid from `HierarchyPanel.tsx`.

## Verification

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

## Acceptance

✓ `npx tsc --noEmit -p .` clean
✓ `npx playwright test --list ui-entity-creation` registers both tests
✓ `cargo test -p editor-bevy --lib` unchanged at 414/0/1
✓ `bun run tools/archcheck/check.ts` green
