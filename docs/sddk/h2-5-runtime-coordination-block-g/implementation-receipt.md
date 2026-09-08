# H2.5 Block G — Implementation Receipt

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-g`
**Path**: A-lite (explore → spec → build → verify → release → archive)
**Tag**: `v0.108.7` (to be released)
**Commit**: TBD (pending release)
**Date**: 2026-09-08

## Scope

Migrate `HOT_RELOAD_BUS` and `PLAY_MODE_REQUEST` thread_locals to
`EditorSession.runtime.{hot_reload_requests, play_mode_request}` via
the dual-write fallback pattern. Extends
`editor_model::runtime::{HotReloadRequest, PlayModeRequest}` from
struct stubs to full enums matching the legacy thread_local
semantics (3-variant and 2-variant).

## Work units landed

| WU     | Files                                                                 | Lines |
|--------|-----------------------------------------------------------------------|-------|
| WU-G-1 | `crates/editor-model/src/runtime/hot_reload.rs` (struct → enum)       | +28 / -7 |
| WU-G-2 | `crates/editor-bevy/src/hot_reload_state.rs` (re-exports + rename)    | +35 / -28 |
| WU-G-3 | `crates/editor-bevy/src/wasm_hot_reload.rs` (dual-write)              | +25 / -15 |
| WU-G-4 | `crates/editor-bevy/src/preview_runtime.rs` (dual-write Bevy systems) | +45 / -10 |
| WU-G-5 | `crates/editor-bevy/src/lib.rs` (import update)                       | +1 / -1 |
| WU-G-6 | `crates/editor-bevy/src/state.rs` (re-export update)                  | +1 / -1 |
| WU-G-7 | `crates/editor-bevy/tests/hot_reload_play_mode_parity.rs` (new, 6 tests) | +131 / -0 |
| WU-G-8 | `docs/architecture/state-ownership-matrix.md` (2 retired + progress)  | +5 / -5 |
| WU-G-9 | `tools/archcheck-globals/globals-inventory.yaml` (2 retired → 2 FALLBACK OPEN) | +8 / -8 |
| WU-G-10 | `Cargo.toml` (0.108.6 → 0.108.7)                                     | +1 / -1 |

## Code changes

### WU-G-1: editor_model enum extension

```rust
// crates/editor-model/src/runtime/hot_reload.rs
pub enum HotReloadRequest {
    Source { file_id: String },
    Asset { asset_id: String },
    ForceReloadAll,
}

pub enum PlayModeRequest {
    Enter,
    Exit,
}
```

### WU-G-2: re-export + rename

```rust
// crates/editor-bevy/src/hot_reload_state.rs
pub use editor_model::runtime::{HotReloadRequest, PlayModeRequest};

thread_local! {
    pub static HOT_RELOAD_BUS_FALLBACK: RefCell<Vec<HotReloadRequest>> = ...;
    pub static PLAY_MODE_REQUEST_FALLBACK: RefCell<Option<PlayModeRequest>> = ...;
}
```

### WU-G-3/4: dual-write

Production code (was hot_reload_state.rs, preview_runtime.rs):
```rust
// Push: session first, FALLBACK otherwise
let via_session = with_session_mut(|s| {
    s.runtime_hot_reload_requests_mut().push(req.clone());
    true
});
if via_session.unwrap_or(false) { return; }
HOT_RELOAD_BUS_FALLBACK.with(|bus| bus.borrow_mut().push(req));
```

Read (Bevy system):
```rust
let requests: Vec<HotReloadRequest> = with_session_mut(|s| {
    std::mem::take(s.runtime_hot_reload_requests_mut())
}).unwrap_or_else(|| HOT_RELOAD_BUS_FALLBACK.with(|bus| std::mem::take(&mut *bus.borrow_mut())));
```

## Verification status

- `cargo check --workspace --locked` ✅
- `cargo check -p editor-model --target wasm32-unknown-unknown --locked` ✅
- `cargo test -p editor-bevy --test hot_reload --locked` ✅ (4/4 — existing)
- `cargo test -p editor-bevy --test hot_reload_play_mode_parity --locked` ✅ (6/6 — new)
- `archcheck-globals` ✅ (29 entries = 29 declarations)

## H2.5 progress after Block G

| Cell                        | Status                |
|-----------------------------|-----------------------|
| `ACTUATOR_OUTPUT_BUS`       | RETIRED (Block A2)    |
| `PREVIEW_METRICS`           | RETIRED (Block E)     |
| `PREVIEW_MAPPING`           | RETIRED (Block E)     |
| `PREVIEW_PROVENANCE`        | RETIRED (Block E)     |
| `COMMAND_BUS`               | RETIRED (Block F)     |
| `EVENT_BUS`                 | RETIRED (Block F)     |
| `HOT_RELOAD_BUS`            | RETIRED (Block G)     |
| `PLAY_MODE_REQUEST`         | RETIRED (Block G)     |
| `KEYBOARD_STATE`            | OPEN (Block H planned) |

**8 of 9 H2.5 cells retired.** One cell remains.

## Forward work after G

- **Block H**: `KEYBOARD_STATE` → Bevy `Resource InputState`
  (`crates/editor-bevy/src/logic_evaluator.rs:1049`). Different
  migration target (Bevy Resource, not EditorSession field) — likely
  A-lite or A-full because the migration target is architectural.
