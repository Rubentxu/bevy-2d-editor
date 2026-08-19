# Target Architecture — Bevy-Native Runtime, Semantic-First Authoring

**Status:** Proposed  
**Related ADRs:** 0030, 0032, 0034, 0036, 0046, 0047, 0053–0064

## Architectural north star

```text
                         USER / AGENT
                              |
                              v
                     React / TypeScript
                              |
                  Commands / Queries / Notifications
                              |
                              v
                         editor-wasm
                       COMPOSITION ROOT
                              |
             +----------------+----------------+
             |                                 |
             v                                 v
     editor-application                 Bevy Editor Host
     use cases/policies                   runtime
             |                                 |
             v                      +----------+-----------+
      Transaction Kernel            |                      |
             |                      v                      v
             v                 EditorWorld             PreviewWorld
      Semantic Editor Model         Bevy ECS             Bevy ECS
       AUTHORITATIVE                editor state          game state
             |
      +------+-------+----------------------+
      |              |                      |
      v              v                      v
 ProjectStore    Durable Journal      Project Graph
 deterministic     ChangeSets         projections/indexes
```

## Five distinct models

### Semantic Editor Model — truth
Stable, deterministic, migratable, diffable, Bevy-free authoring concepts.

### Durable Change Journal — history
Approved/applied `ChangeSet` and transaction metadata for audit, undo/redo support, review, checkpoints and provenance. Not equivalent to Bevy event traffic.

### Project Graph — relationships
Derived graph for dependencies, instances, lineage, logic references, sources, runtime projections and causal links.

### EditorWorld — editor execution
Bevy `World` for selection, dirty state, validation runtime, indexes, graph runtime, async/job state and diagnostics.

### PreviewWorld — game projection
Bevy runtime/game components, rendering, gameplay, animation and Logic execution.

## Identity mapping

```text
StableId
   |
   +----> EditorWorld Entity
   |
   +----> PreviewWorld Entity
```

Rules:
- StableId is canonical authoring identity.
- Bevy entities are runtime handles only.
- mappings are indexes, never persistence.
- runtime rebuild may assign new Bevy IDs without semantic change.

## Dependency direction

```text
editor-model
    ^
    |
editor-application
    ^
    |
+---+------------------------------+
|                                  |
editor-bevy-*                 editor-storage-*
|                                  |
+----------------+-----------------+
                 |
             editor-wasm
```

Forbidden:
- `editor-application -> editor-bevy`;
- `editor-application -> editor-storage-web`;
- `editor-model -> wasm-bindgen/web-sys/bevy`.

## Runtime schedule

Suggested phases:

```text
ReceiveCommands
  -> ApplyTransactions
  -> EmitSemanticChanges
  -> UpdateGraphProjection
  -> PropagateInvalidation
  -> Validate
  -> UpdateDerivedIndexes
  -> UpdatePreviewProjection
  -> EmitNotifications
  -> RecordDiagnostics
```

## Graph + ECS responsibility

Graph answers relationships, dependency, paths and downstream impact. ECS answers active state, change detection, scheduling and efficient updates.

## Frontend boundary

React owns docking, forms, asset browser, inspector, CodeMirror, graph UI, UAT wizard and AI surfaces.

Bevy owns editor ECS, preview ECS, schedules, events/observers, viewport, picking/gizmos, simulation, graph runtime and runtime probes.

## Operational invariants

1. No authoring mutation bypasses Transaction Kernel.
2. Preview mutation never becomes authoring truth implicitly.
3. Every materialized graph is rebuildable.
4. Every cache/index is discardable.
5. Destructive actions can provide impact before apply.
6. UAT probes are read-only except through normal commands.
