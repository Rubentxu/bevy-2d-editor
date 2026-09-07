# Roadmap Convergence — Pre-v1.0

## Why converge

The repository accumulated several roadmap programs as capabilities were delivered:

- architecture foundation;
- production authoring;
- change/runtime workbench;
- agent runtime;
- semantic agents;
- ecosystem/importers;
- application stabilization;
- UI workflow overhaul;
- AI-native editor;
- v1 stabilization.

Most older documents are valuable history but should no longer compete as active execution plans.

## Active execution sources

1. `docs/roadmaps/MASTER_ROADMAP.md` — canonical ordering and product gates.
2. `docs/roadmaps/v1.0-architecture-ux-hardening.md` — active engineering program.
3. `docs/roadmaps/v1.0-stabilization.md` — release/product proof after hardening slices land.
4. Specs/ADRs/UAT referenced by those roadmaps.

## Historical/reference status

| Roadmap | New status | What survives |
|---|---|---|
| v0.87 architecture foundation | Historical | principles, delivered extraction history |
| v0.88 production authoring | Historical | capability/product rationale |
| v0.89 change/runtime | Historical | ChangeWorkbench/runtime causality intent |
| v0.90 agent runtime | Parked post-v1 | only requirements compatible with typed capabilities/ChangeSet |
| v0.91 semantic agents | Parked post-v1 | retrieval/runtime-aware direction |
| v0.92 ecosystem/importers | Delivered historical | compatibility and provenance requirements |
| application stabilization | Superseded active plan | release-health lessons, deterministic test cohorts |
| UI workflow overhaul | Superseded active plan | hierarchy/inspector/search/runtime UX requirements not fully hardened |
| AI-native editor | Parked post-v1 | product vision, typed operations, reviewability |

## What is intentionally deferred

Until v1 product gates pass:

- new autonomous agent runtime;
- background agent jobs;
- new large editor modes not required by canonical game proof;
- marketplace;
- collaborative editing;
- generic 3D parity;
- speculative framework work.

## Pull-forward exception

A deferred item may be pulled forward only when it directly blocks a v1 gate. The change must state:

- blocking evidence;
- smallest required scope;
- why an existing capability cannot solve it;
- UAT added to the v1 gate.

## Existing sddk cycle mapping (2026-09-07)

This section maps the in-flight `sddk/` cycles and `sddk/active/` workspaces to the converged H0–H10 slots so the active program does not lose lineage. Status reflects the cycle's own proposal/explore-report at the moment of convergence; `done` means the code already shipped in a tagged version, `continue` means the cycle keeps working under the new slot, `absorb` means the slot owns the work and the cycle becomes the implementation record, `parked` means explicitly deferred per the table above.

| Cycle path | Current state | Converged slot | Status |
|---|---|---|---|
| `sddk/active/ext-importers` | v0.93 external source importers, spec + tasks + verify-report present | **H9** (data safety / BSN / import-reimport recovery) | continue |
| `sddk/active/v0.82-p2-floating-multi-select` | Floating panels + multi-select shipped (v0.82) | **H7** (UX/perf, already-shipped retrospective) | done |
| `sddk/active/v0.82-p3-asset-thumbnails` | Asset browser thumbnails shipped (v0.83) | **H7** | done |
| `sddk/active/v092-sdk-ext-reg` | v0.92 extension SDK + extension registry | **H3/H4** (typed backend + mutation path; SDK capability surface is the input for capability segregation) | continue |
| `sddk/asset-pipeline` | AssetFile + OPFS binary + thumbnails + import UI | **H9** (data integrity for binary assets; feeds H7 thumb corpus) | continue |
| `sddk/auto-layer-generation` | Level auto-layer tooling | **H7** (level tools strengthening) | continue |
| `sddk/bsn-file-import` | BSN file import path | **H9** (BSN compatibility contract) | continue |
| `sddk/build-and-run-loop` | Run loop / enhanced preview wiring | **H3/H6** (typed backend surface for runtime) | continue |
| `sddk/code-aware-ai-debt` | v0.83.0 debt: forbidden-commands wiring, UTF-8 panic, doc drift | **pre-H10** (AI panel maintenance before v1 freeze) | continue |
| `sddk/code-editor-foundation` | Hito 4 code-editor foundation | **H3/H6** (typed backend + feature slices) | continue |
| `sddk/defold-inspired-redesign` | Dock/panel polish (shipped) | **H7** | done |
| `sddk/editor-shell-integrity` | Shell integrity work | **H6** (App/shell decomposition) | continue |
| `sddk/level-design-tools` | Level design toolset | **H7** (level strengthening) | continue |
| `sddk/level-inspector-and-override-panel` | Level inspector + override panel | **H7** (Inspector hardening) | continue |
| `sddk/logic-bricks-2d-recipes` | Logic Bricks 2D recipes | **H8** + feature strengthening (logic) | continue |
| `sddk/logic-bricks-graph-editor` | Logic Bricks graph editor (v0.36.0, mostly shipped) | **H8** | mostly done |
| `sddk/logic-graph-authoring-ui` | Logic graph authoring UI | **H7** (feature strengthening — logic) | continue |
| `sddk/logic-graph-data-model` | Logic graph data model (shipped) | **H8** | done |
| `sddk/logic-graph-validation` | Logic graph validation | **H8** (GraphKernel correctness properties) | continue |
| `sddk/logic-registry-and-metadata` | Logic registry + metadata | **H8** (capability contract cleanup) | continue |
| `sddk/opfs-catalog-flake-fix` | OPFS catalog persistence flake (shipped v0.81) | **H9** (fault injection corpus) | done — evidence feeds H9.1 FaultInjectingProjectStore |
| `sddk/refactor` | Open refactor backlog | TBD per PR | TBD |
| `sddk/rig-agent-runtime-foundation` | Rig agent runtime foundation | **post-v1** | parked |
| `sddk/rust-source-integration` | Rust source integration (Hito 4 Order 6) | **H3/H6** (typed backend sources surface) | done |
| `sddk/seguimos-roadmap` | Exploration (decide next Hito) | **consumed** — superseded by this convergence document | parked |
| `sddk/ui-workflow-overhaul` | UI workflow overhaul (mostly shipped) | **H6/H7** | done |
| `sddk/ui-workflow-overhaul-pr4-debt` | PR4 leftover debt | **H7** (Inspector / palette debt) | continue |
| `sddk/ux-overhaul` | UX overhaul (shipped) | **H7** | done |
| `sddk/workflow-first-program` | Workflow-first program (shipped) | **H6** | done |
| `sddk/workflow-surface-convergence` | Workflow surface convergence | **H6** (feature slices contributions) | continue |
| `cycles/adr-0030-crate-split` | Crate split work (shipped v0.94) | **H1** (already shipped; remaining work is enforcement gate = ADR-0056 → H0.2 archcheck v2) | done |

## Rule for new cycles

A new `sddk/cycle-name/` MUST declare a target H0–H10 slot in its `proposal.md` `## Scope / Not in Scope` section. Cycles that cannot pick a slot belong to the research backlog (`docs/research/RESEARCH_BACKLOG.md`) or the post-v1 backlog, never to a competitive parallel roadmap.

