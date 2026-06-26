# Verify Report: schema-registry-persistence

> Phase: sddk-verify · Path: A-lite · Verdict: **PASS**

## Lens 1: Spec Compliance

### §2 schema-registry-persistence

| Requirement | Status | Evidence |
|---|---|---|
| save_schema writes to OPFS | PASS | `test_save_schema_roundtrip` |
| save_schema for built-in | PASS | Built-ins are saveable |
| save_schema missing fails | PASS | Returns Err "Schema not found" |
| load_schema reads + registers | PASS | `test_load_schema_registers_in_combined` |
| load_schema replaces existing | PASS | `test_load_schema_replaces_existing_user` |
| load_schema missing fails | PASS | JS bridge returns `File not found` |
| load_schema malformed fails | PASS | serde_json::from_str returns error |
| list_schemas returns built-ins | PASS | 5 schemas on fresh init |
| list_schemas returns user too | PASS | Combined registry iter |
| delete_schema removes user | PASS | `test_delete_schema_user` |
| delete_schema rejects built-in | PASS | Returns Err "Cannot delete built-in schema" |
| Per-schema file granularity | PASS | `schemas/<type_id>.schema.json` per file |

**§2 Coverage: 12/12 (100%)**

### §3 schema-registry-mutable

| Requirement | Status | Evidence |
|---|---|---|
| register_schema adds in-memory | PASS | `test_register_schema_adds_user` |
| register_schema replaces existing | PASS | `test_register_schema_replaces_existing_user` |
| register_schema rejects built-in | PASS | Returns Err "CannotRegisterBuiltin" |
| register malformed JSON fails | PASS | serde_json::from_str returns error |
| unregister_schema removes user | PASS | `test_unregister_schema_removes_user` |
| unregister_schema rejects built-in | PASS | Returns Err "CannotUnregisterBuiltin" |
| unregister_schema no-op missing | PASS | Returns Ok |
| combined_registry returns merged | PASS | Built-ins + user; validation uses this |

**§3 Coverage: 8/8 (100%)**

### §4 schema-registry-restore

| Requirement | Status | Evidence |
|---|---|---|
| load_project restores scenes + schemas | PASS | Playwright E2E verifies end-to-end |
| load_project atomic on missing schema | PASS | Returns Err, no partial state |
| project.json includes schemas list | PASS | `ProjectMetadata.schemas: Vec<String>` with `#[serde(default)]` |

**§4 Coverage: 3/3 (100%)**

## Lens 2: Test Quality

| Metric | Value |
|---|---|
| Rust unit tests | **97 passed** (11 new schema + 2 new persistence + 84 existing) |
| WASM build | **PASS** in 38.46s |
| Playwright E2E tests | **21/21 passed** (2 new schema + 19 existing) |
| Edge cases | Built-in protection, missing file, malformed JSON, replace existing |
| Backward compat | All 19 prior Playwright tests pass unchanged |
| Schema versioning | `#[serde(default)]` ensures old project.json files parse |

**Score: 10/10** — Comprehensive coverage including atomic restore.

## Lens 3: Design Coherence

| Invariant | Status | Evidence |
|---|---|---|
| OPFS directory structure (§5.2) | PASS | `schemas/<type_id>.schema.json` |
| Schemas global (§6.3) | PASS | combined_registry accessible from any code path |
| Forward compatibility (ADR-0003) | PASS | Roundtrip preserves unknown fields |
| Single Bevy canvas (ADR-0002) | PASS | Registry lives outside Bevy World |
| JSON source of truth (ADR-0001) | PASS | Each schema is JSON file |
| Built-in protection | PASS | `is_builtin_type` check in register/unregister/delete |
| Stable IDs preserved | PASS | StableId type unchanged |
| Atomic restore | PASS | load_project fails before any side-effect on error |

**Score: 8/8 (100%)**

### Architectural decisions honored
1. ✅ Two-layer registry (built-ins in OnceLock, user in thread_local RefCell)
2. ✅ Per-schema files (granular save/load)
3. ✅ Built-in protection via `editor.` prefix
4. ✅ Combined registry for validation
5. ✅ `ProjectMetadata.schemas` with backward-compat `#[serde(default)]`
6. ✅ `load_project()` atomic restore
7. ✅ All 9 wasm_bindgen functions exposed
8. ✅ `opfsDeleteFile` JS bridge extension

## Acceptance Criteria (from spec §6)

- [x] Every §2 scenario passes (12/12)
- [x] Every §3 scenario passes (8/8)
- [x] Every §4 scenario passes (3/3)
- [x] Combined registry returns built-ins + user schemas
- [x] Built-in protection enforced
- [x] Auto-restore via `load_project()` works end-to-end
- [x] `project.json` includes `schemas: Vec<String>`
- [x] Roundtrip preserves unknown fields
- [x] All 19 existing Playwright tests pass
- [x] All 84 existing Rust tests pass
- [x] 2 new Playwright tests pass
- [x] WASM builds clean

## Issues Found

- **0 critical**
- **0 warnings** (only existing unused-code warnings)
- **0 suggestions**

## Verdict

**PASS** — Ready for archive.