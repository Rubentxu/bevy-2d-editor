# Archive Report: entity-drag-drop

## Change Summary

**Change**: entity-drag-drop
**Archived**: 2026-06-27
**Mode**: greenfield (no openspec existed prior)

## Phase Artifacts

| Artifact | Status | Location |
|----------|--------|----------|
| proposal.md | ✅ | docs/sddk/entity-drag-drop/proposal.md |
| spec.md | ✅ | docs/sddk/entity-drag-drop/spec.md |
| tasks.md | ✅ | docs/sddk/entity-drag-drop/tasks.md |
| verify-report.md | ⚠️ NOT PRESENT | — |
| archive-report.md | ✅ | docs/sddk/entity-drag-drop/archive-report.md |

**Note**: `verify-report.md` was not present at archive time. No verification gate was recorded in the artifact registry.

## What Was Implemented

**entity-reparent-dnd** — HTML5 drag-and-drop reparenting in `HierarchyPanel`

### Behavior
- Drag an entity row → reduced opacity visual feedback
- Drop onto another entity → `ReparentEntity { entity_id, new_parent }` dispatched
- Drop onto panel background → `ReparentEntity { entity_id, new_parent: null }` (roots the entity)
- Self-drop guard (no-op, avoids round-trip)
- Root drop zone on panel container
- CSS visual feedback: `.dragging { opacity: 0.5 }`, `.drag-over { outline: 2px solid #3b82f6 }`

### Technical Approach
- Local state `draggedId` / `dragOverId` in `HierarchyPanel`
- `onDragStart` / `onDragEnd` / `onDragOver` / `onDrop` handlers on entity rows and panel container
- `onDrop` guard: entity must still exist in `scene.entities`
- `window.dispatch_command()` for `ReparentEntity` envelope
- Cycle safety is backend-owned via existing `would_create_cycle` in `processor.rs`
- No Rust changes — frontend-only

### Key Files
| File | Change |
|------|--------|
| `frontend/src/components/HierarchyPanel.tsx` | Added drag state, draggable rows, handlers, root-drop zone, visual feedback classes |
| `frontend/src/styles.css` | Added `.entity.dragging`, `.entity.drag-over`, `.hierarchy-root-zone.drag-over` |

### Test Results
No verify-report.md was generated. Tasks.md planned:
- Phase 5.1: `npx tsc --noEmit` — type check
- Phase 5.2: Playwright E2E (`entity-drag-drop.spec.ts`, `rename-inline.spec.ts`, `smoke.spec.ts`)

Test results were not recorded in the artifact registry.

## Spec Sync

**No openspec existed** — greenfield project.

The delta spec (`docs/sddk/entity-drag-drop/spec.md`) is promoted to the main spec:

```
docs/sddk/entity-drag-drop/spec.md
  → openspec/specs/entity-reparent-dnd/spec.md
```

| Domain | Action | Details |
|--------|--------|---------|
| entity-reparent-dnd | Created | 5 requirements: Drag Start, Drop Target Highlight, Drop on Entity Reparents, Drop on Root Reparents, Drop Guards |

## Knowledge Impact

- **Specs created**: `openspec/specs/entity-reparent-dnd/spec.md`
- **Specs made stale**: None
- **ADRs superseded**: None
- **Jurisprudence candidate**: No (verify-report not present, cannot assess)

## Entropy Trend

Not computed — `verify-report.md` absent, no entropy metrics recorded.

## Roadmap Update

The ROADMAP.md entry for "Entity drag-and-drop" under Medium Priority should be moved to Completed Milestones as v0.11.0.

```
Medium Priority (Hito 0 residual):
| **Entity drag-and-drop** | Reorder entities in hierarchy via drag |
```

This item is now implemented and archived.

## SDD Cycle Complete

The change has been planned and (according to the artifact chain) implemented. Verification report is missing — recommend running verification before closing the loop.

---

*Archive generated: 2026-06-27*
*Mode: greenfield (no pre-existing openspec)*
*Router context: not persisted (no verify-report)*