# Explore Report: `h2-5-runtime-coordination-block-a2`

> **Cycle:** `p-28fce7028ac3c497/h2-5-runtime-coordination-block-a2`
> **Phase:** explore · **Path:** A-lite
> **Date:** 2026-09-08 · **Explored by:** orchestrator (inline)
> **Context quality:** C3 (carry-over from cycle h2-5-runtime-coordination with full inventory)

---

## Context Carry-Over

The previous cycle `h2-5-runtime-coordination` (closed at `v0.108.2`) landed Block A:
- `editor_model::runtime::{LinearBus, PortValue, ActuatorBus, HotReloadRequest, PlayModeRequest}`
- 5 `EditorSessionPort` methods
- WASM trampolines routed via `with_session_mut`

Block A2 was deferred because:

> "1. Pre-step: rename `logic_evaluator::PortValue` → `LogicPortValue` to avoid double-identity mismatch when re-introducing `editor_model::runtime::PortValue`."

This explore phase verifies that diagnosis and determines the actual scope.

---

## Inventory of `logic_evaluator::PortValue` references

```
crates/editor-bevy/src/logic_evaluator.rs:31       pub enum PortValue { Bool, Float, Vec2, EntityRef, Action }
crates/editor-bevy/src/logic_evaluator.rs:50       pub enum PortValueType { ... }
crates/editor-bevy/src/logic_evaluator.rs:143      fn evaluate(&self, node: &LogicNode, inputs: &[PortValue]) -> Vec<PortValue>;
crates/editor-bevy/src/logic_evaluator.rs:351      let mut port_values: HashMap<(NodeId, PortId), PortValue> = HashMap::new();
crates/editor-bevy/src/logic_evaluator.rs:386, 394, 412, 414
crates/editor-bevy/src/logic_evaluator.rs:464-468  PortValue::Vec2 { ... }
crates/editor-bevy/src/logic_evaluator.rs:492      PortValue::Action(s) => ...
crates/editor-bevy/src/logic_validation.rs:218-224 fn port_type_name(t: &PortValueType) -> &'static str
crates/editor-bevy/src/actuator_bus.rs:15          use crate::logic_evaluator::PortValue;
crates/editor-bevy/src/actuator_bus.rs:27         pub value: PortValue,
crates/editor-bevy/src/actuator_bus.rs:67         pub fn submit_actuator_output(entity: Entity, field: &str, value: PortValue)
crates/editor-bevy/src/actuator_bus.rs:202-228    unit tests using PortValue
crates/editor-bevy/src/sensor_event.rs            use crate::logic_evaluator::PortValue;
crates/editor-bevy/src/logic_dispatch.rs          references PortValue in dispatch path
```

## Key Observation: the two PortValue are **structurally identical**

```rust
// crates/editor-bevy/src/logic_evaluator.rs:30-37
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortValue {
    Bool(bool),
    Float(f32),
    Vec2 { x: f32, y: f32 },
    EntityRef(String),
    Action(String),
}

// crates/editor-model/src/runtime/port_value.rs:9-25
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortValue {
    Bool(bool),
    Float(f32),
    Vec2 { x: f32, y: f32 },
    EntityRef(String),
    Action(String),
}
```

**They are LITERALLY identical**. This is not a coincidence — the new one in `editor-model` was copy-pasted as part of Block A to keep the public API surface bevy-free.

## Diagnosis update

The "double-identity" concern was real at the time Block A was authored. But the **actual** current state is:

- `editor_model::runtime::PortValue` exists, is **public**, and is **identical** to `logic_evaluator::PortValue`.
- `editor_model::runtime::PortValue` is re-exported at `editor_model` root (`crates/editor-model/src/lib.rs:144`).
- `editor_bevy::logic_evaluator::PortValue` is still the local definition in `crates/editor-bevy/src/logic_evaluator.rs:31`.

The diagnosis in the previous implementation-receipt was correct in spirit (these are two different types by identity) but **misleading in practice** because the user could simply replace `crate::logic_evaluator::PortValue` with `editor_model::runtime::PortValue` (or alias it) at every call site, and they'd be identical after the swap.

## Approaches considered

### Approach A: Full rename `logic_evaluator::PortValue` → `editor_model::PortValue` (or alias `LogicPortValue`)

- Replace `use crate::logic_evaluator::PortValue` with `use editor_model::runtime::PortValue` at 4 sites.
- **DELETE** the local enum definition in `logic_evaluator.rs`.
- Pros: Clean. The local enum is gone; one canonical name; no double identity.
- Cons: 200+ usages across `logic_evaluator.rs` itself need rewriting.

### Approach B: Type alias only

```rust
// In editor_bevy/src/logic_evaluator.rs:
pub use editor_model::runtime::PortValue;
pub type LogicPortValue = editor_model::runtime::PortValue; // back-compat shim
```

- Pros: Minimal call-site change. Existing `logic_evaluator::PortValue` continues to work.
- Cons: Keeps the path `logic_evaluator::PortValue` working, which means double-name remains in some documentation.

### Approach C: Approach A + thread_local migration in one cycle

- Same as A plus thread_local `ACTUATOR_OUTPUT_BUS` removal (Block A2's main work).

### **Chosen: Approach C**

The original Block A2 plan was approach C. The current explore confirms it is still the right choice:

1. Single code path for `PortValue` — no two identical enums.
2. `editor_bevy::logic_evaluator::PortValue` becomes a re-export of `editor_model::runtime::PortValue`.
3. `actuator_bus.rs` is rewritten to use the session-owned `editor_model::runtime::ActuatorBus`.
4. Thread_local `ACTUATOR_OUTPUT_BUS` is deleted.
5. Parity tests added demonstrating the session-backed bus behaves identically to the legacy thread-local.

## Scope

| Item | Type | WU count |
|------|------|----------|
| PortValue rename to re-export | refactor | 1 |
| Migrate `actuator_bus.rs` to session | refactor | 2 |
| Update `actuator_bus.rs` tests | test-fix | 3 |
| Add parity test (legacy vs session) | new-test | 4 |
| Update globals-inventory.yaml (cannot do here, blocked by Block D) | n/a | — |

## Constraints

- ADR-0030 (bevy-free editor-model): preserved. editor-model still has no bevy deps.
- ADR-0057 (single WASM composition root): preserved. No new ambient cells.
- ADR-0059 (single transaction dispatch): preserved. dispatch is unchanged.

## Out of Scope

- Block B (preview typed fields): cannot land until Block A2 + PortValue rename done.
- Block C (InputState Resource): independent of A2 but kept for next cycle.
- Block D (inventory + matrix + parity tests for retired cells): depends on A2 + B + C.

## Risks

- **R1**: `logic_evaluator.rs` has 2327 lines. The re-export line must be inserted carefully.
- **R2**: Some external crates may import `editor_bevy::logic_evaluator::PortValue`. Quick scan shows only `crates/editor-bevy/src/*.rs` references it; safe to migrate.

## Exit Criterion

Explore complete when:
- [x] Inventory of PortValue references enumerated.
- [x] Diagnosis confirmed: rename via re-export is feasible.
- [x] Approach C chosen with rationale.
- [x] Scope bounded to 4 WUs (all within Block A2).
- [x] Out-of-scope items registered for future cycles.
