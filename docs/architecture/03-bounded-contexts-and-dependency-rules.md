# Bounded Contexts and Dependency Rules

## Bounded contexts

### Project
Owns project identity, format version, resources, configuration and persistence orchestration.

### Scene
Owns `SceneDocument`, scene entities, hierarchy and scene-scoped operations.

### Scene Asset
Owns reusable definitions, local identities, asset relationships, versioning and explicit instance synchronization semantics.

### Level / World
Owns Level Scene Asset organisation and world topology. It references existing Scene Assets rather than inventing a parallel prefab model.

### Logic
Owns LogicGraph definitions, nodes/edges, recipes, validation and compile-time/runtime evaluator identifiers.

### Runtime Preview
Owns ephemeral projection/mapping/metrics/causality. It does not own authoring data.

### Source / Build
Owns source workspace abstractions, build/run operations and compiler diagnostics.

### Change Management
Owns `ChangeSet`, transactions, approval, history, semantic diff and post-apply verification.

### Agent / Automation
Owns planning, retrieval, delegation and policy-aware execution requests. It consumes capabilities from other contexts.

## Cross-context communication

Cross-context calls must use:

1. typed IDs/references;
2. application services/capabilities;
3. domain events or `ChangeEffect`s;
4. explicit DTOs.

They must not use:

- direct access to another context's mutable container;
- shared `serde_json::Value` as an untyped protocol when a stable semantic type exists;
- global registries as an implicit service locator.

## Identity policy

Each bounded context may preserve its correct identity type (`StableId`, `LocalId`, `NodeId`, etc.). Do not force a universal ID merely to generalise the transaction layer.

Global project references should use a typed resource locator, for example:

```rust
pub enum ResourceRef {
    Scene(SceneId),
    SceneAsset(SceneAssetId),
    LogicGraph(LogicGraphId),
    World(WorldId),
    Source(SourcePath),
    Asset(AssetFileId),
}
```

## Capability boundary

A capability represents a stable user/tool action, not a storage operation:

```text
PlaceSceneInstance
ExtractSelectionAsSceneAsset
ApplyOverridesToAsset
RegenerateAutoLayer
CreatePlatformerCharacter
ReimportExternalSource
ApplyRuntimeChanges
```

Capabilities may internally coordinate several bounded contexts through a `ChangeSet`.
