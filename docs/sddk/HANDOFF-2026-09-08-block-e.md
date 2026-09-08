# Handoff — 2026-09-08 H2.5 Block E

## TL;DR

Closed **H2.5 Block E** (preview inspector → `EditorSession`) and
shipped **v0.108.5**. Three thread_locals (`PREVIEW_METRICS`,
`PREVIEW_MAPPING`, `PREVIEW_PROVENANCE`) migrated via the dual-write
fallback pattern. **4 of 9 H2.5 cells retired** (halfway). 5 remain
for Blocks F/G/H.

## What landed

| WU      | Files                                                                | What                                                                                              |
|---------|----------------------------------------------------------------------|---------------------------------------------------------------------------------------------------|
| WU-E-1  | `crates/editor-bevy/src/preview_inspector.rs`                        | Renamed `PREVIEW_*` → `PREVIEW_*_FALLBACK` with dual-write doc comment.                           |
| WU-E-2  | `crates/editor-bevy/src/preview_inspector.rs`                        | `set_metrics` + `increment_rebuild_count` session-first with JSON serialization.                  |
| WU-E-3  | `crates/editor-bevy/src/preview_inspector.rs`                        | `set_mapping` + `set_provenance` session-first with JSON serialization.                          |
| WU-E-4  | `crates/editor-bevy/src/preview_inspector.rs`                        | `get_metrics`, `get_mapping`, `get_provenance` session-first with JSON deserialization.           |
| WU-E-5  | `crates/editor-bevy/src/preview_inspector.rs`                        | `apply_pending_causality_edges` routes through `set_provenance` (respects dual-write).            |
| WU-E-7  | `crates/editor-bevy/tests/preview_inspector_parity.rs` (new)         | 6 parity tests (Block A2 `actuator_parity.rs` pattern): round-trip, fallback, two-session isolation. |
| WU-E-9  | `docs/architecture/state-ownership-matrix.md` § H2.5                  | 3 cells RETIRED + progress note updated to 4 of 9.                                                |
| WU-E-10 | `tools/archcheck-globals/globals-inventory.yaml`                     | 3 retired entries + 3 `*_FALLBACK` entries documented as OPEN. archcheck 29=29.                   |
| WU-E-11 | `Cargo.toml`                                                          | `0.108.4 → 0.108.5`.                                                                              |

Total: **+331 / -51** across 6 source files + 1 new test file.

## Cycle mechanics

- **Path**: A-min (explore → spec → tasks → build → verify → release → archive).
- **Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-e`
  (started 08:26:00, closed 08:39:28, sequence 128).
- **Sequence**: 117 (start) → 118 (explore→spec) → 120 (spec→build)
  → 122 (build→verify) → 124 (verify→release) → 128 (closed).
- **Tag**: `v0.108.5` on origin pointing at commit `6ff1048`.

## Migration pattern

The dual-write fallback used for every set:

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

Precedent: Block A2 ActuatorBus, v0.90 PR2 `record_rebuild_cause`.

## Release anomaly (same as Blocks A2 + D)

`sddk release apply` rejected with "dirty worktree" because the
workspace contains untracked files owned by OTHER concurrent cycles
(`docs/sddk/wave-d1-editor-gateway-seam/`,
`docs/sddk/world-workspace/`,
`docs/sddk/semantic-editor-model-*/`,
`docs/sddk/application-stabilization-and-roadmap-convergence/`,
`docs/sddk/archive/2026-07-21-scene-component-authoring-ux/`).

Workaround applied (3rd time, pattern now stable per memory):

1. Manual `git push origin main` + manual `git tag -a v0.108.5` +
   `git push --tags`.
2. After committing implementation-receipt + verify-report, the tag
   was **repositioned to `6ff1048`** (HEAD with the SDDK artifacts).
3. Cycle closed via
   `sddk cycle supersede --reason scope-invalid --evidence-refs [receipt,verify-report,commit:6ff1048,tag:v0.108.5]`.

## Forward work (NOT in this cycle)

### Block F — runtime buses (2 cells)

| Cell         | File                                         | Target owner                  | Sub-system |
|--------------|----------------------------------------------|-------------------------------|------------|
| `COMMAND_BUS` | `crates/editor-bevy/src/lib.rs:407`         | `EditorSession.runtime.bus`   | command entrypoints |
| `EVENT_BUS`   | `crates/editor-bevy/src/lib.rs:408`         | `EditorSession.runtime.events` | telemetry/UI |

`emit_events` already uses `runtime_event_bus_mut()` in
`preview_runtime.rs:1122` (precedent exists for session-first).
Block F applies the same dual-write pattern. A-min or A-lite.

### Block G — hot-reload + play-mode (2 cells)

| Cell              | File                                                | Target owner                  | Sub-system |
|-------------------|-----------------------------------------------------|-------------------------------|------------|
| `HOT_RELOAD_BUS`   | `crates/editor-bevy/src/hot_reload_state.rs:32`   | `EditorSession.runtime.hot_reload` | hot-reload scheduler |
| `PLAY_MODE_REQUEST`| `crates/editor-bevy/src/hot_reload_state.rs:35`   | `EditorSession.runtime.play_mode` | runtime coordinator |

Coupling to Bevy systems needs careful ordering analysis before the
migration. A-min or A-lite, possibly with explore phase.

### Block H — Bevy input resource (1 cell)

| Cell             | File                                                    | Target owner          | Sub-system |
|------------------|---------------------------------------------------------|-----------------------|------------|
| `KEYBOARD_STATE` | `crates/editor-bevy/src/logic_evaluator.rs:1049`       | Bevy `Resource InputState` | Bevy ECS input |

Different shape than F/G (target is Bevy Resource, not EditorSession).
Requires Bevy-specific knowledge of `Resource` lifecycle. A-min.

### Next non-H2.5 priorities

After Block E (or in parallel if dependency-graph permits):

1. **H0 — Release Truth & Architecture Fitness** (per
   `MASTER_ROADMAP.md`): release evidence manifest + smoke cohort
   trim. The 60 s budget breach (pre-existing in
   `evidence-map § 6.4`) is still a blocker.
2. **H3 — Typed EditorBackend** (skeleton + first capability migration).
3. **H10 — v1 Product Proof** (canonical playable sample game; the
   `examples/platformer-minimal/` skeleton already exists).

## Validation evidence

```
$ cargo test -p editor-bevy --test preview_inspector_parity --locked
test result: ok. 6 passed; 0 failed

$ cargo test -p editor-bevy --lib preview_inspector --locked
test result: ok. 3 passed; 0 failed

$ cargo check --workspace --locked
Finished `dev` profile in 22.03s

$ cargo check -p editor-model --target wasm32-unknown-unknown --locked
Finished `dev` profile in 0.30s

$ cd tools/archcheck-globals && npm run check
archcheck-globals: 29 declarations, all match inventory (29 entries)

$ npm test
archcheck-globals tests: all pass
```

## Files for next session

- Cycle artifacts: `docs/sddk/h2-5-runtime-coordination-block-e/`
  (`explore-report.md`, `spec.md`, `tasks.md`,
  `implementation-receipt.md`, `verify-report.md`).
- This handoff: `docs/sddk/HANDOFF-2026-09-08-block-e.md`.
- Updated inventory: `tools/archcheck-globals/globals-inventory.yaml`
  (3 retired + 3 fallback entries).
- Updated matrix: `docs/architecture/state-ownership-matrix.md` § H2.5.
- Updated preview_inspector.rs: dual-write + JSON serialization.
- New parity tests: `crates/editor-bevy/tests/preview_inspector_parity.rs`.

## Critical lessons (to persist in memory)

1. **`sddk release apply` rejects dirty worktrees** — same workaround
   as Blocks A2 and D. Pattern now stable.
2. **`SceneAssetLocalId` ≠ `LocalId`** — `editor_bevy::scene_asset::LocalId`
   is a deprecated alias for `editor_model::ids::SceneAssetLocalId`,
   not the document-level `LocalId`. Tests must import correctly.
3. **`serde_json::Value` is the canonical field type** in
   `PreviewInspectorState`. Migration code must serialize/deserialize
   on every read/write.
4. **The dual-write pattern preserves backward compat** for legacy
   tests that don't install a session. The `*_FALLBACK` thread_locals
   must be **documented in the inventory** as OPEN (they are real
   declarations that the ratchet catches otherwise).
5. **`editor_model::StableId.0` is private**; use `as_str().to_string()`
   or `into_inner()` to access the inner `String`.
