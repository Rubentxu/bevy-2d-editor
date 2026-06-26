# Proposal: UI Panels for Hito 0

## Intent

Hito 0 §4 calls for Hierarchy Panel + Inspector Panel, but only a minimal spike UI exists (canvas + X/Y inputs). Without real panels, users cannot see/edit the SceneDocument tree, browse components, undo/redo via buttons, or save/load projects interactively. This change delivers a 3-column React layout (Hierarchy | Canvas | Inspector), a top bar with project status + undo/redo + save/load buttons, and a `useSceneState` hook for clean component-to-WASM wiring.

## Scope

### In Scope
- `get_scene_snapshot()` wasm_bindgen for non-mutating read
- 3-column CSS layout: Hierarchy Panel | Canvas | Inspector Panel
- Top bar: project name + undo/redo buttons + save/load buttons
- Hierarchy Panel: entity list with selection, indented tree
- Inspector Panel: components display, field editors per FieldType, Add/Remove component
- `useSceneState` React hook (scene + refresh + dispatchCommand wrapper)
- `useLogState` React hook (can_undo, can_redo)
- Component editors: Vec2 (2 inputs), Color (4 inputs), Anchor (dropdown), F32 (number), Bool (checkbox), String (text), AssetReference (text)
- Add Component button with dropdown of all schemas
- Remove Component button per component
- Rename Entity inline editor
- 3 Playwright E2E tests for UI workflows

### Out of Scope
- Reparent drag-and-drop (deferred)
- Bulk operations (multi-select)
- Template instantiation UI (deferred)
- Schema authoring UI (deferred)
- Asset preview (no assets yet)
- Code editor / Monaco integration (way future)
- Live preview while typing (debounce OK for MVP)

## Capabilities

### New Capabilities
- `hierarchy-panel` — React component displaying entity tree with selection
- `inspector-panel` — React component for entity components with edit
- `ui-state-hooks` — React hooks for scene state, undo/redo state, dispatch wrapper
- `scene-snapshot-read` — `get_scene_snapshot()` wasm_bindgen for non-mutating read

### Modified Capabilities
None.

## Approach

**3-column CSS grid layout** with hierarchy on left, canvas in center, inspector on right. Top bar for global actions.

**React state** at App level: `scene` (current SceneDocument or null), `selectedEntityId` (string or null), `logState` ({can_undo, can_redo, size}). Hooks provide clean accessors.

**Component editing flow:**
1. User edits a field input in Inspector
2. On blur or button click, dispatch `SetComponentField { entity_id, type_id, field_path, value }`
3. If dispatch succeeds, update local state from response snapshot
4. If dispatch fails, display error

**Add Component flow:**
1. User clicks "Add Component" → dropdown shows `combined_registry` schemas
2. On select, dispatch `AddComponent { entity_id, type_id, values: <defaults> }`
3. If validation fails (e.g., unknown schema — shouldn't happen but defensive), display error

**Save/Load flow:**
- "Save" button: opens prompt for scene name → calls `save_scene(name)`
- "Load" button: opens dropdown of saved scenes → calls `load_scene(name)`
- "Load Project" button: calls `load_project()` (full restore)
- Errors displayed in top bar

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/lib.rs` | Modified | Add `get_scene_snapshot()` wasm_bindgen |
| `frontend/src/engine-bridge.ts` | Modified | Expose `get_scene_snapshot`, add `getSceneSnapshot` helper |
| `frontend/src/hooks/useSceneState.ts` | New | React hook for scene + dispatch wrapper |
| `frontend/src/hooks/useLogState.ts` | New | React hook for undo/redo state |
| `frontend/src/components/HierarchyPanel.tsx` | New | Entity tree + selection |
| `frontend/src/components/InspectorPanel.tsx` | New | Components display + edit |
| `frontend/src/components/TopBar.tsx` | New | Project status + undo/redo + save/load |
| `frontend/src/components/ComponentEditor.tsx` | New | Per-type field editor |
| `frontend/src/App.tsx` | Modified | 3-column layout + state |
| `frontend/src/main.tsx` | Unchanged | |
| `frontend/tests/engine.spec.ts` | Modified | Add 3 UI E2E tests |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| UI refresh too frequent (perf) | Low | React state updates only on dispatch |
| Component value parsing errors | Med | Validate JSON before dispatch |
| Schema dropdown lists 0 schemas | Low | Fallback: only built-ins if combined_registry unavailable |
| Selected entity deleted by undo | Low | Inspector handles missing entity gracefully |
| Long entity names overflow layout | Low | CSS `overflow: hidden; text-overflow: ellipsis` |
| Async save/load race with dispatch | Low | Disable buttons during async |
| Infinite render loops | Low | useEffect dependencies carefully chosen |

## Rollback Plan

Revert frontend changes to single-sidebar spike UI. Remove `get_scene_snapshot()` from lib.rs. Single-PR makes revert clean.

## Dependencies

Existing: React 19, all wasm functions. No new deps.

## Success Criteria

- [ ] `get_scene_snapshot()` returns SceneDocument JSON or null
- [ ] App renders 3-column layout (hierarchy | canvas | inspector)
- [ ] Hierarchy Panel lists all entities with selection
- [ ] Selecting entity shows its components in Inspector
- [ ] Inspector edits Vec2 with 2 inputs, Color with 4 inputs
- [ ] Inspector edits Anchor with dropdown, F32 with number input
- [ ] Add Component button works (dropdown of schemas)
- [ ] Remove Component button works
- [ ] Rename Entity inline editor works
- [ ] Undo/Redo buttons work via UI
- [ ] Save Scene button works (with name prompt)
- [ ] Load Project button restores scenes + schemas + templates
- [ ] All 23 existing Playwright tests pass (no regression)
- [ ] All 112 existing Rust tests pass (no regression)
- [ ] 3 new Playwright UI tests pass
- [ ] WASM builds clean