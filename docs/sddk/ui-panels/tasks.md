# Tasks: UI Panels

> Change: `ui-panels` · Phase: sddk-tasks · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

## Task Overview

11 atomic single-commit tasks in 4 phases.

### Dependency Graph

```
Phase 1: Foundation
  Task 1.1 — Rust: get_scene_snapshot wasm_bindgen
  Task 1.2 — Frontend: engine-bridge helper + useSceneState/useLogState hooks
  ↓
Phase 2: Layout
  Task 2.1 — App.tsx 3-column layout
  Task 2.2 — CSS styles
  Task 2.3 — TopBar component
  ↓
Phase 3: Panels
  Task 3.1 — HierarchyPanel
  Task 3.2 — ComponentEditor + ComponentCard
  Task 3.3 — InspectorPanel
  Task 3.4 — AddComponentButton
  ↓
Phase 4: Tests + validation
  Task 4.1 — Playwright UI tests (3 tests)
  Task 4.2 — Full test suite + WASM build
```

## Detailed Tasks

### Phase 1: Foundation

#### Task 1.1 — Rust: get_scene_snapshot wasm_bindgen
- **File:** `crates/editor-core/src/lib.rs`
- **Content:** Add `#[wasm_bindgen] pub fn get_scene_snapshot() -> JsValue` returning SceneDocument JSON or null
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(lib): add get_scene_snapshot wasm_bindgen`

#### Task 1.2 — Frontend: bridge helper + hooks
- **File:** `frontend/src/engine-bridge.ts`, `frontend/src/hooks/useSceneState.ts` (new), `frontend/src/hooks/useLogState.ts` (new)
- **Content:**
  - Expose `get_scene_snapshot` on window
  - Add `getSceneSnapshot()` helper returning `SceneDocument | null`
  - Implement `useSceneState()` hook with `scene`, `refresh`, `dispatch`
  - Implement `useLogState()` hook polling every 500ms
- **Verify:** TypeScript compiles.
- **Commit:** `feat(frontend): add get_scene_snapshot bridge + useSceneState/useLogState hooks`

### Phase 2: Layout

#### Task 2.1 — App.tsx 3-column layout
- **File:** `frontend/src/App.tsx`
- **Content:** Replace spike layout with 3-column (Hierarchy | Canvas | Inspector) + TopBar. State management for scene, selectedEntityId, logState, error.
- **Verify:** TypeScript compiles.
- **Commit:** `feat(frontend): replace spike UI with 3-column layout`

#### Task 2.2 — CSS styles
- **File:** `frontend/src/styles.css` (new)
- **Content:** Styles for `.app`, `.topbar`, `.main`, `.panel`, `.entity`, `.component-card`, `.dropdown`, etc. Dark theme matching spike.
- **Verify:** TypeScript compiles.
- **Commit:** `style(frontend): add dark theme styles for UI panels`

#### Task 2.3 — TopBar component
- **File:** `frontend/src/components/TopBar.tsx` (new)
- **Content:** Top bar with title, Undo/Redo buttons, Save/Load buttons, error display
- **Verify:** TypeScript compiles.
- **Commit:** `feat(frontend): add TopBar component with undo/redo + save/load`

### Phase 3: Panels

#### Task 3.1 — HierarchyPanel
- **File:** `frontend/src/components/HierarchyPanel.tsx` (new)
- **Content:** List entities with name + ID, indented by parent depth, selection highlight, click handler
- **Verify:** TypeScript compiles.
- **Commit:** `feat(frontend): add HierarchyPanel with entity list + selection`

#### Task 3.2 — ComponentEditor + ComponentCard
- **File:** `frontend/src/components/ComponentCard.tsx` (new), `frontend/src/components/ComponentEditor.tsx` (new)
- **Content:**
  - `ComponentCard`: header with type_id + Remove button + nested field editors
  - `ComponentEditor`: per-type widget (Vec2, Color, Anchor dropdown, F32, Bool, String)
- **Verify:** TypeScript compiles.
- **Commit:** `feat(frontend): add ComponentCard + ComponentEditor with per-type widgets`

#### Task 3.3 — InspectorPanel
- **File:** `frontend/src/components/InspectorPanel.tsx` (new)
- **Content:** Entity name input (rename on blur), components list, Add Component button
- **Verify:** TypeScript compiles.
- **Commit:** `feat(frontend): add InspectorPanel with entity name + components`

#### Task 3.4 — AddComponentButton
- **File:** `frontend/src/components/AddComponentButton.tsx` (new)
- **Content:** Button + dropdown listing all schemas from combined_registry
- **Verify:** TypeScript compiles.
- **Commit:** `feat(frontend): add AddComponentButton with schema dropdown`

### Phase 4: Tests + Validation

#### Task 4.1 — Playwright UI tests
- **File:** `frontend/tests/engine.spec.ts` (add tests)
- **Tests:**
  - `UI hierarchy shows entities and supports selection`
  - `UI inspector shows components and edits Vec2 field`
  - `UI undo button works`
- **Verify:** `just test` passes (23 + 3 = 26 tests).
- **Commit:** `test(e2e): add UI panel Playwright tests`

#### Task 4.2 — Full test suite
- **Action:** Run `cargo test --lib` (harness), `just wasm`, `just test`
- **Acceptance:** All Rust tests pass. WASM builds. 26 Playwright tests pass.
- **Commit:** `chore(tests): verify ui-panels suite green`

## Forecast

- **Total tasks:** 11 atomic commits
- **Estimated LOC:** ~50 Rust + ~800 TypeScript
- **Estimated time:** 2-3 hours focused work
- **Delivery:** Single branch `feat/ui-panels` + 1 PR against `main`

## Per-Task Verification

After each task:
- `cargo check --target wasm32-unknown-unknown` (Rust changes)
- `tsc --noEmit` or `npm run build` (TS changes)
- After Task 4.1: `just test` must pass

## Backward Compatibility Strategy

- All 23 existing Playwright tests test wasm directly, not UI → unaffected
- `get_scene_snapshot()` is additive
- Spike UI (X/Y inputs + Move Sprite button) is replaced but its wasm functions still work

## PR Circuit (after this cycle)

1. Push `feat/ui-panels` to origin
2. `gh pr create --base main --title "feat(ui-panels): Hierarchy + Inspector React panels with command dispatch"`
3. Self-merge with squash
4. Tag `v0.5.0` on main