# ADR-0053 — Bevy ECS Is the Runtime Substrate of the Editor

**Status:** Accepted  
**Date:** 2026-08-19

## Context

The editor already uses Bevy for 2D preview, while the architectural programme has extracted a semantic model and application layer. The product requirement is stronger: Bevy must remain a genuine foundational technology of the editor, including ECS and WASM, without turning runtime state into the persisted authoring model.

## Decision

Adopt **Bevy-native runtime, semantic-first authoring**. `editor-model` remains the authoritative Bevy-free semantic model. A Bevy-based editor runtime projects semantic state into ECS for efficient execution, change detection, scheduling, runtime caches, graph projections and diagnostics. Bevy also remains the 2D preview/runtime engine.

## Considered Options

1. Make Bevy World the project source of truth.
2. Keep Bevy only as a preview renderer.
3. Build a custom editor ECS/reactive runtime independent of Bevy.

## Consequences

- Real Bevy dogfooding and editor/game parity.
- Semantic persistence is insulated from Bevy API churn.
- Requires explicit projection/mapping layers.
- Architecture fitness must prevent Bevy types leaking inward.

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
