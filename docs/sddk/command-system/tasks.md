# Tasks: Command System

> Change: `command-system` · Phase: sddk-tasks · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

## Task Overview

12 atomic single-commit tasks organized in 5 phases.

### Dependency Graph

```
Phase 1: Foundation (deps + types)
  Task 1.1 — Cargo deps (thiserror already there)
  Task 1.2 — command.rs types (Command enum + metadata + envelope + error)
  Task 1.3 — command.rs serialization tests
  ↓
Phase 2: Processor core
  Task 2.1 — processor.rs: validate + apply skeleton + 4 simple commands
  Task 2.2 — processor.rs: SetComponentField with field path parser
  Task 2.3 — processor.rs: ReparentEntity with cycle detection
  Task 2.4 — processor.rs: Batch with atomic rollback
  Task 2.5 — processor.rs: unit tests for all 8 commands
  ↓
Phase 3: Bevy integration
  Task 3.1 — SceneDocumentState resource + SceneEntity marker
  Task 3.2 — rebuild_preview_world system
  Task 3.3 — dispatch_command wasm_bindgen + dirty flag
  ↓
Phase 4: Frontend
  Task 4.1 — Expose dispatch_command via engine-bridge
  Task 4.2 — Playwright dispatch test
  ↓
Phase 5: Validation
  Task 5.1 — Full test suite + WASM build + Playwright
```

## Detailed Tasks

### Phase 1: Foundation

#### Task 1.1 — Cargo deps
- **File:** `crates/editor-core/Cargo.toml`
- **Action:** No new deps needed (serde, serde_json, thiserror, bevy 0.19 all already present).
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `chore(deps): verify command-system deps already present`

#### Task 1.2 — Command types
- **File:** `crates/editor-core/src/command.rs` (new)
- **Content:**
  - `Command` enum (9 variants including `Batch`) with `#[serde(tag = "type", rename_all = "PascalCase")]`
  - `CommandMetadata { authorship, timestamp, rationale }`
  - `CommandEnvelope { command, metadata }`
  - `CommandResult { inverse, snapshot }`
  - `CommandError` enum with `thiserror` derives
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(command): add Command enum, metadata, error types`

#### Task 1.3 — Command serialization tests
- **File:** `crates/editor-core/src/command.rs` (add `#[cfg(test)] mod tests`)
- **Tests:**
  - `test_create_entity_serializes_with_type_tag`
  - `test_delete_entity_serializes_with_type_tag`
  - `test_add_component_serializes`
  - `test_set_component_field_serializes`
  - `test_reparent_entity_serializes_with_optional_parent`
  - `test_batch_serializes_with_label`
  - `test_envelope_roundtrip`
  - `test_metadata_roundtrip`
- **Verify:** `cargo test --lib command` passes all.
- **Commit:** `test(command): add Command enum serialization tests`

### Phase 2: Processor Core

#### Task 2.1 — Processor skeleton + 4 simple commands
- **File:** `crates/editor-core/src/processor.rs` (new)
- **Content:**
  - Module skeleton with `validate(doc, cmd)` and `apply(doc, cmd) -> Result<Command, CommandError>`
  - Implement `CreateEntity`, `DeleteEntity`, `AddComponent`, `RemoveComponent`
  - Helper `find_entity_mut(doc, id) -> Result<&mut Entity, CommandError>`
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(processor): implement Create/Delete/AddComponent/RemoveComponent`

#### Task 2.2 — SetComponentField with field path
- **File:** `crates/editor-core/src/processor.rs`
- **Content:**
  - Implement `SetComponentField` logic
  - Helper `set_field_path(value: &mut Value, path: &str, new: Value) -> Result<Value, CommandError>`
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(processor): implement SetComponentField with field path parser`

#### Task 2.3 — ReparentEntity with cycle detection
- **File:** `crates/editor-core/src/processor.rs`
- **Content:**
  - Implement `ReparentEntity` logic (captures pre-state, applies new parent)
  - Helper `would_create_cycle(doc, entity_id, proposed_parent) -> Result<bool, CommandError>`
  - Implement `RenameEntity` (captures pre-state name)
  - Stub `InstantiateEntityTemplate` returning `TemplateNotFound`
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(processor): implement ReparentEntity with cycle detection`

#### Task 2.4 — Batch with atomic rollback
- **File:** `crates/editor-core/src/processor.rs`
- **Content:**
  - Implement `Batch` logic with rollback on first failure
  - Inverse is reversed-list of inverses
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(processor): implement Batch with atomic rollback`

#### Task 2.5 — Processor unit tests
- **File:** `crates/editor-core/src/processor.rs` (add `#[cfg(test)] mod tests`)
- **Tests (cover all spec §2 + §3 scenarios):**
  - CreateEntity: adds entity, rejects duplicate
  - DeleteEntity: removes leaf, reparents children, fails on missing
  - AddComponent: succeeds with valid schema, rejects unknown, preserves unknown fields
  - RemoveComponent: removes existing, no-op on absent
  - SetComponentField: simple path, nested path, fails on missing
  - ReparentEntity: valid parent, rejects cycle, captures pre-state
  - RenameEntity: updates name, preserves id
  - InstantiateEntityTemplate: stub returns error
  - Batch: applies all, atomic rollback on failure, inverse in reverse order
  - Validation: failed validation leaves doc unchanged
  - Forward+inverse roundtrip per command
- **Verify:** `cargo test --lib processor` passes all.
- **Commit:** `test(processor): add comprehensive processor unit tests`

### Phase 3: Bevy Integration

#### Task 3.1 — SceneDocumentState resource
- **File:** `crates/editor-core/src/lib.rs`
- **Content:**
  - Add `#[derive(Resource, Clone)] pub struct SceneDocumentState { document: SceneDocument, dirty: bool }`
  - Add `#[derive(Component)] pub struct SceneEntity` marker
  - Modify `setup()` to insert `SceneDocumentState` resource initialized from `SCENE_DOC` or default
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(lib): add SceneDocumentState resource and SceneEntity marker`

#### Task 3.2 — rebuild_preview_world system
- **File:** `crates/editor-core/src/lib.rs`
- **Content:**
  - Add `rebuild_preview_world` system that:
    - Checks `state.dirty`
    - Despawns all entities with `SceneEntity` marker
    - Spawns entities from `state.document.entities`
  - Register in `Update` schedule after `process_commands`
  - Spawn entities with `SceneEntity` marker in `spawn_entity()`
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes; `just wasm` succeeds.
- **Commit:** `feat(lib): add rebuild_preview_world system`

#### Task 3.3 — dispatch_command wasm_bindgen
- **File:** `crates/editor-core/src/lib.rs`
- **Content:**
  - Add `#[wasm_bindgen] pub fn dispatch_command(json: &str) -> Result<String, JsValue>`
  - Deserialize `CommandEnvelope`, call `processor::apply`, set dirty flag, return inverse as JSON
  - Add `thread_local! static DIRTY: RefCell<bool>` for cross-system visibility
  - `rebuild_preview_world` reads DIRTY and sets state.dirty
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes; `just wasm` succeeds.
- **Commit:** `feat(lib): add dispatch_command wasm_bindgen entry point`

### Phase 4: Frontend

#### Task 4.1 — Expose dispatch_command on window
- **File:** `frontend/src/engine-bridge.ts`
- **Content:**
  - Add `(window as any).dispatch_command = (json: string) => wasm.dispatch_command(json)`
  - Add helper `dispatchCommand(cmd: object)` that JSON.stringifies and calls window function
- **Verify:** TypeScript compiles (`tsc`).
- **Commit:** `feat(bridge): expose dispatch_command on window`

#### Task 4.2 — Playwright dispatch test
- **File:** `frontend/tests/engine.spec.ts` (add new test)
- **Test:** `dispatch CreateEntity from JS and verify scene updates`
- **Steps:**
  1. `page.goto("/")`, wait for "Bevy running"
  2. `await page.evaluate(...)` calling `window.dispatch_command(JSON.stringify({command: {...}, metadata: {...}}))` with a `CreateEntity` command
  3. Wait a tick for Bevy to rebuild
  4. Reload page
  5. Wait for "Bevy running"
  6. Verify by calling `window.dispatch_command` with a `SetComponentField` to position the new entity, then verifying position via the existing move-sprite event
- **Verify:** `just test` passes (existing 10 + 1 new = 11 tests).
- **Commit:** `test(e2e): add dispatch_command Playwright test`

### Phase 5: Validation

#### Task 5.1 — Full test suite
- **Action:** Run `cargo test --lib`, `just wasm`, `just test`.
- **Acceptance:** All Rust unit tests pass (~30+ new). WASM builds clean. All 11 Playwright tests pass.
- **Commit:** `chore(tests): verify full command-system test suite green`

## Forecast

- **Total tasks:** 12 atomic commits
- **Estimated LOC:** ~800 Rust (command.rs ~200, processor.rs ~400, lib.rs ~150, tests ~250) + ~50 TypeScript
- **Estimated time:** 2-3 hours focused work
- **Delivery:** Single PR
- **Branching risk:** Low — additive change to existing code

## Per-Task Verification

After each task:
- `cargo check --target wasm32-unknown-unknown` must pass
- After tasks that add tests: `cargo test --lib` must pass the relevant module
- After Task 3.3: `just wasm` must succeed
- After Task 4.2: `just test` must pass

## Risks (from design)

- **Medium:** Bevy rebuild system timing — use thread_local DIRTY flag visible to both dispatch_command and rebuild_preview_world
- **Medium:** Cycle detection deep hierarchies — walk full chain
- **Medium:** Batch inverse order — reverse the inverses vector explicitly
- **Low:** WASM string allocation per command — acceptable for human-speed interactions