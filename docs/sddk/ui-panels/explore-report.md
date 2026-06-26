# Explore Report: ui-panels

> Change: `ui-panels` · Phase: sddk-explore · Path: A-lite · Context quality: C1
> Model: MiniMax-M3 (orchestrator)

---

## 1. Current State

### 1.1 Previous cycles delivered
- **scene-document**: SceneDocument with entities, components, hierarchy
- **command-system**: typed Command enum (9 variants) + processor + dispatch
- **operation-log**: undo/redo
- **opfs-persistence**: save/load scene
- **schema-registry-persistence**: mutable schemas
- **entity-template-persistence**: templates with tree instantiation

### 1.2 Current frontend state

`frontend/src/App.tsx` is the minimal spike:
- Single canvas + sidebar with X/Y inputs + Move Sprite button
- Listens to sprite position via LinearBus events
- Uses `sendMoveSprite` (raw LinearBus command) and `dispatch_command` (typed)

### 1.3 Frontend bridge surface

`engine-bridge.ts` exposes 22+ functions on `window`:
- Scene: `load_scene_json`, `dispatch_command`, `undo`, `redo`, `get_log_state`
- OPFS: `save_scene`, `load_scene`, `list_scenes`, `project_exists`, `load_project`
- Schemas: `save_schema`, `load_schema`, `delete_schema`, `list_schemas`, `register_schema`, `unregister_schema`, `is_builtin_type`, `combined_registry_size`
- Templates: `save_template`, `load_template`, `delete_template`, `list_templates`, `is_template_loaded`
- OPFS bridge: `opfs_save_file`, `opfs_load_file`, etc.

### 1.4 What's missing for UI panels

| Need | Current state | Gap |
|------|---------------|-----|
| Hierarchy panel | None | Need React component showing entity tree |
| Inspector panel | None | Need React component showing entity components |
| Read scene snapshot | None — UI must dispatch command to read | Need `get_scene_snapshot()` non-mutating read |
| Selection state | None | Need local React state for selected entity ID |
| Dispatch + refresh flow | Manual | Need React hook to dispatch and refresh |
| Project metadata UI | None | Optional: name display, save/load buttons |

### 1.5 Bevy 0.19 + WASM considerations

- Single-threaded WASM (no React rerender race)
- All state in `SCENE_DOC` thread_local
- `dispatch_command` is sync (not async)
- `load_scene`/`save_scene`/`load_project` are async (OPFS I/O)

---

## 2. Gap Analysis — UI Panel Requirements

Per Hito 0 §4 + previous cycle decisions:

**Hierarchy Panel:**
- Display all entities (name + ID) in tree view (root → children)
- Click entity to select
- Visual indicator for selection
- Optional: drag-and-drop to reparent (deferred — reparent via right-click or button)

**Inspector Panel:**
- Display selected entity's components
- Show component type_id and values (formatted)
- Allow editing values (dispatch `SetComponentField`)
- Allow adding components (dispatch `AddComponent`)
- Allow removing components (dispatch `RemoveComponent`)

**Top bar / Status:**
- Show "Bevy running" status
- Show operation log state (can_undo, can_redo)
- Project name display
- Save / Load / New Project buttons

### 2.1 MVP scope

For Hito 0 MVP, focus on:
- Hierarchy panel: list + select
- Inspector panel: view + edit component values
- Undo/Redo buttons
- Save scene / Load scene buttons
- NO reparent drag-and-drop, NO bulk operations, NO template UI

---

## 3. Binding Constraints (from Hito 0 + ADR-0001 + ADR-0002)

1. **React never owns document state** (ADR-0002) — only UI state (selection, form inputs)
2. **Unidirectional commands** (Hito 0 §5.3) — UI dispatches commands, never mutates SceneDocument directly
3. **JSON source of truth** (ADR-0001) — SceneDocument shape unchanged
4. **Operation Log for undo/redo** — UI uses existing undo/redo functions
5. **Single Bevy canvas** (ADR-0002) — React doesn't render entities, just panels

---

## 4. Codebase Risks

### 4.1 Read snapshot on every render (Medium)

If UI re-renders Hierarchy on every Bevy frame, performance degrades.

**Mitigation:** UI refreshes on user actions (select, edit, dispatch). Use React state for local cache. Refresh snapshot only after dispatch.

### 4.2 Selection state sync (Low)

Selection in Hierarchy must sync with Inspector focus. Use single `selectedEntityId` React state lifted to App.

**Mitigation:** Standard React pattern with `useState` at App level, pass down via props.

### 4.3 Component editing schema validation (Low)

Inspector edits component values via `SetComponentField`. Validation happens server-side. Show error on dispatch failure.

**Mitigation:** Use dispatch result error to display error message in Inspector.

### 4.4 Concurrent dispatch (Low)

User clicks "Add Component" twice rapidly. Each dispatch is sync; second waits for first. No race.

**Mitigation:** None needed (single-threaded).

### 4.5 Large component values (Low)

Some component values might be large JSON. Inspector renders as JSON tree.

**Mitigation:** Use `<details>` for nested objects, simple text input for primitives.

### 4.6 Fresh snapshot after dispatch (Low)

UI calls `dispatch_command` and gets back snapshot. Use that for re-render.

**Mitigation:** Always re-render with `result.snapshot`. No additional fetch.

### 4.7 Missing `get_scene_snapshot()` (Medium)

Currently no way to read scene without mutating. Adding new wasm_bindgen is cleanest.

**Mitigation:** Add `get_scene_snapshot() -> JsValue` in lib.rs. Returns `SceneDocument` JSON or null if no scene loaded.

---

## 5. Effort Estimate

| Work item | Size | Notes |
|-----------|------|-------|
| `get_scene_snapshot()` wasm_bindgen in lib.rs | XS | New function returning SceneDocument JSON |
| Hierarchy panel React component | M | List + select + tree structure |
| Inspector panel React component | M | View components + edit values |
| App layout: 3-column (canvas / hierarchy / inspector) | S | CSS grid |
| Top bar: project name + undo/redo + save/load buttons | S | Status display |
| React hook for dispatch + refresh | S | `useSceneState` |
| Component value editing UI | M | Text inputs, nested objects |
| Add/Remove component UI | M | Buttons + dropdown for type_id |
| Tests: Playwright E2E | M | 2-3 tests for UI workflows |

**Total:** Medium. ~600 LOC across Rust + TS.

---

## 6. Architecture Decisions Needed (for design phase)

1. **Snapshot read mechanism** — New `get_scene_snapshot()` wasm_bindgen vs reuse `dispatch_command` with no-op. Recommend new function.
2. **React state management** — Plain useState vs useReducer vs Zustand. For MVP, plain useState at App level + props.
3. **Hierarchy display** — Flat list with indent vs nested <ul>. Flat + indent simpler.
4. **Inspector editing** — Per-field inputs vs JSON textarea. Per-field more usable but more code.
5. **Component value types** — Vec2: 2 inputs, Color: 4 inputs (rgba), Anchor: dropdown, others: text input.
6. **Add Component dropdown** — Use combined_registry list. Built-ins + user schemas.
7. **Layout** — 3-column CSS grid or flexbox. Flexbox simpler.
8. **Refresh after dispatch** — `useSceneState` hook returns `{scene, dispatch}` and refreshes on dispatch.

---

## 7. Recommendations for Proposal

1. **Capabilities (NEW):**
   - `hierarchy-panel` — React component displaying entity tree with selection
   - `inspector-panel` — React component displaying selected entity's components with edit
   - `ui-state-hooks` — React hooks for dispatch + refresh (useSceneState, useUndoRedoState)

2. **Approach:**
   - Add `get_scene_snapshot()` wasm_bindgen for non-mutating read
   - Replace single sidebar with 3-column layout: Hierarchy | Canvas | Inspector
   - Top bar with project status + undo/redo + save/load buttons
   - Lift `selectedEntityId` state to App
   - Hooks: `useSceneState()` returns scene + refresh + dispatch helpers

3. **Reuse existing:** `dispatch_command`, `undo`, `redo`, `get_log_state`, `save_scene`, `load_scene`, `list_scenes`, `load_project`, `combined_registry`

4. **Component editing:**
   - Built-in editors per FieldType: Vec2 (2 inputs), Color (4 inputs), Anchor (dropdown), F32 (number), Bool (checkbox), String (text), AssetReference (text)
   - Add Component button → dropdown of all schemas
   - Remove Component button → removes via `RemoveComponent` command

5. **Tests:**
   - Playwright E2E: open App, see default scene in hierarchy, click entity, see components in inspector, edit a value, verify scene changed
   - Playwright E2E: undo/redo buttons work via UI
   - Playwright E2E: save/load project via UI

6. **Backward compat:** All existing 23 Playwright tests pass unchanged. Spike UI (X/Y inputs) replaced by panels.