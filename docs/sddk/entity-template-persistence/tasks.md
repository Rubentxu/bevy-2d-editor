# Tasks: Entity Template Persistence + Instantiation

> Change: `entity-template-persistence` · Phase: sddk-tasks · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

## Task Overview

12 atomic single-commit tasks in 5 phases.

### Dependency Graph

```
Phase 1: Foundation
  Task 1.1 — template.rs: types + ID minter
  Task 1.2 — template.rs: validator
  Task 1.3 — template.rs: cache + instantiator
  Task 1.4 — Template unit tests
  ↓
Phase 2: Persistence extension
  Task 2.1 — persistence.rs: ENTITIES_DIR + template_path + ProjectMetadata.templates
  ↓
Phase 3: Processor integration
  Task 3.1 — processor.rs: full InstantiateEntityTemplate impl
  ↓
Phase 4: WASM surface
  Task 4.1 — lib.rs: save/load/list/delete/is_loaded wasm_bindgen
  Task 4.2 — lib.rs: update load_project to include templates
  Task 4.3 — engine-bridge.ts: expose new functions on window
  ↓
Phase 5: Playwright E2E + validation
  Task 5.1 — 2 new Playwright tests
  Task 5.2 — Full test suite + WASM build
```

## Detailed Tasks

### Phase 1: Foundation

#### Task 1.1 — template.rs: types + ID minter
- **File:** `crates/editor-core/src/template.rs` (new)
- **Content:**
  - `EntityTemplate { template_id, display_name, version, entities }` struct
  - `TemplateEntity { local_id, name, parent_local_id, components }` struct
  - `mint_stable_id() -> StableId` using thread_local counter + timestamp
- **Verify:** Compiles.
- **Commit:** `feat(template): add EntityTemplate, TemplateEntity types and ID minter`

#### Task 1.2 — template.rs: validator
- **File:** `crates/editor-core/src/template.rs`
- **Content:**
  - `TemplateError` enum with thiserror
  - `validate(template: &EntityTemplate) -> Result<(), TemplateError>`:
    - Empty template check
    - Multiple roots check (exactly one)
    - Dangling parent check
    - Cycle detection (walk parent chain)
    - Component schema validation via combined_registry()
- **Verify:** Compiles.
- **Commit:** `feat(template): add validator with cycle and schema checks`

#### Task 1.3 — template.rs: cache + instantiator
- **File:** `crates/editor-core/src/template.rs`
- **Content:**
  - `thread_local! TEMPLATE_CACHE: RefCell<HashMap<String, EntityTemplate>>`
  - `cache_template`, `get_cached_template`, `remove_cached_template`, `clear_template_cache`
  - `instantiate(template, target_parent, doc) -> Result<Vec<StableId>, TemplateError>`
- **Verify:** Compiles.
- **Commit:** `feat(template): add in-memory cache and tree instantiator`

#### Task 1.4 — Template unit tests
- **File:** `crates/editor-core/src/template.rs` (add tests)
- **Tests:**
  - `test_entity_template_single_root_serialization`
  - `test_entity_template_tree_serialization`
  - `test_validate_empty_template_fails`
  - `test_validate_multiple_roots_fails`
  - `test_validate_dangling_parent_fails`
  - `test_validate_cycle_detected`
  - `test_validate_unknown_schema_fails`
  - `test_validate_valid_template_succeeds`
  - `test_instantiate_single_root`
  - `test_instantiate_tree`
  - `test_instantiate_with_target_parent`
  - `test_instantiate_twice_different_ids`
  - `test_mint_stable_id_unique`
  - `test_template_cache_basic`
- **Verify:** `cargo test --lib` passes.
- **Commit:** `test(template): add template unit tests`

### Phase 2: Persistence Extension

#### Task 2.1 — persistence.rs: ENTITIES_DIR + template_path + ProjectMetadata.templates
- **File:** `crates/editor-core/src/persistence.rs`
- **Content:**
  - Add `ENTITIES_DIR: &str = "entities"`
  - Add `template_path(template_id) -> String` helper
  - Extend `ProjectMetadata` with `templates: Vec<String>` field with `#[serde(default)]`
- **Verify:** Compiles; existing tests pass.
- **Commit:** `feat(persistence): add entity template paths and ProjectMetadata.templates`

### Phase 3: Processor Integration

#### Task 3.1 — processor.rs: full InstantiateEntityTemplate
- **File:** `crates/editor-core/src/processor.rs`
- **Content:**
  - Replace `InstantiateEntityTemplate` stub with full implementation
  - Use `template::get_cached_template` and `template::instantiate`
  - Inverse: `Batch` of `DeleteEntity` for each minted entity
- **Verify:** Compiles; existing processor tests pass.
- **Commit:** `feat(processor): implement full InstantiateEntityTemplate with tree minting`

### Phase 4: WASM Surface

#### Task 4.1 — lib.rs: save/load/list/delete/is_loaded wasm_bindgen
- **File:** `crates/editor-core/src/lib.rs`
- **Content:**
  - Add `save_template(template_id, template_json)` async wasm_bindgen
  - Add `load_template(template_id)` async wasm_bindgen
  - Add `list_templates()` async wasm_bindgen
  - Add `delete_template(template_id)` async wasm_bindgen
  - Add `is_template_loaded(template_id)` sync wasm_bindgen
  - Helper `update_project_templates`
- **Verify:** Compiles; WASM builds.
- **Commit:** `feat(lib): add entity template wasm_bindgen surface`

#### Task 4.2 — lib.rs: update load_project to include templates
- **File:** `crates/editor-core/src/lib.rs`
- **Content:** Extend `load_project()` to also `load_template(template_id)` for each in `project.templates`
- **Verify:** Compiles.
- **Commit:** `feat(lib): extend load_project to load templates`

#### Task 4.3 — engine-bridge.ts: expose new functions
- **File:** `frontend/src/engine-bridge.ts`
- **Content:** Expose `save_template`, `load_template`, `list_templates`, `delete_template`, `is_template_loaded` on window
- **Verify:** TypeScript compiles.
- **Commit:** `feat(bridge): expose template functions on window`

### Phase 5: Playwright E2E + Validation

#### Task 5.1 — 2 new Playwright tests
- **File:** `frontend/tests/engine.spec.ts` (add tests)
- **Tests:**
  - `save template and instantiate end-to-end with tree`
    1. Register needed schema (or use built-in editor.Transform2D)
    2. Save template with 3 entities: root + 2 children
    3. Load empty scene
    4. Load template into cache
    5. Dispatch `InstantiateEntityTemplate`
    6. Verify scene has 3 entities with tree hierarchy
  - `template lifecycle with load_project restore`
    1. Save template with 1 entity
    2. Reload page (state lost)
    3. Call `load_project()`
    4. Verify `is_template_loaded` returns true
    5. Instantiate via command — succeeds
- **Verify:** `just test` passes (21 + 2 = 23 tests).
- **Commit:** `test(e2e): add entity template lifecycle tests`

#### Task 5.2 — Full test suite + WASM build
- **Action:** Run `cargo test --lib` (harness), `just wasm`, `just test`
- **Acceptance:** All Rust tests pass. WASM builds. 23 Playwright tests pass.
- **Commit:** `chore(tests): verify entity-template-persistence suite green`

## Forecast

- **Total tasks:** 12 atomic commits
- **Estimated LOC:** ~600 Rust + ~50 TypeScript
- **Estimated time:** 2 hours focused work
- **Delivery:** Single branch + 1 PR against `main`

## Per-Task Verification

After each task:
- `cargo check --target wasm32-unknown-unknown` must pass
- After Task 4.2: `just wasm` must succeed
- After Task 5.1: `just test` must pass

## Backward Compatibility Strategy

- `InstantiateEntityTemplate` was stub (always failed) — now succeeds
- `ProjectMetadata` gets `templates: Vec<String>` with `#[serde(default)]` — old files parse
- All 21 existing Playwright tests + 97 existing Rust tests pass unchanged

## PR Circuit (after this cycle)

1. Push `feat/entity-template-persistence` to origin
2. `gh pr create --base main --title "feat(entity-template-persistence): entity templates with tree instantiation"`
3. Self-merge with squash
4. Tag `v0.4.0` on main