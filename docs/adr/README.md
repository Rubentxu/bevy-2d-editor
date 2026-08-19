# Architecture Decision Records (ADR)

This directory records architecture decisions for the Bevy 2D Editor.

An ADR captures a decision that is hard to reverse, surprising without context, and the result of a real trade-off. Each file follows the structure: **Status · Context · Decision · Considered Options · Consequences · References**.

ADR numbering is monotonic and never reused. Superseded decisions keep their original number and have their `Status` updated.

## Index

| Number                                                                           | Title                                                                          | Status                                                               |
| -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------------- |
| [ADR-0001](./0001-scene-document-json-as-source-of-truth.md)                     | SceneDocument uses JSON as source of truth, not RON or DynamicScene            | Accepted — superseded in source-of-truth semantics by ADR-0046       |
| [ADR-0002](./0002-single-bevy-renders-canvas.md)                                 | Single Bevy renders the canvas                                                 | Accepted                                                             |
| [ADR-0003](./0003-forward-compat-via-serde-json-value.md)                        | Forward compatibility via `serde_json::Value`                                  | Accepted                                                             |
| [ADR-0004](./0004-dynamic-scene-export-bevy-native-anchor.md)                    | DynamicScene Export as the Bevy-native anchor                                  | Accepted                                                             |
| [ADR-0005](./0005-scene-asset-bsn-aligned-reusable-scene-model.md)               | Scene Asset as the BSN-aligned reusable scene model                            | Accepted (2026-06-28)                                                |
| [ADR-0006](./0006-authoring-first-roadmap-after-bsn-migration.md)                | Authoring-first roadmap after the BSN migration                                | Accepted (2026-06-29)                                                |
| [ADR-0007](./0007-separate-asset-command-surface.md)                             | Separate Asset command surface for Scene Asset Authoring                       | Accepted (2026-06-29)                                                |
| [ADR-0008](./0008-path-based-scene-asset-opfs-layout.md)                         | Path-based OPFS layout for Scene Assets                                        | Accepted (2026-06-29) — amended by ADR-0033                          |
| [ADR-0009](./0009-component-override-ecs-bsn-replacement-for-override-patch.md)  | ComponentOverride as the ECS/BSN-friendly replacement for OverridePatch        | Accepted (2026-06-30)                                                |
| [ADR-0010](./0010-bsn-exporter-trait-file-export.md)                             | BsnExporter trait — output-only .bsn file export                               | Accepted (2026-06-30)                                                |
| [ADR-0011](./0011-logic-bricks-compiled-rust-controllers.md)                     | Logic Bricks — compiled Rust controllers and dispatch scheduler (no VM)        | Accepted (2026-07-01)                                                |
| [ADR-0012](./0012-editor-choice-codemirror-6.md)                                 | Editor Choice — CodeMirror 6 over Monaco                                       | Accepted                                                             |
| [ADR-0013](./0013-build-run-loop-enhanced-preview.md)                            | Build & Run Loop — Enhanced Preview Mode for v1                                | Accepted (2026-07-03)                                                |
| [ADR-0014](./0014-data-hot-reload.md)                                            | Data-Only Hot Reload for Source and Asset Files                                | Accepted (2026-07-18)                                                |
| [ADR-0015](./0015-code-aware-ai-context-model.md)                                | Code-Aware AI Context Model                                                    | Accepted (2026-07-19) — Hito 4 Order 6 (`code-aware-ai`)             |
| [ADR-0016](./0016-scene-component-authoring.md)                                  | Scene-Component Authoring                                                      | Accepted (2026-07-19) — Hito 4 Order 7 (`scene-component-authoring`) |
| [ADR-0017](./0017-e2e-test-failure-root-cause.md)                                | E2E Test Failure Root Cause (Hito 4 final cleanup)                             | Investigation complete (2026-07-19)                                  |
| [ADR-0018](./0018-deferred-scene-component-command-handlers-keep-unsupported.md) | Deferred SceneComponent command handlers remain Unsupported                    | Accepted (2026-07-20) — Hito 7 (`scene-component-authoring-ux`)      |
| [ADR-0019](./0019-opfs-scene-asset-catalog-persistence-ordering.md)              | OPFS Scene-Asset Catalog Persistence Ordering                                  | Accepted (2026-07-21) — Hito 7 (`scene-component-authoring-ux`)      |
| [ADR-0020](./0020-number-skipped.md)                                             | Number skipped — reserved                                                      | Skipped (2026-07-29)                                                 |
| [ADR-0021](./0021-defold-inspired-layout.md)                                     | Defold-Inspired Layout for Docks                                               | Accepted — Hito 6 (`defold-inspired-dock-polish`)                    |
| [ADR-0022](./0022-drag-and-dock-region-swap-renumbered.md)                       | Drag-and-Dock Region Swap — renumbered to ADR-0024                             | Renumbered (2026-07-29)                                              |
| [ADR-0023](./0023-number-skipped.md)                                             | Number skipped — reserved                                                      | Skipped (2026-07-29)                                                 |
| [ADR-0024](./0024-drag-dock-swap.md)                                             | Drag-and-Dock Region Swap                                                      | Accepted — Hito 6 (`defold-inspired-dock-polish`)                    |
| [ADR-0025](./0025-floating-panels-multi-select.md)                               | Floating Panels + Inspector Multi-Select                                       | Accepted (2026-07-23) — v0.82 (`v0.82-p2-floating-multi-select`)     |
| [ADR-0026](./0026-asset-browser-thumbnails.md)                                   | Asset Browser Thumbnails — Optional `preview_resource` + Lazy Native Blob URLs | Accepted (2026-07-24) — v0.83 (`v0.82-p3-asset-thumbnails`)          |
| [ADR-0027](./0027-rig-agentic-editor-architecture.md)                            | Rig-Based Agent Runtime for the AI-Native Bevy 2D Editor                       | Accepted (2026-07-27) — planning baseline, refined by ADR-0043       |
| [ADR-0028](./0028-workflow-first-before-agentic-ai.md)                           | Workflow-First UI Convergence Before Agentic AI                                | Accepted (2026-07-27) — sequencing prerequisite, extended by v0.87 Architecture Foundation |
| [ADR-0029](./0029-frontend-performance-budget-contract.md)                       | Frontend Performance Budget Contract                                           | Accepted (2026-08-02) — A3 stabilization                             |
| [ADR-0030](./0030-compile-time-hexagonal-boundaries.md)                          | Compile-Time Hexagonal Crate Boundaries                                        | Accepted + Implemented (v0.94.0)                                      |
| [ADR-0031](./0031-explicit-editor-session-state.md)                              | Explicit EditorSession Replaces Domain-Level Global State                      | Accepted (2026-08-14)                                                |
| [ADR-0032](./0032-transaction-kernel-and-changeset.md)                           | Shared Transaction Kernel and ChangeSet, with Domain-Specific Commands         | Accepted (2026-08-14)                                                |
| [ADR-0033](./0033-projectstore-port-opfs-filesystem.md)                          | ProjectStore Port with OPFS and Filesystem Adapters                            | Accepted (2026-08-14)                                                |
| [ADR-0034](./0034-typed-editor-backend-contract.md)                              | Typed EditorBackend Contract Replaces Global Window Bridge                     | Accepted (2026-08-14)                                                |
| [ADR-0035](./0035-clock-and-id-generator-ports.md)                               | Clock and IdGenerator Are Explicit Application Ports                           | Accepted (2026-08-14)                                                |
| [ADR-0036](./0036-runtime-preview-adapter.md)                                    | Bevy Runtime Preview Is an Ephemeral Projection Adapter                        | Accepted (2026-08-14)                                                |
| [ADR-0037](./0037-world-workspace-first-class.md)                                | World Workspace Is a First-Class Product Context                               | Accepted + Implemented (v0.95.0)                                        |
| [ADR-0038](./0038-workflow-and-gameplay-recipes.md)                              | Workflow and Gameplay Recipes Compile Intent into Typed Changes                | Accepted (2026-08-14)                                                |
| [ADR-0039](./0039-change-workbench.md)                                           | Change Workbench Is the Unified Review and Approval Surface                    | Accepted (2026-08-14)                                                |
| [ADR-0040](./0040-editor-extension-sdk.md)                                       | Editor Extension SDK Is Capability-First and Transactional                     | Accepted (2026-08-14)                                                |
| [ADR-0041](./0041-external-source-import-reimport.md)                            | External Authoring Sources Use Provenance-Aware Import/Reimport Pipelines      | Accepted + Implemented (2026-08-17) — v0.93 (`feat/v0.93-external-source-importers`) |
| [ADR-0042](./0042-runtime-apply-back.md)                                         | Runtime Apply-Back Is Explicit, Scoped and Authorable-Field Only               | Accepted (2026-08-14)                                                |
| [ADR-0043](./0043-agent-runtime-capability-boundary.md)                          | Agent Runtime Uses Replaceable Orchestration Behind Typed Editor Capabilities  | Accepted (2026-08-14)                                                |
| [ADR-0044](./0044-ci-and-architecture-fitness-gates.md)                          | CI and Architecture Fitness Gates Are Release-Critical                         | Accepted (2026-08-14)                                                |
| [ADR-0045](./0045-git-friendly-project-format-and-migrations.md)                 | Project Format Is Git-Friendly, Deterministic and Explicitly Migrated          | Accepted (2026-08-14)                                                |
| [ADR-0046](./0046-semantic-editor-model-authority.md)                            | Semantic Editor Model Is the Authoritative Source of Truth                     | Accepted (2026-08-14)                                                |
| [ADR-0047](./0047-logic-graph-model-split-bevy-adapter.md)                       | Logic Graph Model Split — Pure Types in editor-model, Bevy Adapter in editor-core | Accepted (2026-08-15) — v0.87 (`v0.87-architecture-foundation`)    |
| [ADR-0048](./0048-projectstore-v1-is-synchronous.md)                              | ProjectStore v1 Is a Synchronous Port                                          | Accepted (2026-08-15) — v0.87 (`v0.87-architecture-foundation`)    |
| [ADR-0049](./0049-dual-dispatch-gate.md)                                        | Dual Dispatch Gate for TransactionKernel Adoption                               | Accepted (2026-08-16) — v0.89 (`v0.89-change-runtime-workbench`)    |
| [ADR-0050](./0050-apply-back-policy-not-mirrored.md)                              | ApplyBackPolicy Lives in editor-application (Mirror-Pair with editor-core)      | Accepted (2026-08-16) — v0.89 (`v0.89-change-runtime-workbench`)    |
| [ADR-0051](./0051-change-workbench-bottom-dock-tab.md)                           | ChangeWorkbenchPanel Lives in Bottom-Dock as an Internal Tab (ADR-0039/0024)   | Accepted (2026-08-16) — v0.89 (`v0.89-change-runtime-workbench`)    |
| [ADR-0052](./0052-runtime-causality-rebuild-cause.md)                            | Runtime Causality — RebuildCause + LogicActivationRing + CausalityEdge          | Accepted (2026-08-16) — v0.89 (`v0.89-change-runtime-workbench`)    |
| [ADR-0053](./0053-bevy-native-editor-runtime.md)                                 | Bevy ECS Is the Runtime Substrate of the Editor                                  | Accepted (2026-08-19) — M0 runtime foundation                         |
| [ADR-0054](./0054-editor-world-preview-world.md)                                | Separate EditorWorld and PreviewWorld Runtime Responsibilities                     | Accepted (2026-08-19) — M0 runtime foundation                         |
| [ADR-0055](./0055-semantic-graph-reactive-ecs.md)                               | Semantic Graphs Compile to Reactive Bevy ECS Runtime Projections                  | Proposed (2026-08-19)                                                 |
| [ADR-0056](./0056-events-observers-durable-journal.md)                          | Bevy Events/Observers Are Ephemeral Runtime Signals; Change Journal Is Durable    | Proposed (2026-08-19)                                                 |
| [ADR-0057](./0057-compiled-incremental-logic-runtime.md)                      | Logic Graphs Use a Compiled Incremental Runtime                                  | Proposed (2026-08-19)                                                 |
| [ADR-0058](./0058-typed-cqn-wasm-protocol.md)                                  | WASM Boundary Uses Typed Commands, Queries and Notifications                      | Proposed (2026-08-19)                                                 |
| [ADR-0059](./0059-react-shell-bevy-runtime-boundary.md)                        | React Owns Dense Editor UI; Bevy Owns Runtime, Viewport and Simulation           | Proposed (2026-08-19)                                                 |
| [ADR-0060](./0060-unified-causality-impact-trace.md)                           | Unified Causality Model Powers Impact, Why and Trace                             | Proposed (2026-08-19)                                                 |
| [ADR-0061](./0061-uat-semantic-probe-dsl.md)                                   | UAT Uses a Shared Scenario DSL and Semantic/ECS Probes                           | Proposed (2026-08-19)                                                 |
| [ADR-0062](./0062-contribution-registry.md)                                    | UI and Extension Features Register Typed Contributions                            | Proposed (2026-08-19)                                                 |
| [ADR-0063](./0063-system-graph-observability.md)                                | Bevy Schedule and Runtime Systems Are Inspectable Product Data                    | Proposed (2026-08-19)                                                 |
| [ADR-0064](./0064-scene-asset-variants-provenance.md)                           | Scene Asset Variants and Overrides Use Explicit Provenance                        | Proposed (2026-08-19)                                                 |

## Related Documents

- [CONTEXT.md](../../CONTEXT.md) — project domain language (authoritative terminology).
- [docs/sddk/](../sddk/) — SDD change proposals, specs, and designs.
- [docs/specs/](../specs/) — capability specifications.
- [docs/roadmaps/](../roadmaps/) — forward-looking implementation programs.
- [EVOLUTION_INDEX.md](./EVOLUTION_INDEX.md) — provenance of the Architecture & Product Evolution Pack (ADR-0030 → ADR-0046) and its relationship to historical ADRs.
