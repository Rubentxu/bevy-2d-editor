# Tasks: Schema Registry Persistence

> Change: `schema-registry-persistence` · Phase: sddk-tasks · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

## Task Overview

12 atomic single-commit tasks in 5 phases.

### Dependency Graph

```
Phase 1: Schema layer
  Task 1.1 — schema.rs: add remove() + USER_SCHEMAS + register/unregister
  Task 1.2 — Schema unit tests
  ↓
Phase 2: Persistence extension
  Task 2.1 — persistence.rs: schemas_dir + schema_path + extend ProjectMetadata
  Task 2.2 — persistence unit tests
  ↓
Phase 3: Processor integration
  Task 3.1 — processor.rs: use combined_registry() for validation
  ↓
Phase 4: WASM surface + JS bridge
  Task 4.1 — opfs-bridge.ts: add opfsDeleteFile
  Task 4.2 — lib.rs: 7 wasm_bindgen functions (save/load/list/delete/register/unregister/is_builtin)
  Task 4.3 — lib.rs: load_project function
  Task 4.4 — engine-bridge.ts: expose new functions on window
  ↓
Phase 5: Playwright E2E + validation
  Task 5.1 — 2 new Playwright tests (register+validate, save+reload+load_project)
  Task 5.2 — Full test suite + WASM build
```

## Detailed Tasks

### Phase 1: Schema Layer

#### Task 1.1 — schema.rs: mutable registry + register/unregister
- **File:** `crates/editor-core/src/schema.rs`
- **Content:**
  - Add `ComponentSchemaRegistry::remove(type_id) -> Option<ComponentSchema>`
  - Add `thread_local! USER_SCHEMAS: RefCell<ComponentSchemaRegistry>`
  - Add `is_builtin_type(type_id) -> bool` (checks `editor.` prefix)
  - Add `register_schema(schema) -> Result<(), SchemaError>` (rejects built-ins)
  - Add `unregister_schema(type_id) -> Result<(), SchemaError>` (rejects built-ins, no-op if missing)
  - Add `combined_registry() -> ComponentSchemaRegistry` (merges built-ins + user)
  - Add `SchemaError` enum with thiserror
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(schema): add mutable user schema registry with built-in protection`

#### Task 1.2 — Schema unit tests
- **File:** `crates/editor-core/src/schema.rs` (add tests)
- **Tests:**
  - `test_is_builtin_type_editor_prefix_true`
  - `test_is_builtin_type_game_prefix_false`
  - `test_register_schema_rejects_builtin`
  - `test_register_schema_adds_user`
  - `test_register_schema_replaces_existing_user`
  - `test_unregister_schema_removes_user`
  - `test_unregister_schema_rejects_builtin`
  - `test_unregister_schema_nonexistent_is_noop`
  - `test_combined_registry_includes_builtins`
  - `test_combined_registry_includes_user_added`
- **Verify:** `cargo test --lib` in harness passes.
- **Commit:** `test(schema): add mutable registry unit tests`

### Phase 2: Persistence Extension

#### Task 2.1 — persistence.rs: schema path + ProjectMetadata.schemas
- **File:** `crates/editor-core/src/persistence.rs`
- **Content:**
  - Add `SCHEMAS_DIR: &str = "schemas"` constant
  - Add `schema_path(type_id) -> String` helper
  - Extend `ProjectMetadata` with `schemas: Vec<String>` field with `#[serde(default)]`
  - Update `Default` impl
- **Verify:** Compiles.
- **Commit:** `feat(persistence): add schema_path helper and ProjectMetadata.schemas field`

#### Task 2.2 — persistence unit tests
- **File:** `crates/editor-core/src/persistence.rs` (add tests)
- **Tests:**
  - `test_schema_path_format`
  - `test_project_metadata_default_has_schemas`
  - `test_project_metadata_roundtrip_with_schemas`
  - `test_project_metadata_without_schemas_field_deserializes` (backward compat)
- **Verify:** `cargo test --lib` passes.
- **Commit:** `test(persistence): add schema path and metadata tests`

### Phase 3: Processor Integration

#### Task 3.1 — processor.rs uses combined_registry
- **File:** `crates/editor-core/src/processor.rs`
- **Content:** Replace `global_registry().get(...)` with `combined_registry().get(...)` in validate() (8 occurrences)
- **Verify:** Compiles; existing 30 processor tests still pass.
- **Commit:** `refactor(processor): use combined_registry for validation`

### Phase 4: WASM Surface + JS Bridge

#### Task 4.1 — opfs-bridge.ts: add opfsDeleteFile
- **File:** `frontend/src/opfs-bridge.ts`
- **Content:** Add `opfsDeleteFile(path)` async function
- **Verify:** TypeScript compiles.
- **Commit:** `feat(bridge): add opfsDeleteFile`

#### Task 4.2 — lib.rs: 7 wasm_bindgen functions
- **File:** `crates/editor-core/src/lib.rs`
- **Content:**
  - Add `opfs_delete_file_raw` extern
  - Add helpers: `get_schema_json`, `update_project_schemas`
  - Add wasm_bindgen functions: `save_schema`, `load_schema`, `delete_schema`, `list_schemas`, `register_schema_from_json`, `unregister_schema`, `is_builtin_type`, `combined_registry_size`
- **Verify:** Compiles; WASM builds.
- **Commit:** `feat(lib): add schema persistence wasm_bindgen surface`

#### Task 4.3 — lib.rs: load_project function
- **File:** `crates/editor-core/src/lib.rs`
- **Content:** Add `load_project()` wasm_bindgen that reads project.json + loads schemas + loads first scene
- **Verify:** Compiles; WASM builds.
- **Commit:** `feat(lib): add load_project for atomic project restore`

#### Task 4.4 — engine-bridge.ts: expose new functions
- **File:** `frontend/src/engine-bridge.ts`
- **Content:** Expose all 9 new wasm functions on window: `save_schema`, `load_schema`, `delete_schema`, `list_schemas`, `register_schema`, `unregister_schema`, `is_builtin_type`, `combined_registry_size`, `load_project`, `opfs_delete_file`
- **Verify:** TypeScript compiles.
- **Commit:** `feat(bridge): expose schema persistence on window`

### Phase 5: Playwright E2E + Validation

#### Task 5.1 — 2 new Playwright tests
- **File:** `frontend/tests/engine.spec.ts` (add tests)
- **Tests:**
  - `register custom schema and use it in AddComponent`
    1. Wait for engine ready
    2. Call `register_schema` with JSON for `game.PlayerHealth`
    3. Load empty scene
    4. Dispatch `CreateEntity` + `AddComponent` with `game.PlayerHealth`
    5. Verify success
  - `save schema, reload page, load_project, validate schema available`
    1. Register + save `game.EnemyAI`
    2. Reload page
    3. Call `load_project()`
    4. Verify schema is in combined registry
    5. Use it in `AddComponent`
- **Verify:** `just test` passes (19 existing + 2 new = 21 tests).
- **Commit:** `test(e2e): add schema registry persistence tests`

#### Task 5.2 — Full test suite
- **Action:** Run `cargo test --lib` (harness), `just wasm`, `just test`
- **Acceptance:** All Rust tests pass (~95+ unit). WASM builds. 21 Playwright tests pass.
- **Commit:** `chore(tests): verify schema-registry-persistence suite green`

## Forecast

- **Total tasks:** 12 atomic commits
- **Estimated LOC:** ~400 Rust + ~100 TypeScript
- **Estimated time:** 1.5-2 hours focused work
- **Delivery:** Single branch `feat/schema-registry-persistence` + 1 PR against `main`

## Per-Task Verification

After each task:
- `cargo check --target wasm32-unknown-unknown` must pass
- After Task 4.3: `just wasm` must succeed
- After Task 5.1: `just test` must pass

## Backward Compatibility Strategy

- `global_registry()` still returns built-ins only (unchanged)
- `combined_registry()` is new function returning built-ins + user
- `processor::validate` switches to `combined_registry` (built-ins still validate)
- `ProjectMetadata` gets `schemas: Vec<String>` with `#[serde(default)]` — old project.json files parse
- All 19 existing Playwright tests + 84 existing Rust tests pass unchanged

## PR Circuit (after this cycle)

1. Push `feat/schema-registry-persistence` to origin
2. `gh pr create --base main --title "feat(schema-registry-persistence): save/load user schemas to OPFS"`
3. Self-merge with squash
4. Tag `v0.3.0` on main