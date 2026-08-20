# ADR-0011: Logic Bricks — Compiled Rust Controllers and Dispatch Scheduler

## Status

Accepted (2026-07-01)

## Context

The Bevy 2D Editor has authored static scenes (Scene Assets, Scene Instances,
Level Layers, BSN export) but **no behavior layer**. Users need a visual way to
wire common 2D gameplay — jump, collision response, health/damage, timers,
proximity triggers — without leaving the editor and without hand-writing Rust.

The deferral gate (ROADMAP §Deferred: "Visual scripting/state machines — revisit
when Scene Asset workflows and runtime preview inspection are mature") has
**passed**: Hito 2 Orders 1–9 and Hito 3 Order 4 (`bsn-file-import` v0.36.0)
are complete. The editor substrate is mature enough to introduce logic.

Four architectural questions must be decided before any implementation:

1. **What kind of scripting?** Blueprint-style dynamic VM vs compiled Rust
   controllers vs codegen-only.
2. **How does behavior reach the runtime?** Dynamic evaluation vs generated
   Rust source vs trait-dispatch registry.
3. **What does the preview evaluator look like?** Per-frame loop vs
   event/change-driven scheduler.
4. **How does logic relate to BSN?** Does a logic graph project to `.bsn`, or
   is it editor-owned behavior that BSN never sees?

Established invariants that constrain the answer:

- Editor-owned JSON is the source of truth (ADR-0001); Bevy is a consumer.
- Scene Asset / Scene Instance symmetry with separate command surfaces
  (ADR-0005, ADR-0007) is the proven extension model.
- BSN is a scene-composition format with a lossy one-way IR (`BsnIr`); the
  `BsnExporter` trait is output-only (ADR-0010).
- `spawn_preview_entity` (`lib.rs:1666`) has a silent `_ => {}` skip-arm that
  drops any component type it does not recognize — logic bindings would be
  invisible unless explicitly handled.
- `rebuild_preview_world` (`lib.rs:1347`) is a full despawn/respawn on dirty —
  any per-frame logic state (timers, flip-flops) resets on every rebuild.
- The user has a strong "no scripting VM" stance (Engram #3720: compiled-Rust
  direction).

## Decision

We adopt **Logic Bricks** as the visual behavior authoring system, built on
five binding sub-decisions:

### D1 — Logic Bricks over Blueprint-style scripting

We use a **node-based Logic Bricks** model (Sensor → Controller → Actuator),
not a Blueprint-style event-graph VM that executes arbitrary user-authored
logic text.

**Rationale**: Blueprint VMs (Unreal) drift into scripting; the user explicitly
rejects a VM. Logic Bricks with a curated node set keeps behavior declarative
and auditable — every node is a known built-in with typed ports.

### D2 — Compiled RustController trait registry over dynamic scripting

Extension beyond built-in bricks is satisfied by a **`RustController` node
kind** backed by a compiled trait registry:

```rust
pub trait NodeEvaluator: Send + Sync {
    fn evaluate(&self, node: &LogicNode, inputs: &[PortValue]) -> Vec<PortValue>;
}
```

A `RustController` node references a `controller_id` resolved at runtime to a
compiled `NodeEvaluator` impl. This is the Unity/C#-like compiled extension
point — Rust-native, trait-dispatched, no scripting, no user-authored Rust
snippets inside graph nodes.

**v1 scope**: built-in controllers compiled into the editor binary only.
External/WASM plugin controllers are deferred (blocked on plugin system,
ROADMAP §Deferred).

### D3 — Dispatch scheduler first, optional codegen later

The preview runtime is an **event/change-driven dispatch scheduler** in
`editor-core`, not a codegen pipeline. Sensors emit events; controllers and
actuators run **only when their inputs changed**. The Bevy system may run a cheap
gate in `Update`, but it must not evaluate every graph every frame.

Graph → Rust source codegen (the exploration's Approach 3) is **deferred** to a
future change. The dispatch scheduler provides live preview, which codegen
cannot.

**Rationale**: Live preview is essential for a visual editor; codegen blocks
iteration on a compile cycle and provides no runtime feedback. The scheduler is
framed as **dispatch over compiled bricks, not a VM** — it executes no user
text.

### D4 — React Flow view-only; WASM JSON is source of truth

The graph editor uses React Flow (`@xyflow/react`) as the **view layer only**.
The domain graph lives as editor-owned JSON in WASM (`LogicGraphAsset`). React
mirrors it via `useNodesState`/`useEdgesState` and dispatches typed
`LogicCommand`s through the existing WASM bridge — mirroring how
`HierarchyPanel`/`InspectorPanel` already operate. React state is never the
source of truth.

### D5 — BSN isolation

Logic graphs **do not project to `.bsn`**. BSN is a scene-composition format;
logic is editor-owned behavior. Direct `BsnExporter` calls reject assets with
`SceneAssetRole::Logic`; future bulk export may skip them only with an explicit
warning. `BsnIr` stays scene-only.

### Preview state model (v1)

Preview logic is **stateless across rebuilds**. `rebuild_preview_world` is a
full despawn/respawn; stateful logic (persistent timers, flip-flops surviving
rebuild) is deferred. This is the simplest correct model and avoids survival
logic across the rebuild boundary.

## Considered Options

### Option A — Blueprint-style event-graph VM

Rejected. A dynamic VM that executes user-authored logic drifts toward
scripting, violating the no-VM invariant. Unreal Blueprints demonstrate the
danger of implicit default propagation and silent state resets. The user
explicitly rejects this direction.

### Option B — Pure codegen (graph → Rust source, no evaluator)

Rejected for v1. `code_export.rs` and `bsn_codegen.rs` are precedents for
pure-string codegen, but a codegen-only path provides **no live preview** of
logic — rebuild-on-dirty cannot simulate generated Rust, and iteration is
blocked on a compile cycle. Codegen remains a candidate for a future
optimization/export change.

### Option C — Inline `editor.LogicGraph` component on entities

Rejected. Embedding a graph JSON blob as a `ComponentInstance` reuses the
schema registry but loses reuse across entities, has no override/resync story,
bloats component values, and conflicts with the established asset/instance
model (ADR-0005). No natural authoring-mode seam.

### Option D — Logic Bricks + compiled RustController + dispatch scheduler (chosen)

Chosen. `LogicGraphAsset` (reuse the Scene Asset identity/role and typed-edge principles) +
`LogicInstance` binding, event-driven dispatch scheduler in editor-core,
compiled `RustController` trait-dispatch node, React Flow view-only, BSN
isolated. Honors every established invariant while providing live preview and a
Rust-native extension seam.

## Consequences

### Positive

- **No scripting VM**: the dispatch scheduler executes compiled bricks and
  built-in evaluators only. No user-authored evaluator nodes, no user text
  execution. The no-VM invariant is preserved.
- **Live preview**: the dispatch scheduler gives immediate feedback during
  authoring, which codegen cannot.
- **Compiled-Rust extension**: `RustController` trait dispatch satisfies the
  Unity/C#-like compiled-controller direction without a codegen pipeline.
- **Architecture reuse**: `LogicGraphAsset`/`LogicInstance` mirror
  `SceneAssetDocument`/`SceneInstance`; undo/redo, validation, and the override
  story come for free if logic overrides are added later.
- **BSN insulation**: logic never enters `BsnIr`; the `BsnExporter` boundary
  rejects direct logic-role exports. Bevy BSN PRs (#23639/#23648) do not affect
  this layer.
- **OCP-compliant node extensibility**: adding node kinds is trait registration,
  not central `match` modification.

### Negative

- **New asset kind + binding + projection + evaluation**: larger initial
  surface than an inline component. Must be split across multiple
  implementation changes if >400 LOC.
- **`spawn_preview_entity` skip-arm** (`lib.rs:1666`) is a hard blocker: logic
  bindings attached to projected entities are currently invisible to the Bevy
  world. Implementation must explicitly handle logic projection.

  **Status (2026-08-20)**: RESOLVED at `preview_runtime.rs:979`. The `spawn_preview_entity` skip-arm now correctly handles `editor.LogicBinding` — the projection is inserted into the preview entity alongside the existing component set. No logic bindings are silently dropped.
- **Stateless preview** (v1) means timers and flip-flops reset on every dirty
  rebuild — acceptable for preview, but limits testing of time-based behaviors.
- **Third parallel command surface** (`LogicCommand` alongside `Command` and
  `AssetCommand`) adds maintenance burden. Justified by ADR-0007's identity-split
  precedent (`NodeId` vs `LocalId` vs `StableId`).
- **React Flow is a new frontend dependency** (`@xyflow/react`). Controlled but
  requires enforcing view-only discipline.

## v1 Assumptions

- `RustController` nodes resolve only to controllers compiled into the editor
  binary. No external plugin ABI.
- Logic bindings are non-overridable (no Component Override story for logic in
  v1).
- Preview logic is stateless across rebuilds.
- No user-authored Rust snippets inside graph nodes.
- The dispatch scheduler is gated on event/change presence — it may run a cheap
  `Update` gate, but never evaluates every graph every frame.

## References

- [ADR-0001](./0001-scene-document-json-as-source-of-truth.md) — JSON as source of truth.
- [ADR-0005](./0005-scene-asset-bsn-aligned-reusable-scene-model.md) — Scene Asset / Scene Instance symmetry, roles, relationships.
- [ADR-0007](./0007-separate-asset-command-surface.md) — Separate command surface precedent.
- [ADR-0010](./0010-bsn-exporter-trait-file-export.md) — BsnExporter trait, output-only BSN.
- `sddk/logic-bricks-graph-editor/explore-report.md` — exploration with entropy analysis.
- `sddk/logic-bricks-graph-editor/proposal.md` — proposal (Approach 2+4 fused).
- `sddk/logic-bricks-graph-editor/design.md` — technical design (D1–D10).
- Engram #3720 — compiled-Rust direction (user lean).
- `crates/editor-core/src/scene_asset.rs` — `SceneAssetDocument`, `SceneAssetRole`, `RelationshipKind::Custom`.
- `crates/editor-core/src/asset_command.rs` — parallel command surface pattern.
- `crates/editor-core/src/lib.rs:1666` — `spawn_preview_entity` skip-arm (hard blocker).
