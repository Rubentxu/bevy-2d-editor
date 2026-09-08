# Design: `h2-5-runtime-coordination-block-a2`

> **Cycle:** `p-28fce7028ac3c497/h2-5-runtime-coordination-block-a2`
> **Phase:** design · **Path:** A-lite
> **Date:** 2026-09-08

---

## Decision Log

### D1: Replace local PortValue with re-export (Approach C from explore)

**Decision:** In `crates/editor-bevy/src/logic_evaluator.rs`, replace the local `pub enum PortValue { ... }` (lines 30-37) with `pub use editor_model::runtime::PortValue;`.

**Rationale:** Both enums are byte-for-byte identical. Keeping two definitions invites drift. A re-export preserves the existing path `crate::logic_evaluator::PortValue` for all 200+ references while making `editor_model::runtime::PortValue` the single source of truth.

**Alternatives considered:**
- Type alias `pub type LogicPortValue = ...;` — keeps double-name in API surface.
- Full rename at all call sites — 200+ references to rewrite, no functional benefit.

### D2: ActuatorBus via with_session_mut, drop thread_local

**Decision:** Rewrite `crates/editor-bevy/src/actuator_bus.rs` to use `editor_model::ports::with_session_mut(|s| s.runtime_actuator_outputs_mut())` for both submit and drain. Delete the `thread_local! { static ACTUATOR_OUTPUT_BUS }` declaration.

**Rationale:** This is the literal H2.5 state-ownership goal: retire ambient thread-locals, route through session.

**Sub-decisions:**
- `submit_actuator_output` becomes a thin wrapper: it gets a `&mut ActuatorBus` from the session and pushes. Returns nothing.
- `drain_actuator_outputs` returns `Vec<ActuatorOutput>` from the session's bus.
- Where session is not initialized: graceful no-op + `tracing::warn!` (NOT a panic — actuator evaluators may run during graph evaluation outside a frame).

### D3: Parity tests at `editor-bevy/tests/actuator_parity.rs`

**Decision:** New test file `crates/editor-bevy/tests/actuator_parity.rs` that exercises the session-backed bus through the public API and asserts FIFO ordering, drain semantics, and cross-frame persistence.

**Rationale:** Block D will need parity tests at the inventory level; this is the per-feature parity test that proves the actuator bus works after migration. Same shape as `asset_state.rs` parity tests in editor-bevy.

### D4: PortValueType stays in editor-bevy (no change)

**Decision:** `editor_bevy::logic_evaluator::PortValueType` stays as a local enum. It is **NOT** part of the editor-model public API surface.

**Rationale:** `PortValueType` is metadata describing node ports, not the value boundary itself. It belongs with the logic graph evaluator in editor-bevy.

## ADRs Referenced

| ADR | Status | Evidence |
|-----|--------|----------|
| ADR-0030 (bevy-free editor-model) | PRESERVED | editor-model still has no bevy deps |
| ADR-0057 (single WASM composition root) | PRESERVED | session is the only mutable state |
| ADR-0059 (single transaction dispatch) | PRESERVED | `evaluate_logic_binding` path unchanged |

## Archcheck

| Check | Status | Reason |
|-------|--------|--------|
| `bevy-free editor-model` | PASS | editor-model gains nothing new |
| `thread_locals only in legacy paths` | PASS — IMPROVED | ACTUATOR_OUTPUT_BUS retired |
| `archcheck baseline` | UNCHANGED | pre-existing failures on asset_operation_log.rs |

## Implementation Order (tasks → WUs)

| WU | Description | Depends on |
|----|-------------|------------|
| WU-A2-1 | Replace local PortValue with re-export | — |
| WU-A2-2 | Migrate actuator_bus.rs to session, drop thread_local | WU-A2-1 |
| WU-A2-3 | Update existing actuator_bus.rs tests | WU-A2-2 |
| WU-A2-4 | Add `tests/actuator_parity.rs` parity tests | WU-A2-3 |

## Files Changed

| File | Change |
|------|--------|
| `crates/editor-bevy/src/logic_evaluator.rs` | Replace enum with re-export (1 line) |
| `crates/editor-bevy/src/actuator_bus.rs` | Rewrite submit/drain (~50 LOC change) |
| `crates/editor-bevy/tests/actuator_parity.rs` | New file (~80 LOC) |

## Risks

- **R1**: Bevy system `apply_actuator_outputs` may run outside `with_session_mut` context. Test that it works in both contexts.
- **R2**: Multiple Bevy systems on different threads may try to access the session. Session is already designed to be thread-local in editor-model::ports — confirmed compatible.

## Exit Criterion

Design complete when:
- [x] Decision log finalized.
- [x] ADRs checked.
- [x] Archcheck impact assessed.
- [x] WU dependency graph specified.
- [x] Files to change enumerated.

```yaml
status: designed
decisions: 4
adrs_preserved: 3
archcheck: pre_existing_failures_unchanged
files_changed: 3
wus: 4
path: A-lite
```
