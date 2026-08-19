# SPEC-RUNTIME-001 — Bevy Editor Runtime

**Status:** Proposed  
**ADRs:** 0053, 0054, 0056, 0063

## Purpose

Define the runtime that makes the editor itself Bevy-native while preserving the semantic/application core.

## Runtime host

Initial shape:

```rust
pub struct BevyEditorRuntime {
    editor_world: bevy_ecs::world::World,
    editor_schedule: bevy_ecs::schedule::Schedule,
    preview_app: bevy_app::App,
}
```

Exact types may evolve. Separation of editor runtime and preview runtime responsibilities is invariant.

## EditorWorld resources

```text
EditorRevision
ActiveProjectRef
SemanticRevisionRef
ProjectGraphRuntime
SelectionStateRuntime
ValidationRuntime
EditorEventCorrelation
RuntimeDiagnostics
AssetRuntimeIndex
StableIdToEditorEntity
StableIdToPreviewEntity
```

## Components

```text
EditorSubject { stable_id, subject_kind }
Selected
Dirty(reason)
ValidationState
ProjectionVersion
DependencyRuntimeNode
CausalityRuntimeNode
CachedDerivedState
```

## System sets

```text
CommandIngress
Transactions
SemanticEvents
GraphProjection
Invalidation
Validation
PreviewProjection
Notifications
Diagnostics
```

Ordering:

```text
CommandIngress -> Transactions -> SemanticEvents -> GraphProjection
 -> Invalidation -> Validation -> PreviewProjection -> Notifications -> Diagnostics
```

Parallelism is allowed inside a set when data access permits it.

## Runtime events

Minimum typed events:

```text
ChangeSetApplied
SemanticResourceChanged
GraphProjectionChanged
ValidationInvalidated
PreviewProjectionRequested
PreviewProjectionCompleted
LogicActivationRecorded
EditorNotificationReady
```

Carry correlation IDs where useful.

## Lifecycle

### Project open
1. load/migrate semantic project;
2. build EditorWorld projections/indexes;
3. build project graph;
4. validate;
5. build preview on demand;
6. emit readiness.

### Semantic change
1. command -> use case;
2. Transaction Kernel applies;
3. semantic revision advances;
4. runtime receives ChangeSetApplied;
5. graph delta applied/rebuilt;
6. affected projections become dirty;
7. validation/preview update incrementally;
8. notifications emitted.

### Project close
All ECS worlds, caches and projections are disposable.

## Headless mode

Must run without renderer/browser for schedules, graph projection, invalidation, Logic runtime, causality and UAT probes.

## Acceptance

- no semantic data depends on Bevy entity IDs;
- EditorWorld rebuilds from semantic project;
- preview rebuild does not change semantic hash;
- application crate does not depend on runtime crate;
- headless runtime tests pass.
