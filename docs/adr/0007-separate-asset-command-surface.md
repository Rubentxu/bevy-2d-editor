# ADR-0007: Separate Asset Command Surface for Scene Asset Authoring

## Status

Accepted (2026-06-29)

## Context

ADR-0005 establishes `SceneAssetDocument` as the editor-owned source of truth for reusable Scene Assets, with an entity identity model that differs from `SceneDocument`:

```text
SceneDocument        → StableId identity, parent-field hierarchy
SceneAssetDocument   → LocalId identity, relationships-based hierarchy, exposed properties
```

ADR-0006 §Normative Rules mandates that features which mutate scene data "must use the typed command pipeline **or define why a new command surface is required**." Scene Asset Authoring Mode (`scene-asset-authoring` capability) needs reversible entity and component edits against `SceneAssetDocument`. The question is whether those edits should flow through the existing scene `Command` / `Processor` / `OperationLog` pipeline, or through a dedicated, parallel surface.

The existing scene pipeline (`command.rs`, `processor.rs`, `operation_log.rs`) is keyed on `StableId`, uses a dotted-string `field_path`, and mutates `SCENE_DOC`. The asset substrate (`scene_asset.rs`) is keyed on `LocalId` and already uses `Vec<String>` field paths for `ExposedProperty` and `SceneAssetRelationship` (`scene_asset.rs:93,101`).

Catalog management (create, rename, duplicate, delete) is a separate concern from document-body editing: it operates on the `SceneAssetCatalog` and OPFS files, not on a single document body, so it has no meaningful per-body inverse in an operation log.

## Decision

We introduce a **separate command surface** for Scene Asset Authoring, living in a new `asset_command.rs` module:

```text
AssetCommand       (enum)         — parallels Command, keyed on LocalId
AssetProcessor     (apply/inverse) — parallels Processor
AssetOperationLog  (undo/redo)     — parallels OperationLog, per-asset
AssetCommandError / AssetCommandResult
```

Three rules govern the split:

1. **Identity split.** `AssetCommand` variants carry `local_id` (the `LocalId` of a `SceneAssetEntity`), never a scene `StableId`. The processor validates-then-mutates a `SceneAssetDocument`, and `dispatch_asset_command` touches only the `SCENE_ASSET_DOC` thread-local — never `SCENE_DOC`, `SCENE_REGISTRY`, or the scene `OPERATION_LOG`.

2. **Catalog CRUD is not command-log material.** Create, rename, duplicate, and delete are dedicated `#[wasm_bindgen]` functions (`create_scene_asset`, `rename_scene_asset`, `duplicate_scene_asset`, `delete_scene_asset`) that operate on the catalog and OPFS, mirroring `scene_create`/`scene_rename`. They do **not** appear in `AssetOperationLog`. Only entity and component mutations (`AddEntity`, `RemoveEntity`, `RenameEntity`, `AddComponent`, `RemoveComponent`, `SetComponentValue`, `Batch`) are `AssetCommand`s.

3. **Module-local conventions.** `field_path` is `Vec<String>` (the asset module's existing convention), not the dotted string used by scene `Command::SetComponentField`. This divergence is contained inside the asset module.

## Considered Options

### Option A — Reuse `Command` via a `StableId` ↔ `LocalId` adapter

Rejected. A value adapter that translates `LocalId` to `StableId` on every dispatch is itself a StableId/LocalId bug surface. This is the exact Godot editable-children fragility class that ADR-0005 explicitly rejects: two identity models silently crossing at a translation boundary, where an adapter bug corrupts either the scene or the asset document without a type error. The scene pipeline also hardcodes the dotted-string `field_path` convention and a `parent`-field hierarchy, neither of which matches `SceneAssetDocument`'s `relationships`-based model.

### Option B — Generic `OperationLog<C, D>` shared by both documents

Rejected for now. A generic refactor of the proven `OperationLog`/`Processor`/`Command` trio would touch the stable, tested scene pipeline for the sake of code reuse that is not yet justified by a second or third consumer. The cost of a wrong generic abstraction is paid by the scene pipeline, which must not regress. A concrete parallel type is cheaper today; unification remains a candidate for a future refactor once the asset surface has a second implementation to learn from.

### Option C — Separate `AssetCommand` surface, dedicated module (chosen)

Chosen. A concrete, parallel type in a new module gives type-level isolation between scene and asset edits, keeps the scene pipeline untouched, and documents the boundary as a deliberate seam rather than an accidental coupling.

## Consequences

### Positive

- Type-level separation prevents scene/asset identity confusion at compile time — an `AssetCommand` cannot be applied to a `SceneDocument` and vice versa.
- The scene command pipeline (`Command`/`Processor`/`OperationLog`) is untouched and cannot regress.
- The asset module keeps its own conventions (`Vec<String>` field paths, `LocalId` keys, `relationships`-based hierarchy) without forcing them onto the scene module or vice versa.
- Catalog CRUD stays out of the operation log, matching its actual semantics (catalog + OPFS mutations have no per-body inverse).
- The seam is explicit and auditable for future AI-assisted editing, satisfying ADR-0006's AI-auditability norm.

### Negative

- Some processor concepts (validate-then-mutate, mechanical inverse generation, batch-with-rollback) are duplicated between `processor.rs` and `asset_command.rs`. The duplication is bounded: only the `set_field_path_vec` helper shape is shared; full unification is deferred.
- A second operation-log implementation must stay consistent with the scene one in invariants (cursor semantics, dirty flag). This is a maintenance burden until a future unification.

## References

- [ADR-0005](./0005-scene-asset-bsn-aligned-reusable-scene-model.md) — Scene Asset as the BSN-Aligned Reusable Scene Model (identity model, LocalId, relationships).
- [ADR-0006](./0006-authoring-first-roadmap-after-bsn-migration.md) §Normative Rules — "use the typed command pipeline or define why a new command surface is required".
- [CONTEXT.md](../../CONTEXT.md) — Scene Asset, Scene Asset Authoring Mode, Operation Log terminology.
- `docs/sddk/project-asset-browser-and-scene-asset-authoring/design.md` §3 Decision D1, §5 (`asset_command.rs` module design).
- `crates/editor-core/src/command.rs`, `processor.rs`, `operation_log.rs` — the scene pipeline this decision deliberately mirrors but does not reuse.
