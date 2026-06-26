# Explore Report: opfs-persistence

> Change: `opfs-persistence` · Phase: sddk-explore · Path: A-lite · Context quality: C2
> Model: MiniMax-M3 (orchestrator)

---

## 1. Current State (from previous cycles)

The previous cycles delivered a complete editor core:
- `SceneDocument` with stable IDs, hierarchy, components
- `ComponentSchemaRegistry` with 5 built-in schemas
- `Command` system with 9 variants + reversibility + validation
- `OperationLog` with undo/redo
- Bevy integration via `SceneDocumentState` Resource + `rebuild_preview_world`
- wasm_bindgen surface: `create_buses`, `load_scene_json`, `dispatch_command`, `undo`, `redo`, `get_log_state`
- 79 Rust unit tests + 16 Playwright E2E tests passing

### 1.1 Existing wasm_bindgen pattern

`load_scene_json(&str)` is the canonical pattern for browser↔WASM JSON I/O:
```rust
#[wasm_bindgen]
pub fn load_scene_json(json: &str) -> Result<(), JsValue> {
    let doc: SceneDocument = serde_json::from_str(json)?;
    SCENE_DOC.with(|s| *s.borrow_mut() = Some(doc));
    Ok(())
}
```

### 1.2 Existing Cargo.toml deps

```toml
[dependencies]
bevy = { version = "0.19", default-features = false, features = ["2d"] }
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"

[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys = { version = "0.3", features = ["console"] }
```

`web-sys` has only `console`. For OPFS direct access we'd need to add Window/WorkerGlobalScope features. **But that's the wrong approach** — see §4.1.

---

## 2. What is OPFS?

**Origin Private File System** is a browser API for filesystem-like storage scoped to the origin. Key properties:
- Browser-native, no server needed
- Filesystem-like (files and directories)
- Async API accessible from main thread (`navigator.storage.getDirectory()`)
- Sync API only available in **Web Workers** (`FileSystemSyncAccessHandle`)
- Supports binary files (useful for future assets: spritesheets, fonts)

### 2.1 Hito 0 §5.2 OPFS directory structure

```
project.json              ← Project metadata, registry reference
scenes/                   ← SceneDocument files (*.scene.json)
schemas/                  ← Component Schema Registry entries (*.schema.json)
assets/                   ← Asset files (images, future: spritesheets, fonts)
entities/                 ← Entity Template files (*.template.json)
.editor/                  ← Editor state (selection, viewport, preferences)
```

### 2.2 Worker broker pattern

Hito 0 spec mentions "Sync access via worker broker pattern". This is needed because:
- OPFS sync API requires worker context (not main thread)
- Bevy's main loop is single-threaded WASM (can't block)
- Worker handles file I/O, posts messages back to main

For Hito 0 MVP, we can use the **async API** from the main thread (no worker needed). This is simpler and sufficient for save/load of small JSON files (SceneDocuments are typically <100KB).

---

## 3. Gap Analysis

| Need | Current state | Gap |
|------|---------------|-----|
| Save SceneDocument to OPFS | None | Need JS bridge for OPFS calls |
| Load SceneDocument from OPFS | Only `load_scene_json(json_string)` | Need OPFS file read |
| Project metadata | None | Need `project.json` schema |
| Scenes directory structure | None | Need to create dirs |
| Error handling | JsValue string errors | Need typed errors |
| Roundtrip without data loss | Verified for JSON | Need to verify across OPFS save/load |

---

## 4. Binding Constraints (from Hito 0 §5.2 + CONTEXT.md)

1. **OPFS is the persistence layer** (§5.2) — chosen over localStorage and IndexedDB
2. **Defold-inspired directory structure** (§5.2) — `scenes/`, `schemas/`, `assets/`, `entities/`, `.editor/`
3. **Editor-data-first** — physical folders for editor artifacts, not Bevy runtime
4. **Roundtrip without data loss** (Hito 0 Success Criterion #2) — save 50+ entities, reload, verify
5. **Project metadata** — `project.json` references the registry and contains scene list
6. **Browser-native** — no server, no IndexedDB abstractions beyond OPFS
7. **Forward compatibility** — preserve unknown fields across save/load (ADR-0003)

---

## 5. Codebase Risks

### 5.1 OPFS API surface in WASM (High risk if direct)

Trying to expose OPFS to WASM via `web-sys` would require enabling many features:
- `Window`, `WorkerGlobalScope`, `StorageManager`, `FileSystemDirectoryHandle`, `FileSystemFileHandle`, `FileSystemWritableFileStream`, etc.
- ~10+ feature flags
- Type conversion between Rust and JS bindings
- Fragile across browser versions

**Mitigation:** Use a **JS bridge** instead. Define `wasm_bindgen` extern functions that call into JS, where the OPFS API runs natively. The JS side does the actual file I/O.

### 5.2 OPFS Browser Support (Low-Medium)

OPFS is supported in:
- Chrome 86+ (full)
- Firefox 111+ (full)
- Safari 16.4+ (full)

**Mitigation:** Feature detection in JS bridge. If OPFS unavailable, return error.

### 5.3 Worker broker complexity (Medium)

Hito 0 spec mentions worker broker. For MVP we can use async API directly.

**Mitigation:** Defer worker broker to future change. Use `navigator.storage.getDirectory()` async API in JS bridge. Bevy's main loop doesn't block on OPFS calls (async I/O).

### 5.4 OPFS Quota (Low)

Browsers limit OPFS storage (~10% of disk by default).

**Mitigation:** Catch quota errors in JS bridge. Surface typed errors.

### 5.5 Concurrent access (Low)

OPFS allows concurrent reads, exclusive writes. Single-threaded WASM means no contention in Hito 0.

**Mitigation:** Document the constraint. Future change for multi-tab sync.

### 5.6 Async vs sync API (Medium)

OPFS async is easier but slower per call. Sync API is faster but worker-only.

**Mitigation:** For Hito 0 MVP, async is fine. SceneDocuments are small (<100KB).

---

## 6. Architecture Options Considered

### Option A: Direct OPFS via web-sys
- ❌ Complex feature surface, fragile bindings, ~500 LOC just for type glue

### Option B: JS bridge with wasm_bindgen externs (RECOMMENDED)
- ✅ Clean: Rust defines extern `opfs_save_file(path, bytes)` → `opfs_save_file = (path, bytes) => ...`
- ✅ JS implements actual OPFS calls
- ✅ ~50 LOC Rust (extern decls), ~100 LOC JS (bridge)
- ✅ Future-proof: can add features without touching Rust

### Option C: Worker broker
- ⏳ Overkill for MVP. Defer.

---

## 7. Effort Estimate

| Work item | Size | Notes |
|-----------|------|-------|
| `wasm_bindgen extern` declarations for OPFS | XS | 4 functions: save, load, list, exists |
| JS bridge module `opfs-bridge.ts` | M | Async OPFS calls, path resolution, error mapping |
| Rust `save_scene(name)` and `load_scene(name)` | S | Wraps extern calls + serde |
| `project.json` schema | S | Project metadata + scene list |
| Auto-create directory structure on first save | XS | Idempotent mkdir |
| Tests: OPFS roundtrip in Playwright | M | Save scene → reload → verify |
| Tests: Rust unit tests for save/load logic | S | Mock extern, test path resolution |

**Total:** Small-medium. ~300 LOC across Rust + TS.

---

## 8. Architecture Decisions Needed (for design phase)

1. **JS bridge location** — New module `frontend/src/opfs-bridge.ts` vs inline in `engine-bridge.ts`. Recommend separate module for clarity.
2. **Path format** — Hito 0 says `scenes/level_01.scene.json`. Forward slashes work in OPFS. Use template `scenes/{name}.scene.json`.
3. **Project metadata location** — `project.json` at root of OPFS. Contains scene list.
4. **First-run behavior** — If OPFS empty, save creates directories + `project.json` automatically.
5. **Error mapping** — JS bridge catches OPFS errors, returns typed JSON `{ok: bool, error: string}` (wasm_bindgen externs can't return `Result`).
6. **Sync vs async** — Use async API in MVP. Worker broker deferred.
7. **Conflict resolution** — If file exists, overwrite. No versioning in MVP.

---

## 9. Recommendations for Proposal

1. **Capabilities (NEW):**
   - `opfs-persistence` — save/load SceneDocument to/from OPFS via JS bridge
2. **Approach:** JS bridge module with `wasm_bindgen` extern declarations. Async OPFS API. Rust provides high-level `save_scene(name)` / `load_scene(name)` that wrap the extern calls. Auto-create directory structure. Typed errors.
3. **Reuse existing types:** `SceneDocument`, `serde_json` — do NOT reimplement.
4. **wasm_bindgen surface:**
   - `save_scene(name: &str) -> Result<String, JsValue>` — saves current SCENE_DOC
   - `load_scene(name: &str) -> Result<(), JsValue>` — loads from OPFS into SCENE_DOC
   - `list_scenes() -> Result<String, JsValue>` — returns JSON array of scene names
   - `project_exists() -> bool` — quick check for first-run
5. **JS bridge (`frontend/src/opfs-bridge.ts`):**
   - Exposes 4 functions: `opfsSaveFile`, `opfsLoadFile`, `opfsListFiles`, `opfsExists`
   - Feature-detects OPFS availability
   - Returns `{ok, value, error}` JSON for typed error handling
6. **Project metadata:** `project.json` at OPFS root with `version`, `name`, `scenes[]`. Created on first save.
7. **Tests:**
   - Rust unit: mock extern, test path resolution, error mapping
   - Playwright E2E: save scene → reload page → load scene → verify entities match
8. **Backward compat:** All existing 16 Playwright tests + 79 Rust tests pass unchanged. OPFS is additive (only used if explicitly called).