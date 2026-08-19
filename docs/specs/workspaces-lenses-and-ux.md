# SPEC-UX-001 — Workspaces, Lenses and Workflow UX

**Status:** Proposed  
**ADRs:** 0059, 0062

## Workspaces

### World
World topology, level navigation/links and world validation.

### Design
Scene/level editing, Scene Assets, tiles, sprite authoring and placement.

### Logic
Logic Graph, recipes, validation and activation preview.

### Animate
Sprite animation, timeline and later state/blend graph.

### Debug
Runtime, systems, graph, trace, performance and changes.

### Code
Rust/source editing and diagnostics.

## Persistent lower area

```text
Assets | Problems | Changes | Console | Trace
```

## Mode rules

- Play/Pause/Step is global runtime state.
- Compatible workspace switches preserve subject context.
- Opening a resource chooses suitable editor without unexpectedly resetting context.
- breadcrumbs/history preserve navigation.

## Lenses

Hierarchy, Spatial, Dependencies, Changes, Causality and Runtime.

## Global search

Classify entity, scene, Scene Asset, asset, Logic Graph, world/level, source, validation, change and runtime subject. Results expose contextual actions and dependency/Why shortcuts.

## Provenance labels

Local, Inherited from Variant, Inherited from Base, Instance Override, Runtime-only, Applied Back, Imported Source.

## Destructive actions

Delete/rename/reimport show Impact where material.

## Accessibility

Critical workflows keyboard-operable. Graph canvases need a structural/non-canvas alternative for essential navigation/validation.
