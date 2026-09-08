# H2.5 Block G — Explore Report

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-g`
**Path**: A-min → **escalating to A-lite** (architectural decision required)
**Date**: 2026-09-08

## Scope

Migrate the legacy `HOT_RELOAD_BUS` and `PLAY_MODE_REQUEST`
thread_locals to `EditorSession.runtime.hot_reload_requests` /
`EditorSession.runtime.play_mode_request` via the dual-write fallback
pattern (Blocks A2/D/E/F).

## Path escalation: A-min → A-lite

**Reason**: type-shape mismatch between current thread_local types
and the session-owned types already declared in `editor_model::runtime`.

### The mismatch

| Aspect | editor-bevy::hot_reload_state (current thread_local types) | editor_model::runtime (session target) |
|--------|-----------------------------------------------------------|---------------------------------------|
| `HotReloadRequest` | `enum { Source { file_id }, Asset { asset_id }, ForceReloadAll }` | `struct { asset_id: String }` (single struct) |
| `PlayModeRequest` | `enum { Enter, Exit }` | `struct { play: bool }` (single struct) |

The session-side already has `hot_reload_requests: Vec<HotReloadRequest>`
and `play_mode_request: Option<PlayModeRequest>` fields in
`editor_application::session::RuntimeSessionState`
(`crates/editor-application/src/session.rs:385,387`) and the trait
accessors `runtime_hot_reload_requests_mut` /
`runtime_play_mode_request_mut` in
`crates/editor-model/src/session_port.rs:135,138`.

**The session-side drains are defined but unused** —
`drain_hot_reload_requests` / `drain_play_mode_request` at lines 461,466
have no callers. The `process_hot_reload_requests` Bevy system still
reads the thread_local directly.

### Why this is more than a rename

Block G is structurally different from Blocks A2/D/E/F because:

1. **The thread_local types are richer** (3 variants) than the
   session types (1 field each). To make the session path useful, the
   session types need to be **extended to match** the runtime
   semantics — that is an architectural change touching
   `editor-model`, `editor-application`, and `editor-bevy`.
2. **The Bevy system `process_hot_reload_requests` (line 607)**
   reads the thread_local directly (not via session port), so the
   migration must change the system to either:
   - Read via session (requires Bevy system to receive session somehow)
   - Use the `*_FALLBACK` thread_local (which is what dual-write
     pattern is for)
3. **The play-mode Bevy system** at `preview_runtime.rs:343,367` reads
   `PLAY_MODE_REQUEST.with(...)` directly, same architectural problem.
4. **4 existing tests** (`crates/editor-bevy/tests/hot_reload.rs:145`)
   exercise the thread_local types via `hot_reload_bus_depth_for_tests`
   and `process_hot_reload_requests`.

### Three options analyzed

**Option A: Extend `editor_model::runtime::{HotReloadRequest, PlayModeRequest}` to enum**

- Pros: matches existing thread_local semantics 1:1; minimal call-site
  changes in `wasm_hot_reload.rs` and `preview_runtime.rs`.
- Cons: bumps `editor_model::runtime` shape (binary-breaking for any
  out-of-tree consumer); must update `RuntimeSessionState::new()` and
  the trait `runtime_*_mut` signatures accordingly.
- Touch: `editor-model/src/runtime/hot_reload.rs`,
  `editor-application/src/session.rs`,
  `editor-bevy/src/wasm_hot_reload.rs`,
  `editor-bevy/src/preview_runtime.rs`,
  `editor-bevy/src/hot_reload_state.rs` (now a thin adapter).

**Option B: Keep the enum in editor-bevy; convert at session boundary**

- Pros: `editor_model` shape unchanged.
- Cons: every push/drain site needs a conversion. Awkward
  because `Vec<HotReloadRequest>` (editor-bevy) cannot equal
  `Vec<HotReloadRequest>` (editor-model). The two types would
  collide on name.
- Touch: same files but with adapter functions per call site.
- NOT recommended (cognitive load + name collision).

**Option C: Use editor_model as-is, refactor call sites to use 1-field shape**

- Pros: cleanest from architectural standpoint (single source of truth).
- Cons: Source/Asset/ForceReloadAll semantics must collapse into
  `asset_id: String`. `ForceReloadAll` becomes a special string
  sentinel (e.g. `"*"` or `None`). Call sites must change.
- Touch: `editor-bevy/src/wasm_hot_reload.rs` (3 functions),
  `editor-bevy/src/preview_runtime.rs` (`process_hot_reload_requests`,
  play-mode match).

### Decision: Option A (extend editor_model)

Rationale:
1. The runtime semantics of `Source{file_id}` vs `Asset{asset_id}`
   are genuinely different (file path vs asset path), and `ForceReloadAll`
   is a distinct trigger. Collapsing them into a single field loses
   information that downstream code depends on (`source_files::invalidate_cache`
   vs `with_asset_body_cache_mut`).
2. Extending `editor_model::runtime::HotReloadRequest` is a
   non-breaking change for current consumers (the struct is a stub
   with `asset_id: String`; promoting to enum keeps `asset_id` as one
   variant).
3. The existing `editor_application::session.rs` `RuntimeSessionState`
   is a recent Block A addition with 0 callers of `drain_*` — easy
   to update.

## Block G work plan (A-lite, 5 WUs)

| WU     | Description                                                              | Files |
|--------|--------------------------------------------------------------------------|-------|
| WU-G-1 | Extend `editor_model::runtime::HotReloadRequest` and `PlayModeRequest` to enums matching the editor-bevy semantics | `crates/editor-model/src/runtime/hot_reload.rs` |
| WU-G-2 | Rename `HOT_RELOAD_BUS` → `HOT_RELOAD_BUS_FALLBACK`, `PLAY_MODE_REQUEST` → `PLAY_MODE_REQUEST_FALLBACK` with dual-write docs | `crates/editor-bevy/src/hot_reload_state.rs` |
| WU-G-3 | Update `wasm_hot_reload.rs` to push via session (with `_FALLBACK.with(...)` fallback) | `crates/editor-bevy/src/wasm_hot_reload.rs` |
| WU-G-4 | Update `preview_runtime.rs` `process_hot_reload_requests` and play-mode match to read via session (with `_FALLBACK.with(...)` fallback) | `crates/editor-bevy/src/preview_runtime.rs` |
| WU-G-5 | Parity tests (4-6 tests) covering session path for hot-reload + play-mode | `crates/editor-bevy/tests/hot_reload_play_mode_parity.rs` (new) |
| WU-G-6 | Matrix + inventory updates (2 cells RETIRED, 2 FALLBACK OPEN) | `docs/architecture/state-ownership-matrix.md`, `tools/archcheck-globals/globals-inventory.yaml` |
| WU-G-7 | Version bump 0.108.6 → 0.108.7                                          | `Cargo.toml` |

## Existing test surface

- `crates/editor-bevy/tests/hot_reload.rs` (145 lines, 4 tests):
  - `hot_reload_source_wasm_pushes_source_request` — uses thread_local
  - `process_drains_bus_and_invalidates_cache` — uses thread_local
  - `asset_request_invalidates_body_cache` — uses thread_local
  - `force_reload_emits_force_variant` — uses thread_local

These tests will continue to pass with the `*_FALLBACK` rename +
dual-write because the test code paths don't install a session, so
the `_FALLBACK` thread_local path will be exercised (same as Block E
`fallback_path_without_session` parity test pattern).

## Files NOT in scope (forward work)

- `KEYBOARD_STATE` → Bevy `Resource InputState` — Block H.
- `RuntimeDelta` buffer migration — orthogonal to H2.5.

## Risks

1. **Type extension in editor-model** is observable by
   `editor-application::session::RuntimeSessionState`. Any external
   consumer (none in this repo) would need to update. Risk: zero
   in-tree, low.
2. **`process_hot_reload_requests` is a Bevy system** — migrating
   its source of truth from thread_local to session requires either
   `Res<EditorSession>` (Bevy resource wrapper) or a function-level
   call that pulls from session. Dual-write keeps the thread_local
   as fallback so the Bevy system keeps working without a session
   install.
3. **Hot-reload timing**: `process_hot_reload_requests` runs every
   frame in `Update`. If session is not installed (tests), the
   `_FALLBACK` thread_local is the source of truth — same as Block E
   fallback semantics.

## Lessons applied

- Dual-write fallback pattern (Blocks A2/D/E/F) — same shape:
  production code goes through session, `_FALLBACK` thread_local
  carries the legacy path.
- archcheck-globals ratchet: new entries (`HOT_RELOAD_BUS_FALLBACK`,
  `PLAY_MODE_REQUEST_FALLBACK`) must be added to
  `tools/archcheck-globals/globals-inventory.yaml` as OPEN.
- Matrix progress note: 6 of 9 → 8 of 9 retired after Block G.
