# ADR-0060 — Unified Causality Model Powers Impact, Why and Trace

**Status:** Proposed  
**Date:** 2026-08-19

## Context

The project already tracks rebuild causes, logic activations and causal links. These can evolve from isolated diagnostics into a major capability explaining data provenance, change provenance and runtime execution.

## Decision

Define a unified causal vocabulary linking FrameId, ChangeId, semantic revision, graph revision, rebuild cause, logic activation and runtime projection. Expose three surfaces from the same data: **Impact** (what would be affected), **Why** (why state/object exists or differs) and **Trace** (what happened over time). Keep data bounded/observable by default.

## Considered Options

1. Keep diagnostic features independent.
2. Persist every low-level ECS operation forever.
3. Rely on console logs.

## Consequences

- Differentiating debugging UX.
- Richer UAT evidence and AI reasoning.
- Instrumentation cost must be controlled.
- Requires retention/sampling policy.

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
