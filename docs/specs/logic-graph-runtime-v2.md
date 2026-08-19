# SPEC-LOGIC-002 — Logic Graph Runtime v2

**Status:** Proposed  
**ADR:** 0057

## Problem

Authoring-friendly graphs use stable IDs and metadata. Runtime should not repeatedly resolve those structures.

## Compiler

Input: LogicGraphAsset revision N + node descriptors + evaluator registry.

Compiler phases:
1. schema validation;
2. resolve node type/evaluator;
3. resolve PortIds;
4. type-check connections;
5. reject dangling ports;
6. compute topological/phase order;
7. allocate node/port slots;
8. build forward/reverse adjacency;
9. initialize caches/dirty masks;
10. emit diagnostics.

## Evaluator contract

Prefer:

```rust
fn evaluate(
    &self,
    ctx: &EvaluationContext,
    inputs: &NodeInputs,
) -> Result<NodeOutputs, EvaluationError>;
```

over positional `Vec<PortValue>` at semantic boundaries. Compile PortId -> PortSlot once.

## Execution phases

```text
Sensors -> Controllers -> Actuators
```

These can map to Bevy SystemSets. Nodes do not become one Bevy System each.

## Dirty evaluation

```text
dirty source -> evaluate -> compare cached outputs
 -> changed output marks fan-out dirty -> continue in compiled order
```

Actuator effects are queued/flushed after pure evaluation.

## Per-node runtime cache

```text
input fingerprint/version
cached outputs
last result
last activation id
duration sample
```

## Activation trace

```text
ActivationId
GraphId
GraphRevision
Trigger
NodesVisited
OutputsChanged
ActuatorsQueued
Errors
Duration
```

## Errors

Compile-time: missing node/port/type mismatch/illegal cycle/malformed parameter.  
Runtime: evaluator failure/missing entity/actuator failure/stale projection.

## Compatibility

Keep existing semantic LogicGraph readable where possible. Runtime compiler may evolve without project migration.

## UAT

- valid graph executes;
- type mismatch blocked;
- graph edit recompiles once per revision;
- sensor -> controller -> actuator trace visible;
- parameter update propagates;
- broken reference appears in Problems/Trace;
- save/reload preserves graph.
