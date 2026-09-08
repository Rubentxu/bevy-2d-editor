# H2.5 Block G — Spec

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-g`
**Path**: A-lite (explore → spec → tasks → build → verify → release → archive)
**Tag**: `v0.108.7` (to be released)
**Date**: 2026-09-08

## Goals

1. Extend `editor_model::runtime::{HotReloadRequest, PlayModeRequest}`
   to enums matching the current thread_local semantics (3 variants
   and 2 variants respectively).
2. Migrate `HOT_RELOAD_BUS` and `PLAY_MODE_REQUEST` thread_locals to
   `EditorSession.runtime.{hot_reload_requests, play_mode_request}`
   via the dual-write fallback pattern.
3. Update `process_hot_reload_requests` and the play-mode Bevy system
   to read from session (with `_FALLBACK.with(...)` fallback).
4. Pass all 4 existing `hot_reload.rs` tests without modification
   (they exercise the `_FALLBACK` path because no session is
   installed in those tests).

## Scenarios

### §S-G-1: Session-installed — `hot_reload_source_wasm` pushes to session

**Given** a `FakeSession` is registered with the default EditorSession
**When** `hot_reload_source_wasm("foo.rs")` is called
**Then** `session.runtime.hot_reload_requests` contains exactly one
`HotReloadRequest::Source { file_id: "foo.rs" }`
**And** `HOT_RELOAD_BUS_FALLBACK` is unchanged (production code
writes only to session)

### §S-G-2: Session-installed — `hot_reload_asset_wasm` pushes to session

**Given** a session is registered
**When** `hot_reload_asset_wasm("asset_42")` is called
**Then** `session.runtime.hot_reload_requests` contains exactly one
`HotReloadRequest::Asset { asset_id: "asset_42" }`

### §S-G-3: Session-installed — `force_reload_wasm` pushes to session

**Given** a session is registered
**When** `force_reload_wasm()` is called
**Then** `session.runtime.hot_reload_requests` contains exactly one
`HotReloadRequest::ForceReloadAll`

### §S-G-4: No session — fallback path (existing tests)

**Given** no session is registered
**When** `hot_reload_source_wasm("foo.rs")` is called
**Then** `HOT_RELOAD_BUS_FALLBACK` contains the request
**And** `hot_reload_bus_depth_for_tests()` returns 1
**And** `process_hot_reload_requests()` drains and processes the request
(verifies the legacy path still works — covers the existing
`hot_reload.rs` tests).

### §S-G-5: Session-installed — play-mode Enter

**Given** a session is registered
**When** `play_mode_request_set_for_tests(PlayModeRequest::Enter)` is
called (new test helper)
**Then** `session.runtime.play_mode_request == Some(PlayModeRequest::Enter)`
**And** the play-mode Bevy system reads from session, snapshots
transforms, sets `PlayMode::Playing`, and clears the request.

### §S-G-6: Session-installed — play-mode Exit

**Given** a session is registered with a pending `PlayModeRequest::Exit`
**When** the play-mode Bevy system runs
**Then** it computes runtime deltas, restores transforms, sets
`PlayMode::Editing`, and clears the request.

### §S-G-7: Two-session isolation

**Given** session A pushes `hot_reload_source_wasm("a.rs")`
**When** session B replaces A
**Then** session B's `hot_reload_requests` is empty (no inheritance)

### §S-G-8: archcheck-globals ratchet

**Given** the inventory is updated with `HOT_RELOAD_BUS_FALLBACK` and
`PLAY_MODE_REQUEST_FALLBACK` entries (OPEN)
**When** `npm run check` runs in `tools/archcheck-globals`
**Then** 31 entries = 31 declarations (29 from Block F + 2 new for Block G)

## Out-of-scope

- Bevy system signature changes (no `Res<EditorSession>` plumbing —
  dual-write fallback lets the system keep using `with(...)`).
- `RuntimeDelta` buffer migration (orthogonal).
- `KEYBOARD_STATE` (Block H).

## Migration type semantics

### `editor_model::runtime::HotReloadRequest` (after Block G)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HotReloadRequest {
    /// A source file (e.g. `.rs`) was saved; invalidate its cached content.
    Source { file_id: String },
    /// An asset file (e.g. `.bsn`) was saved or deleted; invalidate its body cache.
    Asset { asset_id: String },
    /// Full reload: clear source cache, asset body cache, and logic graph doc.
    ForceReloadAll,
}
```

### `editor_model::runtime::PlayModeRequest` (after Block G)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayModeRequest {
    Enter,
    Exit,
}
```

These match `editor_bevy::hot_reload_state::{HotReloadRequest,
PlayModeRequest}` 1:1. The migration de-duplicates the two
declarations: editor-bevy keeps them as re-exports
(`pub use editor_model::runtime::{HotReloadRequest, PlayModeRequest}`)
once Block G lands.

## Files to be modified

| File | Change |
|------|--------|
| `crates/editor-model/src/runtime/hot_reload.rs` | Replace struct with enum (HotReloadRequest); replace struct with enum (PlayModeRequest) |
| `crates/editor-model/src/runtime/mod.rs` | Re-export updated types (no change needed — already re-exports both) |
| `crates/editor-application/src/session.rs` | Update `use` statements (if struct literals are used); trait impl unchanged (signature already uses `editor_model::runtime::*`) |
| `crates/editor-bevy/src/hot_reload_state.rs` | Rename thread_locals to `*_FALLBACK`; add dual-write doc comments; re-export types from editor_model |
| `crates/editor-bevy/src/wasm_hot_reload.rs` | Update `HOT_RELOAD_BUS` → `HOT_RELOAD_BUS_FALLBACK`; add session-first dual-write |
| `crates/editor-bevy/src/preview_runtime.rs` | Update `HOT_RELOAD_BUS.with` / `PLAY_MODE_REQUEST.with` in `process_hot_reload_requests` and play-mode Bevy system; add session-first dual-write |
| `crates/editor-bevy/tests/hot_reload.rs` | Update `use` paths to point at renamed `*_FALLBACK` (already works because `pub use` re-export preserves names) |
| `crates/editor-bevy/tests/hot_reload_play_mode_parity.rs` (new) | 6 parity tests covering §S-G-1..§S-G-7 |
| `crates/editor-bevy/tests/support/mod.rs` | `FakeSession` already has `hot_reload_requests` and `play_mode_request` fields (verify) |
| `docs/architecture/state-ownership-matrix.md` | 2 rows: `HOT_RELOAD_BUS` + `PLAY_MODE_REQUEST` → RETIRED (Block G); progress note: 6 of 9 → 8 of 9 |
| `tools/archcheck-globals/globals-inventory.yaml` | 2 entries: `HOT_RELOAD_BUS` → `HOT_RELOAD_BUS_FALLBACK` (OPEN); `PLAY_MODE_REQUEST` → `PLAY_MODE_REQUEST_FALLBACK` (OPEN). Total: 31 entries. |
| `Cargo.toml` | 0.108.6 → 0.108.7 |

## Compatibility

- Public API of `editor_model::runtime::{HotReloadRequest,
  PlayModeRequest}` changes from struct → enum. Any external
  consumer of these types would need to update. (Risk: zero in-tree
  consumers; the struct shape was a stub.)
- `editor_bevy::hot_reload_state::{HotReloadRequest, PlayModeRequest}`
  continue to be available (now as `pub use` re-exports from
  `editor_model::runtime`).
- WASM exports `hot_reload_source_wasm`, `hot_reload_asset_wasm`,
  `force_reload_wasm`, `play_mode_*_wasm` unchanged.
- All 4 existing tests in `hot_reload.rs` continue to pass
  unmodified (they exercise the `_FALLBACK` path).
