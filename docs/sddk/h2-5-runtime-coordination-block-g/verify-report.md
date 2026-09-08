# H2.5 Block G — Verify Report

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-g`
**Path**: A-lite (explore → spec → build → verify → release → archive)
**Tag**: `v0.108.7` (to be released)
**Date**: 2026-09-08

## Verification scope

Block G is the fourth thread_local migration under the dual-write
fallback pattern (Block A2 ActuatorBus, Block E preview inspector,
Block F runtime buses, Block G hot-reload + play-mode). It also
extends `editor_model::runtime` types from struct stubs to full
enums. Verification covers four axes:

1. **Parity (new)**: session-backed behaviour for hot-reload +
   play-mode (6 tests).
2. **Regression**: existing `hot_reload.rs` tests (4 tests) continue
   to pass without modification.
3. **Build hygiene**: `cargo check --workspace --locked` + WASM
   target.
4. **H0.3 ratchet**: `archcheck-globals` 29=29.

## Test results

### 1. Parity suite (new — 6 tests)

```
$ cargo test -p editor-bevy --test hot_reload_play_mode_parity --locked
running 6 tests
test parity_force_reload_via_session ... ok
test parity_hot_reload_asset_via_session ... ok
test parity_hot_reload_source_via_session ... ok
test parity_play_mode_enter_via_session ... ok
test parity_play_mode_exit_via_session ... ok
test parity_two_session_isolation_hot_reload ... ok

test result: ok. 6 passed; 0 failed
```

### 2. Existing hot_reload tests (4 tests, unchanged)

```
$ cargo test -p editor-bevy --test hot_reload --locked
running 4 tests
test hot_reload_source_wasm_pushes_source_request ... ok
test process_drains_bus_and_invalidates_cache ... ok
test force_reload_emits_force_variant ... ok
test asset_request_invalidates_body_cache ... ok

test result: ok. 4 passed; 0 failed
```

### 3. Build hygiene

```
$ cargo check --workspace --locked
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.50s

$ cargo check -p editor-model --target wasm32-unknown-unknown --locked
Finished `dev` profile in 0.21s
```

### 4. H0.3 ratchet (archcheck-globals)

```
$ cd tools/archcheck-globals && npm run check
archcheck-globals: 29 declarations, all match inventory (29 entries)
```

The rename `HOT_RELOAD_BUS` → `HOT_RELOAD_BUS_FALLBACK` and
`PLAY_MODE_REQUEST` → `PLAY_MODE_REQUEST_FALLBACK` (plus removal of
the old entries from the inventory) keeps the ratchet at 29 entries
(same total).

## Code diff summary

```
crates/editor-model/src/runtime/hot_reload.rs              | +28 / -7   (struct → enum)
crates/editor-bevy/src/hot_reload_state.rs                 | +35 / -28  (re-exports + rename)
crates/editor-bevy/src/wasm_hot_reload.rs                  | +25 / -15  (dual-write)
crates/editor-bevy/src/preview_runtime.rs                  | +45 / -10  (dual-write Bevy systems)
crates/editor-bevy/src/lib.rs                              | +1 / -1    (import update)
crates/editor-bevy/src/state.rs                            | +1 / -1    (re-export update)
crates/editor-bevy/tests/hot_reload_play_mode_parity.rs   | +131 / -0  (new, 6 tests)
docs/architecture/state-ownership-matrix.md                | +8 / -8    (2 retired + progress)
tools/archcheck-globals/globals-inventory.yaml             | +8 / -8    (2 retired → 2 FALLBACK)
Cargo.toml                                                 | +1 / -1    (0.108.6 → 0.108.7)
```

Net: **+283 / -79** lines (excluding block-G artifacts).

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

## Architectural change in Block G

Unlike Blocks A2/D/E/F (which were pure renames with dual-write),
Block G extended `editor_model::runtime::{HotReloadRequest,
PlayModeRequest}` from struct stubs to full enums. The struct
stubs were placeholders — the legacy thread_local types had 3 and 2
variants respectively. Migrating the call sites to the session
without extending the types would have lost semantics
(`Source{file_id}` vs `Asset{asset_id}` are routed differently in
`process_hot_reload_requests`).

The migration collapses the duplicated declarations:
`editor_bevy::hot_reload_state::{HotReloadRequest, PlayModeRequest}`
are now `pub use` re-exports of `editor_model::runtime::*`.

## Out-of-scope (not verified)

- Full UAT cohort (`playwright.*.config.ts`) — Block G is internal to
  the Bevy preview subsystem; no UI changes.
- Bevy system ordering changes — dual-write lets `process_hot_reload_requests`
  and the play-mode system keep their signatures.
- `KEYBOARD_STATE` (Block H).

## Compatibility

- `editor_model::runtime::HotReloadRequest` shape changed from
  struct → enum. In-tree consumers: zero (the struct shape was a
  stub with no callers outside `editor_application::session.rs`,
  which uses `Vec<HotReloadRequest>` as an opaque type — only the
  push/drain methods are used, which still work).
- `editor_model::runtime::PlayModeRequest` shape changed from
  struct → enum. Same compatibility profile as above.
- `editor_bevy::hot_reload_state::{HotReloadRequest, PlayModeRequest}`
  continue to be available as `pub use` re-exports.
- WASM exports unchanged.
- All 4 existing `hot_reload.rs` tests pass unmodified.

## Conclusion

Block G is **verified**. The dual-write fallback pattern preserves
backward compatibility while migrating the canonical owner to the
session. The 9 H2.5 cells are now 8 of 9 retired, with only
`KEYBOARD_STATE` remaining (Block H — Bevy `Resource InputState`,
different migration target).

Ready for `phase.verify.complete.a-min` → release → archive.
