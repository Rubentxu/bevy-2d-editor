# Explore Report: scene-asset-catalog

> Change: `scene-asset-catalog` · Phase: explore · Mode: C1

## Status

Completed. Proceed to proposal.

## Context Quality

- **Level:** C1 — strong architectural guidance (ADR-0005, Fase 0 types), but no prior catalog implementation or runtime integration.
- **Evidence Present:** ADR-0005 §Detailed Rules; Fase 0 `SceneAssetDocument`/`SceneInstance` types shipped (`scene_asset.rs`, `scene_instance.rs`); `ProjectMetadata` persistence schema (`persistence.rs`); Bevy issue #23637 + PR #23648 (runtime catalog API); cross-editor patterns (Unity AssetDatabase, Godot ResourceLoader) from Fase 0 explore.
- **Missing Context:** No runtime catalog sync (we diverge by design); no broken-reference surfacing convention yet; no migration path for catalog schema changes.
- **Recommended Effort:** deepen.

## 1. What is the Scene Asset Catalog in Bevy's Roadmap?

Bevy issue #23637 (*"BSN editor infrastructure: write-back, asset catalog, persistent document"*) lists 7 infrastructure items needed for a BSN-based editor, authored by `jbuebler23` based on the Jackdaw editor. Item 4 is the **Asset Catalog** — shared named assets in `.bsn` format, referenced across scenes.

PR #23648 (*"Add BSN asset catalog: load, save, and labeled sub-asset registration"*) implements the runtime side in `bevy_scene2::bsn_asset_catalog`:

| Capability | Description |
|---|---|
| **`load_bsn_assets()`** | Parses `.bsn` text containing named asset definitions, inserts them into `Assets<T>` stores via reflection. Called explicitly (editor startup, level load) — nothing triggers automatically. |
| **`serialize_assets_to_bsn()`** | Serializes named assets from the World back to `.bsn` text with default-diffing. `Handle` fields resolve to asset path strings. Called on "save" — nothing saves automatically. |
| **Labeled sub-asset registration** | `DynamicBsnLoader` scans `.bsn` AST for named asset entries, creates them via `ReflectAsset` + `ReflectDefault`, registers as labeled sub-assets: `asset_server.load("scenes/materials.bsn#PolishedMetal")`. **This** is automatic. |
| **`bevy_asset` additions** | `ReflectAsset::into_loaded_asset()`, `LoadContext::add_loaded_labeled_asset_erased()` for reflection-based asset creation. |

The runtime catalog's job: **named `.bsn` assets → `Handle<T>` resolution at runtime**. It lives inside Bevy's asset server + ECS world.

**How the editor-side catalog differs:** The Bevy 2D Editor's `SceneAssetCatalog` is an **editor-owned index** of `SceneAssetDocument` metadata — `asset_id`, `logical_path`, `role`, `version`, tags. It does NOT parse `.bsn` files, does NOT resolve `Handle<T>`, and does NOT touch the Bevy asset server. The editor catalog is a metadata registry (like Unity's `AssetDatabase` index, not the runtime asset loader). The runtime catalog and the editor catalog are different concepts with different lifecycles — they converge only when/if `.bsn` export lands and the editor's `logical_path` maps to a Bevy asset path. Until then, the editor catalog is standalone.

## 2. What Does the Editor's Scene Asset Catalog Own?

Based on ADR-0005 §Detailed Rules + Fase 0 types (`scene_asset.rs`), the catalog owns:

### Core data structures

```rust
pub struct SceneAssetCatalogEntry {
    pub asset_id: String,
    pub logical_path: String,
    pub role: SceneAssetRole,
    pub current_version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
}

pub struct SceneAssetCatalog {
    entries: BTreeMap<String, SceneAssetCatalogEntry>,  // asset_id → entry
    path_index: BTreeMap<String, String>,                // logical_path → asset_id
    role_index: BTreeMap<SceneAssetRole, Vec<String>>,   // role → Vec<asset_id>
    broken_references: BTreeSet<String>,                  // asset_ids referenced but missing
}
```

### Helper methods

| Method | Signature |
|---|---|
| `register` | `(&mut self, entry: SceneAssetCatalogEntry)` — inserts/upserts, updates all 3 indices |
| `unregister` | `(&mut self, asset_id: &str)` — removes entry + cleans indices |
| `resolve` | `(&self, logical_path: &str) -> Option<&SceneAssetCatalogEntry>` — path → entry |
| `resolve_id` | `(&self, asset_id: &str) -> Option<&SceneAssetCatalogEntry>` — id → entry |
| `list_by_role` | `(&self, role: SceneAssetRole) -> Vec<&SceneAssetCatalogEntry>` |
| `list_all` | `(&self) -> Vec<&SceneAssetCatalogEntry>` |
| `broken_references` | `(&self) -> &BTreeSet<String>` — asset_ids referenced but not in catalog |

### Relationship to existing types

- `SceneAssetDocument` already has `asset_id`, `logical_path`, `role`, `version` (Fase 0). The catalog entry is a **projection** of document-level fields plus catalog-specific fields (`tags`, `created_at`, `updated_at`). The entry does NOT hold the full document.
- `SceneInstance.asset_ref` is `AssetReference(String)` — the catalog resolves it to a catalog entry by `logical_path`.
- Broken references are computed by scanning `SceneInstance.asset_ref` across all scenes and checking membership in the catalog.

## 3. What Does the Editor Do with the Catalog Today?

**Nothing.** The current state:

- `SceneAssetDocument` exists as a type (`scene_asset.rs:54-66`) but is **never instantiated** anywhere in the runtime. No module creates or references it.
- `SceneInstance` exists (`scene_instance.rs:31-40`) with `asset_ref: AssetReference` but is **never created, stored, or resolved**.
- No command references the catalog (`command.rs`, `processor.rs` untouched).
- No persistence: `persistence.rs` has `PROJECT_FILE`, `SCENES_DIR`, `SCHEMAS_DIR`, `ENTITIES_DIR` but no catalog path. `ProjectMetadata` has `scenes`, `schemas`, `templates` lists but no `assets` or `catalog` field.
- No `wasm_bindgen` function exists for catalog operations (search of `lib.rs` confirms no `save_asset`/`load_asset`/`list_assets`).
- The catalog is greenfield — a clean spike with zero coupling to existing runtime code.

## 4. Persistence Model — How Does the Catalog Fit into OPFS?

### Current persistence layout

```
OPFS root
├── project.json              ← ProjectMetadata (version, name, scenes[], schemas[], templates[])
├── scenes/                   ← SceneDocument JSON files
├── schemas/                  ← ComponentSchema JSON files
└── entities/                 ← EntityTemplate JSON files
```

Source: `persistence.rs:10-20`, `lib.rs` OPFS functions.

### Recommended: new `assets/` directory + `catalog.json`

```
OPFS root
├── project.json              ← add `assets: Vec<String>` (asset_ids) for compatibility
├── scenes/
├── schemas/
├── entities/
├── assets/                   ← NEW: SceneAssetDocument JSON files
│   └── <asset_id>.asset.json
└── catalog.json              ← NEW: SceneAssetCatalog (all entries + indices)
```

**Rationale:**
- `catalog.json` is a single Project-level metadata file — mirrors how `project.json` works today. Loading the catalog is one `opfs_load_file` call.
- Individual `SceneAssetDocument` bodies live in `assets/` — mirrors `scenes/`, `schemas/`, `entities/` convention. The catalog is the *index*; the document files are the *content*.
- `project.json` gets a new `assets: Vec<String>` field with `#[serde(default)]` for backward compat (same pattern used for `schemas` and `templates` additions — see `persistence.rs:31-36`).

**Migration:** When loading an old `project.json` without `assets`, the field defaults to empty `Vec`. When `catalog.json` doesn't exist, the catalog is empty. No destructive migration needed — it's purely additive.

**Format:** JSON via `serde`, same conventions as `SceneDocument` JSON (`#[serde(default)]` + `#[serde(skip_serializing_if)]` for clean output).

### Alternative considered: extend `project.json` with full catalog

Embedding the entire catalog inside `project.json` would avoid a second file but couples catalog growth to project metadata I/O. As assets grow (hundreds), `project.json` balloons. The `catalog.json` split is cleaner and matches the existing "one file per concern" convention.

## 5. Cross-Editor Lessons

### Unity — `AssetDatabase` (GUID-based)

Unity's `AssetDatabase` is the closest analog. Every asset gets a GUID (stable across renames, moves, and content changes). References store `GUID + fileID` (internal object ID). `AssetDatabase.LoadAssetAtPath()` resolves paths. `.meta` files store the GUID alongside each asset.

**Pattern we adopt:** stable `asset_id` (our GUID equivalent) + `logical_path` (our path) — dual identity. Renaming a path keeps the `asset_id` stable, so `SceneInstance.asset_ref` (which uses `logical_path` today but could be upgraded to `asset_id`) survives moves.

**Pattern we skip:** Unity's per-asset `.meta` files. We centralize in `catalog.json` instead — simpler for a WASM browser editor with OPFS.

### Godot — `ResourceLoader` (path-based)

Godot uses path-based references exclusively (`"res://player.tscn"`). No GUIDs. If you move a file, all references break unless Godot's editor rewrites them. `.uid` files were added later as a band-aid.

**Pattern we skip:** path-only references. ADR-0005 already mandates opaque `asset_id` as the stable anchor — this is the right call. `logical_path` is human-facing only.

### Defold — no project-level catalog

Defold collections live at arbitrary paths with no central index. References resolve at build time. No catalog = no validation, no broken-reference detection, no "list all actors."

**Pattern we skip:** no central catalog. The Bevy 2D Editor's catalog is explicitly a validation and discovery layer — Defold's approach is the anti-pattern.

### Blender — Asset Libraries + Catalogs

Blender 3.0+ introduced Asset Libraries with catalog metadata (tags, categories). Assets are marked from objects and indexed by UUID. Library overrides track source changes (see Fase 0 explore for details).

**Pattern we adopt:** `tags: Vec<String>` on catalog entries for discovery/filtering. The catalog is the tag index, not the document.

## 6. Risks / Unknowns

1. **`asset_id` minting strategy.** Fase 0 stores `asset_id` as a plain `String` with no minting function. UUID v4 (random, universally unique) vs UUID v7 (time-sortable, lexicographic) vs nano-id? **Mitigation:** the spike should define a `mint_asset_id()` helper. Recommend UUID v7 (time-sortable aids debugging and catalog ordering) but the proposal can decide. No external dependency needed — a small implementation or `uuid` crate feature.

2. **Concurrency.** Single-threaded WASM means no data races on the catalog. But `thread_local!` patterns (like `SCENE_REGISTRY` in `lib.rs:83`) suggest the catalog will live in a `thread_local! RefCell<Option<SceneAssetCatalog>>`. **Mitigation:** follow the existing `with_registry`/`with_registry_mut` pattern. No `Mutex` needed for WASM.

3. **Broken reference surfacing.** How do we tell the user "asset not found"? The catalog tracks `broken_references` but there's no convention for surfacing them. **Mitigation:** spike-only: expose `broken_references()` as a method. Frontend integration (warnings on save, validation badge) is out of scope for Fase 2.

4. **Catalog schema versioning.** If `SceneAssetCatalogEntry` gains fields, old `catalog.json` files must still load. **Mitigation:** use `#[serde(default)]` on all new fields (same pattern as `ProjectMetadata`). No explicit migration framework needed until breaking changes.

5. **Path normalization.** Should `logical_path` be case-sensitive? What about leading/trailing slashes? Windows-style backslashes? **Mitigation:** spike should document the canonical form: lowercase, forward slashes, no leading slash (e.g., `"assets/characters/player"`). Reject non-normalized paths at registration time. This matches `AssetReference` usage in Fase 0 spec (S1: `"assets/player.bsn"`).

6. **Cross-project references.** Should a catalog entry reference assets from other projects? **Mitigation:** No — out of scope. Catalog is Project-local. `logical_path` is relative to the current Project's OPFS root.

7. **Index consistency after crashes.** If the browser crashes between writing `assets/<id>.asset.json` and updating `catalog.json`, the indices and files can diverge. **Mitigation:** spike doesn't need atomicity guarantees — it's a metadata type layer. Production atomicity is a future concern (OPFS transactions, write-ahead log).

## 7. Out of Scope for Fase 2 (Spike Discipline)

Mirroring Fase 0 (`scene-asset-document`) and Fase 1 (`code-export-bsn`) discipline:

- **No `bsn!` codegen changes** — Fase 1 owns that (`bsn_codegen.rs`).
- **No Scene Instance override resolution** — Fase 3 territory (resync, rebind, orphaned/stale handling).
- **No `.bsn` asset file export** — requires Bevy's `.bsn` loader/parser to stabilize.
- **No frontend changes** — no React components, no hooks, no panels.
- **No commands / undo integration** — no `Command` enum variants, no `processor.rs` changes.
- **No migration of existing data** — `EntityTemplate` → `SceneAssetDocument` migration is a separate change.
- **No catalog sync with Bevy runtime's `SceneAssetCatalog`** (PR #23648) — different concept, different lifecycle. The editor catalog is Project-level metadata; the runtime catalog is ECS-world asset management. They converge only when `.bsn` export lands.
- **No broken-reference UI** — the spike tracks `broken_references` but doesn't surface them to users.
- **No `SceneAssetDocument` body persistence** — the spike defines the catalog index type. Individual asset document files (`assets/<id>.asset.json`) are saved/loaded in a future change. The spike may define the path convention but not the I/O functions.

## 8. Context Quality Classification

**C1** — Strong architectural foundation:
- ADR-0005 provides clear ownership boundaries (§Detailed Rules, §Implementation Direction).
- Fase 0 shipped the types the catalog indexes (`SceneAssetDocument`, `SceneInstance`).
- Persistence patterns are well-established (`ProjectMetadata`, `scenes/` directory convention).
- Bevy's runtime catalog (PR #23648) is researched enough to know we diverge.

Missing: no direct implementation experience with a project-level catalog in this codebase, no broken-reference conventions, no `asset_id` minting. These are design decisions for the proposal, not blockers.

---

## Evidence Citations

1. `docs/adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md` — §Identity, §Detailed Rules, §Implementation Direction
2. `crates/editor-core/src/scene_asset.rs:54-66` — `SceneAssetDocument` fields (`asset_id`, `logical_path`, `role`, `version`)
3. `crates/editor-core/src/scene_instance.rs:31-40` — `SceneInstance.asset_ref: AssetReference`
4. `crates/editor-core/src/persistence.rs:22-54` — `ProjectMetadata` schema + `#[serde(default)]` backward compat pattern
5. `crates/editor-core/src/lib.rs:83` — `thread_local! { SCENE_REGISTRY }` pattern for catalog placement
6. `docs/sddk/scene-asset-document/explore-report.md:179` — Fase 0 explicitly deferred "Scene Asset Catalog" as out of scope
7. `docs/sddk/scene-asset-document/archive-report.md:141-143` — "Fase 2 — Scene Asset Catalog" recommendation
8. Bevy issue #23637 — BSN editor infrastructure roadmap (7 items, asset catalog = item 4)
9. Bevy PR #23648 — BSN asset catalog runtime implementation (`load_bsn_assets`, `serialize_assets_to_bsn`, labeled sub-asset registration)
10. Bevy PR #23630 — scene → world serialization rename (context for naming divergence)

---

## Next Step

Proceed to `sddk-propose` with these key proposal points:

1. Introduce `SceneAssetCatalog` + `SceneAssetCatalogEntry` as Rust types in `crates/editor-core/src/scene_asset_catalog.rs` (new module).
2. Three indices: `asset_id → entry`, `logical_path → asset_id`, `role → Vec<asset_id>`.
3. Helper methods: `register`, `unregister`, `resolve`, `resolve_id`, `list_by_role`, `list_all`, `broken_references`.
4. No commands, no UI, no OPFS I/O, no migration — pure type + index layer with serde round-trip tests.
5. Define path convention: lowercase, forward slashes, no leading slash.
6. Define `mint_asset_id()` helper (recommend UUID v7).
7. Wire into `lib.rs` via `pub mod scene_asset_catalog` + `pub use` re-exports.
