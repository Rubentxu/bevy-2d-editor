# ADR-0061 — UAT Uses a Shared Scenario DSL and Semantic/ECS Probes

**Status:** Proposed  
**Date:** 2026-08-19

## Context

The existing E2E suite is valuable but browser automation alone does not express user acceptance goals, and DOM assertions are insufficient for many Bevy runtime behaviours.

## Decision

Introduce a versioned UAT scenario schema with persona, goal, fixture, actions, expectations and evidence policy. Support guided-human, Playwright and headless execution where applicable. Add a dev/test-only `UatProbePlugin` exposing read-only semantic/runtime queries. The probe cannot mutate outside normal editor commands.

## Considered Options

1. Treat existing Playwright tests as UAT.
2. Create a separate manual spreadsheet/test-plan process.
3. Expose unrestricted test mutation APIs.

## Consequences

- Reusable acceptance definitions.
- Semantic evidence reduces UI-only flakiness.
- Improves defect reproduction.
- Requires schema tooling and fixture discipline.

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
