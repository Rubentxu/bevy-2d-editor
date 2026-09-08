# Spec: `h2-5-runtime-coordination-block-a2`

> **Cycle:** `p-28fce7028ac3c497/h2-5-runtime-coordination-block-a2`
> **Phase:** specify · **Path:** A-lite
> **Date:** 2026-09-08
> **Carry-over from:** `h2-5-runtime-coordination` (REQ-A-1..REQ-A-17 in original spec.md)

---

## Goals

G1. Eliminate the double-identity ambiguity between `editor_bevy::logic_evaluator::PortValue` and `editor_model::runtime::PortValue` by replacing the local definition with a re-export of the canonical `editor_model::runtime::PortValue`.

G2. Migrate `editor_bevy::actuator_bus.rs` from its private `thread_local!` `ACTUATOR_OUTPUT_BUS` to the session-owned `editor_model::runtime::ActuatorBus`.

G3. Add parity tests proving the session-backed actuator bus behaves identically to the legacy thread-local pattern.

## Requirements

### REQ-A2-1: Single canonical PortValue
The type `editor_bevy::logic_evaluator::PortValue` SHALL be a re-export of `editor_model::runtime::PortValue`. No two distinct types with the same name in the build graph.

#### Scenario S-A2-1.1: local definition removed
- GIVEN the local `pub enum PortValue { ... }` block in `editor_bevy::logic_evaluator.rs:30-37`
- WHEN the change is applied
- THEN that block is replaced with `pub use editor_model::runtime::PortValue;`
- AND `cargo check -p editor-bevy` passes without re-declaration errors

#### Scenario S-A2-1.2: existing references still resolve
- GIVEN code in `crates/editor-bevy/src/logic_validation.rs`, `actuator_bus.rs`, `sensor_event.rs`, `logic_dispatch.rs` that uses `crate::logic_evaluator::PortValue`
- WHEN the re-export lands
- THEN all those references compile unchanged
- AND `cargo check -p editor-bevy --locked` produces zero new warnings

### REQ-A2-2: Session-owned ActuatorBus
The function `submit_actuator_output(entity, field, value)` SHALL push to `editor_model::runtime::ActuatorBus` accessible via `editor_model::ports::with_session_mut` instead of writing to a `thread_local!`.

#### Scenario S-A2-2.1: thread_local removed
- GIVEN the existing `thread_local! { static ACTUATOR_OUTPUT_BUS: RefCell<Option<ActuatorBus>> = ... }` block
- WHEN the migration is applied
- THEN that `thread_local!` declaration is deleted
- AND `submit_actuator_output` writes to the session's `ActuatorBus`

#### Scenario S-A2-2.2: drain reads from session
- GIVEN a Bevy frame that calls `apply_actuator_outputs`
- WHEN `drain_actuator_outputs` runs
- THEN it reads from the session's `ActuatorBus` (via `with_session_mut`)
- AND returns the same `Vec<ActuatorOutput>` as the legacy thread-local would have

### REQ-A2-3: Parity tests
A test module SHALL verify that submitting actuator outputs through the session produces the same observable behavior as the legacy thread-local bus.

#### Scenario S-A2-3.1: parity at submit+drain
- GIVEN the test sets up a session with an empty `ActuatorBus`
- WHEN it submits N outputs and drains
- THEN it gets back exactly N outputs in the same order

#### Scenario S-A2-3.2: parity under concurrent submit
- GIVEN a session shared across submit calls
- WHEN multiple submits happen before drain
- THEN drain returns all of them in insertion order (FIFO)
- AND no output is lost or duplicated

### REQ-A2-4: No new wasm_bindgen in editor-model
The editor-model crate SHALL remain free of `wasm_bindgen` imports.

#### Scenario S-A2-4.1: archcheck wasm32 unchanged
- GIVEN `cargo check -p editor-model --target wasm32-unknown-unknown --locked`
- WHEN run before and after the changes
- THEN exit code is 0 in both cases
- AND no new dependencies appear in `crates/editor-model/Cargo.toml`

## Edge Cases

### EC-A2-1: Session not yet initialized
- GIVEN a test or call path that invokes `submit_actuator_output` before any session exists
- THEN the call SHALL fail-soft (return without panicking)
- AND a panic message SHALL indicate "session not initialized"

### EC-A2-2: Empty drain
- GIVEN `drain_actuator_outputs` called on a session with no pending outputs
- THEN it returns `Vec::new()`
- AND the actuator bus is left in its empty state (no spurious re-init)

## Constraints

- ADR-0030 (bevy-free editor-model): preserved — no bevy dep added to editor-model.
- ADR-0057 (single WASM composition root): preserved — no new ambient cells.
- ADR-0059 (single transaction dispatch): preserved.

## Out of Scope (deferred)

- Block B: typing `serde_json::Value` preview fields with `editor_model::runtime::PortValue`.
- Block C: collapse `InputState` into a Bevy `Resource`.
- Block D: update `globals-inventory.yaml` + `state-ownership-matrix.md` § H2.5 + parity tests at the inventory level.

## Acceptance Gate

The cycle's `implementation-complete` gate SHALL pass when:
- All REQ-A2-1..REQ-A2-4 scenarios are demonstrable via `cargo test`.
- Code review of `actuator_bus.rs` confirms no `thread_local!` cell remains.
- `cargo check -p editor-model --target wasm32-unknown-unknown --locked` passes.
- `cargo check --workspace --locked` passes.

```yaml
status: specified
requirements: 4
scenarios: 5
edge_cases: 2
wus_planned: 4
block: A2
path: A-lite
previous_cycle: h2-5-runtime-coordination (v0.108.2)
next_cycle_hint: h2-5-runtime-coordination-block-b (after A2 lands)
```
