# Logic Bricks Graph Editor — Durable Capability Spec

> Status: **Draft durable spec** (generated from `sddk/logic-bricks-graph-editor/spec.md`).
> Source of truth for shared review: **this file**. The SDDK spec remains phase provenance and may be local-only.
> Authoritative references: [ADR-0005](../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md) · [ADR-0007](../adr/0007-separate-asset-command-surface.md) · [ADR-0010](../adr/0010-bsn-exporter-trait-file-export.md) · [ADR-0011](../adr/0011-logic-bricks-compiled-rust-controllers.md).

This spec is the durable capability contract for the Logic Bricks Graph Editor. It describes **observable behaviour** — what the implementation must satisfy — not the implementation itself.

## Purpose

Add a visual Logic Bricks system to the Bevy 2D Editor so authors can wire common 2D gameplay (jump, collision response, health/damage, timers, proximity) without leaving the editor, while preserving the editor's "Rust-compiled, no scripting VM" posture.

## Scope

In scope for v1:
- `LogicGraphAsset` (editor-owned JSON; nodes/edges) + `LogicInstance` binding.
- A new Logic Authoring Mode (`EditorMode == "logic"`) with a React Flow panel as **view only**.
- Built-in `RustController` nodes backed by a compiled trait registry (`fn evaluate(&self, &LogicInputs) -> LogicOutputs`).
- Versioned built-in recipes for common 2D patterns.
- An event/change-driven **dispatch scheduler** in editor-core that evaluates projected graphs in the preview.
- A `LogicInstance` is **non-overridable** in v1; one per Scene Instance.

Out of scope for v1 (deferred to later changes):
- Rust controller **codegen** (graph → Rust source). Closed set of behaviours in v1: built-in bricks + compiled-in `RustController` impls.
- External/WASM plugin controllers — blocked on the plugin system.
- `.bsn` import of logic graphs.
- Override/Resync Workbench UI for logic bindings.

## Capabilities

| Capability | Direction | Summary |
|---|---|---|
| `logic-graph-authoring` | NEW | Asset/instance/binding/mode + React Flow view-only + `LogicCommand` surface |
| `compiled-logic-controllers` | NEW | `RustController` kind + compiled trait registry (built-in only) |
| `logic-bricks-2d-recipes` | NEW | Versioned built-in `LogicGraphAsset` recipes |
| `logic-graph-evaluation` | NEW | Event/change-driven dispatch scheduler in editor-core |
| `logic-graph-bsn-isolation` | NEW | `LogicGraph` MUST NOT project to `.bsn` |
| `runtime-preview-inspector` | MODIFIED | Project `LogicInstance`s; expose evaluator diagnostics |
| `validation-center` | MODIFIED | Surface graph validation issues through `get_validation_issues_wasm` |
| `bsn-export` | MODIFIED | Direct exporter calls MUST reject `Logic`-role assets |

The detailed phase spec lives in [`sddk/logic-bricks-graph-editor/spec.md`](../../sddk/logic-bricks-graph-editor/spec.md), but the requirements below are self-contained so reviewers do not need local SDDK artifacts.

## Invariants (non-negotiable)

1. **No scripting VM.** Evaluator is a dispatch scheduler over compiled bricks. The set of executable behaviours is **closed** in v1. Any change that introduces a free-form `code: String`, `eval`, `wasm_instance`, or plugin loader for `RustController` SHALL be rejected as a contract violation.
2. **React Flow is view-only.** The source of truth for any graph is editor-core/WASM JSON. After every `LogicCommand` ack, React Flow MUST re-mirror state from editor-core. Reload MUST restore from editor-core, never from React state.
3. **Logic does NOT project to `.bsn`.** `BsnIr` stays scene-only. Direct `BsnExporter` calls MUST reject `Logic`-role assets with `BsnExportError::UnsupportedShape`. Future bulk export MAY skip logic assets only with an explicit warning. Bevy BSN draft PRs (#23639, #23648) MUST NOT influence the logic surface — logic is insulated by design.
4. **Asset/instance symmetry.** `LogicGraphAsset` mirrors the identity and relationship principles of `SceneAssetDocument` (opaque IDs, typed edges, stable JSON, validation), but it exposes explicit `LogicEdge` records. `LogicInstance` is bound to one Scene Instance in v1, using a parallel `LogicCommand` surface per ADR-0007 (separate processor, separate operation log, identity = `asset_id`).
5. **v1 binding constraints.** Exactly one `LogicInstance` per Scene Instance; binding is non-overridable (`editor.LogicBinding` is not a valid `component_overrides` target); `RustController` is built-in only.

## v1 Assumptions

| Assumption | Why it matters | Revisit when |
|---|---|---|
| `RustController` is built-in only | Plugin ABI not yet defined; shipping a pluggable surface now locks bad design | Plugin system lands |
| One graph per Scene Instance | Simplifies binding model and round-trip in v1; supports the dominant 2D pattern (per-actor behaviour) | Multi-graph binding demand is observed |
| Logic bindings non-overridable | Override/Resync Workbench semantics and validation get complex fast; defer | Reuse pressure forces multi-graph or per-node overrides |
| Preview logic resets on every rebuild | Avoids coupling Bevy ECS persistence with editor ephemeral state in v1 | Preview iteration UX requires stable timers or state |
| Evaluator dispatches compiled Rust + built-in bricks only | Honors the "Rust-compiled, no VM" posture | A safer, typed extension seam is proposed |

## Detailed Requirements

### `logic-graph-authoring`

- `LogicGraphAsset` MUST be editor-owned JSON with explicit `nodes` and `edges`, stable `asset_id`, `logical_path`, `role = Logic`, and monotonic `version`.
- Each `LogicNode` MUST carry `node_id`, `role` (`Sensor | Controller | Actuator`), `node_type_id`, and persisted `field_values`.
- Each `LogicEdge` MUST connect `from_node/from_port` to `to_node/to_port` and MUST be validated by editor-core.
- A Scene Instance MUST accept at most one v1 `LogicInstance`, represented by `editor.LogicBinding`; that binding MUST NOT be a Component Override target.
- React Flow MUST be view-only: it MAY keep transient UI state, but durable graph mutations MUST go through typed `LogicCommand`s and re-mirror from editor-core after acknowledgement.

### `compiled-logic-controllers`

- `RustController` is a Controller-role node with `node_type_id = "rust-controller"` and a `controller_id` resolved through a compiled registry.
- v1 registry entries MUST be built into the editor binary. Unknown `controller_id`s MUST produce `dangling-controller-ref` validation errors and MUST NOT fire actuator outputs.
- User-authored code strings, `eval`, dynamic-loaded controllers, and WASM plugin controller instances are prohibited in v1.

### `logic-bricks-2d-recipes`

- Built-in recipes MUST be versioned immutable `LogicGraphAsset`s under `recipes/<name>` logical paths.
- Users MAY instantiate recipes through `LogicInstance`. Direct edits to built-in recipes MUST be rejected with `recipe-immutable`.
- Future "duplicate recipe as user asset" is allowed; the duplicate is no longer the built-in recipe.

### `logic-graph-evaluation`

- The preview evaluator MUST be an event/change-driven dispatch scheduler over projected `LogicInstance`s.
- The Bevy system MAY run a cheap `Update` gate, but graph nodes MUST evaluate only when relevant sensor/event/change inputs exist.
- The scheduler MUST NOT execute user-authored source code and MUST NOT evaluate every graph every frame.
- Preview logic state MUST reset across `rebuild_preview_world` in v1.
- Dispatch MUST use a `NodeEvaluator` registry keyed by `node_type_id` for built-ins and `controller_id` for `RustController` nodes, not a central growing match over all concrete node types.

### `logic-graph-bsn-isolation` and `bsn-export`

- Logic graphs MUST NOT enter `BsnIr`.
- Direct `.bsn` export of a `Logic` role asset MUST return `BsnExportError::UnsupportedShape` and produce no `.bsn` text.
- Scene roles such as `actor` and `level` MUST keep their pre-change export behavior.

### `runtime-preview-inspector` and `validation-center`

- Preview projection MUST make `LogicInstance` bindings visible to the evaluator and diagnostics; unknown component skipping MUST NOT hide logic bindings silently.
- Runtime Preview Inspector diagnostics MUST expose active graph count, last-triggered sensor id, and last-fired actuator id.
- Validation Center MUST surface `invalid-port-type`, `cycle`, `dangling-controller-ref`, and `missing-binding` with graph id and offending node ids where applicable.

## Acceptance Criteria (per capability)

The implementation must demonstrate, in order, that each capability satisfies the durable requirements above and the phase scenarios in `sddk/logic-bricks-graph-editor/spec.md`. Concretely:

1. `logic-graph-authoring` — create round-trips a graph; second binding rejected; React Flow re-initializes from editor-core after reload.
2. `compiled-logic-controllers` — built-in controller dispatches; unknown `controller_id` rejected by validator; no user-authored code path exists.
3. `logic-bricks-2d-recipes` — built-in recipes list in browser; recipe edits rejected.
4. `logic-graph-evaluation` — Sensor→Controller→Actuator fires on the right trigger; idle frame skips the graph; rebuild resets state.
5. `logic-graph-bsn-isolation` — exporter rejects Logic role; `BsnIr` stays scene-only.
6. `runtime-preview-inspector` — `LogicBinding` projects; diagnostics expose active graphs and last triggers.
7. `validation-center` — surfaces cycle, dangling-controller-ref, invalid-port-type, missing-binding.
8. `bsn-export` — Logic role rejected; scene roles export unchanged.

## Sequencing

This change is **docs only**. Implementation lands in one or more follow-up changes. Recommended order:

1. `editor-core` data model + `LogicGraphAsset` / `LogicInstance` persistence + parallel `LogicCommand` surface.
2. `BsnExporter` direct-reject for `Logic` role + `validation-center` logic issues.
3. Frontend Logic Authoring Mode + React Flow panel + `engine-bridge` WASM surface.
4. Runtime evaluator: built-in bricks + `RustController` registry + evaluation schedule + preview projection.
5. Built-in recipes (read-only) + browser integration.

Each slice must keep `cargo test` and Playwright green for prior scope. Any slice that exceeds ~400 LOC SHOULD be split.

## Glossary

- **LogicGraph** — editor-owned JSON document: `nodes` + `edges`. Source of truth for behaviour wiring.
- **LogicGraphAsset** — reusable `LogicGraph` stored at `recipes/<name>` or user-authored, with `asset_id`, `logical_path`, `role = Logic`, `version`. It is behavior data, not a `.bsn` scene.
- **LogicInstance** — placement of one `LogicGraphAsset` on exactly one Scene Instance in v1 via `editor.LogicBinding`.
- **Logic Brick** — a single node in a `LogicGraph`, assigned a **role** (`Sensor`, `Controller`, or `Actuator`) and a concrete `node_type_id`.
- **`RustController`** — a Controller brick whose behaviour is provided by a compiled trait impl in the editor binary.
- **Recipe** — built-in, versioned, immutable `LogicGraphAsset`.
- **Logic Evaluation Schedule** — the Bevy system set that runs the dispatch scheduler after `rebuild_preview_world`.

## References

- [`sddk/logic-bricks-graph-editor/spec.md`](../../sddk/logic-bricks-graph-editor/spec.md) — change-level spec (required reading).
- [`sddk/logic-bricks-graph-editor/proposal.md`](../../sddk/logic-bricks-graph-editor/proposal.md) — change proposal.
- [`sddk/logic-bricks-graph-editor/explore-report.md`](../../sddk/logic-bricks-graph-editor/explore-report.md) — exploration report.
- [CONTEXT.md](../../CONTEXT.md) — authoritative domain language.
- [ROADMAP.md](../../ROADMAP.md) — Logic Bricks roadmap entry.
- [`post-bsn-authoring-roadmap.md`](post-bsn-authoring-roadmap.md) — predecessor spec; this file extends the "Deferred until After Hito 3" → "Visual scripting/state machines" entry by graduating it to a concrete capability.
