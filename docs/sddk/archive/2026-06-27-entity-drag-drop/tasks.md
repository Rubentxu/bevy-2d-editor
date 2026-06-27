# Tasks: Entity Drag-and-Drop Reparenting

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 160-220 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | single PR |
| Delivery strategy | single-pr |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Foundation

- [ ] 1.1 Add CSS rules in `frontend/src/styles.css`: `.entity.dragging { opacity: 0.5 }`, `.entity.drag-over { outline: 2px solid #3b82f6 }`, `.hierarchy-root-zone.drag-over { background: rgba(59,130,246,0.08); outline: 2px dashed #3b82f6 }`.
- [ ] 1.2 Extend `Props` in `HierarchyPanel.tsx` — no new prop needed (helper is inlined per task 3.2); keep current `Props` as-is.
- [ ] 1.3 Add `const [draggedId, setDraggedId] = useState<string | null>(null)` and `const [dragOverId, setDragOverId] = useState<string | null>(null)` inside `HierarchyPanel`.

## Phase 2: Drag handlers on entity rows

- [ ] 2.1 On each `.entity` row in `HierarchyPanel.tsx`: add `draggable`, `onDragStart` (`e.dataTransfer.effectAllowed = "move"; setDraggedId(entity.id)`), `onDragEnd` (`setDraggedId(null); setDragOverId(null)`).
- [ ] 2.2 Add `onDragOver` (`e.preventDefault(); setDragOverId(entity.id)`) and `onDragLeave` (`if (e.currentTarget === e.target) setDragOverId(null)`) on each row.
- [ ] 2.3 Add `onDrop`: guard `draggedId && draggedId !== entity.id && scene.entities.some(e => e.id === draggedId)`, then `e.stopPropagation(); reparent(draggedId, entity.id)`.
- [ ] 2.4 Build `className` to include `"dragging"` when `draggedId === entity.id` and `"drag-over"` when `dragOverId === entity.id`; set inline `opacity: 0.5` when dragging.

## Phase 3: Root-drop zone + reparent helper

- [ ] 3.1 On the entity-list container `<div>` in `HierarchyPanel.tsx` (`data-testid="hierarchy-panel"`): add `onDragOver` (`e.preventDefault()`) and `onDrop` (`e.stopPropagation(); if (draggedId && scene.entities.some(e => e.id === draggedId)) reparent(draggedId, null)`).
- [ ] 3.2 Inside `HierarchyPanel`, define `function reparent(entityId: string, newParent: string | null)` that calls `window.dispatch_command(JSON.stringify({ command: { type: "ReparentEntity", entity_id: entityId, new_parent: newParent ?? undefined }, metadata: { authorship: "user", timestamp: Date.now() } }))`. Resolve `currentParentOf` via `scene.entities.find(e => e.id === entityId)?.parent`.
- [ ] 3.3 No `App.tsx` changes required — helper is self-contained inside `HierarchyPanel`. Cycle safety stays backend-owned (per proposal §Approach).

## Phase 4: E2E test

- [ ] 4.1 Create `frontend/tests/entity-drag-drop.spec.ts` with three tests using `page.dragAndDrop` between `[data-testid="hierarchy-entity-e2"]` and `[data-testid="hierarchy-panel"]`: (a) child → root reparents to `null`; (b) child → sibling reparents to sibling id; (c) self-drop is no-op. Verify via `await page.evaluate(() => (window as any).get_scene_snapshot())` and parse JSON to assert `entities.find(e=>e.id==="e2").parent === null`.
- [ ] 4.2 Seed scenes via `window.load_scene_json` with `e1` as root, `e2` with `parent: "e1"` for test (a); two siblings `e1`, `e2` both with `parent: null` for test (b).

## Phase 5: Verify

- [ ] 5.1 `cd frontend && npx tsc --noEmit` — zero type errors.
- [ ] 5.2 `cd frontend && npx playwright test tests/entity-drag-drop.spec.ts tests/rename-inline.spec.ts tests/smoke.spec.ts` — new + regression pass.