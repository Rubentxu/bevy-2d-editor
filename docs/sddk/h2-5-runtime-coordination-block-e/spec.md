# H2.5 Block E — Specification

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-e`
**Path**: A-min
**Phase**: Specify
**Date**: 2026-09-08
**Based on**: `explore-report.md`

## Goal

Migrate the three preview-inspector thread_locals (`PREVIEW_METRICS`,
`PREVIEW_MAPPING`, `PREVIEW_PROVENANCE`) from `editor-bevy` into
`EditorSession.preview_inspector`, following the established dual-write
fallback pattern (Block A2 ActuatorBus, v0.90 PR2 record_rebuild_cause).

## Functional requirements

### FR-E1 — `set_metrics` writes to session, falls back to thread_local

```rust
pub fn set_metrics(metrics: PreviewMetrics) {
    let payload = serde_json::to_value(&metrics).expect("PreviewMetrics serializes");
    let written = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().metrics = payload.clone();
    }).is_some();
    if !written {
        PREVIEW_METRICS_FALLBACK.with(|m| *m.borrow_mut() = metrics);
    }
}
```

### FR-E2 — `increment_rebuild_count` reads+mutates session, falls back

```rust
pub fn increment_rebuild_count() -> u32 {
    let updated = editor_model::ports::with_session_mut(|sess| {
        let mut m: PreviewMetrics = serde_json::from_value(
            &sess.preview_inspector_mut().metrics
        ).unwrap_or_default();
        m.rebuild_count = m.rebuild_count.saturating_add(1);
        sess.preview_inspector_mut().metrics = serde_json::to_value(&m).expect("...");
        m.rebuild_count
    });
    match updated {
        Some(n) => n,
        None => PREVIEW_METRICS_FALLBACK.with(|m| {
            let mut m = m.borrow_mut();
            m.rebuild_count = m.rebuild_count.saturating_add(1);
            m.rebuild_count
        }),
    }
}
```

### FR-E3 — `set_mapping` writes JSON array to session, falls back

```rust
pub fn set_mapping(entries: Vec<PreviewMappingEntry>) {
    let payload: Vec<serde_json::Value> = entries.iter()
        .map(|e| serde_json::to_value(e).expect("..."))
        .collect();
    let written = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().mapping = payload;
    }).is_some();
    if !written {
        PREVIEW_MAPPING_FALLBACK.with(|m| *m.borrow_mut() = entries);
    }
}
```

### FR-E4 — `set_provenance` writes JSON map to session, falls back

```rust
pub fn set_provenance(entries: BTreeMap<StableId, PreviewProvenance>) {
    let payload: BTreeMap<String, serde_json::Value> = entries.iter()
        .map(|(k, v)| (k.0.clone(), serde_json::to_value(v).expect("...")))
        .collect();
    let written = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().provenance = payload;
    }).is_some();
    if !written {
        PREVIEW_PROVENANCE_FALLBACK.with(|p| *p.borrow_mut() = entries);
    }
}
```

### FR-E5 — `get_metrics`, `get_mapping`, `get_provenance` read session first, fallback

```rust
pub fn get_metrics() -> PreviewMetrics {
    if let Some(v) = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().metrics.clone()
    }) {
        return serde_json::from_value(v).unwrap_or_default();
    }
    PREVIEW_METRICS_FALLBACK.with(|m| m.borrow().clone())
}

pub fn get_mapping() -> Vec<PreviewMappingEntry> {
    if let Some(arr) = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().mapping.clone()
    }) {
        return arr.into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
    }
    PREVIEW_MAPPING_FALLBACK.with(|m| m.borrow().clone())
}

pub fn get_provenance(stable_id: &str) -> Option<PreviewProvenance> {
    let sid_str = stable_id.to_string();
    if let Some(map) = editor_model::ports::with_session_mut(|sess| {
        sess.preview_inspector_mut().provenance.clone()
    }) {
        return map.get(&sid_str).and_then(|v| serde_json::from_value(v.clone()).ok());
    }
    PREVIEW_PROVENANCE_FALLBACK.with(|p| {
        p.borrow().get(&StableId::new(stable_id)).cloned()
    })
}
```

### FR-E6 — `apply_pending_causality_edges` (already session-based) keeps writing to PREVIEW_PROVENANCE_FALLBACK

Currently this function reads from session (`pending_causality_edges_mut`)
and writes to `PREVIEW_PROVENANCE.with(...)`. After Block E it must
write via `set_provenance` so the fallback path is preserved.

### FR-E7 — Fallback thread_locals are renamed to make their role explicit

```rust
thread_local! {
    static PREVIEW_METRICS_FALLBACK: RefCell<PreviewMetrics> = ...;
    static PREVIEW_MAPPING_FALLBACK: RefCell<Vec<PreviewMappingEntry>> = ...;
    static PREVIEW_PROVENANCE_FALLBACK: RefCell<BTreeMap<StableId, PreviewProvenance>> = ...;
}
```

## Non-functional requirements

### NFR-E1 — Backward compatibility

`wasm_preview.rs` and `preview_runtime.rs` callers must continue to
work without modification **except** `apply_pending_causality_edges`
(which now calls `set_provenance` instead of writing the thread_local
directly).

### NFR-E2 — Type safety

`serde_json::to_value` / `from_value` failures must `.expect()` with
descriptive messages because the typed structs derive
`Serialize + Deserialize + PartialEq` and the round-trip is exercised
by the existing tests (`metrics_round_trip`, `mapping_round_trip`).

### NFR-E3 — Test coverage

The migration must:

- Keep all 3 existing tests passing (`metrics_round_trip`,
  `mapping_round_trip`, `mapping_contains_no_bevy_entity_id_field`).
- Add **session-installed parity tests** in
  `crates/editor-bevy/tests/preview_inspector_parity.rs` (matching
  Block A2's `actuator_parity.rs` pattern). Coverage:
  - `set_metrics` round-trip via session
  - `set_mapping` round-trip via session
  - `set_provenance` round-trip via session
  - `increment_rebuild_count` increments via session
  - Fallback path: when no session is installed, the renamed
    `PREVIEW_METRICS_FALLBACK` etc. thread_locals still serve reads.
  - Two-session isolation: two `EditorSession`s do not leak state
    across each other.

### NFR-E4 — Documentation

- The doc comments on the new functions explain the dual-write
  pattern and reference ADR-0052 + Block A2 precedent.
- `docs/architecture/state-ownership-matrix.md` § H2.5 is updated to
  mark the three cells as RETIRED with the same pattern as Block D.
- `tools/archcheck-globals/globals-inventory.yaml` removes the three
  entries and adds them to `retired:` with the new Block E
  attribution.

## Out-of-scope (explicit)

- Removing the fallback thread_locals entirely (single-write) — that
  is a future follow-up after all test surfaces install a session.
- Migration of the `last_rebuild_cause` getter (`last_rebuild_cause`
  already session-owned at `preview_inspector.rs:143`).
- Bevy system changes — `preview_inspector.rs` has no Bevy systems.
- New WASM exports — `wasm_preview.rs` API unchanged.

## Acceptance criteria

- [ ] All 3 existing unit tests pass: `cargo test -p editor-bevy --lib preview_inspector`.
- [ ] All new parity tests pass: `cargo test -p editor-bevy --test preview_inspector_parity`.
- [ ] `cargo check --workspace --locked` succeeds.
- [ ] `cargo check -p editor-model --target wasm32-unknown-unknown --locked` succeeds.
- [ ] `cd tools/archcheck-globals && npm run check` reports 26
  declarations (was 29), all match inventory (was 29 → 26 after
  retirement of 3 H2.5 cells).
- [ ] No production code path references the old names
  `PREVIEW_METRICS` / `PREVIEW_MAPPING` / `PREVIEW_PROVENANCE`
  (renamed to `*_FALLBACK`).
- [ ] `state-ownership-matrix.md` § H2.5 shows 4 of 9 cells retired
  (was 1 of 9 after Block D).

## Risks and mitigations

| Risk                                                | Severity | Mitigation                                                                  |
|-----------------------------------------------------|----------|-----------------------------------------------------------------------------|
| Test failures from missing session in lib tests     | Medium   | Fallback path keeps tests green; new parity tests cover both paths.          |
| Serialization overhead in `emit_events` hot loop    | Low      | `serde_json::to_value` on a 3-field struct is ~tens of nanoseconds; budget OK. |
| `StableId::0` field access from editor_bevy         | Low      | `StableId` is a tuple struct with `pub String`; precedent at preview_inspector.rs:175. |
| `Vec<serde_json::Value>` allocation churn           | Low      | Mapping size is bounded by document instance count; rebuilds are infrequent. |
| Two-session isolation regression                    | Low      | Parity test explicitly covers this case.                                     |

## Open questions

None. The exploration confirmed all architectural questions:
target field exists, pattern precedent exists, dual-write fallback
is the proven approach.
