# MASTER ROADMAP — Bevy 2D Workbench

**Approved:** 2026-08-14  
**Starting point:** v0.86.x

## Vision

Reach v1.0 as a production-capable 2D Bevy authoring workbench whose defining strengths are semantic/reversible editing, specialised 2D workflows, runtime causality and AI/extension operations over safe typed capabilities.

## Dependency graph

```mermaid
flowchart LR
  A[v0.87 Architecture Foundation]
  B[v0.88 Production Authoring]
  C[v0.89 Change + Runtime Workbench]
  D[v0.90 Agent Runtime]
  E[v0.91 Semantic Retrieval + Agent Workbench]
  F[v0.92 Ecosystem / SDK / Importers]
  G[v1.0 Stabilization]
  A --> B --> C --> D --> E --> F --> G
```

## v0.87 — Architecture Foundation

### Outcome
The existing editor behaves the same, but the architecture can enforce future direction.

### Must ship
- CI workflows + required checks;
- `editor-model` extraction;
- initial `editor-application`;
- `EditorSession` migration foundation;
- `Clock`/`IdGenerator` ports;
- `ProjectStore` port + existing OPFS adapter;
- Transaction Kernel v1 around existing operation logs;
- `ChangeSet` v1 metadata/effects;
- typed backend foundation and no-new-`window as any` gate;
- architecture fitness tests.

### Exit gate
No new feature code depends directly on old global-state paths unless explicitly allowlisted as migration debt.

## v0.88 — Production Authoring Foundation

### Outcome
The editor becomes materially faster for real 2D level production.

### Must ship
- 2D direct manipulation toolkit;
- World Workspace v1;
- scope-of-change UX for instance vs definition;
- filesystem-backed project spike/adapter or approved native companion mechanism;
- Git-friendly deterministic format/migration corpus;
- recipe engine + initial 5–8 recipes;
- improved large-scene hierarchy/asset performance.

## v0.89 — Change & Runtime Workbench

### Outcome
Bulk/refactor/import/runtime changes share one trustworthy workflow.

### Must ship
- Change Workbench;
- semantic diff renderer;
- checkpoints/history improvements;
- Runtime Causality Inspector v1;
- Runtime Apply-Back v1;
- ChangeSet cross-resource effects/verification;
- search/navigation links across causality graph.

## v0.90 — Agent Runtime Foundation

### Outcome
Rig-based runtime is integrated without bypassing editor architecture.

### Must ship
- `editor-protocol` tool contracts;
- `agent-runtime` crate;
- manager + 2 specialist agents initially;
- `/v1/propose` compatibility adapter;
- read/planning tools;
- ChangeSet proposal generation;
- policy/approval enforcement;
- telemetry/diagnostics.

Do not start with every specialist. Prove the architecture with Scene + Validation/Runtime first.

## v0.91 — Semantic Retrieval & Agent Workbench

### Outcome
Agents understand the project and can safely execute multi-step workflows.

### Must ship
- semantic/typed retrieval;
- full Change Workbench integration for agents;
- scene/asset/logic/code/world specialists;
- runtime-aware diagnostics;
- post-apply verification loops;
- limited safe background maintenance tasks.

## v0.92 — Ecosystem, SDK & Import Pipelines

### Outcome
The workbench participates in the wider 2D toolchain and has a credible extension model.

### Must ship
- internal Editor Extension SDK;
- Aseprite import/reimport;
- LDtk import/reimport;
- Tiled import/reimport;
- recipe packs as extensions;
- validator extension example;
- capability permissions/versioning.

### Delivered (v0.92.0)
SDK + capability permissions + 3 built-in extensions shipped. Importers deferred to v0.93 per SDK-061 dependency.

## v1.0 — Stabilization

### Product gates

- create a small complete 2D game without hand-editing editor data;
- filesystem/Git workflow documented and stable;
- browser-local workflow remains supported;
- round-trip/migration compatibility guaranteed for declared v1 formats;
- crash/data-loss recovery story tested;
- performance corpus meets budgets;
- accessibility critical paths pass;
- extension and agent capability APIs have documented compatibility policy;
- no critical architecture fitness exceptions remain.

## Architecture evolution addendum (2026-08-19)

The post-v0.95 Bevy-native evolution programme is defined in
[BEVY_NATIVE_MASTER_ROADMAP.md](./BEVY_NATIVE_MASTER_ROADMAP.md).

Its governing principle is **Bevy-native runtime, semantic-first authoring** and
it introduces M0–M4 gates for EditorWorld, reactive project graphs, compiled Logic
runtime, causality/trace, workflow UX and semantic UAT.

## Explicitly post-v1 unless pulled by evidence

- marketplace;
- multiplayer collaborative CRDT editing;
- full visual scripting VM;
- pixel-art editor;
- full audio DAW tooling;
- mobile/touch-first editor;
- general 3D editor parity.
