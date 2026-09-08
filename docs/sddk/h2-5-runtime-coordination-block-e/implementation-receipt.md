# H2.5 Block E — Implementation Receipt

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-e`
**Path**: A-min
**Tag**: `v0.108.5`
**Commit**: `6c7c0fc` (`main`)
**Date**: 2026-09-08

## Scope

Migrate the preview-inspector thread_locals to `EditorSession` via the
established dual-write fallback pattern (Block A2 ActuatorBus, v0.90
PR2 `record_rebuild_cause`). Three thread_locals retired; production
path goes through the session, fallback thread_locals renamed
(`*_FALLBACK`) for legacy tests.

## Work units landed

| WU      | Files                                                            | Lines |
|---------|------------------------------------------------------------------|-------|
| WU-E-1  | `crates/editor-bevy/src/preview_inspector.rs` (rename + doc)     | +12 / -3  |
| WU-E-2  | `crates/editor-bevy/src/preview_inspector.rs` (set_metrics + increment) | +28 / -8 |
| WU-E-3  | `crates/editor-bevy/src/preview_inspector.rs` (set_mapping + set_provenance) | +30 / -6 |
| WU-E-4  | `crates/editor-bevy/src/preview_inspector.rs` (get_metrics + get_mapping + get_provenance) | +34 / -6 |
| WU-E-5  | `crates/editor-bevy/src/preview_inspector.rs` (apply_pending_causality_edges + helper) | +24 / -8 |
| WU-E-7  | `crates/editor-bevy/tests/preview_inspector_parity.rs` (new, 6 tests) | +163 / -0 |
| WU-E-9  | `docs/architecture/state-ownership-matrix.md` § H2.5              | +18 / -4 |
| WU-E-10 | `tools/archcheck-globals/globals-inventory.yaml` (retire 3 + add 3 fallback) | +21 / -15 |
| WU-E-11 | `Cargo.toml` (0.108.4 → 0.108.5)                                 | +1 / -1 |

Total: **+331 / -51** across 6 source files (+ the new test file).

## Migration pattern (dual-write fallback)

For every set:

```rust
let written = editor_model::ports::with_session_mut(|sess| {
    sess.preview_inspector_mut().metrics = payload;
}).is_some();
if !written {
    PREVIEW_METRICS_FALLBACK.with(|m| *m.borrow_mut() = metrics);
}
```

For every get:

```rust
if let Some(v) = editor_model::ports::with_session_mut(|sess| {
    sess.preview_inspector_mut().metrics.clone()
}) {
    return serde_json::from_value(v).unwrap_or_default();
}
PREVIEW_METRICS_FALLBACK.with(|m| m.borrow().clone())
```

This pattern (Block A2 ActuatorBus precedent) keeps production code
session-bound while allowing legacy tests that don't install a session
to continue working unchanged.

## Type conversion

`PreviewInspectorState` stores `serde_json::Value` (not the typed
structs from `editor-bevy`). Every set serializes with
`serde_json::to_value`; every get deserializes with
`serde_json::from_value` (or `unwrap_or_default()` on the metrics
read). The typed structs derive `Serialize + Deserialize + PartialEq`,
and the existing `metrics_round_trip` and `mapping_round_trip` tests
exercise the round-trip path.

## `apply_pending_causality_edges` change

Before: drained `EditorSession::pending_causality_edges_mut` and
wrote directly into the legacy `PREVIEW_PROVENANCE.with(...)`.

After: drains the pending map, reads current provenance via the
new `get_provenance_map_for_internal_write` helper (session-first),
extends edges, then writes back via `set_provenance` (which respects
the dual-write contract).

## Validation evidence

### Parity tests (new)

```
$ cargo test -p editor-bevy --test preview_inspector_parity --locked
running 6 tests
test parity_fallback_path_without_session ... ok
test parity_metrics_round_trip_via_session ... ok
test parity_increment_rebuild_count_via_session ... ok
test parity_mapping_round_trip_via_session ... ok
test parity_two_session_isolation ... ok
test parity_provenance_round_trip_via_session ... ok

test result: ok. 6 passed; 0 failed
```

### Lib tests (existing, kept green)

```
$ cargo test -p editor-bevy --lib preview_inspector --locked
running 3 tests
test preview_inspector::tests::metrics_round_trip ... ok
test preview_inspector::tests::mapping_round_trip ... ok
test preview_inspector::tests::mapping_contains_no_bevy_entity_id_field ... ok

test result: ok. 3 passed; 0 failed
```

### Workspace + WASM

```
$ cargo check --workspace --locked
Finished `dev` profile in 22.03s

$ cargo check -p editor-model --target wasm32-unknown-unknown --locked
Finished `dev` profile in 0.30s
```

### Inventory sync

```
$ cd tools/archcheck-globals && npm run check
archcheck-globals: 29 declarations, all match inventory (29 entries)

$ npm test
archcheck-globals tests: all pass
```

29 declarations (was 26 in v0.108.4 because 3 new `*_FALLBACK`
thread_locals appeared; 3 original PREVIEW_* entries retired; net 0).

## Closes

- **H2.5 Block E** (preview inspector migration): 3 of 9 cells retired.
- Combined with Block A2 + Block D: **4 of 9 H2.5 cells retired**
  (ACTUATOR_OUTPUT_BUS, PREVIEW_METRICS, PREVIEW_MAPPING,
  PREVIEW_PROVENANCE).

## Does NOT close (forward work)

The remaining 5 H2.5 cells (COMMAND_BUS, EVENT_BUS, HOT_RELOAD_BUS,
PLAY_MODE_REQUEST, KEYBOARD_STATE) are scheduled for:

- **Block F**: COMMAND_BUS + EVENT_BUS (runtime buses, both in
  `crates/editor-bevy/src/lib.rs:407-408`).
- **Block G**: HOT_RELOAD_BUS + PLAY_MODE_REQUEST (both in
  `crates/editor-bevy/src/hot_reload_state.rs:32,35`).
- **Block H**: KEYBOARD_STATE → Bevy `Resource InputState`
  (`crates/editor-bevy/src/logic_evaluator.rs:1049`).

The dual-write fallback pattern from Block A2 + Block E is the
template for each subsequent block.

## Compatibility

- Public API unchanged (same function signatures).
- Production code paths unchanged semantically.
- Tests that previously called `set_*`/`get_*` against the legacy
  thread_locals now read/write through the session when installed
  (the typical case in `cargo test --workspace`); tests that
  intentionally bypass the session continue to work via the renamed
  `*_FALLBACK` thread_locals.
- WASM exports (`get_preview_metrics_wasm`, `get_preview_mapping_wasm`,
  `get_preview_provenance_wasm`) unchanged: they call the same public
  getters, which now read from session-or-fallback transparently.

## Rollback

Revert commit `6c7c0fc` (single commit). The change touches only
`preview_inspector.rs` (internal), `wasm_preview.rs` (unchanged),
`preview_runtime.rs` (unchanged), the new parity test file, the
inventory YAML, the matrix, and the version bump. The session
fields (`EditorSession.preview_inspector.{metrics,mapping,provenance}`)
already existed pre-Block E, so no schema migration is needed in
either direction.
