# Tasks: Delete Key Shortcut for Entity Removal

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~150 (hook +25, App +20, test +100) |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | single PR |
| Delivery strategy | single-pr |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: stacked-to-main
400-line budget risk: Low

## Phase 1: Hook Restructure

- [ ] 1.1 Extend `UseKeyboardShortcutsOptions` in `frontend/src/hooks/useKeyboardShortcuts.ts` with `selectedEntityId: string | null` and `onDelete: (id: string) => void`.
- [ ] 1.2 Move input-guard (`target.closest("input,textarea,[contenteditable=\"true\"]")`) above the `if (!modKey) return;` line so it gates both modifier and bare-key gestures.
- [ ] 1.3 Add bare-key branch after the input-guard: if `e.key === "Delete" || e.key === "Backspace"`, `preventDefault()`, gate on `selectedEntityId !== null`, call `onDelete(selectedEntityId)`.
- [ ] 1.4 Add `selectedEntityId` and `onDelete` to the `useEffect` dependency array in `frontend/src/hooks/useKeyboardShortcuts.ts`.

## Phase 2: App Wiring

- [ ] 2.1 Add `handleDeleteEntity(id: string)` async handler in `frontend/src/App.tsx` that `await dispatch({ command: { type: "DeleteEntity", id }, metadata: { authorship: "user", timestamp: Date.now() } })` and clears `selectedEntityId` via `setSelectedEntityId(null)`.
- [ ] 2.2 Pass `selectedEntityId` and `onDelete: handleDeleteEntity` to `useKeyboardShortcuts` call in `frontend/src/App.tsx` L127.

## Phase 3: Verification

- [ ] 3.1 Run `cd frontend && npx tsc --noEmit` to confirm no TS regressions in hook or App.

## Phase 4: E2E Test

- [ ] 4.1 Create `frontend/tests/delete-shortcut.spec.ts` with test "Delete key removes selected entity": load scene, dispatch `CreateEntity` for `del-e1`, wait for `[data-testid="hierarchy-entity-del-e1"]`, click to select, save `tests/baselines/delete-key-before.png`, press `Delete`, assert entity hidden, save `delete-key-after.png`, assert `get_scene_snapshot().entities.length === 0`.
- [ ] 4.2 Add second test "Delete key is no-op when focus is in input": create entity, click to select, focus `input.entity-name`, press `Delete`, assert entity still visible and `can_undo` unchanged.
- [ ] 4.3 Add third test "Delete key does nothing when no entity selected": empty scene, press `Delete`, assert no error and scene unchanged.
- [ ] 4.4 Run `cd frontend && npx playwright test delete-shortcut.spec.ts` — all three tests pass.

## Phase 5: Regression

- [ ] 5.1 Run `cd frontend && npx playwright test keyboard-shortcuts.spec.ts` to confirm undo/redo/guard tests still pass (no regression from hook restructure).