# Spec: Schema Registry Persistence

> Change: `schema-registry-persistence` · Phase: sddk-spec (draft) · Path: A-lite

## §1. Spec Metadata

- **Change:** `schema-registry-persistence`
- **Phase:** spec (draft, awaiting design)
- **Path:** A-lite
- **Capabilities (NEW):**
  - `schema-registry-persistence` — save/load per-schema to OPFS at `schemas/<type_id>.schema.json`
  - `schema-registry-mutable` — register/unregister user-defined schemas at runtime
  - `schema-registry-restore` — auto-load schemas referenced by project metadata
- **Source proposal:** [`docs/sddk/schema-registry-persistence/proposal.md`](../schema-registry-persistence/proposal.md)
- **Source explore:** [`docs/sddk/schema-registry-persistence/explore-report.md`](../schema-registry-persistence/explore-report.md)
- **Authoritative references:**
  - [Hito 0 §5.2 (Persistence — OPFS, schemas/ directory)](../../hito-0-spec.md)
  - [Hito 0 §6.3 (Component Schemas with Metadata)](../../hito-0-spec.md)
  - [Hito 0 §6.9 (Forward Compatibility)](../../hito-0-spec.md)
  - [ADR-0002 — Single Bevy renders canvas](../../adr/0002-single-bevy-renders-canvas.md)
  - [ADR-0003 — Forward compat via serde_json::Value](../../adr/0003-forward-compat-via-serde-json-value.md)
  - Previous cycles: [`docs/sddk/scene-document/`](../scene-document/), [`docs/sddk/command-system/`](../command-system/), [`docs/sddk/opfs-persistence/`](../opfs-persistence/)

---

## §2. Capability: `schema-registry-persistence`

### Requirement: save_schema writes one schema to OPFS

The system MUST provide `save_schema(type_id: &str)` that serializes the schema with that type_id and writes it to `schemas/<type_id>.schema.json`.

#### Scenario: Save a user-defined schema
- GIVEN a schema registered as `game.PlayerHealth`
- WHEN `save_schema("game.PlayerHealth")` is called
- THEN `schemas/game.PlayerHealth.schema.json` exists in OPFS
- AND the file contents match the JSON serialization of the schema
- AND the function returns `Ok("schemas/game.PlayerHealth.schema.json")`

#### Scenario: Save a built-in schema
- GIVEN the built-in `editor.Transform2D` schema
- WHEN `save_schema("editor.Transform2D")` is called
- THEN the function returns `Ok` (built-ins are also saveable for project portability)
- AND `schemas/editor.Transform2D.schema.json` exists

#### Scenario: Save non-existent schema fails
- GIVEN no schema with type_id `game.NonExistent`
- WHEN `save_schema("game.NonExistent")` is called
- THEN the function returns `Err` with "Schema not found"

### Requirement: load_schema reads from OPFS and registers

The system MUST provide `load_schema(type_id: &str)` that reads `schemas/<type_id>.schema.json` from OPFS and registers the schema in the combined registry.

#### Scenario: Load a user-defined schema
- GIVEN `schemas/game.PlayerHealth.schema.json` exists in OPFS
- AND the schema is not in the combined registry
- WHEN `load_schema("game.PlayerHealth")` is called
- THEN the combined registry now contains `game.PlayerHealth`
- AND the schema fields match the file content
- AND the function returns `Ok`

#### Scenario: Load replaces existing user schema
- GIVEN `game.PlayerHealth` is registered with version 1
- AND `schemas/game.PlayerHealth.schema.json` exists with version 2
- WHEN `load_schema("game.PlayerHealth")` is called
- THEN the combined registry now has version 2 of `game.PlayerHealth`

#### Scenario: Load non-existent file fails
- GIVEN no `schemas/missing.schema.json` in OPFS
- WHEN `load_schema("missing")` is called
- THEN the function returns `Err` with "File not found" or "Schema not found"

#### Scenario: Load malformed JSON fails
- GIVEN `schemas/broken.schema.json` contains invalid JSON
- WHEN `load_schema("broken")` is called
- THEN the function returns `Err` with parse error

### Requirement: list_schemas returns all registered schemas

The system MUST provide `list_schemas() -> Vec<String>` that returns all type_ids in the combined registry (built-ins + user).

#### Scenario: List returns built-ins on fresh init
- GIVEN the editor just initialized (no user schemas added)
- WHEN `list_schemas()` is called
- THEN it returns the 5 built-in type_ids: `editor.Name`, `editor.Transform2D`, `editor.Sprite2D`, `editor.Visible`, `editor.Locked`

#### Scenario: List returns user schemas too
- GIVEN `game.PlayerHealth` and `game.EnemyAI` registered
- WHEN `list_schemas()` is called
- THEN it returns all 7 schemas (5 built-in + 2 user)

### Requirement: delete_schema removes from OPFS and registry

The system MUST provide `delete_schema(type_id: &str)` that removes the schema from OPFS and unregisters it. Built-in schemas (`editor.*`) MUST be protected.

#### Scenario: Delete user schema
- GIVEN `game.PlayerHealth` is registered
- AND `schemas/game.PlayerHealth.schema.json` exists
- WHEN `delete_schema("game.PlayerHealth")` is called
- THEN the file is removed from OPFS
- AND the schema is removed from the combined registry
- AND the function returns `Ok`

#### Scenario: Delete built-in schema fails
- GIVEN the built-in `editor.Transform2D`
- WHEN `delete_schema("editor.Transform2D")` is called
- THEN the function returns `Err` with "Cannot delete built-in schema"
- AND the schema and file are unchanged

### Requirement: Per-schema file granularity

The system MUST use one OPFS file per schema (path: `schemas/<type_id>.schema.json`), NOT a single combined `schemas.json`.

#### Scenario: Each schema is its own file
- GIVEN 3 user schemas registered: `game.A`, `game.B`, `game.C`
- WHEN `list_schemas()` returns type_ids
- AND OPFS `schemas/` directory is inspected
- THEN 3 files exist: `game.A.schema.json`, `game.B.schema.json`, `game.C.schema.json`

---

## §3. Capability: `schema-registry-mutable`

### Requirement: register_schema adds schema in-memory without saving

The system MUST provide `register_schema(schema_json: &str)` that parses the JSON, validates, and adds the schema to the combined registry. Does NOT save to OPFS.

#### Scenario: Register a new user schema
- GIVEN the combined registry has 5 built-ins
- WHEN `register_schema('{"type_id": "game.PlayerHealth", "display_name": "Player Health", "fields": [...], "exports_to_bevy": true}')` is called
- THEN the combined registry has 6 schemas
- AND `game.PlayerHealth` is in the registry
- AND the file is NOT created in OPFS

#### Scenario: Register replaces existing user schema
- GIVEN `game.PlayerHealth` registered with field `hp: f32`
- WHEN `register_schema('{"type_id": "game.PlayerHealth", "display_name": "Player Health", "fields": [{"name": "mana", "field_type": "F32"}], "exports_to_bevy": true}')` is called
- THEN the registry's `game.PlayerHealth` now has field `mana` instead of `hp`

#### Scenario: Register built-in schema fails
- WHEN `register_schema('{"type_id": "editor.NewName", ...}')` is called
- THEN the function returns `Err` with "Cannot register built-in schema"

#### Scenario: Register malformed JSON fails
- WHEN `register_schema('not valid json')` is called
- THEN the function returns `Err` with parse error

### Requirement: unregister_schema removes user schema in-memory

The system MUST provide `unregister_schema(type_id: &str)` that removes a user schema from the combined registry without touching OPFS.

#### Scenario: Unregister user schema
- GIVEN `game.PlayerHealth` registered
- WHEN `unregister_schema("game.PlayerHealth")` is called
- THEN the registry no longer contains it
- AND the OPFS file is unchanged

#### Scenario: Unregister built-in schema fails
- WHEN `unregister_schema("editor.Transform2D")` is called
- THEN the function returns `Err` with "Cannot unregister built-in schema"

#### Scenario: Unregister non-existent schema is no-op
- GIVEN no schema `game.Missing`
- WHEN `unregister_schema("game.Missing")` is called
- THEN the function returns `Ok` (no-op success)

### Requirement: combined_registry returns merged view

The system MUST provide `combined_registry()` that returns a registry containing all built-ins + user schemas. Used by `processor::validate()`.

#### Scenario: Validation uses combined registry
- GIVEN `game.PlayerHealth` registered (user)
- WHEN `dispatch_command({command: {type: "AddComponent", entity_id: ..., type_id: "game.PlayerHealth", values: {...}}})` is called
- THEN validation succeeds
- AND the component is added

#### Scenario: Validation fails for unregistered schema
- WHEN `dispatch_command({command: {type: "AddComponent", type_id: "game.NotRegistered", ...}})` is called
- THEN validation fails with `UnknownSchema`

---

## §4. Capability: `schema-registry-restore`

### Requirement: load_project restores scenes + schemas

The system MUST provide `load_project() -> Result<(), JsValue>` that reads `project.json`, loads all referenced scenes, and registers all referenced schemas.

#### Scenario: Load complete project
- GIVEN OPFS has `project.json` with `scenes: ["level_01"]` and `schemas: ["game.PlayerHealth"]`
- AND `scenes/level_01.scene.json` exists
- AND `schemas/game.PlayerHealth.schema.json` exists
- WHEN `load_project()` is called
- THEN `project.json` is parsed
- AND `level_01` is loaded into SCENE_DOC
- AND `game.PlayerHealth` is registered in the combined registry
- AND the function returns `Ok`

#### Scenario: Load project with missing schema file
- GIVEN `project.json` references `game.Missing` schema
- AND `schemas/game.Missing.schema.json` does NOT exist
- WHEN `load_project()` is called
- THEN the function returns `Err` with "Schema file not found: game.Missing"
- AND no partial state is left (atomic restore)

### Requirement: project.json includes schemas list

The system MUST extend `ProjectMetadata` with `schemas: Vec<String>` field.

#### Scenario: project.json shape after schema save
- GIVEN a save happened including schema registration
- WHEN `project.json` is read
- THEN it contains `schemas: ["game.PlayerHealth"]` (or similar)

---

## §5. Out-of-Scope Behaviors (explicit non-goals)

- UI for authoring schemas (deferred to ui-panels cycle)
- Schema migration / versioning beyond the `version: "0.1"` field
- Schema inheritance / composition
- Built-in schema modifications
- Scene validation against updated schemas at load time

---

## §6. Acceptance Criteria

1. Every §2 scenario passes via Rust unit + Playwright E2E tests.
2. Every §3 scenario passes via Rust unit tests.
3. Every §4 scenario passes via Playwright E2E tests.
4. Combined registry returns built-ins + user schemas.
5. Built-in protection: cannot register/unregister/delete `editor.*` schemas.
6. Auto-restore via `load_project()` works end-to-end.
7. `project.json` includes `schemas: Vec<String>`.
8. Roundtrip preserves unknown fields (ADR-0003).
9. All 19 existing Playwright tests pass (no regression).
10. All 84 existing Rust tests pass (no regression).
11. 2+ new Playwright tests pass.
12. WASM builds clean.

---

## §7. Test Plan

| Section | Scenarios | Test type | Rough count |
|---|---|---|---|
| §2 save | user, built-in, missing | Rust unit + E2E | 3 |
| §2 load | user, replace, missing, malformed | Rust unit + E2E | 4 |
| §2 list | fresh, with user | Rust unit + E2E | 2 |
| §2 delete | user, built-in fails | Rust unit + E2E | 2 |
| §2 granularity | per-file check | E2E | 1 |
| §3 register | new, replace, built-in fails, malformed | Rust unit + E2E | 4 |
| §3 unregister | user, built-in fails, no-op | Rust unit | 3 |
| §3 combined | validation uses merged | Rust unit + E2E | 2 |
| §4 load_project | complete, missing file | E2E | 2 |
| §4 metadata | schemas list | Rust unit | 1 |
| **Total** | | | **~24 tests** |

Dev cycle: `cargo test --lib` (harness) + `just wasm` + `just test`.