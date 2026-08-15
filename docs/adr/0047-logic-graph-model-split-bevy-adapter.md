# ADR-0047: Logic Graph Model Split — Pure Types in editor-model, Bevy Adapter in editor-core

## Status

Accepted — 2026-08-15 — v0.87 (`v0.87-architecture-foundation`)

## Context

The v0.87 Architecture Foundation plan (PR2, `editor-model` crate) requires the crate to be free of `bevy` and `wasm_bindgen` dependencies per ADR-0030 (compile-time hexagonal boundaries) and ADR-0046 (semantic editor model authority). The exploration phase identified `crates/editor-core/src/logic_graph.rs` as one of nine "pure enough" modules ready to move without modification.

However, inspection during the design phase found that `LogicBinding` (logic_graph.rs:155) carries `#[derive(bevy::prelude::Component)]` because it is the bridge type between editor-owned logic graphs and the Bevy preview ECS world. Stripping the derive is not safe:

- `LogicBinding` is queried by preview systems to resolve which `LogicGraphAsset` each preview entity executes.
- The derive participates in Bevy's ECS reflection and `Component` trait object machinery; removing it breaks the preview integration that ships in v0.86.1.

The original spec text said "strip the `bevy::prelude::Component` derive" — this assumed the derive was decorative, but the code reality is that it is load-bearing for the preview.

## Decision

Split `logic_graph.rs` along the Bevy/pure seam:

- Move eight pure types to `editor_model::logic_graph`:
  - `LogicGraphAsset`
  - `LogicNode`
  - `LogicEdge`
  - `LogicNodeRole` (sensor / controller / actuator)
  - `NodeId`
  - `PortId`
  - `NodeTypeId`
  - `LogicInstance`
- Keep `LogicBinding` in `editor-core` as a thin Bevy adapter. Rename the file to `bevy_logic_binding.rs` so the Bevy-coupling signal is explicit at the file level.
- Re-export the pure types from `editor-core::logic_graph` so legacy import paths (`editor_core::logic_graph::LogicGraphAsset`) keep compiling during the PR2 transition window.

`LogicBinding`'s implementation stays unchanged: it continues to hold an `InstanceId` and reference the asset via `editor_model::logic_graph::LogicGraphAsset`. The `#[derive(Component)]` is preserved on the adapter file.

## Considered options

### Strip the `Component` derive
Rejected: breaks preview integration; the derive is not decorative.

### Leave `logic_graph.rs` whole in editor-core
Rejected: gives up the purity goal for PR2. Other modules move but logic graphs remain Bevy-coupled at the crate boundary, which defeats the testability and dependency-direction objectives of ADR-0030.

### Split into pure types + adapter
Accepted: keeps editor-model Bevy-free, isolates Bevy coupling to one named file, preserves preview behavior unchanged.

## Consequences

- `editor-model` remains free of `bevy`/`wasm_bindgen`/`web_sys`/`js_sys` dependencies after PR2.
- The Bevy coupling in `editor-core` shrinks to the named adapter (`bevy_logic_binding.rs`) plus the existing `SceneEntity`/`SceneDocumentState`/`OperationLogState`/`PlayMode`/`TransformSnapshot` types that PR2 leaves in `editor-core` for v0.88+.
- Preview integration tests must continue to pass without modification.
- Future Bevy-free refactors can move `LogicBinding` to `editor-bevy` once that crate exists (per ADR-0030).
- This is a deviation from the original PR2 spec text ("strip the Component derive"). The deviation is recorded here to keep the rationale visible; no separate ADR-0047 amendment is needed because the deviated behavior is the new decision.

## References

- ADR-0030 — Compile-Time Hexagonal Crate Boundaries
- ADR-0046 — Semantic Editor Model Is the Authoritative Source of Truth
- v0.87 cycle spec — `docs/roadmaps/v0.87-architecture-foundation.md`
- v0.87 explore report — `cycles/v0.87-architecture-foundation/explore-report.md`
- v0.87 design — `cycles/v0.87-architecture-foundation/design.md` (decision D1, risk R1)