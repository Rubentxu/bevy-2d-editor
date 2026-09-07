# Specification — Frontend Feature Slices and Workspace

## Goal

Reduce cross-feature connascence and make adding a new document/editor surface local rather than App-wide.

## Target source layout

```text
frontend/src/
  app/
    App.tsx
    shell/
    providers/
    workspace/
  backend/
  features/
    scene/
    assets/
    logic/
    world/
    code/
    runtime/
    validation/
    changes/
  shared/ui/
```

## App responsibilities

`App.tsx` should primarily:

- compose providers;
- initialize backend/application lifecycle;
- render shell/workspace;
- surface unrecoverable initialization errors.

It should not know every feature callback.

## Feature slice responsibilities

Each feature owns:

- feature hooks/controllers;
- capability calls;
- panels/components;
- local transient state;
- tests;
- commands shown in command palette through contributions.

## Workspace direction

The current global mode concept should be decomposed experimentally into orthogonal state:

```text
ActiveDocument / OpenDocuments
RuntimeMode
WorkspaceLayout
ContextSelection
```

A possible active document union:

```ts
SceneDocumentRef
SceneAssetDocumentRef
LogicGraphDocumentRef
WorldDocumentRef
SourceDocumentRef
```

This is a directional target, not a requirement to build a full IDE tab system in one PR.

## Contribution model

Instead of App switches, features may contribute:

- commands;
- panels;
- document renderers;
- breadcrumbs;
- validation navigation handlers.

Start with explicit registries/configuration; do not create dynamic plugin infrastructure until needed.

## State management

No mandatory Redux/Zustand migration.

First choice:

- React Context for stable capabilities;
- reducers for cohesive workspace state;
- component state for transient UI;
- external store only if profiling demonstrates Context update fan-out problems.

## Success criteria

Adding a future animation document surface should not require editing a central 50-handler contract and multiple editor-mode switches.

