# ADR-0014: Data-Only Hot Reload for Source and Asset Files

## Status
Accepted (2026-07-18)

## Context

Hito 4 Order 5 (hot-reload) enables the editor to detect when a user saves a source file (`.rs`) or asset file (`.bsn`) and immediately refresh the preview without requiring a full scene reload or page refresh.

The editor already has:
- An OPFS-backed `sources/` directory for Rust source files (`source_files.rs`)
- An OPFS-backed asset pipeline (`asset_files.ts`)
- A `DIRTY_FLAG` + `rebuild_preview_world()` pattern for triggering scene refresh
- A `COMMAND_BUS`/`EVENT_BUS` pattern for WASM↔Bevy communication

We need to extend this to support fine-grained file-level invalidation:
- Source files: invalidate the in-memory content cache for the specific file
- Asset files: invalidate the `ASSET_BODY_CACHE` entry and mark dirty
- Force-reload: clear all caches + logic graph doc

## Decision

**Data-only hot reload.** The hot-reload system invalidates cached data structures. It does NOT recompile Rust code or reload textures in this implementation.

### Architecture

```
TypeScript save hooks
  ├── code-files.ts: writeSourceFile → emit {type:'hot-reload-source', fileId}
  └── asset-files.ts: importAssetFile / deleteAssetFile → emit {type:'hot-reload-asset', assetId}

engine-bridge.ts (subscribes to event bus)
  ├── hotReloadSource(id) → window.hot_reload_source_wasm(id)
  ├── hotReloadAsset(id)  → window.hot_reload_asset_wasm(id)
  └── forceReload()       → window.force_reload_wasm()

Rust (wasm32)
  ├── HOT_RELOAD_BUS: thread_local Vec<HotReloadRequest>
  ├── process_hot_reload_requests(): drains + dedupes bus
  │   ├── Source{file_id}  → source_files::invalidate_cache(file_id)
  │   ├── Asset{asset_id}  → ASSET_BODY_CACHE.remove(asset_id) + mark_dirty()
  │   └── ForceReloadAll   → clear all caches + LOGIC_GRAPH_DOC=None + mark_dirty()
  └── Schedule: Update, process_hot_reload_requests.before(rebuild_preview_world)
```

### Key Design Decisions

1. **HOT_RELOAD_BUS is a simple `Vec`**, not a `HashMap` or deduping structure, because:
   - Deduplication happens at drain time via `HashSet<(u8,String)>`
   - Multiple rapid saves should all be coalesced into one invalidation per (variant, key)
   - The `Vec` naturally supports the "request queue" pattern used by `PLAY_MODE_REQUEST`

2. **Source cache uses `BTreeMap`** (not `HashMap`) to allow `const` initialization.
   `HashMap::new()` is not `const fn` in stable Rust as of 1.83.

3. **Dedup is by `(variant_index, key)`** — `(Source, "a.rs")` and `(Asset, "a.rs")` are distinct keys.

4. **`mark_dirty()` is always called on Asset and ForceReloadAll** — the preview world must rebuild. Source invalidation does NOT call `mark_dirty()` because source files affect code generation only, not the scene graph directly.

5. **No double-mark_dirty guard needed** — the dedup step ensures at most one `mark_dirty()` per (variant, key) per frame.

## Deferred: Texture Reload

Texture assets (`.png`, `.jpg`) are handled by a separate pipeline. Per design.md, texture hot-reload requires:
- Re-issuing `bevy::render::texture::Image` handles
- Updating GPU texture views
- This is out of scope for Hito 4 Order 5.

A follow-up ADR will address texture hot-reload once the data-invalidation foundation is validated.

## Consequences

### Positive
- File-level invalidation (not full-scene rebuild)
- Non-blocking: drain happens next frame, not synchronously
- wasm-bindgen exports are `Result<(), JsValue>` — errors propagate to JS console
- Deduplication prevents redundant invalidations from rapid saves
- Mirrors existing codebase patterns (COMMAND_BUS, EVENT_BUS, DIRTY_FLAG)

### Negative
- Data-only: does not recompile Rust source to WASM
- Texture assets not handled in this PR
- No invalidation batching across frames (handled by dedup, but not cross-frame coalescing)

## Reference

- Related ADR-0013: Build & Run Loop — Enhanced Preview Mode
- `crates/editor-core/src/lib.rs`: `HotReloadRequest`, `HOT_RELOAD_BUS`, `process_hot_reload_requests`
- `crates/editor-core/src/source_files.rs`: `SOURCE_FILE_REGISTRY`, cache functions
- `frontend/src/services/hot-reload.ts`: TypeScript event bus service
- `frontend/src/hooks/useHotReloadStatus.ts`: React hook for UI binding
