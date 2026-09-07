# Proposal: H2.5 — Collapse Runtime / Preview / Hot-Reload thread_locals

> **Status:** H2.5 — five slices of `EditorSession` migration (H2 program, milestone H2).
> **Predecessors:** H2.1 (`docs/architecture/state-ownership-matrix.md`), H2.2 (#185, #186), H2.3 (#187), H2.4 (#188).
> **Authority:** `state-ownership-matrix.md` § H2.5.

## Intent

Nine `thread_local!` cells in `crates/editor-bevy/src/` currently hold live runtime,
preview-inspector and hot-reload coordination state that has no business being
thread-local. `state-ownership-matrix.md` § H2.5 has pre-assigned each cell to a
canonical owner; this change retires the cells, moves the state behind the
existing `EditorSession` / `EditorSessionPort` seam, and — for one cell — onto
a typed Bevy `Resource` whose lifetime is strictly bound to the Bevy world.

The intent mirrors the four prior H2.x slices (H2.2 registries, H2.3 scene
family, H2.4 asset family): one PR family, one parity test suite, one matrix
deletion per retired cell.

### Cells in scope (9)

| # | Cell | Crate / declared_at | Target owner | Group |
|---|------|---------------------|--------------|-------|
| 1 | `COMMAND_BUS` | editor-bevy/src/lib.rs (LinearBus) | `EditorSession.runtime.command_bus` | A |
| 2 | `EVENT_BUS` | editor-bevy/src/lib.rs (LinearBus) | `EditorSession.runtime.event_bus` | A |
| 3 | `ACTUATOR_OUTPUT_BUS` | editor-bevy/src/actuator_bus.rs | `EditorSession.runtime.actuator_outputs` | A |
| 4 | `HOT_RELOAD_BUS` | editor-bevy/src/hot_reload_state.rs | `EditorSession.runtime.hot_reload_requests` | A |
| 5 | `PLAY_MODE_REQUEST` | editor-bevy/src/hot_reload_state.rs | `EditorSession.runtime.play_mode_request` | A |
| 6 | `PREVIEW_METRICS` | editor-bevy/src/preview_inspector.rs | `EditorSession.preview_state.preview_inspector.metrics_typed` | B |
| 7 | `PREVIEW_MAPPING` | editor-bevy/src/preview_inspector.rs | `EditorSession.preview_state.preview_inspector.mapping_typed` | B |
| 8 | `PREVIEW_PROVENANCE` | editor-bevy/src/preview_inspector.rs | `EditorSession.preview_state.preview_inspector.provenance_typed` | B |
| 9 | `KEYBOARD_STATE` | editor-bevy/src/logic_evaluator.rs | Bevy `Resource` `editor_bevy::InputState` | C |

## Scope

### In scope
- New typed fields on `EditorSession.runtime` and `EditorSession.preview_state.preview_inspector`
  (groups A and B).
- Five new methods on `EditorSessionPort` for groups A: `runtime_command_bus_mut`,
  `runtime_event_bus_mut`, `runtime_actuator_outputs_mut`,
  `runtime_hot_reload_requests_mut`, `runtime_play_mode_request_mut`.
- A new Bevy `Resource` `InputState` (group C) and the small plugin/registration
  that initializes it.
- Thin trampoline wrappers in `editor-bevy/src/preview_inspector.rs`,
  `editor-bevy/src/hot_reload_state.rs`, `editor-bevy/src/actuator_bus.rs`,
  `editor-bevy/src/logic_evaluator.rs` that route reads/writes through
  `editor_model::ports::with_session_mut` (group A + B) and through
  `Res<InputState>` (group C).
- Removal of the nine `thread_local!` declarations and the `LinearBus` /
  `ActuatorBus` helpers that exist only to back them.
- Updates to `tools/archcheck-globals/globals-inventory.yaml`: remove 9 entries,
  document them as RETIRED in `state-ownership-matrix.md` § H2.5.
- Parity tests for runtime apply-back, hot reload, preview rebuild,
  save/load, validation refresh; the multi-session re-entrancy guard.

### Out of scope
- Promoting strongly-typed preview inspector data to the WASM/TS surface
  (group B decision **b** keeps the JSON projection alive for the existing
  consumers; see "Approach / Group B").
- A new InputState plugin refactor beyond the minimum needed for group C
  migration. The existing `update_keyboard_state(Res<ButtonInput<KeyCode>>)`
  system already populates the state; we only move ownership.
- H2.6 (`C` id-mint) and later H2.x cells documented in the matrix.
- Bevy 0.19 → 0.20+ keyboard event API upgrade (assumed not yet landed).
- Any frontend / TS / WASM-bindgen signature changes; the WASM surface
  continues to call the trampolines.

## Capabilities

> CONTRACT with sddk-tasks. Canonical `openspec/specs/` already exist for the
> runtime coordination domain. The delta is in the **migration of ownership**
> of the existing types to `EditorSession` / Bevy `Resource`.

### References to canonical specs
- `openspec/specs/hot-reload-bus/spec.md` — defines `HotReloadRequest`
  variants, queue semantics, drain system, WASM exports.
- `openspec/specs/hot-reload-event-bus/spec.md` — frontend publish/subscribe.
- `openspec/specs/hot-reload-status-ui/spec.md` — auto-reload status line.
- `openspec/specs/source-file-cache/spec.md` — already RETIRED in H2.2.

### New capabilities introduced by this change
- `runtime-state-owned-by-session` — Runtime coordination buses
  (COMMAND/EVENT/ACTUATOR_OUTPUT/HOT_RELOAD/PLAY_MODE) live on
  `EditorSession.runtime`, not as thread-locals. Trampolines preserve the
  existing public API.
- `preview-inspector-state-owned-by-session` — Preview inspector's metrics,
  mapping, and provenance move to `EditorSession.preview_state.preview_inspector`
  with both a typed backing field and the existing JSON projection.
- `keyboard-input-owned-by-bevy-resource` — Keyboard sensor state lives on
  Bevy `Resource InputState` (NOT on `EditorSession`, because its lifetime is
  bound to the Bevy world / input frame).

### Modified capabilities
None of the canonical specs change; we modify only the *storage location* of
their runtime state. Trampolines keep observable behaviour identical.

## Approach

### Group A — EditorSession.runtime (5 cells, LinearBus + small queues)

Pattern follows H2.4 (`AssetFocus` ADT + trampolines over `with_session_mut`):

1. Extend `RuntimeSessionState` (`crates/editor-application/src/session.rs`)
   with five typed fields:
   ```rust
   pub command_bus: LinearBus,
   pub event_bus: LinearBus,
   pub actuator_outputs: ActuatorBus,
   pub hot_reload_requests: Vec<HotReloadRequest>,
   pub play_mode_request: Option<PlayModeRequest>,
   ```
   `LinearBus` moves from `editor-bevy/src/lib.rs` into
   `editor-model::runtime::linear_bus` (a new pure module) so `EditorSession`
   can own it without an `editor-application → editor-bevy` dependency edge.
   `ActuatorBus` similarly moves into
   `editor-model::runtime::actuator_bus`. `HotReloadRequest` /
   `PlayModeRequest` already live in `editor-bevy`; they move to
   `editor-model::runtime::hot_reload` (the canonical spec already documents
   the type, this is just a location fix).

2. Extend `EditorSessionPort` (`crates/editor-model/src/session_port.rs`)
   with five `*_mut` accessors and provide impls on `EditorSession` that
   route to `self.runtime.<field>_mut()`.

3. Replace each thread_local `with()` / `borrow_mut()` site in
   `crates/editor-bevy/src/lib.rs`, `hot_reload_state.rs`, `actuator_bus.rs`
   with a `editor_model::ports::with_session_mut(|s| ...)` closure.

4. Keep the public Bevy-system / WASM-export functions as thin trampolines
   so no Bevy system signature changes.

### Group B — EditorSession.preview_state.preview_inspector (3 cells, typed ↔ JSON)

**Decision: option (b) — typed backing field + JSON projection on the same struct.**

Rationale (against options a and c):
- **(a) Promote PreviewInspectorState to typed** — breaks the JSON projection
  that the WASM/TS consumers and existing preview inspector UI rely on. Would
  touch the WASM bridge for a structural-storage win.
- **(c) Move typed state to EditorSession.runtime.preview_typed, keep JSON in
  PreviewInspectorState** — splits the conceptual model across two session
  fields and requires a sync layer; increases surface for drift bugs.
- **(b) Add typed backing fields alongside the JSON projection** — minimal
  blast radius. Bevy systems write the typed fields (cheap, no JSON
  round-trip). The JSON projection is regenerated from the typed fields on
  each Bevy tick (or on demand for the WASM read).

Concretely:
```rust
pub struct PreviewInspectorState {
    /// Typed backing for PREVIEW_METRICS (replaces thread_local).
    pub metrics_typed: PreviewMetrics,
    /// Typed backing for PREVIEW_MAPPING (replaces thread_local).
    pub mapping_typed: Vec<PreviewMappingEntry>,
    /// Typed backing for PREVIEW_PROVENANCE (replaces thread_local).
    pub provenance_typed: BTreeMap<StableId, PreviewProvenance>,
    /// JSON projection for WASM/TS consumers (regenerated on tick).
    pub metrics: serde_json::Value,
    pub mapping: Vec<serde_json::Value>,
    pub provenance: BTreeMap<String, serde_json::Value>,
    pub last_rebuild_cause: Option<RebuildCause>,
}
```
The existing `preview_inspector::set_metrics`, `set_mapping`, `set_provenance`
become trampolines that write the typed field and queue a deferred projection
update. A small Bevy system `project_preview_inspector_json` runs each tick
(or on demand before the WASM read) and regenerates the JSON fields from
the typed backing.

`PreviewMetrics` / `PreviewMappingEntry` / `PreviewProvenance` already exist
in `editor-model`; they only need `Serialize` / `Clone` derives confirmed.
The proxy through `with_session_mut` mirrors `record_rebuild_cause`.

### Group C — Bevy Resource InputState (1 cell)

`KEYBOARD_STATE` is fundamentally a Bevy-frame concern: the input system
reads `Res<ButtonInput<KeyCode>>` and writes a `HashSet<String>` of held
keys. Move the cell into a Bevy `Resource`:

```rust
// crates/editor-bevy/src/input_state.rs
#[derive(Resource, Default, Debug)]
pub struct InputState {
    pub held_keys: HashSet<String>,
}
```

The existing `update_keyboard_state(Res<ButtonInput<KeyCode>>, ResMut<InputState>)`
system writes `ResMut<InputState>::held_keys`. A new tiny
`InputStatePlugin` (in the same file) initializes the resource on
`App::add_plugins(InputStatePlugin)`. Sensor nodes that consume
`KEYBOARD_STATE.with(...)` become `Res<InputState>` readers.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-application/src/session.rs` | Modified | `RuntimeSessionState` gains 5 fields; impl accessors. |
| `crates/editor-model/src/session_port.rs` | Modified | 5 new `*_mut` methods on the trait. |
| `crates/editor-model/src/runtime/` (new) | Added | `linear_bus.rs`, `actuator_bus.rs`, `hot_reload.rs`. |
| `crates/editor-model/src/lib.rs` | Modified | Re-export new runtime types. |
| `crates/editor-bevy/src/lib.rs` | Modified | Remove `thread_local! COMMAND_BUS / EVENT_BUS`; trampolines. |
| `crates/editor-bevy/src/preview_inspector.rs` | Modified | Typed fields; remove thread_locals; add projection system. |
| `crates/editor-bevy/src/hot_reload_state.rs` | Modified | Remove thread_locals; types move to editor-model; trampolines. |
| `crates/editor-bevy/src/actuator_bus.rs` | Modified | Remove thread_local; trampoline over `with_session_mut`. |
| `crates/editor-bevy/src/logic_evaluator.rs` | Modified | `KEYBOARD_STATE` removed; `Res<InputState>` readers. |
| `crates/editor-bevy/src/input_state.rs` (new) | Added | `InputState` Resource + plugin. |
| `crates/editor-bevy/src/lib.rs` plugin list | Modified | Register `InputStatePlugin`. |
| `tools/archcheck-globals/globals-inventory.yaml` | Modified | Remove 9 entries. |
| `docs/architecture/state-ownership-matrix.md` | Modified | Mark 9 H2.5 rows as RETIRED (this PR). |
| `crates/editor-bevy/tests/hot_reload.rs` | Modified | Existing parity — must pass. |
| `crates/editor-bevy/tests/play_mode.rs` | Modified | Existing parity — must pass. |
| `crates/editor-bevy/tests/rebuild_cause_unified.rs` | Modified | Existing parity — must pass. |
| `crates/editor-bevy/tests/apply_actuator_outputs_in_preview.rs` | Modified | Existing parity — must pass. |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| **JSON projection drift**: typed and JSON fields go out of sync. | Medium | Single owning write site (the Bevy system); invariant test asserts equality after each tick. |
| **Lazy init assumptions**: some thread_locals relied on `RefCell<None>` first-write wins. | Low | `EditorSession::new()` initializes all fields eagerly. |
| **WASM bindings assume direct pointer access** (LinearBus `ptr()`/`len()` exports). | Medium | New `LinearBus::ptr()` / `len()` accessors on `EditorSession.runtime.*_bus`; trampolines mirror the previous cell-byte semantics. |
| **KEYBOARD_STATE consumers outside Bevy systems** (e.g. WASM export). | Low | `Res<InputState>` readable via `Res::inner()` or via the plugin; WASM export is a Bevy system already. |
| **`update_keyboard_state` runs before `InputState` is registered**. | Low | `InputStatePlugin` initializes `InputState::default()` on `App::add_plugins`; test for fresh-app boot order. |
| **Tests that depend on thread-local storage state** (`rebuild_cause_unified.rs` writes `PREVIEW_PROVENANCE` directly). | Medium | Replace direct `with()` writes with `with_session_mut`; update test fixtures. |
| **Multi-session leakage**: `EditorSession::new()` constructs owned buses; if a Bevy world is shared across sessions, lifetime blurs. | Medium | Each session registers its own `Arc<Mutex<dyn EditorSessionPort>>`; the Bevy world has no cross-session reach (H1.4 invariant). |
| **archcheck-globals drift**: missed entries. | Low | CI grep covers 9 exact names; matrix update is the audit trail. |

## Migration Sequencing

The three groups are independent and can land as three separate PRs within
this change (or as one PR if the reviewer prefers atomicity). Recommended
order:

1. **Group A** (runtime buses) — broadest scope, most trampolines, mirrors H2.4.
2. **Group B** (preview state) — touches the JSON/typed decision; lands after A
   so the session-port extension is already proven.
3. **Group C** (InputState) — smallest and most isolated; can land last.

All three can land in one PR if the diff stays under 1500 lines (precedent:
H2.4 was ~700 lines). The parity suite from `crates/editor-bevy/tests/`
must pass for each landing.

## Open Questions (resolved by this proposal)

- **Group B a/b/c decision**: (b) — typed backing + JSON projection on the same
  struct. Rationale in "Approach / Group B".
- **LinearBus location**: editor-model (not editor-bevy) so the dep graph stays
  `editor-application → editor-model` (not editor-bevy).
- **HotReloadRequest location**: editor-model (not editor-bevy) for the same
  reason. Canonical spec already documents the type.
- **KEYBOARD_STATE → Resource vs Local**: Resource — Local would only serve one
  system; Resource is the Bevy idiom for cross-system shared state.

## Success Criteria

- [ ] `cargo fmt --all --check` green.
- [ ] `cargo check --workspace --all-targets` green.
- [ ] `cargo test --workspace` green; in particular:
      `hot_reload.rs`, `play_mode.rs`, `rebuild_cause_unified.rs`,
      `apply_actuator_outputs_in_preview.rs` parity.
- [ ] `tools/archcheck-globals` reports zero matches for `COMMAND_BUS`,
      `EVENT_BUS`, `ACTUATOR_OUTPUT_BUS`, `HOT_RELOAD_BUS`,
      `PLAY_MODE_REQUEST`, `PREVIEW_METRICS`, `PREVIEW_MAPPING`,
      `PREVIEW_PROVENANCE`, `KEYBOARD_STATE`.
- [ ] `state-ownership-matrix.md` § H2.5 rows are RETIRED with PR reference.
- [ ] Two independent `EditorSession` instances can be constructed in the same
      process and each retain their own runtime state (UAT-ARCH-004).
- [ ] Fresh-app boot order does not depend on hidden initialization
      (UAT-ARCH-003): `EditorSession::new()` → `InputState::default()` →
      Bevy world → `update_keyboard_state` runs → `InputState.held_keys`
      reflects `ButtonInput<KeyCode>` within one frame.