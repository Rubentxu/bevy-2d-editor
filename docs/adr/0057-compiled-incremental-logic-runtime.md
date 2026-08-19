# ADR-0057 — Logic Graphs Use a Compiled Incremental Runtime

**Status:** Proposed  
**Date:** 2026-08-19

## Context

The current Logic evaluator performs work that belongs to compilation/resolution during activation. As graphs grow, repeated ordering/map/port work reduces performance and correctness.

## Decision

Compile each LogicGraph revision into node/port slots, validated typed connections, topological/phase order, forward/reverse adjacency, evaluator references, dirty masks and cached outputs. Evaluate only affected nodes. Separate pure node evaluation from actuator/effect execution. Authoring uses semantic PortIds; runtime compiles them to slots.

## Considered Options

1. Evaluate the authoring graph from scratch on every activation.
2. Generate a Bevy System per node.
3. Introduce a general visual scripting VM.

## Consequences

- Predictable performance.
- Typed connection errors caught before runtime.
- Better traces.
- Compiler complexity and cache invalidation by revision.

## Architecture Guardrails

- preserve stable semantic identity;
- preserve Transaction Kernel ownership of authoring mutations;
- keep generated/derived runtime state rebuildable;
- add architecture fitness checks before relying on convention;
- migration must be incremental and covered by UAT.

## References

- ADR-0030 — Compile-Time Hexagonal Crate Boundaries
- ADR-0032 — Shared Transaction Kernel and ChangeSet
- ADR-0034 — Typed EditorBackend Contract
- ADR-0036 — Runtime Preview Adapter
- ADR-0046 — Semantic Editor Model Authority
- ADR-0047 — Logic Graph Model Split
