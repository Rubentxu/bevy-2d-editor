# Crate and Module Migration Plan

## Baseline

Current workspace: `editor-model`, `editor-application`, `editor-protocol`, `editor-storage-web`, `editor-bevy`, `editor-wasm`, `ai-proxy`.

Do not explode the repository into many crates immediately.

## Stage A — enforce current boundaries

- remove concrete storage and Bevy dependencies from `editor-application`;
- keep `editor-model` pure;
- make `editor-wasm` the composition root;
- move WASM glue out of application;
- move adapter/global registries out of model;
- strengthen archcheck using `cargo metadata`.

## Stage B — internal modules inside editor-bevy

```text
editor-bevy/src/
  runtime/
    editor_world.rs
    schedules.rs
    events.rs
    projection.rs
  preview/
    app.rs
    scene_projection.rs
  graph/
    runtime.rs
    compiled.rs
    invalidation.rs
  logic/
    compiler.rs
    evaluator.rs
    activation.rs
  diagnostics/
    causality.rs
    trace.rs
    metrics.rs
```

## Stage C — optional extraction

Only after API/ownership stabilizes:

```text
editor-graph
editor-bevy-runtime
editor-bevy-preview
editor-bevy-graph
editor-bevy-logic
```

`editor-graph` is the strongest early extraction candidate because its algorithms/types should be Bevy-free and reusable.

## Bevy feature policy

Use narrow ECS/app/state/reflect/asset dependencies for editor runtime where practical. Keep the heavier 2D render stack close to preview/viewport.

## Architecture fitness checks

1. actual Cargo dependency graph;
2. forbidden dependencies by crate;
3. no domain service globals;
4. no OPFS concrete types in application/model;
5. no Bevy type in serialized semantic DTOs;
6. no raw `bevy::Entity` in protocol;
7. no unmanaged window bridge growth;
8. hotspot/file-size warnings;
9. protocol generation drift;
10. UAT schema validity.
