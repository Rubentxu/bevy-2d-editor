# ADR-0009: ComponentOverride as the ECS/BSN-Friendly Replacement for OverridePatch

## Status

Accepted (2026-06-30)

## Context

ADR-0005 introduced `SceneInstance` as a placed use of a `SceneAssetDocument`: an asset reference plus local non-destructive patches and a durable `id_map`. The current implementation names those patches `OverridePatch`:

```rust
pub struct OverridePatch {
    pub target_local_id: LocalId,
    pub field_path: Vec<String>,
    pub value: serde_json::Value,
    pub status: OverrideStatus,
}
```

In practice, the implementation relies on `field_path[0]` carrying the component type ID and the remaining path segments pointing inside that component. That convention works, but it hides an important ECS boundary: the patch does not target an opaque property on a prefab-like object; it targets a field inside a specific Component Instance on an asset-local Entity.

Order 5 (`level-design-layers-research`) sharpened two additional constraints:

1. The editor should stay aligned with Bevy's ECS model because Bevy is also the preview/runtime execution engine.
2. The model must remain friendly to Bevy 0.19 BSN semantics (`bsn!` / `bsn_list!`), where scenes are composed from entities, components, relationships, and patches.

Keeping `OverridePatch` as the primary term and data shape would preserve a Unity/prefab-flavored mental model and make the next layer of design (`instance_components` for placed Scene Instances) harder to reason about.

## Decision

We will replace `OverridePatch` conceptually and technically with **`ComponentOverride`**.

The future canonical shape is:

```rust
pub struct ComponentOverride {
    pub target_local_id: LocalId,
    pub component_type_id: ComponentTypeId,
    pub field_path: Vec<String>,
    pub value: serde_json::Value,
    pub status: OverrideStatus,
}
```

`ComponentTypeId` is an explicit transparent identity wrapper, following the existing `StableId`, `LocalId`, and `AssetReference` pattern:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentTypeId(pub String);
```

`ComponentTypeId` lives in `schema.rs` because component identity belongs to the Component Schema Registry language. A shared ID module is deferred until several ID wrappers need common behavior.

Rules:

1. `component_type_id` explicitly identifies the Component Instance being patched.
2. `field_path` addresses fields **inside** that component only; it MUST NOT contain the component type ID as segment 0.
3. `SceneInstance` will use `component_overrides` and `orphaned_component_overrides` as the canonical fields.
4. Legacy JSON using `overrides` / `orphaned_overrides` and `field_path[0] = component_type_id` is not part of the forward-compatible contract. This migration is an intentional internal breaking schema reset before public compatibility.
5. `instance_components` are distinct from `component_overrides`:
   - `instance_components` are components owned by the placed occurrence itself, such as `editor.Transform2D` placement.
   - `component_overrides` are non-destructive patches against Component Instances inside the referenced Scene Asset.

## Considered Options

### Option A — Keep `OverridePatch` as-is

Rejected. It avoids immediate churn but preserves a hidden convention (`field_path[0]` as component type) that is neither ECS-friendly nor BSN-friendly. It also makes the boundary between occurrence-owned components and asset-field patches ambiguous.

### Option B — Rename only, keep the same shape

Rejected. Renaming `OverridePatch` to `ComponentOverride` without separating `component_type_id` from `field_path` would improve language but leave the underlying model flaw intact.

### Option C — Introduce `ComponentOverride` with explicit component identity (chosen)

Chosen. This makes the ECS boundary explicit, keeps field paths local to a component, and prepares the model for Level Scene Assets with Scene Instance Layers and `instance_components`.

### Option D — Remove overrides and model all instance changes as `instance_components`

Rejected. Occurrence-owned components and non-destructive asset-local patches are different concepts. Collapsing both into `instance_components` would lose resync semantics and make asset evolution harder to validate.

## Consequences

### Positive

- The model becomes explicit about ECS component identity.
- `field_path` becomes simpler and safer: it only addresses fields within a component.
- BSN projection can distinguish occurrence-owned components from patches against included/composed Scene Asset content.
- Validation and resync logic can report issues against `{target_local_id, component_type_id, field_path}` instead of reverse-engineering component identity from a path vector.
- Order 5 can build Level Scene Assets on a cleaner distinction between `instance_components` and `component_overrides`.

### Negative

- This is not a cosmetic rename. It touches persisted JSON shape, Rust types, resync/validation functions, WASM bridge payloads, frontend TypeScript types, UI copy, tests, and documentation.
- Existing specs and roadmap entries mention `OverridePatch`; they must be updated or marked legacy/transitional.
- Existing OPFS projects written with the legacy `OverridePatch` JSON shape may fail to load or require manual reset/reload. This is accepted because the project is still before public compatibility commitments.
- The migration delays Level Layer implementation, but reduces downstream design ambiguity.

## Migration Notes

The migration should be implemented as a separate slice before Level Layers implementation:

1. Add `ComponentOverride` with explicit `component_type_id`.
2. Remove `OverridePatch` as the canonical persisted shape.
3. Rename canonical fields:
   - `overrides` → `component_overrides`
   - `orphaned_overrides` → `orphaned_component_overrides`
4. Update all producers and consumers together: commands, processor, override/resync logic, WASM functions, frontend services, UI labels, tests, and docs.
5. Reset or regenerate test fixtures using the new JSON shape.
6. Document that existing OPFS data may need a manual reset/reload.
7. Do **not** add compatibility shims unless a later product milestone introduces real external users or saved projects that must be preserved.

## References

- [ADR-0005](./0005-scene-asset-bsn-aligned-reusable-scene-model.md) — Scene Asset as the BSN-aligned reusable scene model.
- [ADR-0006](./0006-authoring-first-roadmap-after-bsn-migration.md) — Authoring-first roadmap after the BSN migration.
- [CONTEXT.md](../../CONTEXT.md) — Component Override, Scene Instance, Component Instance terminology.
- `crates/editor-core/src/scene_instance.rs` — current `OverridePatch` / `SceneInstance` types.
- `crates/editor-core/src/schema.rs` — current schema registry type IDs that should migrate toward `ComponentTypeId` usage where appropriate.
- `crates/editor-core/src/scene_instance_overrides.rs` — override validation, effective-values merge, resync, and rebind logic.
- `crates/editor-core/src/bsn_ir.rs` — BSN semantic projection boundary.
- `crates/editor-core/src/bsn_codegen.rs` — `bsn!` / `bsn_list!` code generation target.
