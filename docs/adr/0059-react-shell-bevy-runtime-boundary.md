# ADR-0059 — React Owns Dense Editor UI; Bevy Owns Runtime, Viewport and Simulation

**Status:** Proposed  
**Date:** 2026-08-19

## Context

The application contains dense forms, docking, code editing, accessibility, graph canvases and browser-native workflows. Reimplementing all of these in Bevy UI would consume large effort with limited product advantage.

## Decision

Keep React/TypeScript as the main desktop/web shell. Keep Bevy responsible for editor ECS, preview ECS, 2D viewport, picking/gizmos, simulation, runtime graph execution and probes. UI state references semantic subjects/capabilities, not runtime Bevy IDs.

## Considered Options

1. Rewrite the complete editor UI in Bevy UI.
2. Move preview/runtime simulation into TypeScript.
3. Embed many independent Bevy canvases.

## Consequences

- Uses each ecosystem where strongest.
- Preserves accessibility/tooling.
- Requires disciplined typed bridge.
- Some state projection remains necessary.

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
