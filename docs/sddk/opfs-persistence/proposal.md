# Proposal: OPFS Persistence for Hito 0

## Intent

Hito 0 §5.2 mandates OPFS as the persistence layer with a Defold-inspired directory structure (`scenes/`, `schemas/`, `assets/`, `entities/`, `.editor/`), but no implementation exists. Without persistence, every page reload loses all editor work — Success Criterion #2 ("Save/load roundtrip: save a scene with 50+ entities, reload, and verify zero data loss") cannot be validated. This change delivers save/load of `SceneDocument` to OPFS via a JS bridge (clean separation from Rust), establishing the persistence foundation that project metadata, schema registry, and asset storage will build on.

## Scope

### In Scope
- JS bridge module `frontend/src/opfs-bridge.ts` with async OPFS calls
- `wasm_bindgen extern` declarations for 4 OPFS operations (save, load, list, exists)
- Rust `save_scene(name)`, `load_scene(name)`, `list_scenes()`, `project_exists()` wasm_bindgen functions
- `project.json` schema at OPFS root with version, name, scenes list
- Auto-create directory structure on first save (idempotent)
- Roundtrip save → reload → load → verify (Playwright E2E)
- Rust unit tests for save/load logic (path resolution, error mapping)
- Backward compat: all 16 existing Playwright tests + 79 Rust tests pass unchanged

### Out of Scope
- Worker broker pattern (defer — async OPFS API sufficient for MVP)
- Schema registry persistence (separate change — schemas/)
- Asset storage (separate change — assets/, requires asset loading pipeline)
- Entity template persistence (separate change — entities/)
- Editor state persistence (separate change — .editor/)
- Conflict resolution / multi-tab sync (separate change)
- OPFS quota management (future — error handling now, mitigation later)

## Capabilities

### New Capabilities
- `opfs-persistence` — save/load SceneDocument to/from OPFS via JS bridge with directory structure
- `project-metadata` — project.json at OPFS root with version, name, scene list

### Modified Capabilities
None.

## Approach

**JS bridge pattern.** Rust defines `wasm_bindgen extern` declarations for 4 OPFS operations. JS bridge implements them using native `navigator.storage.getDirectory()` async API. Rust wraps extern calls in high-level functions (`save_scene`, `load_scene`, `list_scenes`, `project_exists`) that handle serialization and error mapping.

JS bridge returns `{ok: bool, value?: any, error?: string}` JSON for typed error handling (wasm_bindgen externs can't return `Result`).

Path convention (from Hito 0 §5.2):
- `project.json` at root
- `scenes/<name>.scene.json` for SceneDocuments
- (Future: `schemas/`, `assets/`, `entities/`, `.editor/`)

First-run behavior: if OPFS is empty, `save_scene` creates `project.json` automatically with the scene entry. Subsequent saves update `project.json` scene list.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `frontend/src/opfs-bridge.ts` | New | OPFS wrapper module (feature detection, async calls, error mapping) |
| `frontend/src/engine-bridge.ts` | Modified | Expose OPFS functions on window for tests |
| `crates/editor-core/src/lib.rs` | Modified | Add `wasm_bindgen extern` for 4 OPFS ops, high-level `save_scene`/`load_scene`/`list_scenes`/`project_exists` |
| `crates/editor-core/src/persistence.rs` | New | Rust helper: project metadata, path resolution, save/load logic |
| `frontend/tests/engine.spec.ts` | Modified | 2 Playwright E2E tests: save/load roundtrip, list scenes |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| OPFS unavailable in browser (Firefox <111, Safari <16.4) | Low | Feature detection in JS bridge, typed error |
| OPFS quota exceeded | Low | Catch QuotaExceededError, surface as typed error |
| Direct web-sys OPFS exposure | High if attempted | Avoided — JS bridge pattern is cleaner |
| Concurrent access from multiple tabs | Low (Hito 0 single-tab) | Document constraint; future sync change |
| `load_scene` replaces SCENE_DOC, but `dispatch_command` checks "No scene loaded" | Med | After `load_scene`, mark dirty so Bevy rebuilds |
| Roundtrip data loss | Med | Playwright E2E test with 50+ entities |

## Rollback Plan

Revert lib.rs to remove new wasm_bindgen functions; delete opfs-bridge.ts and persistence.rs. Single-PR makes revert a clean `git revert`.

## Dependencies

Existing: `serde`, `serde_json`, `wasm-bindgen`. No new crates.

JS side: native OPFS API (no npm deps).

## Success Criteria

- [ ] `save_scene(name)` writes current SceneDocument to `scenes/<name>.scene.json`
- [ ] `load_scene(name)` reads `scenes/<name>.scene.json` into SCENE_DOC
- [ ] `list_scenes()` returns JSON array of scene names from project.json + filesystem
- [ ] `project_exists()` returns true if project.json exists
- [ ] First-run: save_scene creates `project.json` automatically
- [ ] Roundtrip test: save scene with 50+ entities → reload page → load scene → all entities present
- [ ] Unknown fields preserved across save/load (ADR-0003)
- [ ] OPFS unavailable → typed error, no panic
- [ ] All 16 existing Playwright tests pass (no regression)
- [ ] All 79 existing Rust unit tests pass (no regression)
- [ ] 2 new Playwright tests pass
- [ ] WASM builds clean