# ADR-0008: Path-Based OPFS Layout for Scene Assets

## Status

Accepted (2026-06-29)

## Context

ADR-0005 gives Scene Assets two identities: an opaque stable `asset_id` and a human-readable `logical_path` (e.g. `characters/player`). ADR-0006 §Normative Rules keeps `SceneAssetDocument` as the editor-owned source of truth and keeps physical `.bsn` files out of scope until Bevy's loader/write-back APIs stabilize. The editor therefore persists Scene Assets as its own editor-owned JSON in OPFS today.

The Project Asset Browser + Scene Asset Authoring change (`project-asset-browser-and-scene-asset-authoring`) needs to decide:

1. What physical OPFS path holds a Scene Asset body — keyed on `asset_id` or on `logical_path`?
2. Where does the catalog index live — a separate `catalog.json`, or embedded in `ProjectMetadata`?
3. What write order guarantees that the catalog never references a body file that does not exist?

`ProjectMetadata` already embeds additive collections with `#[serde(default)]` (the `schemas` and `active_scene` precedent at `persistence.rs:27-33`), so a back-compatible catalog field is feasible without a separate file.

## Decision

We persist Scene Assets using a **path-based** OPFS layout with the catalog embedded in `ProjectMetadata`:

```text
assets/<logical_path>.asset.json          ← SceneAssetDocument body
ProjectMetadata.scene_assets              ← Vec<SceneAssetCatalogEntry>, embedded in project.json
```

Four rules govern the layout:

1. **Path is identity-derived, not ID-derived.** `asset_path(logical_path)` returns `assets/{logical_path}.asset.json` (e.g. `characters/player` → `assets/characters/player.asset.json`). The normalized catalog `logical_path` IS the physical path. Bodies are named for what they are, not for an opaque `asset_id`.

2. **Catalog lives in `ProjectMetadata`.** `ProjectMetadata` gains `scene_assets: Vec<SceneAssetCatalogEntry>` with `#[serde(default)]` for back-compat with `project.json` files written before this change. There is no separate `catalog.json`. Old projects parse cleanly with an empty asset catalog.

3. **Save is body-first, catalog-second.** `save_scene_asset` serializes `SCENE_ASSET_DOC` and writes the body file **before** updating `ProjectMetadata.scene_assets` (and bumping `current_version`) and writing `project.json`. If the body write fails, the catalog is never updated, so the catalog never references a missing file. This mirrors the existing `save_scene` body-first → metadata order (`lib.rs:1219-1241`).

4. **Rename is a file move plus a catalog update.** Renaming a Scene Asset moves the body file from `assets/<old_path>.asset.json` to `assets/<new_path>.asset.json`, then updates the catalog entry's `logical_path` and bumps `current_version`. Orphan catalog entries (body missing on `load_project`) are detected and surfaced as `CatalogWarning { code: "orphaned_index" }` and **kept** — never silently deleted (spec scenario S16).

## Considered Options

### Option A — ID-based filenames plus a separate `catalog.json`

Rejected. Naming bodies `assets/{asset_id}.asset.json` makes the OPFS tree opaque to humans and to external tools, hurting debuggability and browsability. A separate `catalog.json` adds a second file that can drift from `project.json` and from the body files, multiplying the catalog↔file divergence failure modes instead of collapsing them.

### Option B — Path-based layout with catalog embedded in `ProjectMetadata` (chosen)

Chosen. Debuggability and OPFS browsability outweigh the rename-as-file-move cost. The catalog is small and serializes cleanly (`SceneAssetCatalogEntry` derives `Serialize`), and embedding it in `ProjectMetadata` reuses the existing back-compat pattern instead of introducing a new file.

### Option C — Single-file project (all bodies inside one JSON)

Rejected. Cramming all Scene Asset bodies into one `project.json` bloats reads/writes, blocks lazy body loading, and makes partial failures catastrophic. Scene Assets must load lazily on `open_scene_asset`, which requires per-asset files.

## Consequences

### Positive

- The OPFS tree is human-readable: `assets/characters/player.asset.json` is self-describing, browsable in devtools, and greppable.
- Back-compat is automatic: old `project.json` files without `scene_assets` parse via `#[serde(default)]` with no migration step (spec scenario S17).
- Body-first/catalog-second ordering collapses one whole class of divergence (catalog pointing at a missing body) at the write boundary.
- Catalog state mirrors the existing `SCENE_REGISTRY` thread-local pattern and is rebuilt from `ProjectMetadata.scene_assets` on `load_project`.

### Negative

- Rename is a file move, not just a catalog mutation. A crash between body move and catalog update can leave a body file at the old path with a catalog entry pointing at the new path. This is mitigated by body-first/catalog-second ordering within `rename_scene_asset` (write new body, then delete old body, then update catalog) and by load-time orphan detection that keeps orphaned entries rather than deleting them silently.
- Orphan-entry handling is a real ongoing concern: `load_project` must detect bodies missing from disk and surface `CatalogWarning`s, and the future Validation Center (Capability 4) is the proper home for surfacing and resolving them project-wide. The current change detects orphans but defers full aggregation.
- Path-based naming couples the catalog `logical_path` to the filesystem, so `logical_path` normalization and `..`/absolute-path rejection (`validate_logical_path`) are security- and correctness-critical, not cosmetic.

## References

- [ADR-0005](./0005-scene-asset-bsn-aligned-reusable-scene-model.md) — Scene Asset identity (`asset_id` + `logical_path`), roles, versioning.
- [ADR-0006](./0006-authoring-first-roadmap-after-bsn-migration.md) §Normative Rules — editor-owned source of truth; `.bsn` write-back deferred.
- [CONTEXT.md](../../CONTEXT.md) — Scene Asset, Scene Instance, Project Asset Browser, Asset Reference terminology.
- `docs/sddk/project-asset-browser-and-scene-asset-authoring/proposal.md` — Decision 1 (OPFS path convention) and Decision 2 (ProjectMetadata/catalog persistence shape).
- `docs/sddk/project-asset-browser-and-scene-asset-authoring/design.md` §3 Decision D6, §4 (data model), §8 (OPFS persistence algorithm).
- `crates/editor-core/src/persistence.rs` — `ProjectMetadata` additive-field precedent (`#[serde(default)]`).
