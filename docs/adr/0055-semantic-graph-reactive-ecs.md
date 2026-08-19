# ADR-0055 — Semantic Graphs Compile to Reactive Bevy ECS Runtime Projections

**Status:** Accepted  
**Date:** 2026-08-19

## Context

The editor needs graphs for logic, dependencies, inheritance, causality, world topology and rule systems. Persisting every graph as raw ECS or building a second custom reactive runtime would either weaken authoring guarantees or duplicate Bevy.

## Decision

Introduce a Bevy-free graph kernel for stable graph types/algorithms and compile/materialize selected graph dialects into Bevy ECS runtime state. Graphs model relations/dependency propagation; ECS models active state/execution. Runtime representations may use dense slots, adjacency arrays and bitsets rather than one ECS entity per edge.

## Considered Options

1. Use Bevy ECS hierarchy as universal graph representation.
2. Use a graph database as authoritative project store.
3. Implement every graph separately.
4. Keep graph evaluation in ad-hoc hash maps.

## Consequences

- Reusable graph infrastructure.
- Incremental evaluation/invalidation.
- Multiple visual lenses over the same graph.
- Requires compiler/projection layer and revisions.

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
