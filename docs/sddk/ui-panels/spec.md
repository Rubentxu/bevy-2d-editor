# Spec: UI Panels

> Change: `ui-panels` · Phase: sddk-spec (draft) · Path: A-lite

## §1. Spec Metadata

- **Change:** `ui-panels`
- **Phase:** spec (draft, awaiting design)
- **Path:** A-lite
- **Capabilities (NEW):**
  - `hierarchy-panel` — React component displaying entity tree with selection
  - `inspector-panel` — React component for entity components with edit
  - `ui-state-hooks` — React hooks for scene state, undo/redo state
  - `scene-snapshot-read` — `get_scene_snapshot()` wasm_bindgen
- **Source proposal:** [`docs/sddk/ui-panels/proposal.md`](../ui-panels/proposal.md)
- **Source explore:** [`docs/sddk/ui-panels/explore-report.md`](../ui-panels/explore-report.md)
- **Authoritative references:**
  - [Hito 0 §4 (Hierarchy Panel + Inspector Panel + Bevy integration)](../../hito-0-spec.md)
  - [Hito 0 §5.3 (Unidirectional command queue)](../../hito-0-spec.md)
  - [ADR-0002 — Single Bevy renders the canvas](../../adr/0002-single-bevy-renders-canvas.md)
  - [ADR-0001 — JSON source of truth](../../adr/0001-scene-document-json-as-source-of-truth.md)
  - Previous cycles: [`docs/sddk/scene-document/`](../scene-document/), [`docs/sddk/command-system/`](../command-system/), [`docs/sddk/operation-log/`](../operation-log/), [`docs/sddk/opfs-persistence/`](../opfs-persistence/)

---

## §2. Capability: `scene-snapshot-read`

### Requirement: get_scene_snapshot returns current SceneDocument

The system MUST provide `get_scene_snapshot() -> Option<SceneDocument>` that returns the current SceneDocument without mutating state.

#### Scenario: Returns scene with entities
- GIVEN a scene with 3 entities loaded
- WHEN `get_scene_snapshot()` is called
- THEN it returns `Some(SceneDocument)` with 3 entities

#### Scenario: Returns None when no scene loaded
- GIVEN no scene loaded (SCENE_DOC is None)
- WHEN `get_scene_snapshot()` is called
- THEN it returns `None`

### Requirement: get_scene_snapshot does not mutate state

The function MUST NOT mutate SceneDocument, operation log, or dirty flag.

#### Scenario: Read-only — operation log unchanged
- GIVEN scene with 5 entities, operation log empty
- WHEN `get_scene_snapshot()` is called twice
- THEN operation log remains empty

---

## §3. Capability: `hierarchy-panel`

### Requirement: Hierarchy Panel lists all entities

The Hierarchy Panel MUST display all entities from the current scene as a list with their names and stable IDs.

#### Scenario: Hierarchy shows all entities
- GIVEN scene with entities [A, B, C]
- WHEN hierarchy renders
- THEN all 3 entity names are visible

#### Scenario: Empty scene shows empty state
- GIVEN scene with 0 entities
- WHEN hierarchy renders
- THEN "No entities" message is displayed

### Requirement: Selecting entity highlights it

Clicking an entity in the Hierarchy MUST set it as selected, visually highlight it, and trigger Inspector to show its details.

#### Scenario: Click entity selects it
- GIVEN 3 entities
- WHEN user clicks entity B
- THEN B is highlighted in hierarchy
- AND Inspector shows B's components

#### Scenario: Click empty area deselects
- GIVEN entity B selected
- WHEN user clicks empty area in hierarchy
- THEN B is deselected
- AND Inspector shows "No selection"

---

## §4. Capability: `inspector-panel`

### Requirement: Inspector shows entity name and components

When an entity is selected, the Inspector MUST show the entity name (editable) and all its components.

#### Scenario: Inspector shows selected entity
- GIVEN entity "Player" with components [Name, Transform2D]
- WHEN user selects Player
- THEN Inspector shows name="Player" (editable) and 2 components

#### Scenario: Empty selection shows placeholder
- GIVEN no entity selected
- WHEN Inspector renders
- THEN "Select an entity" placeholder is shown

### Requirement: Rename Entity inline

Editing the entity name in Inspector and pressing Enter MUST dispatch `RenameEntity`.

#### Scenario: Rename updates name
- GIVEN entity "OldName" selected
- WHEN user changes name to "NewName" and presses Enter
- THEN `RenameEntity` is dispatched
- AND hierarchy shows "NewName"
- AND Inspector shows "NewName"

### Requirement: Editing Vec2 component field

Vec2 component fields (e.g., `editor.Transform2D.translation`) MUST display as 2 number inputs (x, y). Editing and pressing Enter MUST dispatch `SetComponentField`.

#### Scenario: Edit Vec2.x
- GIVEN entity with Transform2D.translation.x = 0
- WHEN user enters 100 in x field and presses Enter
- THEN `SetComponentField { field_path: "translation.x", value: 100 }` is dispatched
- AND field shows 100

### Requirement: Editing Color component field

Color fields (RGBA) MUST display as 4 number inputs.

#### Scenario: Edit Color.r
- GIVEN Sprite2D.color.r = 0
- WHEN user enters 0.5 and presses Enter
- THEN `SetComponentField { field_path: "color.r", value: 0.5 }` is dispatched

### Requirement: Editing Anchor component field

Anchor fields MUST display as a dropdown with all anchor variants.

#### Scenario: Select anchor from dropdown
- GIVEN Sprite2D.anchor = Center
- WHEN user selects "TopLeft" from dropdown
- THEN `SetComponentField { field_path: "anchor", value: "TopLeft" }` is dispatched

### Requirement: Editing F32 component field

F32 fields MUST display as number input.

### Requirement: Editing Bool component field

Bool fields MUST display as checkbox.

### Requirement: Editing String component field

String fields MUST display as text input.

### Requirement: Add Component button

An "Add Component" button MUST appear in Inspector when entity is selected. Clicking opens a dropdown of all schemas from combined_registry.

#### Scenario: Add Transform2D component
- GIVEN entity with no Transform2D
- WHEN user clicks "Add Component" → selects "Transform2D"
- THEN `AddComponent { type_id: "editor.Transform2D", values: <defaults> }` is dispatched
- AND component appears in Inspector

### Requirement: Remove Component button

Each component in Inspector MUST have a "Remove" button. Clicking dispatches `RemoveComponent`.

#### Scenario: Remove Transform2D
- GIVEN entity with Transform2D
- WHEN user clicks "Remove" on Transform2D
- THEN `RemoveComponent { type_id: "editor.Transform2D" }` is dispatched
- AND component disappears from Inspector

---

## §5. Capability: `ui-state-hooks`

### Requirement: useSceneState provides scene + refresh + dispatch

The `useSceneState` React hook MUST provide:
- `scene: SceneDocument | null` — current state
- `refresh()` — re-reads from WASM
- `dispatch(envelope)` — dispatches command and refreshes

#### Scenario: dispatch updates scene state
- GIVEN scene with 1 entity
- WHEN useSceneState.dispatch({command: {type: "CreateEntity", ...}})
- THEN scene has 2 entities after dispatch

#### Scenario: refresh reads current state
- GIVEN scene modified externally (via wasm_bindgen directly)
- WHEN useSceneState.refresh()
- THEN scene reflects the external modification

### Requirement: useLogState provides undo/redo state

The `useLogState` hook MUST provide:
- `canUndo: boolean`
- `canRedo: boolean`
- `size: number`

#### Scenario: canUndo reflects log state
- GIVEN 2 commands dispatched
- WHEN useLogState called
- THEN `canUndo` is true

#### Scenario: canUndo false on empty log
- GIVEN 0 commands in log
- WHEN useLogState called
- THEN `canUndo` is false

---

## §6. UI Layout

### Requirement: 3-column layout

The App MUST render 3 columns: Hierarchy (left, ~250px) | Canvas (flex) | Inspector (right, ~300px). Plus a top bar (~50px).

#### Scenario: Layout renders correctly
- WHEN App mounts
- THEN 3 columns + top bar are visible
- AND canvas is in the center, hierarchy left, inspector right

### Requirement: Undo/Redo buttons in top bar

Top bar MUST have Undo and Redo buttons. Disabled when can_undo / can_redo is false.

#### Scenario: Undo button works
- GIVEN canUndo = true
- WHEN user clicks Undo
- THEN scene reverts to previous state

#### Scenario: Undo button disabled when no history
- GIVEN canUndo = false
- WHEN top bar renders
- THEN Undo button is disabled (greyed out)

### Requirement: Save/Load Project buttons in top bar

Top bar MUST have Save Scene, Load Project buttons.

#### Scenario: Save Scene prompts for name
- GIVEN scene with entities
- WHEN user clicks Save
- THEN browser prompt appears for scene name
- AND on submit, scene is saved to OPFS

---

## §7. Out-of-Scope Behaviors (explicit non-goals)

- Reparent drag-and-drop
- Bulk operations (multi-select)
- Template instantiation UI
- Schema authoring UI
- Asset preview
- Code editor integration
- Live preview while typing (debounce is OK)
- Undo/redo for UI state (selection, form inputs)

---

## §8. Acceptance Criteria

1. Every §2-§6 scenario passes via Playwright E2E tests.
2. 3-column layout renders correctly.
3. Hierarchy shows entities, supports selection.
4. Inspector edits fields with appropriate widgets per type.
5. Add/Remove component buttons work.
6. Undo/Redo buttons work via UI.
7. Save/Load Project buttons work.
8. All 23 existing Playwright tests pass (no regression).
9. All 112 existing Rust tests pass (no regression).
10. 3 new Playwright UI tests pass.
11. WASM builds clean.

---

## §9. Test Plan

| Section | Scenarios | Test type | Rough count |
|---|---|---|---|
| §2 snapshot | with/without entities, no mutation | Rust unit + E2E | 3 |
| §3 hierarchy | list, select, empty | Playwright E2E | 2 |
| §4 inspector | view, rename, edit Vec2/Color/Anchor, Add, Remove | Playwright E2E | 3 |
| §5 hooks | dispatch, refresh | Playwright E2E | (covered above) |
| §6 layout | render, undo/redo, save | Playwright E2E | 2 |
| **Total** | | | **~10 tests** |

Dev cycle: `cargo test --lib` (harness) + `just wasm` + `just test`.