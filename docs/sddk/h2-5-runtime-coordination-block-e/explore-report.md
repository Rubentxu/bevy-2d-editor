# H2.5 Block E — Exploration Report

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-e`
**Path**: A-min
**Date**: 2026-09-08

## Scope (target)

Migrate the **preview inspector** thread_locals to `EditorSession`:

- `PREVIEW_METRICS` (RefCell<PreviewMetrics>) →
  `EditorSession.preview_inspector.metrics: serde_json::Value`
- `PREVIEW_MAPPING` (RefCell<Vec<PreviewMappingEntry>>) →
  `EditorSession.preview_inspector.mapping: Vec<serde_json::Value>`
- `PREVIEW_PROVENANCE` (RefCell<BTreeMap<StableId, PreviewProvenance>>) →
  `EditorSession.preview_inspector.provenance: BTreeMap<String, serde_json::Value>`

## Inventory of current state

### `crates/editor-bevy/src/preview_inspector.rs` (247 lines)

| Lines   | What                                                                                                                          |
|---------|-------------------------------------------------------------------------------------------------------------------------------|
| 21-25   | `pub struct PreviewMetrics { fps, frame_time_ms, rebuild_count }`                                                            |
| 33-39   | `pub struct PreviewMappingEntry { stable_id, local_id, asset_ref, component_count }`                                          |
| 43-51   | `pub struct PreviewProvenance { stable_id, local_id, asset_ref, components, is_from_instance, causality_edges }`              |
| 57-81   | `thread_local! { PREVIEW_METRICS, PREVIEW_MAPPING, PREVIEW_PROVENANCE }` (the three targets)                                  |
| 84-105  | Setters: `set_metrics`, `set_mapping`, `set_provenance`                                                                       |
| 89-95   | `increment_rebuild_count()` (reads+mutates PREVIEW_METRICS)                                                                   |
| 108-122 | Getters: `get_metrics`, `get_mapping`, `get_provenance(stable_id)`                                                            |
| 133-198 | Already-migrated session code (`record_rebuild_cause`, `last_rebuild_cause`, `stamp_provenance`, `apply_pending_causality_edges`) — v0.90 PR2 |
| 200-246 | 3 unit tests (`metrics_round_trip`, `mapping_round_trip`, `mapping_contains_no_bevy_entity_id_field`)                         |

### `crates/editor-bevy/src/wasm_preview.rs` (37 lines)

Three `#[wasm_bindgen]` exports that read from the getters and serialize to JSON for JS:

- `get_preview_metrics_wasm` → calls `crate::preview_inspector::get_metrics()` then `serde_json::to_string(&m)`
- `get_preview_mapping_wasm` → calls `get_mapping()` then `serde_json::to_string(&m)`
- `get_preview_provenance_wasm(stable_id)` → calls `get_provenance(stable_id)` then serializes

### `crates/editor-bevy/src/preview_runtime.rs` (callers)

- Line 717-769: `push_preview_inspector_state` — calls `set_mapping`, `set_provenance`, `apply_pending_causality_edges`, `increment_rebuild_count`.
- Line 1142-1145 + 1175-1178: `emit_events` — calls `set_metrics` twice (once in `using_session` branch, once in `EVENT_BUS.with` fallback).
- Line 1076: `record_rebuild_cause` (already session-owned — no change needed).

### Target field (already exists)

`crates/editor-model/src/session.rs:203`:

```rust
pub struct PreviewInspectorState {
    pub metrics: serde_json::Value,
    pub mapping: Vec<serde_json::Value>,
    pub provenance: std::collections::BTreeMap<String, serde_json::Value>,
    pub last_rebuild_cause: Option<crate::rebuild_cause::RebuildCause>,
}
```

`crates/editor-model/src/session_port.rs:66`:

```rust
fn preview_inspector_mut(&mut self) -> &mut crate::session::PreviewInspectorState;
```

`crates/editor-model/src/ports.rs:182`:

```rust
pub fn with_session_mut<R, F: FnOnce(&mut dyn EditorSessionPort) -> R>(f: F) -> Option<R>;
```

## Architectural pattern (precedent)

**v0.90 PR2** already migrated `LAST_REBUILD_CAUSE` and
`PENDING_CAUSALITY_EDGES` to `EditorSession` (ADR-0052). Pattern
visible at `preview_inspector.rs:124-198`.

**Block A2** (v0.108.3) migrated `ACTUATOR_OUTPUT_BUS` using a
**dual-write fallback** that tries session first, then falls back to
thread_local when the session is not initialized (the
`is_some()`/`using_session` boolean pattern in `emit_events`).

Both precedents apply to Block E: the target field exists, the API is
established, and the dual-write fallback pattern is proven safe in
tests.

## Type-conversion implications

`PreviewInspectorState` stores `serde_json::Value`, not typed
`PreviewMetrics`/`PreviewMappingEntry`/`PreviewProvenance`. Every set
operation must:

```rust
serde_json::to_value(&typed_payload).expect("...")
```

Every get operation must:

```rust
serde_json::from_value(&stored_value).expect("...")
```

The `expect` calls are safe because we serialize then immediately
deserialize on the same payload (no schema drift) and the typed
structs derive `Serialize + Deserialize + PartialEq`. Tests at
`preview_inspector.rs:213-246` already exercise this serialization
path and pass.

## Dual-write or single-write?

Two options:

**A. Single-write (session-only, like v0.90 PR2)**: Remove the
thread_locals entirely; rely on the session. Tests that don't
initialize a session must install one (as Block A2 did with
`MinimalSession`).

**B. Dual-write (session-first with thread_local fallback, like Block
A2 ActuatorBus)**: Try session first; on `None` (no session),
fallback to thread_local. Tests can still run without explicit
session installation, at the cost of keeping the thread_locals
alive.

**Recommended**: **B (dual-write)** for the migration PR, matching
the Block A2 ActuatorBus pattern. A follow-up PR can collapse to
single-write once all test surfaces install a session.

## Out-of-scope (explicit)

- Migrating `EVENT_BUS` (lib.rs:408) — that is Block F.
- Migrating `COMMAND_BUS` (lib.rs:407) — that is Block F.
- Migrating `HOT_RELOAD_BUS` (hot_reload_state.rs:32) — that is Block G.
- Migrating `PLAY_MODE_REQUEST` (hot_reload_state.rs:35) — that is Block G.
- Migrating `KEYBOARD_STATE` (logic_evaluator.rs:1049) — that is Block H.
- `LAST_REBUILD_CAUSE` / `PENDING_CAUSALITY_EDGES` — already migrated in v0.90 PR2.

## Risk assessment

| Risk                                    | Severity | Mitigation                                                  |
|-----------------------------------------|----------|-------------------------------------------------------------|
| Test failures from session-not-installed | Medium   | Dual-write fallback keeps tests green without session       |
| Serialization perf                      | Low      | Operations are O(n) where n = mapping/provenance size; small |
| StableId ↔ String key drift             | Low      | `EditorSession.provenance` keys are `String`; conversion via `.0` is safe (inner String is identical) — precedent at preview_inspector.rs:175 |
| Bevy system ordering                    | None     | preview_inspector.rs has no Bevy systems; only functions    |
| Breaking the JS contract                | Low      | wasm_preview.rs is unchanged in semantics; serialization unchanged |

## Verdict

**Path: A-min is appropriate.** The exploration is bounded:

- Target field exists.
- Precedent (v0.90 PR2 + Block A2) is established.
- 1 file changes type signatures (preview_inspector.rs);
  1 file unchanged in API (wasm_preview.rs, since it goes through
  the same getters);
  1 file updates callers to a fallback pattern (preview_runtime.rs);
  new tests added.
- No new public API, no architectural decisions open.

Ready for `phase.explore.complete` → `phase.spec.next` → `phase.tasks.next` → `phase.apply.complete`.
