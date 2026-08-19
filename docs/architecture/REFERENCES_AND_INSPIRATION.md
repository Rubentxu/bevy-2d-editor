# References and Inspiration

This document records ideas worth borrowing while preserving Bevy 2D Editor's own architecture.

## Bevy

Borrow directly as foundational runtime concepts:
- ECS World, components/resources and queries;
- schedules/SystemSets;
- events/observers/change detection;
- asset/runtime model;
- 2D rendering, picking/gizmos and WASM.

Do not persist raw runtime identity as authoring truth.

## Defold / Dynamo graph

Key inspiration:
- stable editor pane/workflow model;
- dependency-aware graph evaluation;
- cache/invalidation rather than recomputing everything;
- editor scripts/transactions as structured operations.

Borrow principles, not Clojure implementation details.

## ActiveGraph

Key inspiration:
- behaviours coordinate through graph-visible shared state;
- relation-aware behaviour;
- fork/diff from known revisions/events;
- traceable graph operations;
- packs/capabilities as composable units.

Adaptation for this project:
- semantic model remains source of truth;
- ChangeSet journal provides durable history;
- Project Graph is derived/materialized;
- Bevy ECS is reactive runtime;
- fork/diff becomes a controlled history feature, not universal event sourcing.

## Unity 2D

Ideas to match/improve conceptually:
- prefab/variant/override provenance;
- Sprite Editor workflows;
- Tile Palette/Tilemap authoring;
- Sprite Atlas tooling;
- 2D animation/timeline.

The goal is workflow parity where useful, not UI cloning.

## Godot 2D

Ideas:
- rich TileSet/terrain workflows;
- multi-layer tile authoring;
- animation state/blend graph concepts;
- integrated 2D resource editing.

## Moldable development

Use multiple views/lenses of the same semantic/project graph:
- tree when hierarchy is best;
- table for large structured data;
- graph for relationships;
- timeline for causal history;
- spatial view for world/scene;
- provenance chain for effective values.

A graph-native model should not force a graph canvas everywhere.
