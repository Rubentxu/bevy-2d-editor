# Proposal: Schema Registry Persistence

## Intent

Hito 0 §6.3 mandates a project-global Component Schema Registry that lives outside the Bevy World, but no persistence exists. The 5 built-in schemas (`editor.Name`, `editor.Transform2D`, etc.) are hardcoded at startup — there's no way for users to define custom `game.*` schemas, and no way to persist schema additions across sessions. This change delivers per-schema save/load to OPFS at `schemas/<type_id>.schema.json`, mutable registry state at runtime, and auto-restore on project load. Foundation for the user-defined schema authoring flow that Hito 0 §3.2 explicitly defers (but the data layer must support).

## Scope

### In Scope
- Per-schema files at `schemas/<type_id>.schema.json` (one file per `ComponentSchema`)
- Mutable user-defined schemas (registered at runtime, persisted across sessions)
- Built-in schemas (`editor.*`) are immutable — cannot be unregistered
- `register_schema(json)` / `unregister_schema(type_id)` runtime mutations
- `save_schema(type_id)` / `load_schema(type_id)` / `delete_schema(type_id)` / `list_schemas()` wasm_bindgen functions
- Combined registry function returning built-ins + user schemas
- Auto-restore: when loading a project, load all referenced schemas
- Project metadata `schemas: Vec<String>` field
- Roundtrip preservation including unknown fields (ADR-0003)
- Rust unit tests + Playwright E2E

### Out of Scope
- UI for authoring schemas (deferred to ui-panels cycle)
- Schema migration / versioning beyond the `version: "0.1"` field (future change)
- Schema inheritance / composition (future)
- Dynamic built-in schema modifications (built-ins are immutable)
- Scene validation against updated schemas (existing scenes may reference outdated schemas; commands will fail at next `AddComponent` if schema is gone)

## Capabilities

### New Capabilities
- `schema-registry-persistence` — save/load per-schema to OPFS at `schemas/<type_id>.schema.json`
- `schema-registry-mutable` — register/unregister user-defined schemas at runtime
- `schema-registry-restore` — auto-load schemas referenced by project metadata

### Modified Capabilities
None.

## Approach

**Per-schema files** in OPFS at `schemas/<type_id>.schema.json`. This is granular and supports incremental save/load. Project metadata `schemas: Vec<String>` tracks which schemas belong to the project (similar to `scenes: Vec<String>`).

**Two-layer registry:**
- `OnceLock<ComponentSchemaRegistry>` holds **built-ins** (immutable, seeded once)
- `RefCell<ComponentSchemaRegistry>` holds **user-defined** additions (mutable)
- `combined_registry()` returns a merged registry for validation

The processor switches from `global_registry()` to `combined_registry()`. `global_registry()` still returns built-ins for backward compat.

**Built-in protection:** `register_schema()` rejects `editor.*` type_ids. `unregister_schema()` rejects `editor.*` type_ids. Custom schemas use `game.*` prefix per Hito 0 §6.3.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/schema.rs` | Modified | Add `register()`, `unregister()`, `is_user_type()`, mutable state |
| `crates/editor-core/src/persistence.rs` | Modified | Add `schemas_dir`, `schema_path()`, extend `ProjectMetadata` with `schemas: Vec<String>` |
| `crates/editor-core/src/lib.rs` | Modified | wasm_bindgen: save_schema, load_schema, delete_schema, list_schemas, register_schema, unregister_schema, combined_registry_size, is_builtin_type |
| `crates/editor-core/src/processor.rs` | Modified | Switch from `global_registry()` to `combined_registry()` for validation |
| `crates/editor-core/src/persistence.rs` | Modified | `load_project()` function: reads project.json + all scenes + all schemas |
| `frontend/src/engine-bridge.ts` | Modified | Expose schema persistence functions on window |
| `frontend/tests/engine.spec.ts` | Modified | Add 2 E2E tests: save/load custom schema, register and validate |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Built-in registry mutation breaks existing code | High | Two-layer separation; built-ins immutable |
| Existing scenes reference schemas that get deleted | Med | Document constraint; commands fail at next AddComponent if missing |
| Auto-restore timing (Bevy already started) | Low | `load_project()` is called before `start_engine()` OR after; document |
| Schema file collision with built-ins | Low | Built-in protection in `register_schema` |
| Many schemas slow down auto-restore | Low | Per-schema files, parallelize if needed in future |

## Rollback Plan

Revert schema.rs and lib.rs changes; persistence.rs reverts to v0.2.0 state. Single-PR makes revert clean.

## Dependencies

Existing: `serde`, `serde_json`, `wasm-bindgen`, `wasm-bindgen-futures`, `serde-wasm-bindgen`, `js-sys`, OPFS bridge. No new crates.

## Success Criteria

- [ ] `save_schema("game.PlayerHealth")` writes `schemas/game.PlayerHealth.schema.json`
- [ ] `load_schema("game.PlayerHealth")` reads + registers in combined registry
- [ ] `list_schemas()` returns array of all saved schema type_ids (built-ins + user)
- [ ] `register_schema(json)` adds schema to combined registry without saving to OPFS
- [ ] `unregister_schema("game.PlayerHealth")` removes from combined registry
- [ ] Built-in protection: `register_schema("editor.Foo", ...)` fails
- [ ] `combined_registry()` returns built-ins + user schemas
- [ ] `AddComponent` validates against combined registry
- [ ] Auto-restore: `load_project()` reads scenes + schemas
- [ ] `project.json` includes `schemas: Vec<String>`
- [ ] Roundtrip preserves unknown fields (ADR-0003)
- [ ] All 19 existing Playwright tests pass (no regression)
- [ ] All 84 existing Rust tests pass (no regression)
- [ ] 2+ new Playwright tests pass
- [ ] WASM builds clean