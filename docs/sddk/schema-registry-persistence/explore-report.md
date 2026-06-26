# Explore Report: schema-registry-persistence

> Change: `schema-registry-persistence` · Phase: sddk-explore · Path: A-lite · Context quality: C1
> Model: MiniMax-M3 (orchestrator)

---

## 1. Current State

### 1.1 Previous cycles delivered
- **scene-document**: SceneDocument, Entity, ComponentInstance types
- **command-system**: Command processor with validation against registry
- **operation-log**: undo/redo with semantic commands
- **opfs-persistence**: JS bridge + save_scene/load_scene/list_scenes/project_exists

### 1.2 ComponentSchemaRegistry (existing)

Located in `crates/editor-core/src/schema.rs`:
- `ComponentSchema { type_id, display_name, fields: Vec<FieldDef>, exports_to_bevy: bool }`
- `FieldDef { name, field_type, default, constraints }`
- `FieldType` enum: `String | F32 | Bool | Vec2 | Color | Anchor | AssetReference`
- `Constraint` enum: `Min(f32) | Max(f32) | NonEmpty`
- `ComponentSchemaRegistry { schemas: HashMap<String, ComponentSchema> }`
- 5 built-in schemas seeded via `with_builtin_seeds()`
- Global singleton via `OnceLock<ComponentSchemaRegistry>` exposed as `global_registry()`

**Key issue:** The registry is currently **read-only-after-init** — `with_builtin_seeds()` is called once in `OnceLock::get_or_init`. There's no way to:
1. Add custom user-defined schemas
2. Save registry state to disk
3. Load registry state from disk
4. Persist modifications across sessions

### 1.3 How registry is used today
- `processor::validate()` calls `global_registry().get(type_id)` to validate `AddComponent`
- If schema doesn't exist, validation fails with `UnknownSchema`
- All scenes depend on the registry being populated at startup

### 1.4 OPFS bridge (existing)

From `opfs-persistence` cycle:
- `opfsSaveFile(path, contents)` — async, returns `{ok, error?}`
- `opfsLoadFile(path)` — async, returns `{ok, value?, error?}`
- `opfsListFiles(path)` — async, returns `{ok, value?: string[], error?}`
- `opfsExists(path)` — async, returns boolean
- `opfs_load_file_raw`, etc. — wasm_bindgen externs

Path convention:
- `project.json` at root
- `scenes/<name>.scene.json`

Now extending to:
- `schemas/` directory for schema files

---

## 2. Gap Analysis

| Need | Current state | Gap |
|------|---------------|-----|
| Persist schema registry to OPFS | None | Need wasm_bindgen functions to save/load |
| Path convention `schemas/<type_id>.schema.json` | None | Need helper to compute path |
| Update `ComponentSchemaRegistry` at runtime | Read-only after init | Need `register()`, `clear()`, `replace_all()` |
| Roundtrip preservation | N/A | Need to verify schema serialization |
| Schema versioning | None | Per ADR-0001, `version: "0.1"` field exists; need to preserve |
| List saved schemas | None | Need `list_schemas()` |
| Project metadata integration | `project.json` has `scenes` only | Need to add `schemas` array OR separate registry file |

---

## 3. Binding Constraints (from Hito 0 §5.2 + ADR-0003 + CONTEXT.md)

1. **OPFS directory structure** (§5.2): `schemas/` directory for schema files
2. **Forward compatibility** (ADR-0003): preserve unknown fields across save/load
3. **Schema is project-global** (§6.3): "Schemas live in `schemas/` and are referenced by `type_id`"
4. **Schema is JSON-serializable**: must roundtrip through serde JSON
5. **Editor-owned types**: schema registry lives outside Bevy World (ADR-0002)
6. **Built-in seeds remain available**: `global_registry()` continues to work even if user-defined schemas are added
7. **Type IDs are namespaced**: `editor.Transform2D`, `game.PlayerHealth` etc. Built-ins use `editor.` prefix

---

## 4. Codebase Risks

### 4.1 Replacing the global registry (Medium)

Currently `global_registry()` returns a `&'static ComponentSchemaRegistry` from `OnceLock`. To support dynamic schemas, we need to allow replacement.

**Mitigation:** Use a `RwLock<ComponentSchemaRegistry>` instead of `OnceLock`. The `register()` method adds new schemas; `replace_all()` swaps the entire registry. Existing callers (via `global_registry()`) continue to work — they get a snapshot of the current state.

Actually, simpler: keep `OnceLock` for the **built-in** registry, and add a separate `RwLock<ComponentSchemaRegistry>` for the **user-defined** additions. The processor combines both via `combined_registry()` for validation.

Or even simpler: the `OnceLock` holds the **initial seed**, and a separate `RwLock<Vec<ComponentSchema>>` holds user additions. Validation consults both.

Actually, the cleanest approach: keep `OnceLock<ComponentSchemaRegistry>` for built-ins (immutable), add `RwLock<ComponentSchemaRegistry>` for the **full** registry (built-ins + user additions). The processor calls `combined_registry()` which is a merge.

### 4.2 Schema versioning (Low)

Schemas have a `version` field per ADR-0001. For Hito 0 MVP, version is `"0.1"`. Future changes will need migration. For now, just preserve the field.

**Mitigation:** Document as future work. Roundtrip preserves the field but doesn't enforce version compatibility.

### 4.3 Built-in schema protection (Low)

Built-in schemas should not be deletable. User additions should be removable.

**Mitigation:** `register()` for new schemas only. `unregister_user_schema()` checks against the built-in prefix `editor.` to prevent deletion of built-ins.

### 4.4 Schema file naming (Low)

Type IDs contain dots (`editor.Transform2D`). OPFS file names don't allow `/` but dots are OK.

**Mitigation:** Use `schemas/<type_id>.schema.json` — `<type_id>` is already a valid filename. Example: `schemas/editor.Transform2D.schema.json`.

### 4.5 Concurrent access (Low)

WASM is single-threaded. `RwLock` is overkill but matches Rust idioms.

**Mitigation:** Use `RefCell` (single-threaded WASM) for simplicity.

### 4.6 Schema-aware project metadata (Low)

Project metadata currently has `scenes: Vec<String>`. Should it also track `schemas: Vec<String>`?

**Mitigation:** Add `schemas: Vec<String>` to ProjectMetadata for completeness. Existing project.json files without `schemas` field still work (serde default).

---

## 5. Effort Estimate

| Work item | Size | Notes |
|-----------|------|-------|
| Extend `ComponentSchemaRegistry` with mutable state | S | `register()`, `unregister()`, `iter_all()` |
| Add `RwLock` for user-defined additions | S | Separate from `OnceLock` built-ins |
| `schema_path(type_id)` helper | XS | One line |
| `save_schema(type_id)` wasm_bindgen | S | Serializes single schema, writes to OPFS |
| `load_schema(type_id)` wasm_bindgen | S | Reads from OPFS, registers in registry |
| `list_schemas()` wasm_bindgen | S | Lists saved schema files from OPFS |
| `delete_schema(type_id)` wasm_bindgen | S | Removes from OPFS + unregisters |
| `project.json` schema list | XS | Extend ProjectMetadata with `schemas: Vec<String>` |
| Tests: roundtrip, register/unregister, list | S | Rust unit tests |
| E2E: save/load custom schema | M | Playwright test |

**Total:** Small. ~250 LOC across Rust + TS.

---

## 6. Architecture Decisions Needed (for design phase)

1. **Storage granularity** — One file per schema (`schemas/<type_id>.schema.json`) vs single `schemas.json` index. Per-file is cleaner for incremental save/load.
2. **Built-in protection** — Reject `register()` for `editor.*` type_ids? Or allow override?
3. **Auto-load on startup** — Should OPFS schemas auto-load into the registry? Yes (project restore).
4. **Combined registry for validation** — Should `global_registry()` return built-ins + user, or just built-ins?
5. **Schema naming convention** — `schemas/<type_id>.schema.json` (with dots) is fine for OPFS
6. **Project metadata schema list** — Add `schemas: Vec<String>` to ProjectMetadata
7. **Reload strategy** — After `load_schema`, validate existing scenes still pass? Or just register and let commands fail at next `AddComponent`?

---

## 7. Recommendations for Proposal

1. **Capabilities (NEW):**
   - `schema-registry-persistence` — save/load individual schemas to OPFS at `schemas/<type_id>.schema.json`
   - `schema-registry-mutable` — register/unregister user-defined schemas at runtime
   - `schema-registry-restore` — auto-load all saved schemas on project restore

2. **Approach:**
   - Per-schema files (one JSON file per ComponentSchema)
   - Keep `OnceLock<ComponentSchemaRegistry>` for built-ins (immutable)
   - Add `RefCell<ComponentSchemaRegistry>` for user-defined additions (mutable)
   - Combined registry function returns built-ins + user for validation
   - `register(type_id, schema)` rejects `editor.*` type_ids (built-ins immutable)
   - `unregister(type_id)` rejects `editor.*` type_ids
   - Auto-load on `load_project()` (new function that reads project.json + all scenes + all schemas)

3. **Reuse existing:** `ComponentSchema`, `FieldDef`, `FieldType`, `Constraint`, `ComponentSchemaRegistry`, OPFS bridge

4. **wasm_bindgen surface:**
   - `save_schema(type_id: &str) -> Result<String, JsValue>`
   - `load_schema(type_id: &str) -> Result<(), JsValue>`
   - `list_schemas() -> Result<JsValue, JsValue>` (returns Vec<String>)
   - `delete_schema(type_id: &str) -> Result<(), JsValue>`
   - `register_schema(schema_json: &str) -> Result<(), JsValue>` (registers in memory without saving to OPFS)
   - `combined_registry_size() -> usize` (for UI hooks)
   - `is_builtin_type(type_id: &str) -> bool`

5. **JS bridge:** Reuse existing OPFS bridge (already supports any path). No new functions.

6. **Tests:**
   - Rust unit: roundtrip a schema, register/unregister, built-in protection
   - Playwright E2E: save custom schema → reload → load_schema → register via AddComponent works

7. **Backward compat:** Existing `global_registry()` returns built-ins (5 schemas). New `combined_registry()` returns built-ins + user. Processor changes from `global_registry()` to `combined_registry()`.