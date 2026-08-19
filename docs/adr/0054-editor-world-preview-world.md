# ADR-0054 — Separate EditorWorld and PreviewWorld Runtime Responsibilities

**Status:** Proposed  
**Date:** 2026-08-19

## Context

Editor state and game-preview state have different lifecycles, mutation rules and rebuild semantics. Mixing both into one undifferentiated world makes authoring state, runtime state and tool state difficult to reason about.

## Decision

Maintain two runtime responsibilities: **EditorWorld** for selection, dirty state, indexes, graph runtime, validation and diagnostics; **PreviewWorld** for projected game entities/components, gameplay, rendering, animation and Logic runtime. Both may use Bevy ECS. Stable semantic IDs map to ephemeral Bevy entity IDs through explicit indexes. Initially prefer explicit World+Schedule plus preview App; SubApp is optional and evidence-driven.

## Considered Options

1. Single Bevy World for editor and preview.
2. No EditorWorld; keep all editor state in React/application objects.
3. Mandatory Bevy SubApp from day one.

## Consequences

- Cleaner lifecycles and tests.
- Headless editor runtime becomes possible.
- State exists in semantic/editor/preview representations.
- Projection synchronization becomes first-class.

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
