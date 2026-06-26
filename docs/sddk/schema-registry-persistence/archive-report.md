# Archive Report: schema-registry-persistence

> Phase: sddk-archive · Status: COMPLETED · Cycle complete: true · Branch: `feat/schema-registry-persistence`

## Summary

The `schema-registry-persistence` change delivered runtime-mutable Component Schema Registry with save/load to OPFS at `schemas/<type_id>.schema.json`. Built-in schemas (`editor.*`) are immutable; user-defined schemas can be registered, unregistered, deleted, and persisted across sessions via `load_project()` atomic restore. All 23 spec scenarios verified by 13 new Rust unit tests + 2 new Playwright E2E tests. Full suite (97 Rust + 21 E2E = 118 tests) passing.

## Artifacts (delta vs main)

### New
- 7 SDDK documents in `docs/sddk/schema-registry-persistence/`

### Modified
- `crates/editor-core/src/schema.rs` — `remove()`, mutable user registry, `is_builtin_type`, `register_schema`, `unregister_schema`, `combined_registry`, `SchemaError`
- `crates/editor-core/src/persistence.rs` — `SCHEMAS_DIR`, `schema_path()`, `ProjectMetadata.schemas: Vec<String>`
- `crates/editor-core/src/processor.rs` — uses `combined_registry()` for validation
- `crates/editor-core/src/lib.rs` — 9 new wasm_bindgen functions (`save_schema`, `load_schema`, `delete_schema`, `list_schemas`, `register_schema_from_json`, `unregister_schema`, `is_builtin_type`, `combined_registry_size`, `load_project`)
- `frontend/src/opfs-bridge.ts` — `opfsDeleteFile`
- `frontend/src/engine-bridge.ts` — exposed new functions on window
- `frontend/tests/engine.spec.ts` — 2 new Playwright tests

## Capability Coverage

| Capability | Spec scenarios | Test coverage | Status |
|---|---|---|---|
| `schema-registry-persistence` | 12 | 11 Rust + 1 E2E | ✅ IMPLEMENTED |
| `schema-registry-mutable` | 8 | 8 Rust unit | ✅ IMPLEMENTED |
| `schema-registry-restore` | 3 | 1 Rust + 1 E2E | ✅ IMPLEMENTED |

## Acceptance Criteria (from spec §6)

- [x] Every §2 scenario passes (12/12)
- [x] Every §3 scenario passes (8/8)
- [x] Every §4 scenario passes (3/3)
- [x] Combined registry returns built-ins + user schemas
- [x] Built-in protection enforced
- [x] Auto-restore via `load_project()` works end-to-end
- [x] `project.json` includes `schemas: Vec<String>` (with `#[serde(default)]` for backward compat)
- [x] Roundtrip preserves unknown fields
- [x] All 19 existing Playwright tests pass
- [x] All 84 existing Rust tests pass
- [x] 2 new Playwright tests pass
- [x] WASM builds clean

## Test Results (final)

- **Rust unit tests:** 97 passed (13 new + 84 existing)
- **WASM build:** success in 38.46s
- **Playwright E2E:** 21/21 passed (2 new + 19 existing)

## Decisions Worth Remembering

1. **Two-layer registry** — Built-ins immutable in `OnceLock`, user schemas mutable in `thread_local! RefCell`. `combined_registry()` returns merged view. Processor validates against combined.

2. **Built-in protection** — `is_builtin_type(type_id)` checks `editor.` prefix. `register`, `unregister`, `delete` all reject built-ins. User schemas use `game.*` namespace.

3. **`#[serde(default)]` on ProjectMetadata.schemas** — Old `project.json` files without `schemas` field still parse with empty Vec. Zero migration cost.

4. **OnceLock vs thread_local for user registry** — Originally tried `thread_local!` but `HashMap::new()` is not const. Used `OnceLock<RefCell<...>>` — But `RefCell` is not Sync, so static `OnceLock<RefCell<...>>` doesn't compile. Final: `thread_local!` (non-const initializer) using `ComponentSchemaRegistry::new()` at runtime.

5. **Atomic load_project** — If any schema fails to load, the operation errors before loading scenes. SCENE_DOC remains unchanged. No partial state.

6. **Per-schema files** — One JSON file per schema. Granular save/load, easy to diff, easy to delete. Project metadata tracks which schemas belong.

7. **`opfsDeleteFile` JS bridge addition** — Required for `delete_schema`. Reuses existing pattern with feature detection and `{ok, error?}` JSON response.

## Forward Compatibility

- Unknown fields preserved in schema JSON (via `serde_json::Value` for `default` field in FieldDef)
- Project metadata `schemas: Vec<String>` is additive — old files parse with empty Vec
- Schema versioning per ADR-0001 (`version: "0.1"` in `ComponentSchema`)
- Built-in protection prevents accidental modification of editor.* schemas

## Risks Realized During Implementation

1. **`HashMap::new()` not const** — Original design used `thread_local!` with const initializer. Failed because `HashMap::new()` requires runtime initialization. Solved by using `thread_local!` with runtime initializer (no `const` keyword needed).

2. **`OnceLock<RefCell<T>>` not Sync** — Tried `OnceLock<RefCell<ComponentSchemaRegistry>>` as workaround for the const issue. Failed because `OnceLock` requires `Sync` and `RefCell` isn't Sync. Reverted to `thread_local!` with runtime init.

3. **`JsValue::from_str` requires `&str` not `String`** — Same issue as previous cycles. Use closures: `.map_err(|e| JsValue::from_str(&e))`.

## PR Circuit (next steps)

1. Push `feat/schema-registry-persistence` to origin
2. `gh pr create --base main --title "feat(schema-registry-persistence): mutable user schemas + OPFS persistence"`
3. Self-merge with squash
4. Tag `v0.3.0` on main

## Next Steps (for the next SDD cycle)

1. **Entity template persistence** — Save/load Entity Templates to OPFS at `entities/`
2. **UI panels** — Hierarchy + Inspector that dispatch commands and call save/load
3. **DynamicScene Export** — Hito 0 §9.5 mapping
4. **Undo UI buttons** — React components reading `get_log_state()`

## Metrics

- **Files modified:** 5 (schema.rs, persistence.rs, processor.rs, lib.rs, opfs-bridge.ts, engine-bridge.ts, engine.spec.ts)
- **Files added:** 0 (only docs/sddk/* and pure changes)
- **Lines added (Rust):** ~300 (schema.rs additions + lib.rs new functions + tests)
- **Lines added (TypeScript):** ~100 (opfsDeleteFile + 2 E2E tests + bridge wiring)
- **Spec scenarios covered:** 23/23 (100%)
- **Tests passing:** 97 Rust + 21 E2E (118 total)
- **Cycle phases:** 8 (full SDDK A-lite)
- **Path:** A-lite (3 lenses in verify)
- **Model used:** minimax-coding-plan/MiniMax-M3 (orchestrator, all phases)
- **Branch:** `feat/schema-registry-persistence`

## Cycle Complete

This change is fully planned, implemented, verified, and ready for PR. The Hito 0 schema registry is now mutable and persistent.