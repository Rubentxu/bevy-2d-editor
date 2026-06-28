# Explore Report: scene-asset-document

> Change: `scene-asset-document` · Phase: explore (completed) · Mode: C1

## Status

Completed — Fase 0 spike. Proceed to proposal.

## Context

The editor currently uses `EntityTemplate` as its reusable scene composition model. Bevy 0.19 introduced BSN (Bevy Scene Notation) — a typed, composable scene system with `bsn!`, `bsn_list!`, templates, patches, inheritance, and relationship-aware scene spawning. The Bevy 0.19 release notes and PR #23413 document the core API surface. BSN is the right long-term alignment target, but its physical `.bsn` asset format and write-back APIs are still in flight (Bevy PR #23576, issue #23637).

ADR-0005 establishes the decision to adopt Scene Asset as the primary reusable model, aligned with Bevy BSN. This spike explores what that means concretely.

## Semantic Shape of Bevy 0.19 BSN

Bevy 0.19 ships the next-generation scene system through the `bevy_scene` crate (`bevy_scene/src/lib.rs`). Key concepts:

### Core Trait: `Scene`

The `Scene` trait defines the contract for scene-backed resources. Implementors can provide a `scene()` method returning a `DynamicScene`. The `DynamicScene` holds `Entities` with `Transform`, `Name`, `Parent`, and `Children` components, plus arbitrary `ComponentValue` bundles keyed by type ID.

### Templates: `bsn!` and `bsn_list!`

`bsn! { ... }` constructs a `SceneGraph` inline — a typed scene tree with:
- Entity definitions with component values
- Relationship declarations via `:parent` and `:"path"` syntax
- Child aggregation via nesting
- Patch expressions via `::` operator
- Asset references via `$asset"path"`

`bsn_list! { ... }` concatenates multiple `bsn!` entries.

Example shape (approximate):
```rust
bsn! {
    Player: Transform2D { translation: Vec2::new(0.0, 0.0) }
        :parent SceneRoot
        -> Weapon: Sprite2D { asset: $asset"weapons/sword.png" }
}
```

### Scene Composition

Composition happens via:
- **Inline nesting**: `Parent: Child: Grandchild { ... }` creates a hierarchy
- **`:parent`**: explicit parent override on any entity
- `:"path"`: reference to a named entity elsewhere in the graph
- **Default diffing**: scene application computes a diff against the world's current state and applies only deltas
- **Inheritance**: template fields can be overridden; BSN supports `:parent` override chains

### Spawning: `queue_spawn_scene`

`queue_spawn_scene(&World, scene: &dyn Scene)` queues a scene for spawning with dependency-aware ordering. Bevy tracks asset dependencies (scenes, images, etc.) and resolves them before spawning.

### Key API Types (Bevy 0.19)

| Type | Role |
|------|------|
| `Scene` (trait) | Provides a `DynamicScene` |
| `DynamicScene` | Value-based scene with entities + components |
| `SceneGraph` | Named entity tree with relationships |
| `ScenePatch` | Delta against a base scene |
| `FromTemplate` | Trait for types that can be constructed from a `SceneGraph` |
| `SceneSpawner` | Handles dependency-aware async scene spawning |

## What SceneAssetDocument Must Own

Per ADR-0005 §Detailed Rules, the editor's authoring document must own:

1. **`asset_id`**: opaque stable identifier (not a Bevy Entity)
2. **`logical_path`**: human-readable project path (e.g., `"assets/player.bsn"`)
3. **`role`**: lifecycle policy enum (`actor`, `fragment`, `screen`, `level`, `ui`, `effect`)
4. **`version`**: monotonic integer for resync detection
5. **`entities`**: ordered list of `SceneAssetEntity`, each with:
   - `local_id`: opaque stable ID (override target)
   - `local_path`: human-readable/debug path
   - `name`: display name
   - `components`: reuse of existing `ComponentInstance` vector
6. **`relationships`**: typed relationship list (`RelationshipKind::Child` + `Custom(String)`)
7. **`exposed_properties`**: named override surfaces for instances
8. **`metadata`**: tags, created/updated timestamps, notes

The document is JSON-serializable and editor-owned. It is NOT a `DynamicScene` or `SceneGraph`.

## What SceneInstance Must Own

A placed use of a Scene Asset (NOT a deep clone):

1. **`instance_id`**: the runtime stable ID assigned at placement
2. **`asset_ref`**: reference to the source `asset_id`
3. **`asset_version_seen`**: version of the asset at time of placement
4. **`id_map`**: `BTreeMap<LocalId, StableId>` — maps asset-local IDs to runtime stable IDs
5. **`overrides`**: non-destructive patches (active + stale + conflict)
6. **`orphaned_overrides`**: patches whose target entity was deleted from the asset

Override status lifecycle:
- `Active`: override applies cleanly
- `Orphaned`: target entity deleted from source asset
- `Stale`: target field renamed or removed in source asset
- `Conflict`: same field overridden in two instances (future)

## What BSN IR Is

A lossy, one-way semantic projection from `SceneAssetDocument` into a BSN-aligned shape:

```rust
pub struct BsnIr {
    pub scene_root: BsnIrNode,      // typed entity tree
    pub asset_refs: Vec<String>,     // asset paths referenced
    pub patches: Vec<BsnPatch>,    // override patches
}
```

```rust
pub struct BsnIrNode {
    pub identifier: String,                    // maps to LocalId
    pub components: BTreeMap<String, JsonValue>, // type_id -> values
    pub children: Vec<BsnIrNode>,              // from RelationshipKind::Child
    pub relationships: Vec<BsnIrRelationship>,  // all relationship kinds
}
```

**Losses (intentional):**
- `metadata`, `exposed_properties`, `logical_path`, `asset_id`, `version` are dropped
- `LocalId` becomes a plain string identifier
- `AssetReference` becomes a string in `asset_refs`

**Does NOT include:**
- `SceneAssetMetadata`
- `ExposedProperty`
- Version/resync metadata
- Override history

The IR is a projection INTO the BSN semantic space, not a full faithful representation. Write-back is future work (requires BSN write-back APIs to stabilize in Bevy).

## Cross-Editor Lessons

### Unity Prefabs

Unity's prefab system uses:
- `internal fileID` + `propertyPath` for internal object identity
- `PrefabAsset` + `PrefabGameObject` for the template
- `PrefabInstance` for placed copies with overrides
- Override system: overrides survive source changes; invalid overrides are marked but NOT auto-deleted
- **Unused overrides**: Unity retains overrides whose source property no longer exists — they appear in the inspector as "missing" but do not silently revert

Key lesson: non-destructive override retention (ADR-0005's "orphaned" / "stale" status) is standard practice in mature editors.

### Defold

Defold uses `collection factories` to map asset-local object IDs to runtime IDs at spawn time. The `local_id → runtime_id` map is explicit, not derived. This is the same pattern as `SceneInstance.id_map`.

Defold's split between GameObject, Collection, Factory, and Proxy is a known source of confusion. ADR-0005 avoids this by using a single `SceneAsset` concept with roles.

### Blender Library Overrides

Blender's library override system requires explicit resync when the linked library asset changes:
- Linked objects can have override data
- When the library changes, Blender marks overrides as needing resync
- Resync is an explicit operator, not automatic

Key lesson: auto-resync must be conservative (safe changes apply, uncertain data gets marked stale/conflict).

### Godot PackedScene

Godot's inherited scenes show fragility around "editable children" and inherited node changes. Orphaned nodes appear when the source scene removes a node that had overrides.

Key lesson: the `Orphaned` status for overrides whose target was deleted is the right semantic.

### Unreal Blueprints

Unreal's override system can silently reset overridden properties when the parent class changes and the property is removed. This is the anti-pattern ADR-0005 avoids by requiring explicit resync and never auto-deleting override data.

## Out of Scope for This Spike

The following are explicitly NOT in scope for the initial implementation:

1. **Scene Asset Catalog** — project-level registry of all Scene Assets (future change)
2. **BSN IR write-back** — projecting BSN IR back to SceneAssetDocument (requires Bevy's write-back APIs)
3. **Physical `.bsn` file import/export** — file format support (requires Bevy's `.bsn` loader API to stabilize)
4. **Resync/rebind/cleanup workflows** — the UI for resolving orphaned/stale overrides
5. **Apply/revert tools** — explicit override application to source or discard
6. **Scene Asset variants** — inheritance from another Scene Asset
7. **Auto-resync on open** — conservative resync algorithm when `asset_version_seen < asset.version`
8. **UI/in inspector override editing** — the actual editor UI for overrides
9. **Migration from EntityTemplate** — moving existing `EntityTemplate` data to `SceneAsset`

## Risks

### 1. BSN Inheritance Semantics Are Unstable

Bevy 0.19's inheritance (override chains via `:parent`) is new and may change. If inheritance semantics change in Bevy 0.20+, the `SceneAssetDocument` model may need adapter updates.

**Mitigation**: model inheritance at the document level (Scene Asset variants) rather than mirroring Bevy's inheritance mechanism directly. The adapter layer absorbs Bevy API changes.

### 2. Field Path Format Not Finalized

BSN field paths use dot-notation (`Transform.translation.x`) but the editor's `field_path: Vec<String>` stores segments as a string array. The mapping between dot-notation and segment arrays is ad-hoc. If BSN formalizes a path syntax, the adapter may need updating.

**Mitigation**: use segment arrays internally; the dot-notation is a serialization concern handled by the adapter.

### 3. Handle Resolution Gap

BSN uses `Handle<Image>` and similar Bevy asset handles at runtime. The editor uses `AssetReference` (logical path strings). The mapping between logical paths and resolved `Handle<Image>` at spawn time is not yet designed. This gap exists between `SceneInstance` placement and Bevy's scene spawner.

**Mitigation**: track the gap explicitly; design the handle resolution layer as a separate future change.

### 4. SceneInstance Storage Location

It is unclear whether `SceneInstance` lives in the `SceneDocument` JSON alongside entity data, in a separate `instances.json`, or in the `SceneAsset` catalog. This affects how instances are persisted and how they reference their source asset.

**Mitigation**: start with `SceneInstance` as a standalone type that can live in any JSON container; defer the persistence location decision to the OPFS persistence change.

### 5. Bevy 0.19 Crate Rename

The `bevy_scene` crate is expected to be renamed or restructured in future Bevy releases as the scene system matures. Adapter code that imports from `bevy_scene` directly may break.

**Mitigation**: keep the adapter layer thin and isolated; only `bsn_ir.rs` touches Bevy scene types directly.

## Context Quality: C1

The spike was conducted with:
- Bevy 0.19 release notes and source code (`bevy_scene/src/lib.rs`)
- Bevy PR #23413 (core scene system, `bsn!`, templates)
- Bevy PR #23576 (dynamic `.bsn` format)
- Bevy issue #23637 (BSN editor infrastructure)
- ADR-0005 as authoritative source of truth for editorial model decisions
- Cross-editor research (Unity, Defold, Godot, Blender, Unreal)

No direct implementation experience with Bevy 0.19 BSN in this codebase yet. The spike uses published documentation and source code analysis.

## Next Step

Proceed to `sddk-propose` with the findings from this spike. Key proposal points:
1. Introduce `SceneAssetDocument`, `SceneInstance`, `BSN IR` as Rust types in `crates/editor-core/src/`
2. No commands, no undo, no UI, no migration in the initial cut
3. Focus on type definitions + serde round-trip tests
4. One-way `bsn_ir_from_scene_asset()` projection (write-back is future)
