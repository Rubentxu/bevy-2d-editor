# Tasks: Inline Entity Rename in Hierarchy Panel

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~130 (≤40 prod + ~90 test) |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | single PR |
| Delivery strategy | single-pr |
| Chain strategy | size-exception |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Low

## Phase 1: Foundation (Props Contract)

- [ ] 1.1 Add `onRename: (entityId: string, newName: string) => void` to the `Props` interface in `frontend/src/components/HierarchyPanel.tsx` — same signature already used by `InspectorPanel.tsx` line 9.

## Phase 2: Core Implementation (Inline Edit State)

- [ ] 2.1 In `frontend/src/components/HierarchyPanel.tsx`, import `useState` from React and add two local state vars at top of the component: `const [editingId, setEditingId] = useState<string | null>(null)` and `const [editValue, setEditValue] = useState("")`.
- [ ] 2.2 Define a `commit(entity)` helper inside `HierarchyPanel`: no-op if `editingId !== entity.id`, `editValue.trim() === ""`, or `editValue === entity.name`; otherwise call `onRename(entity.id, editValue)`, then `setEditingId(null)`.
- [ ] 2.3 In `frontend/src/components/HierarchyPanel.tsx` lines 61–62, replace the static `<span className="name">{entity.name}</span>` with a branch: when `editingId === entity.id`, render `<input data-testid="hierarchy-rename-input" autoFocus value={editValue} onChange={(e) => setEditValue(e.target.value)} onBlur={() => commit(entity)} onKeyDown={(e) => { if (e.key === "Enter") commit(entity); else if (e.key === "Escape") setEditingId(null); }} onClick={(e) => e.stopPropagation()} />`; otherwise render `<span className="name" onDoubleClick={(e) => { e.stopPropagation(); setEditingId(entity.id); setEditValue(entity.name); }}>{entity.name}</span>`.
- [ ] 2.4 Guard the entity map with `if (editingId !== null && !scene.entities.some((e) => e.id === editingId)) setEditingId(null)` in a `useEffect` so a stale id clears if the entity disappears mid-edit.

## Phase 3: Integration (App Wiring)

- [ ] 3.1 In `frontend/src/App.tsx` lines 156–160, pass `onRename={handleRename}` to `<HierarchyPanel>`. Reuse existing `handleRename` (lines 79–85), which already dispatches `{ type: "RenameEntity", entity_id, new_name }` via the typed command bus — no App.tsx logic change beyond the prop.

## Phase 4: Verification

- [ ] 4.1 Create `frontend/tests/rename-inline.spec.ts`: load empty scene via `load_scene_json`, dispatch `CreateEntity` "e1", await `[data-testid="hierarchy-entity-e1"]`, double-click `.name`, fill `[data-testid="hierarchy-rename-input"]` with "Renamed", press Enter, assert the `.name` text equals "Renamed" and `get_scene_snapshot` reports the new name.
- [ ] 4.2 Add an Escape-cancel test in the same spec: open editor, press Escape, assert `get_log_state().size` unchanged (no `RenameEntity` dispatched).
- [ ] 4.3 Add an unchanged-name no-op test in the same spec: double-click, press Enter without typing, assert `get_log_state().size` unchanged.
- [ ] 4.4 Run `cd frontend && npx tsc --noEmit` — must exit 0.
- [ ] 4.5 Run `cd frontend && npx playwright test tests/smoke.spec.ts tests/keyboard-shortcuts.spec.ts tests/delete-key.spec.ts tests/anchor-sync.spec.ts` — all must still pass (regression gate).
- [ ] 4.6 Run the new spec `cd frontend && npx playwright test tests/rename-inline.spec.ts` — all 3 tests must pass.