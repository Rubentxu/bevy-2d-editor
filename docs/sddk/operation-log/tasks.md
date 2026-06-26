# Tasks: Operation Log + Undo/Redo

> Change: `operation-log` · Phase: sddk-tasks · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

## Task Overview

10 atomic single-commit tasks in 4 phases.

### Dependency Graph

```
Phase 1: Foundation (types)
  Task 1.1 — operation_log.rs: LogEntry, OperationLogError, OperationLog skeleton
  Task 1.2 — operation_log.rs: LogEntry serialization tests
  ↓
Phase 2: Core operations
  Task 2.1 — record() with truncate + FIFO eviction
  Task 2.2 — undo() with cursor--
  Task 2.3 — redo() with cursor++
  Task 2.4 — can_undo/can_redo/get_log_size accessors
  Task 2.5 — comprehensive unit tests
  ↓
Phase 3: WASM integration
  Task 3.1 — thread_local! OPERATION_LOG + dispatch_command recording
  Task 3.2 — undo/redo wasm_bindgen + OperationLogState Resource
  Task 3.3 — sync_log_state Bevy system
  ↓
Phase 4: Frontend + validation
  Task 4.1 — engine-bridge.ts exposes undo/redo on window
  Task 4.2 — 2 Playwright tests (undo, undo+redo)
  Task 4.3 — full test suite + WASM build verification
```

## Detailed Tasks

### Phase 1: Foundation

#### Task 1.1 — OperationLog types
- **File:** `crates/editor-core/src/operation_log.rs` (new)
- **Content:**
  - `LogEntry { forward, inverse, metadata }` with derives (Debug, Clone, PartialEq, Serialize, Deserialize)
  - `OperationLogError` enum with thiserror
  - `OperationLog` struct skeleton (entries: Vec<LogEntry>, cursor: isize, max_size: usize)
  - `new()` constructor with max_size = 1000
  - `with_max_size(max_size: usize)` constructor
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes.
- **Commit:** `feat(operation-log): add LogEntry, OperationLogError, OperationLog skeleton`

#### Task 1.2 — LogEntry serialization tests
- **File:** `crates/editor-core/src/operation_log.rs` (add tests)
- **Tests:**
  - `test_log_entry_roundtrip`
- **Verify:** `cargo test --lib` in harness passes.
- **Commit:** `test(operation-log): add LogEntry serialization test`

### Phase 2: Core Operations

#### Task 2.1 — record() with truncate + eviction
- **File:** `crates/editor-core/src/operation_log.rs`
- **Content:**
  - `record(envelope, inverse)` method:
    - Truncate entries after cursor (if cursor not at end)
    - Append new entry
    - Evict oldest if over max_size (with cursor adjustment)
    - Advance cursor to new last entry
- **Verify:** Compiles.
- **Commit:** `feat(operation-log): implement record with truncate and FIFO eviction`

#### Task 2.2 — undo() applies inverse
- **File:** `crates/editor-core/src/operation_log.rs`
- **Content:**
  - `undo(doc)` method:
    - If `!can_undo()`, return `NothingToUndo`
    - Apply inverse at cursor via `processor::apply`
    - cursor -= 1
    - Return cloned doc
- **Verify:** Compiles.
- **Commit:** `feat(operation-log): implement undo applying inverse`

#### Task 2.3 — redo() applies forward
- **File:** `crates/editor-core/src/operation_log.rs`
- **Content:**
  - `redo(doc)` method:
    - If `!can_redo()`, return `NothingToRedo`
    - cursor += 1
    - Apply forward at cursor via `processor::apply`
    - Return cloned doc
- **Verify:** Compiles.
- **Commit:** `feat(operation-log): implement redo applying forward`

#### Task 2.4 — Accessors
- **File:** `crates/editor-core/src/operation_log.rs`
- **Content:**
  - `can_undo() -> bool` (cursor >= 0)
  - `can_redo() -> bool` (cursor < len - 1)
  - `get_log_size() -> usize`
  - `get_cursor() -> isize`
  - `get_log() -> &[LogEntry]`
  - `clear()` (reset for new project)
- **Verify:** Compiles.
- **Commit:** `feat(operation-log): add can_undo/can_redo/get_log accessors`

#### Task 2.5 — Comprehensive unit tests
- **File:** `crates/editor-core/src/operation_log.rs` (add tests)
- **Tests covering all spec §2 and §3 scenarios:**
  - `test_apply_appends_and_advances_cursor`
  - `test_apply_invalid_rejected_no_log`
  - `test_batch_logged_as_single_entry`
  - `test_max_size_evicts_oldest`
  - `test_configurable_max_size`
  - `test_undo_applies_inverse`
  - `test_undo_at_start_returns_error`
  - `test_undo_all_the_way_empties`
  - `test_redo_applies_forward`
  - `test_redo_at_end_returns_error`
  - `test_truncate_on_new_command_after_undo`
  - `test_no_truncate_at_end`
  - `test_can_undo_can_redo_state`
  - `test_undo_redo_roundtrip`
  - `test_undo_on_empty_log`
  - `test_redo_on_empty_log`
  - `test_get_log_size_on_empty`
- **Verify:** `cargo test --lib` in harness passes all.
- **Commit:** `test(operation-log): add comprehensive unit tests for all spec scenarios`

### Phase 3: WASM Integration

#### Task 3.1 — thread_local OPERATION_LOG + dispatch_command records
- **File:** `crates/editor-core/src/lib.rs`
- **Content:**
  - Add `thread_local! static OPERATION_LOG: RefCell<OperationLog>`
  - Modify `dispatch_command`: after processor::apply succeeds, call `OPERATION_LOG.with(|l| l.borrow_mut().record(&envelope, inverse.clone()))`
- **Verify:** `cargo check --target wasm32-unknown-unknown` passes; `just wasm` succeeds.
- **Commit:** `feat(lib): record commands to operation log in dispatch_command`

#### Task 3.2 — undo/redo wasm_bindgen + Bevy Resource
- **File:** `crates/editor-core/src/lib.rs`
- **Content:**
  - Add `#[derive(Resource, Clone, Default)] OperationLogState { size, can_undo, can_redo }`
  - Add `#[wasm_bindgen] undo() -> Result<String, JsValue>` and `redo() -> Result<String, JsValue>`
  - Both call `mark_dirty()` after success
- **Verify:** Compiles; WASM builds.
- **Commit:** `feat(lib): add undo/redo wasm_bindgen and OperationLogState Resource`

#### Task 3.3 — sync_log_state Bevy system
- **File:** `crates/editor-core/src/lib.rs`
- **Content:**
  - Add `sync_log_state` system in Update schedule (after rebuild_preview_world)
  - Reads `OPERATION_LOG.with(...)` and updates `ResMut<OperationLogState>`
- **Verify:** Compiles; WASM builds.
- **Commit:** `feat(lib): add sync_log_state Bevy system`

### Phase 4: Frontend + Validation

#### Task 4.1 — engine-bridge.ts exposes undo/redo
- **File:** `frontend/src/engine-bridge.ts`
- **Content:**
  - Add `(window as any).undo = () => wasm.undo()`
  - Add `(window as any).redo = () => wasm.redo()`
- **Verify:** TypeScript compiles.
- **Commit:** `feat(bridge): expose undo and redo on window`

#### Task 4.2 — Playwright undo/redo tests
- **File:** `frontend/tests/engine.spec.ts` (add tests)
- **Tests:**
  - `dispatch CreateEntity → undo → verify entity gone`
  - `dispatch CreateEntity → undo → redo → verify entity back`
- **Verify:** `just test` passes 15 tests (13 existing + 2 new).
- **Commit:** `test(e2e): add undo and redo Playwright tests`

#### Task 4.3 — Full test suite
- **Action:** Run `cargo test --lib` (in harness), `just wasm`, `just test`.
- **Acceptance:** ~75+ Rust unit tests pass. WASM builds clean. 15 Playwright tests pass.
- **Commit:** `chore(tests): verify full operation-log test suite green`

## Forecast

- **Total tasks:** 10 atomic commits
- **Estimated LOC:** ~600 Rust (operation_log.rs ~350 + tests + lib.rs modifications + 50 TypeScript)
- **Estimated time:** 1.5-2 hours focused work
- **Delivery:** Single PR
- **Risks:** Low — additive change, backward compatible

## Per-Task Verification

After each task:
- `cargo check --target wasm32-unknown-unknown` must pass
- After test tasks: `cargo test --lib` in harness must pass relevant module
- After Task 3.2: `just wasm` must succeed
- After Task 4.2: `just test` must pass

## Backward Compatibility Strategy

- `dispatch_command` API unchanged (still returns CommandResult JSON)
- Existing 13 Playwright tests untouched
- LinearBus untouched
- Default scene fallback preserved
- All existing 58 Rust tests still pass (operation_log is new module)