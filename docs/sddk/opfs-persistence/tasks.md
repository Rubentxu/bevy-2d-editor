# Tasks: OPFS Persistence

> Change: `opfs-persistence` · Phase: sddk-tasks · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

## Task Overview

10 atomic single-commit tasks in 5 phases.

### Dependency Graph

```
Phase 1: Foundation
  Task 1.1 — Add wasm-bindgen-futures + serde-wasm-bindgen deps
  Task 1.2 — persistence.rs: ProjectMetadata + path helpers
  Task 1.3 — ProjectMetadata unit tests
  ↓
Phase 2: JS Bridge
  Task 2.1 — frontend/src/opfs-bridge.ts: 4 async OPFS functions
  Task 2.2 — engine-bridge.ts exposes bridge on window
  ↓
Phase 3: Rust wasm_bindgen extern + high-level functions
  Task 3.1 — wasm_bindgen extern declarations (4 ops)
  Task 3.2 — save_scene + update_project_metadata
  Task 3.3 — load_scene + mark_dirty
  Task 3.4 — list_scenes + project_exists
  ↓
Phase 4: Playwright E2E
  Task 4.1 — Roundtrip test (50+ entities)
  Task 4.2 — list_scenes test
  ↓
Phase 5: Validation
  Task 5.1 — Full test suite + WASM build
```

## Detailed Tasks

### Phase 1: Foundation

#### Task 1.1 — Add deps
- **File:** `crates/editor-core/Cargo.toml`
- **Add:** `wasm-bindgen-futures = "0.4"`, `serde-wasm-bindgen = "0.6"`
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `build(deps): add wasm-bindgen-futures and serde-wasm-bindgen`

#### Task 1.2 — persistence.rs types
- **File:** `crates/editor-core/src/persistence.rs` (new)
- **Content:**
  - `ProjectMetadata { version, name, scenes }` struct
  - `Default` impl: `version: "0.1"`, `name: "Untitled Project"`, `scenes: vec![]`
  - Constants: `PROJECT_FILE = "project.json"`, `SCENES_DIR = "scenes"`
  - `scene_path(name: &str) -> String` helper
- **Verify:** Compiles.
- **Commit:** `feat(persistence): add ProjectMetadata type and path helpers`

#### Task 1.3 — ProjectMetadata unit tests
- **File:** `crates/editor-core/src/persistence.rs` (add tests)
- **Tests:**
  - `test_project_metadata_default`
  - `test_project_metadata_serialization_roundtrip`
  - `test_scene_path_format`
- **Verify:** `cargo test --lib` in harness passes.
- **Commit:** `test(persistence): add ProjectMetadata unit tests`

### Phase 2: JS Bridge

#### Task 2.1 — opfs-bridge.ts
- **File:** `frontend/src/opfs-bridge.ts` (new)
- **Content:** 4 functions: `opfsSaveFile`, `opfsLoadFile`, `opfsListFiles`, `opfsExists`. Returns `{ok, value?, error?}` JSON. Feature-detects OPFS. Handles `NotFoundError` for missing files.
- **Verify:** TypeScript compiles.
- **Commit:** `feat(bridge): add OPFS wrapper module`

#### Task 2.2 — engine-bridge.ts exposes OPFS on window
- **File:** `frontend/src/engine-bridge.ts`
- **Content:** Import OPFS functions, expose them on `window` for wasm_bindgen externs.
- **Verify:** TypeScript compiles.
- **Commit:** `feat(bridge): expose OPFS functions on window`

### Phase 3: Rust WASM Bindgen

#### Task 3.1 — wasm_bindgen extern declarations
- **File:** `crates/editor-core/src/lib.rs`
- **Content:** 4 extern functions: `opfs_save_file`, `opfs_load_file`, `opfs_list_files`, `opfs_exists`. Async with `wasm_bindgen_futures`.
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(lib): add wasm_bindgen extern declarations for OPFS`

#### Task 3.2 — save_scene + update_project_metadata
- **File:** `crates/editor-core/src/lib.rs`
- **Content:** `save_scene(name)` and `update_project_metadata(name)` async functions. Serialize SCENE_DOC, write to OPFS, update project.json.
- **Verify:** Compiles.
- **Commit:** `feat(lib): implement save_scene with project metadata update`

#### Task 3.3 — load_scene
- **File:** `crates/editor-core/src/lib.rs`
- **Content:** `load_scene(name)` async function. Read from OPFS, parse, replace SCENE_DOC, mark dirty.
- **Verify:** Compiles.
- **Commit:** `feat(lib): implement load_scene with dirty flag`

#### Task 3.4 — list_scenes + project_exists
- **File:** `crates/editor-core/src/lib.rs`
- **Content:** `list_scenes()` returns `Vec<String>`, `project_exists()` returns `bool`.
- **Verify:** Compiles; WASM builds.
- **Commit:** `feat(lib): add list_scenes and project_exists`

### Phase 4: Playwright E2E

#### Task 4.1 — Roundtrip test
- **File:** `frontend/tests/engine.spec.ts` (add test)
- **Test:** Save scene with 50 entities → reload page → load scene → verify all 50 entities present
- **Verify:** `just test` passes (16 existing + 1 new = 17 tests).
- **Commit:** `test(e2e): add save/load roundtrip with 50 entities`

#### Task 4.2 — list_scenes test
- **File:** `frontend/tests/engine.spec.ts` (add test)
- **Test:** Save 3 scenes → list_scenes returns array with 3 names
- **Verify:** `just test` passes (17 + 1 new = 18 tests).
- **Commit:** `test(e2e): add list_scenes test`

### Phase 5: Validation

#### Task 5.1 — Full test suite
- **Action:** Run `cargo test --lib` (harness), `just wasm`, `just test`.
- **Acceptance:** All tests pass. WASM builds clean.
- **Commit:** `chore(tests): verify opfs-persistence test suite green`

## Forecast

- **Total tasks:** 10 atomic commits
- **Estimated LOC:** ~400 Rust + ~250 TypeScript
- **Estimated time:** 1.5-2 hours focused work
- **Delivery:** Single branch `feat/opfs-persistence` + 1 PR against `main`

## Per-Task Verification

After each task:
- `cargo check --target wasm32-unknown-unknown` must pass
- After Task 3.4: `just wasm` must succeed
- After Task 4.2: `just test` must pass

## Backward Compatibility Strategy

- All existing wasm_bindgen functions unchanged
- OPFS namespace `bevy-2d-editor` isolates from other apps
- OPFS is additive — only used when explicitly called
- Existing 16 Playwright tests + 79 Rust tests pass unchanged

## PR Circuit (after this cycle)

1. Push `feat/opfs-persistence` to origin
2. `gh pr create --base main --title "feat(opfs-persistence): save/load SceneDocument to OPFS via JS bridge"`
3. Self-review (no team), merge with squash
4. Tag `v0.2.0` on main