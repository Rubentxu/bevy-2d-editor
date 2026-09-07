# Spec: h2-5-runtime-coordination

> **Change:** h2-5-runtime-coordination
> **Authority:** `docs/architecture/state-ownership-matrix.md` § H2.5
> **Predecessors:** H2.1 (#184), H2.2 (#185, #186), H2.3 (#187), H2.4 (#188)
> **Canonical references:** `openspec/specs/hot-reload-bus/spec.md`,
> `openspec/specs/hot-reload-event-bus/spec.md`, `openspec/specs/hot-reload-status-ui/spec.md`

This spec retires nine `thread_local!` cells from `crates/editor-bevy/src/`
into three target owners:

- **Group A** (5 cells) → `EditorSession.runtime`
- **Group B** (3 cells) → `EditorSession.preview_state.preview_inspector` (typed backing + JSON projection)
- **Group C** (1 cell) → Bevy `Resource editor_bevy::InputState`

---

## Group A — Runtime Buses → EditorSession.runtime

### REQ-A-1: Typed Fields on EditorSession.runtime

`EditorSession.runtime` MUST expose five typed fields:

```rust
pub command_bus: LinearBus,
pub event_bus: LinearBus,
pub actuator_outputs: ActuatorBus,
pub hot_reload_requests: Vec<HotReloadRequest>,
pub play_mode_request: Option<PlayModeRequest>,
```

**Scenario: EditorSession::new initializes all fields eagerly**
- **Given** a freshly constructed `EditorSession`
- **When** the session is created via `EditorSession::new(store, clock)`
- **Then** `runtime.command_bus` MUST be a default-constructed `LinearBus`
- **And** `runtime.event_bus` MUST be a default-constructed `LinearBus`
- **And** `runtime.actuator_outputs` MUST be an empty `ActuatorBus`
- **And** `runtime.hot_reload_requests` MUST be an empty `Vec`
- **And** `runtime.play_mode_request` MUST be `None`

### REQ-A-2: EditorSessionPort Accessors

`EditorSessionPort` MUST expose five new methods that return mutable
references into `EditorSession.runtime`:

```rust
fn runtime_command_bus_mut(&mut self) -> &mut LinearBus;
fn runtime_event_bus_mut(&mut self) -> &mut LinearBus;
fn runtime_actuator_outputs_mut(&mut self) -> &mut ActuatorBus;
fn runtime_hot_reload_requests_mut(&mut self) -> &mut Vec<HotReloadRequest>;
fn runtime_play_mode_request_mut(&mut self) -> &mut Option<PlayModeRequest>;
```

**Scenario: accessors route through with_session_mut**
- **Given** a registered session
- **When** `editor_model::ports::with_session_mut(|s| s.runtime_command_bus_mut().write(42, &[]))`
- **Then** the write MUST land on the session's `runtime.command_bus`
- **And** a subsequent read via the same path MUST observe `42`

### REQ-A-3: Trampoline Functions Preserved

The existing public trampolines in `editor-bevy/src/lib.rs`,
`hot_reload_state.rs`, and `actuator_bus.rs` MUST remain byte-identical in
signature so Bevy systems and WASM exports compile unchanged:

- `submit_command(cmd_type: u16, payload: &[u8]) -> bool`
- `drain_events() -> Vec<(u16, Vec<u8>)>`
- `submit_actuator_output(entity: Entity, field: &str, value: PortValue)`
- `drain_actuator_outputs() -> Vec<ActuatorOutput>`
- `push_hot_reload_request(req: HotReloadRequest)`
- `drain_hot_reload_requests() -> Vec<HotReloadRequest>`
- `set_play_mode_request(req: PlayModeRequest)`
- `take_play_mode_request() -> Option<PlayModeRequest>`

**Scenario: trampoline bodies use with_session_mut**
- **Given** any of the trampolines above
- **When** the function is invoked
- **Then** its body MUST be a closure passed to
  `editor_model::ports::with_session_mut(|s| ...)`
- **And** the body MUST NOT touch any `thread_local!` cell

### REQ-A-4: Type Movement to editor-model

`LinearBus`, `ActuatorBus`, `HotReloadRequest`, and `PlayModeRequest` MUST
live in `crates/editor-model/src/runtime/` (new sub-module) so
`EditorSession` (in `editor-application`) can own them without an
`editor-application → editor-bevy` dependency edge.

**Scenario: dep graph stays directional**
- **Given** the new types in `editor-model`
- **When** `cargo metadata` is run
- **Then** `editor-application` MUST depend on `editor-model` (not on
  `editor-bevy`) for these types

### REQ-A-5: Parity — Runtime Apply-Back

The runtime apply-back flow MUST continue to drain `actuator_outputs`
after each Bevy tick and apply the writes to components, identical to
the pre-migration behaviour.

**Scenario: actuator outputs are drained each tick**
- **Given** play mode is active and `runtime.actuator_outputs` holds two
  pending `ActuatorOutput` entries from the previous tick
- **When** the Bevy world ticks once
- **Then** `drain_actuator_outputs` MUST return both entries (FIFO)
- **And** after the drain, `runtime.actuator_outputs.pending.len() == 0`
- **And** the corresponding component fields reflect the typed values

### REQ-A-6: Parity — Hot Reload

The hot-reload flow MUST continue to drain `hot_reload_requests` once
per Bevy tick and dispatch per-variant invalidations, satisfying
`openspec/specs/hot-reload-bus/spec.md` REQ-HRB-1..4.

**Scenario: hot-reload requests drain FIFO**
- **Given** `runtime.hot_reload_requests` holds
  `[Source(A), Asset(B), ForceReloadAll]`
- **When** the Bevy world ticks once
- **Then** source invalidation for A MUST execute first
- **And** asset invalidation for B MUST execute second
- **And** `force_reload_wasm`-equivalent MUST execute last
- **And** the vector MUST be empty after the drain

**Scenario: ForceReloadAll preserves play-mode state**
- **Given** play mode is active and a `TransformSnapshot` exists
- **When** `ForceReloadAll` is dispatched
- **Then** `play_mode_request` MUST remain in its prior state
- **And** `TransformSnapshot` MUST be unchanged

### REQ-A-7: Parity — Save / Load

Save and load flows MUST NOT depend on any of the retired cells being
thread-local.

**Scenario: round-trip preserves runtime state**
- **Given** a session with `runtime.command_bus` containing a buffered
  event and `runtime.play_mode_request = Some(Enter)`
- **When** the session is serialized, written to storage, read back, and
  reconstructed
- **Then** the reconstructed `runtime.command_bus` MUST contain the
  buffered event
- **And** `runtime.play_mode_request` MUST be `Some(Enter)`

### Group A — Edge cases

#### EC-A-1: with_session_mut returns None during early boot
If a trampoline is called before the session is registered (Bevy systems
can run before `init_project_store` in tests), the trampoline MUST be a
silent no-op, matching the existing `record_rebuild_cause` precedent.

#### EC-A-2: Two sessions, same thread
If two `EditorSession` instances exist in the same process, each
trampoline call MUST route to its respective session because the port
seam is per-session, not per-thread.

#### EC-A-3: Hot reload during actuator drain
If `drain_hot_reload_requests` runs concurrently with `drain_actuator_outputs`
(same Bevy schedule tick), each MUST drain its own queue without
interference.

---

## Group B — Preview Inspector → EditorSession.preview_state

> **Decision:** option (b) — typed backing field alongside the existing
> JSON projection on `PreviewInspectorState`. Rationale documented in
> `proposal.md` § Approach / Group B.

### REQ-B-1: Typed Backing Fields

`PreviewInspectorState` MUST gain three typed backing fields alongside
the existing JSON fields:

```rust
pub struct PreviewInspectorState {
    // Existing JSON projection (unchanged for WASM/TS consumers).
    pub metrics: serde_json::Value,
    pub mapping: Vec<serde_json::Value>,
    pub provenance: BTreeMap<String, serde_json::Value>,

    // NEW: typed backing fields.
    pub metrics_typed: PreviewMetrics,
    pub mapping_typed: Vec<PreviewMappingEntry>,
    pub provenance_typed: BTreeMap<StableId, PreviewProvenance>,

    pub last_rebuild_cause: Option<RebuildCause>,
}
```

**Scenario: Default-construction is consistent**
- **Given** a freshly constructed `PreviewInspectorState`
- **When** `Default::default()` is called
- **Then** `metrics_typed` MUST be a default `PreviewMetrics`
- **And** `mapping_typed` MUST be an empty `Vec`
- **And** `provenance_typed` MUST be an empty `BTreeMap`

### REQ-B-2: Trampolines Write Typed Backing

The existing `set_metrics`, `set_mapping`, `set_provenance`,
`increment_rebuild_count`, `get_metrics`, `get_mapping`, `get_provenance`
in `editor-bevy/src/preview_inspector.rs` MUST write and read the typed
backing fields through `editor_model::ports::with_session_mut`.

**Scenario: set_metrics writes typed field**
- **Given** an empty `metrics_typed`
- **When** `set_metrics(new_metrics)` is called
- **Then** `metrics_typed` MUST equal `new_metrics` after the call
- **And** the JSON `metrics` field MUST be marked dirty (see REQ-B-3)

**Scenario: get_metrics reads typed field**
- **Given** `metrics_typed == some_metrics`
- **When** `get_metrics()` is called
- **Then** the returned value MUST equal `some_metrics`

### REQ-B-3: JSON Projection Refresh

A small Bevy system `project_preview_inspector_json` MUST run each Bevy
tick (in `Update` schedule, before the WASM read point) and regenerate
the JSON `metrics` / `mapping` / `provenance` fields from the typed
backing. A dirty flag tracks whether the typed fields changed since
the last projection.

**Scenario: dirty flag triggers projection**
- **Given** `metrics_typed` was just updated via `set_metrics`
- **And** the dirty flag is set
- **When** `project_preview_inspector_json` runs
- **Then** the JSON `metrics` field MUST equal `serde_json::to_value(metrics_typed)`
- **And** the dirty flag MUST be cleared

**Scenario: no-op when not dirty**
- **Given** the dirty flag is unset (no writes since last projection)
- **When** `project_preview_inspector_json` runs
- **Then** the system MUST NOT regenerate the JSON fields
- **And** MUST NOT allocate

### REQ-B-4: Parity — Preview Rebuild

`rebuild_preview_world` MUST continue to call the existing trampolines
in the same order as before the migration. The visible behaviour of the
preview inspector UI and the JSON export to WASM MUST be unchanged.

**Scenario: rebuild writes the same JSON as before**
- **Given** a build pipeline that calls `set_mapping(entries)` then
  `set_metrics(metrics)` then `set_provenance(provenance)`
- **When** the rebuild completes and `project_preview_inspector_json` runs
- **Then** the JSON fields MUST match the pre-migration output for the
  same inputs (golden test in `crates/editor-bevy/tests/preview_state_parity.rs`)

**Scenario: rebuild_cause survives typed migration**
- **Given** `record_rebuild_cause(cause)` is called from a rebuild
- **When** the session is queried for `last_rebuild_cause`
- **Then** `Some(cause)` MUST be returned (existing H2.2 behaviour)

### REQ-B-5: Test Fixtures Migrated

`crates/editor-bevy/tests/rebuild_cause_unified.rs` writes to
`PREVIEW_PROVENANCE` via the thread-local `with()`. The fixture MUST be
migrated to use `editor_model::ports::with_session_mut` to set up the
test state.

**Scenario: test setup uses session port**
- **Given** `rebuild_cause_unified.rs` test fixture
- **When** it sets up provenance entries for the test
- **Then** it MUST call
  `with_session_mut(|s| s.preview_inspector_mut().provenance_typed = map)`
- **And** MUST NOT call `PREVIEW_PROVENANCE.with(...)`

### Group B — Edge cases

#### EC-B-1: JSON serialization failures
If `serde_json::to_value` fails for a typed field (e.g. a non-serializable
type), the projection MUST log a warning and leave the JSON field
unchanged rather than corrupting the inspector.

#### EC-B-2: Concurrent reads / writes
If a Bevy system reads the JSON field while another system writes the
typed backing in the same frame, the JSON field reflects the **previous**
projection (no torn read). The next projection tick makes it consistent.

#### EC-B-3: Causality edges drain
The existing `apply_pending_causality_edges` function in
`preview_inspector.rs` MUST continue to drain pending causality edges
into `provenance_typed` (not the old `PREVIEW_PROVENANCE` thread-local).

---

## Group C — KEYBOARD_STATE → Bevy Resource InputState

### REQ-C-1: InputState Resource

A new `InputState` Bevy `Resource` MUST exist in
`crates/editor-bevy/src/input_state.rs`:

```rust
#[derive(Resource, Default, Debug)]
pub struct InputState {
    pub held_keys: HashSet<String>,
}
```

**Scenario: InputState is registered**
- **Given** the Bevy app is initialized with `InputStatePlugin`
- **When** the app starts up
- **Then** `Res<InputState>` MUST be valid (returns `Default::default()`)

### REQ-C-2: InputStatePlugin

A new `InputStatePlugin` MUST register `InputState` and the
`update_keyboard_state` system in the Bevy app.

```rust
pub struct InputStatePlugin;
impl Plugin for InputStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
           .add_systems(Update, update_keyboard_state);
    }
}
```

**Scenario: plugin registers and runs**
- **Given** an App with `InputStatePlugin` added
- **When** the Bevy world ticks
- **Then** `Res<InputState>::held_keys` MUST reflect
  `Res<ButtonInput<KeyCode>>::get_pressed()` after the first tick

### REQ-C-3: update_keyboard_state reads InputState

The existing `update_keyboard_state` system MUST read `Res<ButtonInput<KeyCode>>`
and write `ResMut<InputState>`:

```rust
pub fn update_keyboard_state(
    keys: Res<ButtonInput<KeyCode>>,
    mut input: ResMut<InputState>,
) {
    input.held_keys.clear();
    for key in keys.get_pressed() {
        input.held_keys.insert(format!("{:?}", key));
    }
}
```

**Scenario: update_keyboard_state populates held_keys**
- **Given** `ButtonInput<KeyCode>` has `KeyW` pressed
- **When** the system runs once
- **Then** `InputState::held_keys` MUST contain `"KeyW"`

### REQ-C-4: Sensor Nodes Read InputState

Logic sensor nodes that currently read `KEYBOARD_STATE.with(...)` MUST
read `Res<InputState>` instead.

**Scenario: IsKeyDown sensor reads Res<InputState>**
- **Given** `InputState::held_keys` contains `"Space"`
- **When** a logic graph evaluates an `IsKeyDown("Space")` sensor node
- **Then** the sensor MUST return `true`

### REQ-C-5: WASM Export Updated

If `KEYBOARD_STATE` is exposed via WASM (`key_state_wasm` or similar),
the export MUST read `Res<InputState>` via a small Bevy system.

**Scenario: WASM export returns held_keys**
- **Given** `InputState::held_keys == {"KeyW", "Space"}`
- **When** JavaScript calls `key_state_wasm()`
- **Then** the returned list MUST contain both `"KeyW"` and `"Space"`

### Group C — Edge cases

#### EC-C-1: Plugin registration order
If `InputStatePlugin` is added after a system that reads
`Res<InputState>` is registered, the system panics with
`SystemParamFetchError`. The plugin MUST be added before any consumer
system.

#### EC-C-2: KeyCode formatting
`format!("{:?}", KeyCode::KeyW)` produces `"KeyW"` today. If Bevy changes
the Debug representation in a future version, the WASM contract breaks.

#### EC-C-3: Multiple worlds
In a multi-session setup, each Bevy world has its own `InputState`
because `InputState` is a per-world `Resource`. There is no cross-world
sharing, matching the H1.4 multi-session invariant.