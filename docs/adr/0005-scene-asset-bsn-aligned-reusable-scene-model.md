# ADR-0005: Scene Asset as the BSN-Aligned Reusable Scene Model

## Status

Accepted (2026-06-28)

## Context

Bevy 0.19 introduced the next-generation scene system through BSN (Bevy Scene Notation), including `bsn!`, `bsn_list!`, scene composition, patching, inheritance, templates, relationships, and dependency-aware scene spawning. Bevy 0.19 does **not** yet ship the first-party `.bsn` asset loader or full write-back/editor infrastructure. Those pieces are in flight for future Bevy releases, with public work around dynamic `.bsn`, BSN write-back, default-diffing, asset catalogs, persistent ASTs, and `SceneDocument` / `SceneAssetCatalog` APIs.

The Bevy 2D Editor currently has editor-owned `SceneDocument` JSON, `EntityTemplate` terminology, a `DynamicScene Export` adapter, and Rust code export that generates manual `Commands::spawn` code. This was enough for Hito 0, but it is no longer the best conceptual direction if the editor wants to align with Bevy's roadmap.

Research into Defold, Unity, Godot, Blender, and Unreal showed several useful patterns:

- Defold collection factories map asset-local object IDs to runtime IDs, but its split between game objects, collections, factories, and proxies creates conceptual confusion.
- Unity prefabs use stable asset identity plus internal object IDs, property-path overrides, apply/revert tools, nested prefabs, variants, and non-destructive unused override retention.
- Godot PackedScenes and inherited scenes are powerful but show fragility around editable children, inherited node changes, and orphaned nodes.
- Blender Library Overrides highlight the need for explicit hierarchy roots and resync operations when linked asset structures change.
- Unreal Blueprints demonstrate the danger of implicit default propagation and silent data resets.

The application is still early and not yet stable. We prefer a breaking architecture reset over a long compatibility layer that keeps discarded concepts alive.

## Decision

We will adopt **Scene Asset** as the primary reusable scene composition model, aligned with Bevy BSN.

`EntityTemplate` becomes a legacy/transitional concept and should be migrated away, not kept as a long-term first-class model.

The new model has these core concepts:

| Concept | Decision |
|---------|----------|
| **Scene Asset** | Reusable Bevy-aligned scene composition. Can represent an actor, fragment, screen, level, UI composition, or effect. |
| **Scene Instance** | A placed use of a Scene Asset: asset reference + explicit local patches/overrides + durable identity map. |
| **Scene Asset Catalog** | Project-level registry of Scene Assets, their stable IDs, logical paths, roles, dependencies, and exposed properties. |
| **Scene Asset Document** | Editor-owned durable document for authoring: IDs, metadata, undo/redo, migrations, exposed properties, and override metadata. |
| **BSN IR** | Semantic compatibility boundary that models BSN scenes, components, relationships, patches, and asset references. |
| **Adapters** | Export/import layers from Scene Asset Document / BSN IR to current and future formats. |

BSN is the standard target, but we will not copy unstable `.bsn` implementation details prematurely. The first real compatibility target is **Rust `bsn!` code generation**, because that exists in Bevy 0.19 today. Physical `.bsn` file export/import should be added when Bevy's loader/parser/write-back APIs stabilize.

## Detailed Rules

### Identity

Scene Assets use both stable internal identity and human-readable paths:

```text
SceneAssetRecord
  asset_id: opaque stable ID
  logical_path: human-readable Project path
```

Entities inside a Scene Asset also use two identities:

```text
SceneAssetEntity
  local_id: opaque stable ID
  local_path: human-readable/debug/export path
  name: display name
```

Overrides target `local_id`, not names or paths. `local_path` is for UI, debugging, export, and migration fallback.

Scene Instances persist an explicit mapping from Scene Asset local IDs to scene Stable IDs:

```text
SceneInstance
  asset_ref: asset_id
  id_map:
    root   -> stable-id-a
    weapon -> stable-id-b
    hitbox -> stable-id-c
```

### Overrides

Scene Instances are references plus patches, not deep clones.

Overrides are non-destructive. When a Scene Asset changes, invalid override data is marked as `orphaned`, `stale`, or `conflict`, and resolved through explicit resync / rebind / cleanup tools. The editor must not silently delete override data in response to asset changes.

The inspector should prioritize explicit **Exposed Properties** and keep arbitrary internal field overrides as **Advanced Overrides**.

### Roles

Scene Asset roles are soft validation policies, not separate asset types:

- `actor`
- `fragment`
- `screen`
- `level`
- `ui`
- `effect`

Roles guide lifecycle, multiplicity, validation, and editor UI without recreating Defold's GameObject / Collection / Factory / Proxy split.

### Versioning and Resync

Each Scene Asset has a monotonic numeric `version`. Each Scene Instance stores `asset_version_seen`.

When `asset_version_seen < asset.version`, the editor runs conservative auto-resync on open:

- safe changes apply automatically;
- active overrides remain active;
- uncertain data becomes `stale` or `conflict`;
- no destructive cleanup happens automatically;
- the editor shows a visible resync report.

### Variants

Scene Asset Variants are allowed as a future concept, but they are out of the first implementation cut. The initial scope is:

```text
Scene Asset + Scene Instance + local overrides
```

Variants may be added later, starting with at most one inheritance level.

## Considered Options

### Option A — Keep EntityTemplate as the primary reusable model

Rejected. It preserves legacy terminology and concepts that are drifting away from Bevy's BSN roadmap. Keeping both EntityTemplate and Scene Asset as first-class concepts would create two mental models and unnecessary migration debt.

### Option B — Make physical `.bsn` files the editor's immediate source of truth

Rejected for now. This aligns with the roadmap but couples the editor to APIs and syntax that are not yet stable in Bevy. We should model BSN semantics now and add physical `.bsn` support when Bevy's implementation lands.

### Option C — Scene Asset Document + BSN IR + adapters (chosen)

Chosen. This makes BSN the standard while isolating unstable parser/loader/write-back details behind adapters. It enables aggressive migration away from EntityTemplate without gambling on unfinished Bevy APIs.

### Option D — Use DynamicScene Export as the primary model

Rejected. `DynamicScene Export` remains an integration/export adapter. It is not the editor's source of truth and is not the right long-term reusable authoring model.

### Option E — Copy Defold's GameObject / Collection / Factory / Proxy split

Rejected. Defold's local-ID-to-runtime-ID behavior is useful, but its conceptual split is a known source of confusion. The editor will use one Scene Asset concept with roles and lifecycle policies instead.

## Consequences

### Positive

- Aligns the editor with Bevy's public BSN direction.
- Removes legacy terminology instead of dragging it forward indefinitely.
- Gives reusable content explicit identity, patching, resync, and migration semantics.
- Makes future `.bsn` adoption an adapter problem, not a whole-editor rewrite.
- Keeps `DynamicScene Export` useful without letting it dominate the domain model.
- Avoids Defold-style concept multiplication while retaining its useful identity mapping insight.

### Negative

- This is a breaking architectural reset.
- Existing `EntityTemplate` code and docs must be migrated or deleted.
- The editor needs new infrastructure: Scene Asset Catalog, Scene Asset Document, BSN IR, override health states, and resync reports.
- `bsn!` codegen must replace or supersede the current manual `Commands::spawn` codegen.
- Future Bevy `.bsn` changes may still require adapter updates.

## Implementation Direction

1. Introduce `Scene Asset`, `Scene Instance`, and `Scene Asset Catalog` as first-class Project concepts.
2. Add a BSN-compatible IR that captures scene graph, components, relationships, patches, and asset references.
3. Replace `EntityTemplate` with Scene Asset concepts through an explicit migration; delete the legacy model after migration.
4. Change Rust code export to generate `bsn!` / `bsn_list!` output as the primary Bevy-facing code target.
5. Keep `DynamicScene Export` as an adapter for runtime/preview integration, not as the primary reusable scene model.
6. Add non-destructive override validation, resync, rebind, cleanup, apply, and reset workflows.
7. Add `.bsn` file import/export only after Bevy's asset loader and write-back APIs stabilize.

## References

- Bevy 0.19 release notes — Next Generation Scenes / BSN.
- Bevy PR #23413 — core scene system, `bsn!`, templates.
- Bevy PR #23576 — dynamic BSN (`.bsn` asset format), targeted at future Bevy versions.
- Bevy issue #23637 — BSN editor infrastructure: write-back, asset catalog, persistent document.
- Bevy Editor roadmap — `.bsn` integration is ideal for scene save-back.
- Unity prefab serialization and unused overrides documentation.
- Defold collection factory documentation.
- Godot PackedScene and inherited scene behavior.
- Blender Library Overrides documentation.
