# ADR-0063 — Bevy Schedule and Runtime Systems Are Inspectable Product Data

**Status:** Proposed  
**Date:** 2026-08-19

## Context

Because the editor itself runs on Bevy, schedules and runtime work can be observable. Making this visible supports dogfooding, performance diagnosis and causal explanation.

## Decision

Add a diagnostic representation of editor/preview schedules and systems with stable system IDs, phase/set, read/write metadata where feasible, timings, trigger/cause correlation and recent execution samples. Expose through Debug workspace Systems/Trace lenses. Aggregate rather than leaking Bevy internals verbatim.

## Considered Options

1. Keep scheduler internals opaque.
2. Expose raw scheduler structures to frontend.
3. Instrument every system at maximum detail in production.

## Consequences

- Powerful self-inspection.
- Performance budgets become attributable.
- Instrumentation overhead must be controlled.
- Stable abstraction needed across Bevy upgrades.

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
