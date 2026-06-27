# entity-reparent-dnd Specification

## Purpose
Defines the user-visible behavior for reparenting entities by dragging them within the hierarchy panel. Drag-and-drop MUST commit through the existing `ReparentEntity` command so the operation is undoable, recorded in the Operation Log, and rejected by the backend when it would create a cycle. The frontend MUST defer all correctness checks (cycle detection, entity existence) to the backend and surface any rejection without mutating the document.

## Requirements

### Requirement: Drag Start
The hierarchy panel MUST allow a user to start dragging an entity row. While dragging, the row MUST show a reduced-opacity visual indicator and the panel MUST remember the dragged entity until the gesture ends.

#### Scenario: Starting a drag marks the source row
- GIVEN a scene with at least one entity visible in the hierarchy panel
- WHEN the user starts dragging an entity row
- THEN the row shows reduced opacity
- AND the panel records the dragged entity's stable id internally

#### Scenario: Drag start does not dispatch any command
- GIVEN an entity in the hierarchy panel
- WHEN the user starts dragging it
- THEN no command envelope is dispatched yet

### Requirement: Drop Target Highlight
While dragging, hovering over another entity row MUST show a drop-zone highlight on that row. Hovering over panel background (not on any row) MUST show a root-drop highlight on the container. Hovering outside the panel MUST show no highlight.

#### Scenario: Hovering over a row highlights that row
- GIVEN an entity is being dragged
- WHEN the user hovers over a different entity row
- THEN that target row shows a highlighted border or background
- AND the dragged row remains in its reduced-opacity state

#### Scenario: Hovering over panel background shows root-drop highlight
- GIVEN an entity is being dragged
- WHEN the user hovers over empty panel space (not over any row)
- THEN the panel container shows a root-drop highlight
- AND no entity row shows a drop-zone highlight

### Requirement: Drop on Entity Reparents
Dropping a dragged entity onto a different entity row MUST dispatch exactly one `ReparentEntity` command envelope with `entity_id`, `new_parent` set to the target entity's stable id, and `metadata.authorship = "user"`. The drop MUST clear all drag visuals.

#### Scenario: Drop onto another entity reparents it
- GIVEN entity A is being dragged and entity B is in the hierarchy panel
- WHEN the user drops A onto B's row
- THEN a `ReparentEntity` envelope is dispatched with `entity_id` = A and `new_parent` = B
- AND all drag visuals clear
- AND the tree visibly re-indents to reflect A being a child of B

### Requirement: Drop on Root Reparents
Dropping a dragged entity onto panel background (not on any row) MUST dispatch exactly one `ReparentEntity` envelope with `new_parent` omitted (root level).

#### Scenario: Drop onto panel background makes entity root-level
- GIVEN entity A is being dragged with parent B
- WHEN the user drops A onto empty panel space
- THEN a `ReparentEntity` envelope is dispatched with `entity_id` = A and `new_parent` omitted
- AND A appears at root level in the hierarchy
- AND all drag visuals clear

### Requirement: Drop Guards
The system MUST treat three cases as no-ops: drop on the dragged entity itself, drop on a non-existent entity, and drop that the backend rejects. A no-op MUST NOT mutate the document and MUST clear all drag visuals.

#### Scenario: Drop onto self is a no-op
- GIVEN entity A is being dragged
- WHEN the user drops A onto A's own row
- THEN no command envelope is dispatched
- AND all drag visuals clear
- AND the tree is unchanged

#### Scenario: Drop after backend cycle rejection is a no-op
- GIVEN entity A is being dragged and entity C is a descendant of A
- WHEN the user drops A onto C's row and the backend returns `WouldCreateCycle`
- THEN the document is unchanged
- AND the rejection is surfaced to the user
- AND all drag visuals clear

#### Scenario: Drag end without a valid drop clears state
- GIVEN an entity is being dragged
- WHEN the user releases the drag outside any valid drop target (e.g. presses Escape, drops off-panel)
- THEN no command envelope is dispatched
- AND all drag visuals clear

## E2E Acceptance (Playwright)
The following end-to-end scenario MUST pass against a built editor instance.

#### Scenario: Reparent an entity to root via drag-and-drop
- GIVEN a scene loaded with entity "Parent" (root) and entity "Child" (parented to Parent)
- WHEN the user drags the "Child" row onto the panel background
- THEN the hierarchy panel shows "Child" indented at root level (depth 0)
- AND no console errors occur
- AND pressing Ctrl+Z restores "Child" as a child of "Parent" (undo via Operation Log)

## Verification Notes
- Dispatch shape (frontend leaves `old_parent` omitted; the processor populates the actual previous parent during apply):
  ```json
  { "command": { "type": "ReparentEntity", "entity_id": "<id>", "new_parent": "<target|null>" },
    "metadata": { "authorship": "user", "timestamp": 0 } }
  ```
- `new_parent` omitted (or JSON `null`) means root level, matching the `Option<StableId>` + `skip_serializing_if = "Option::is_none"` shape in `crates/editor-core/src/command.rs` §54-60.
- Cycle safety is backend-owned: `ReparentEntity::validate()` rejects with `WouldCreateCycle` for self-parenting and descendant drops. The frontend does NOT replicate cycle detection; it dispatches and surfaces any `result.error`.
- Reparent reversibility is handled by the existing Operation Log machinery; this spec introduces no new persistence.
- This spec introduces no new commands, no Rust changes, and no backend migration.