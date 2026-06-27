# Proposal: Entity Drag-and-Drop Reparenting in Hierarchy Panel

## Intent
Entity hierarchy today can only be reparented via direct command dispatch — there is no in-canvas affordance. Drag-and-drop is the canonical editor gesture for restructuring a tree: the user drags an entity row onto another entity (or onto the panel background to root it) and the editor dispatches the existing `ReparentEntity` command so the move is undoable and recorded in the Operation Log. This unblocks a core Hito 0 scene-organization interaction. Frontend-only; no Rust changes.

## Scope

### In Scope
- HTML5 drag-and-drop on entity rows in `HierarchyPanel` (no external library)
- Drag an entity → drop onto another entity → `ReparentEntity { entity_id, new_parent }` dispatched
- Drop onto panel background (not on any row) → `ReparentEntity { entity_id, new_parent: null }` (root)
- Visual feedback: dragged row opacity reduction, target row highlight, root-drop zone highlight
- Edge-case guards: drop onto self (no-op), drop onto descendant (backend rejects via `WouldCreateCycle`), dropped entity vanished mid-drag (no-op)

### Out of Scope
- Drag-to-reorder among siblings (positional ordering — separate change)
- Multi-entity drag (no multi-select yet)
- Touch / pointer events (mouse-only for Hito 0)
- Any Rust/backend changes (`ReparentEntity`, cycle validation, and tests already exist)

## Capabilities

> CONTRACT with sddk-spec. No `openspec/specs/` exists yet — greenfield, matching prior changes.

### New Capabilities
- `entity-reparent-dnd`: User-facing behavior of restructuring the entity tree by dragging an entity in the hierarchy panel and dropping it onto a target entity or the panel root, committed through the `ReparentEntity` command (undoable, logged, cycle-safe via backend validation).

### Modified Capabilities
None.

## Approach
- Add local state `draggedId: string | null` and `dragOverId: string | null` in `HierarchyPanel`.
- New prop `onReparent(entityId, newParentId | null)` passed from `App.tsx`, which already holds `dispatch` from `useSceneState` (same pattern as `onRename`, `onDeleteEntity`).
- Each `.entity` row becomes `draggable`; `onDragStart` sets `draggedId` and `dataTransfer.effectAllowed = "move"`.
- `onDragOver` on a row: `preventDefault()` (enables drop), set `dragOverId`. On the panel container: same, with a sentinel `dragOverId = "__root__"`.
- `onDrop` resolution:
  - Guard: `draggedId` must still exist in `scene.entities` (500ms poll may have swapped the scene).
  - Drop on row `T` where `T === draggedId` → no-op.
  - Drop on row `T` (different entity) → `onReparent(draggedId, T)`.
  - Drop on panel background (`__root__`) → `onReparent(draggedId, null)`.
  - Clear `draggedId` / `dragOverId` in `onDragEnd` (fires on both success and cancel).
- `onReparent` dispatches:
  ```json
  { "command": { "type": "ReparentEntity", "entity_id": "<id>", "new_parent": "<target|omitted>" },
    "metadata": { "authorship": "user", "timestamp": <ms> } }
  ```
  (`old_parent` omitted — the processor populates `actual_old` at apply time, line 287.)
- **Cycle safety is backend-owned**: `validate()` (`processor.rs` lines 140–151) calls `would_create_cycle` and returns `WouldCreateCycle` for self-parenting or descendant drops. The frontend does NOT replicate cycle detection — it dispatches and surfaces any `result.error`. Client-side self-drop guard is a UX nicety (avoids a redundant round-trip), not a correctness requirement.
- Visual feedback via conditional `className` / inline `opacity` driven by `draggedId` and `dragOverId` state.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `frontend/src/components/HierarchyPanel.tsx` | Modified | Add `draggedId`/`dragOverId` state, `draggable` rows, drag/drop handlers, root-drop zone on panel container, visual-feedback classes, new `onReparent` prop |
| `frontend/src/App.tsx` | Modified | Add `handleReparent` (wraps `dispatch`) and pass as `onReparent` into `HierarchyPanel` |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Drop event fires on inner `<span>` not row `<div>` | Medium | Attach handlers to the row `<div>`; `onDragOver`/`onDrop` bubble up from children |
| Panel-background drop wrongly catches row drops | Medium | Row `onDrop` calls `stopPropagation`; panel `onDrop` only fires when event reaches container |
| 500ms poll swaps scene mid-drag (entity deleted) | Low | Existence guard in `onDrop` before dispatch; `onDragEnd` always clears state |
| HTML5 DnD quirks: `dragover` must `preventDefault` or drop won't fire | Medium | `preventDefault` in every `onDragOver`; standard documented requirement |
| Rename input inside row interferes with drag start | Low | Input is not `draggable`; drag starts on the row but text selection in input is unaffected |

## Rollback Plan
Revert changes to `HierarchyPanel.tsx` and `App.tsx`. No schema, persistence, or backend migration — `ReparentEntity` and its cycle-validation predate this change.

## Dependencies
- Existing `ReparentEntity` command in `crates/editor-core/src/command.rs` (present, line 54)
- Existing cycle validation `would_create_cycle` + `validate()` arm in `crates/editor-core/src/processor.rs` (present, lines 68 & 140)
- `dispatch` from `useSceneState` hook (present, used by `handleRename` in `App.tsx`)

## Success Criteria
- [ ] Entity row is draggable; cursor and ghost reflect the drag
- [ ] Drop onto another entity → `ReparentEntity` dispatched, tree re-indents, indentation depth updates
- [ ] Drop onto panel background → entity becomes root-level (`new_parent: null`)
- [ ] Drop onto self → no dispatch, no-op
- [ ] Drop onto own descendant → backend returns `WouldCreateCycle`, no tree mutation, error surfaced
- [ ] Dragged entity deleted mid-drag (poll refresh) → drop is a no-op, state cleared
- [ ] Reparent is undoable (Operation Log reverses it)