# PRD — Bevy-Native Editor Evolution

**ID:** PRD-BNE-001  
**Status:** Proposed  
**Date:** 2026-08-19

## Product intent

Evolve Bevy 2D Editor into a production-grade Bevy-native 2D workbench whose differentiators are real Bevy ECS/runtime foundations, semantic and reversible authoring, graph-aware dependency and causality reasoning, production 2D workflows, safe AI/extension automation, transparent runtime inspection and user-goal-driven UAT.

## Product thesis

The editor should feel purpose-built for Bevy rather than like a generic web editor that exports Bevy data.

Use Bevy where it adds unique value:

- ECS and queries;
- schedules/system sets;
- events/observers/change detection;
- asset/runtime integration;
- 2D viewport, picking, gizmos and simulation;
- WASM execution.

Authoring data still requires stable IDs, deterministic Git-friendly serialization, semantic diffs, migration, partial approval, undo/redo, provenance and cross-resource review. Therefore the authoritative persisted model remains Bevy-free.

## Goals

### G1 — Bevy-native execution
Make Bevy ECS the runtime substrate of the editor itself, not only game preview.

### G2 — Strong hexagonal boundaries
Enforce dependency direction at compile time and CI.

### G3 — Graph-native understanding
Provide a reusable graph kernel/materialized project graph for dependencies, inheritance, Logic Graph, causality, validation, world topology and AutoLayer.

### G4 — Incremental/reactive behaviour
Move from broad recomputation/polling to dependency-aware invalidation and event-driven projection.

### G5 — Production 2D workflows
Add Scene Asset variants/provenance, sprite slicing/pivot/collision tools, atlas workflow, animation timeline/state graph, better AutoLayer rules and impact navigation.

### G6 — Understandable execution
The user can answer: Why does this runtime entity exist? Why did preview rebuild? What depends on this asset? Why is this node dirty? Which system caused this? What breaks if I rename/delete this?

### G7 — Workflow-oriented UX
Use stable workspaces: World, Design, Logic, Animate, Debug, Code, with transversal lenses.

### G8 — Real UAT
Validate complete journeys through guided human, Playwright and headless semantic/runtime execution with evidence.

## Non-goals

- no full 3D editor;
- no general-purpose visual programming VM;
- no custom web UI framework replacing React;
- no persistent project representation based on raw Bevy `World`;
- no graph database as sole project truth;
- no event sourcing of every Bevy event;
- no big-bang rewrite;
- no multiplayer CRDT requirement.

## Personas

- technical designer;
- Bevy developer;
- content designer;
- QA/UAT validator;
- extension author;
- AI/agent supervisor.

## Core outcomes

1. Build/run a small 2D game without editing internal JSON.
2. Understand source/impact of significant authoring/runtime objects.
3. Make cross-resource changes safely and review them semantically.
4. Reimport external content without losing authored intent.
5. Diagnose Logic/runtime behaviour without ad-hoc logs.
6. Validate releases through reproducible UAT scenarios.

## Success metrics

Architecture:
- zero `editor-application -> editor-bevy/editor-storage-web/web-sys/js-sys` violations;
- zero new domain-level global service registries;
- generated/checked protocol DTOs;
- no persistent `bevy::Entity` IDs.

Runtime:
- graph compilation amortized;
- dirty/incremental Logic evaluation;
- event-driven frontend updates for migrated areas;
- measurable EditorWorld/PreviewWorld timing and rebuild causes.

UX:
- fewer disruptive mode changes;
- persistent Problems/Changes/Console/Trace surfaces;
- visible provenance for inherited/overridden fields.

UAT:
- every milestone has mandatory UAT gates;
- critical destructive/round-trip workflows capture semantic evidence;
- failures produce reproducible scenario/run IDs.

## Release strategy

- M0 / proposed v0.96 — Bevy-native runtime foundation;
- M1 / v0.97 — reactive project graph;
- M2 / v0.98 — Logic runtime + causality;
- M3 / v0.99 — UX and production authoring;
- M4 / v1.0 stabilization addendum — UAT/release hardening;
- M5 / post-v1 — time travel, advanced graph lenses and ecosystem expansion.

Version labels are sequencing proposals; dependency order is normative.
