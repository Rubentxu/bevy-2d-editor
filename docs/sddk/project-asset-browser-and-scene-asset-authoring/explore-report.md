# Kernel Exploration: Project Asset Browser + Scene Asset Authoring

**Change**: `project-asset-browser-and-scene-asset-authoring`
**Phase**: sddk-explore (Path A-full)
**Date**: 2026-06-29
**Model**: GLM-5.1

---

## Result Contract

| Field | Value |
|-------|-------|
| **status** | `success` |
| **executive_summary** | The Rust substrate for Scene Assets (SceneAssetDocument, SceneAssetCatalog, SceneInstance, BsnIr, scene_instance_overrides, bsn_codegen) is complete and tested — but entirely invisible to users. There are zero `#[wasm_bindgen]` functions for Scene Assets and zero frontend references. This change exposes that substrate as real product workflows: a Project Asset Browser panel, Scene Asset persistence in OPFS, and an isolated Scene Asset Authoring Mode. The architecture is clean; the work is bridge + UI + persistence conventions, not new domain modeling. |
| **context_quality** | C1 — Strong Rust substrate + clear ADRs, but no WASM bridge, no UI precedent, and open OPFS layout decisions. |
| **taxonomy** | `frontend-authoring-workflow` (dominant), `rust-wasm-bridge-surface`, `opfs-persistence-layout`, `project-asset-identity`, `domain-language-preservation` |
| **lenses_used** | architecture-boundary, UX-workflow, persistence/OPFS, command-surface, Bevy-BSN-alignment |
| **skipped_lenses** | Chronos runtime (not a bug), debt-verify (not implementation), impeccable visual craft (design phase) |
| **escalation_needed** | None blocking. Three decisions for `propose`: (1) OPFS path layout for assets, (2) authoring-mode UI strategy (modal vs tab vs split), (3) command surface — reuse Command enum vs new asset-scoped commands. |
| **next_recommended** | `sddk-propose` with A-full path (spec + design + tasks) |
| **risks** | See §Risks below |
| **capabilities_deployed** | Exa web search (3/5 gates answered), direct code reading, ADR/spec cross-reference |
| **model_used** | glm-5.2 |
| **skill_resolution** | `sddk-explore` loaded; `_shared` available; `entropy-sdd` qualitative envelope applied |

---

## Context Quality

- **Level**: C1
- **Evidence Present**:
  - `CONTEXT.md` — full domain language for Scene Asset, Scene Instance, Project Asset Browser, Scene Asset Authoring Mode, all with explicit "Avoid" terms.
  - `docs/adr/0005` — Scene Asset model accepted, implementation steps 1–6 done, step 7 (.bsn) deferred.
  - `docs/adr/0006` — Authoring-first roadmap accepted, Hito 2 sequence normative.
  - `docs/specs/post-bsn-authoring-roadmap.md` — Capability 1 scope, acceptance criteria, out-of-scope.
  - `docs/ROADMAP.md` — Hito 2 planned sequence, change #1 = this change.
  - Rust types: `SceneAssetDocument`, `SceneAssetCatalog` (3-index BTreeMap), `SceneInstance`, `OverridePatch`, `BsnIr`, `bsn_codegen` — all with integration tests.
- **Missing Context**:
  - No WASM bridge functions for Scene Assets exist in `lib.rs`.
  - No OPFS path convention for Scene Assets (`persistence.rs` only defines `scenes/` and `schemas/`).
  - No `ProjectMetadata` field for asset catalog.
  - No frontend component, hook, or service references Scene Assets.
  - No authoring-mode UI precedent in `App.tsx`.
- **Recommended Effort**: `deepen` — the substrate is ready but the bridge/UI/persistence layer is greenfield.

---

## Problem Statement and Scope

**Problem**: The Bevy 2D Editor completed its BSN-aligned Scene Asset architecture in v0.20.0 (ADR-0005 steps 1–6), but none of it is accessible through the UI. Users cannot create, browse, open, edit, or persist Scene Assets. The entire reusable-content capability is Rust-only.

**Scope (from spec Capability 1)**:
- Project Asset Browser panel (list, filter by role, create, rename, duplicate, delete, open).
- Scene Asset persistence under OPFS.
- Isolated Scene Asset Authoring Mode (edit entities/components without mutating the active SceneDocument).
- Stable `asset_id`, normalized `logical_path`, role, version per asset.
- Tests: create/list/open/save/delete + OPFS roundtrip.

**Out of Scope**:
- Scene Instance placement (Capability 2).
- Override/resync UI (Capability 3).
- Variants, nested assets, `.bsn` files, collaboration, plugins.

---

## Current Implementation Map

### Rust substrate (complete, tested)

| Module | What exists | Lines |
|--------|-------------|-------|
| `scene_asset.rs` | `SceneAssetDocument`, `SceneAssetEntity`, `LocalId`, `AssetReference`, `SceneAssetRole` (6 roles), `RelationshipKind`, `ExposedProperty`, `SceneAssetMetadata`, `validate_role()` | 175 |
| `scene_asset_catalog.rs` | `SceneAssetCatalog` (3-index BTreeMap: id, path, role), `SceneAssetCatalogEntry`, `CatalogError`, `CatalogWarning`, `mint_asset_id()`, `normalize_logical_path()`, `validate_logical_path()`, 11 public methods, 12 tests | 315 |
| `scene_instance.rs` | `SceneInstance`, `OverridePatch`, `OverrideStatus` (Active/Orphaned/Stale/Conflict), `patch_status_after_field_rename()` | 55 |
| `scene_instance_overrides.rs` | `effective_values()`, `resync()`, `mint_id_map()`, `reconcile_id_map()`, `validate_overrides()`, `classify_overrides()`, `try_rebind()` — 7 public functions, 11 tests | 1241 |
| `bsn_ir.rs` | `BsnIr`, `BsnIrNode`, `bsn_ir_from_scene_asset()` — one-way lossy projection | 133 |
| `bsn_codegen.rs` | `emit_bsn_source_from_document()` — generates `bsn!`/`bsn_list!` source | — |

### WASM bridge surface (Scene Asset = ZERO)

`lib.rs` exposes `#[wasm_bindgen]` functions for:
- Scene commands: `dispatch_command`, `undo`, `redo`, `get_log_state`
- Scene persistence: `save_scene`, `load_scene`, `list_scenes`, `load_project`
- Multi-scene registry: `scene_create`, `scene_switch`, `scene_switch_commit`, `scene_delete`, `scene_rename`, `list_scenes_extended`, `get_current_scene_id`, `discard_scene_changes`
- Schema registry: `save_schema`, `load_schema`, `delete_schema`, `list_schemas`, `register_schema_from_json`, `unregister_schema`
- Export: `export_code`, `export_dynamic_scene_wasm`

**There are NO `#[wasm_bindgen]` functions for Scene Assets.** No `save_scene_asset`, no `load_scene_asset`, no `list_scene_assets`, no catalog operations. The types are `pub use`-d at the module level but never exposed to JS.

### Frontend (Scene Asset = ZERO)

- `grep` for `scene_asset|SceneAsset|asset_id` across `frontend/` → **no files found**.
- `App.tsx` has no authoring-mode concept — it renders TopBar, SceneTabs, HierarchyPanel, canvas, InspectorPanel, AIAssistantPanel.
- `engine-bridge.ts` exposes `window.*` bindings for scenes/schemas/export but nothing for assets.
- No `ProjectAssetBrowser` component, no `useSceneAssets` hook, no `scene-assets.ts` service.

### Persistence layer

`persistence.rs` defines:
- `PROJECT_FILE = "project.json"`
- `SCENES_DIR = "scenes"` → `scenes/<name>.scene.json`
- `SCHEMAS_DIR = "schemas"` → `schemas/<type_id>.schema.json`
- `ProjectMetadata { version, name, scenes, schemas, active_scene }`

**No `ASSETS_DIR`, no `asset_path()` function, no `scene_assets` list in `ProjectMetadata`.**

---

## Product Gap Map

| User action | Can they do it? | What's missing |
|-------------|-----------------|----------------|
| See all Project assets in one place | ❌ | Project Asset Browser panel |
| Create a Scene Asset from scratch | ❌ | WASM bridge + UI action |
| Open a Scene Asset for editing | ❌ | Authoring mode UI + bridge |
| Edit Scene Asset entities/components | ❌ | Authoring mode (currently only SceneDocument is editable) |
| Save a Scene Asset to persistence | ❌ | OPFS path convention + bridge + `ProjectMetadata` extension |
| Filter assets by role (actor/fragment/level...) | ❌ | Browser UI + catalog `list_by_role` bridge |
| Rename/duplicate/delete a Scene Asset | ❌ | Catalog operations bridge + UI |
| See asset survive page reload | ❌ | `load_project` must load catalog + asset documents |

---

## Affected Areas

| File/Module | Why affected |
|-------------|-------------|
| `crates/editor-core/src/lib.rs` | Add `#[wasm_bindgen]` functions for asset CRUD + catalog ops + load_project extension |
| `crates/editor-core/src/persistence.rs` | Add `ASSETS_DIR`, `asset_path()`, extend `ProjectMetadata` with `scene_assets` list |
| `crates/editor-core/src/scene_asset_catalog.rs` | May need thread-local storage pattern (like `SCENE_REGISTRY`) for catalog state |
| `frontend/src/App.tsx` | Add authoring-mode state, Project Asset Browser panel, mode switching logic |
| `frontend/src/engine-bridge.ts` | Expose new `window.*` bindings for asset operations |
| `frontend/src/components/` (new) | `ProjectAssetBrowser.tsx`, `AssetAuthoringView.tsx` or equivalent |
| `frontend/src/hooks/` (new) | `useSceneAssets.ts` |
| `frontend/src/services/` (new) | `scene-assets.ts` |
| `frontend/tests/` (new) | `project-asset-browser.spec.ts` Playwright E2E |

---

## Approaches

### Approach 1 — Thread-local Catalog + Authoring Mode as SceneDocument Swap (Recommended)

**Description**: Add a thread-local `SCENE_ASSET_CATALOG` and `SCENE_ASSET_DOC` (mirroring the existing `SCENE_REGISTRY`/`SCENE_DOC` pattern). Authoring mode swaps the active document between SceneDocument and SceneAssetDocument. The Project Asset Browser is a new React panel.

- **Pros**:
  - Follows established thread-local + value-swap pattern (low architectural surprise).
  - Reuses existing `ComponentInstance` type inside `SceneAssetEntity` — inspector/hierarchy components can partially work with minimal adaptation.
  - Catalog persistence is additive: extend `ProjectMetadata`, add `ASSETS_DIR`.
  - Isolated authoring mode is a document swap, not a parallel UI tree.
- **Cons**:
  - SceneAssetEntity uses `LocalId` (not `StableId`) and has no `parent` field (hierarchy is in `relationships`). Existing `HierarchyPanel`/`InspectorPanel` expect `SceneDocument.entities` shape — adaptation needed.
  - Thread-local state means only one asset editable at a time (acceptable for Hito 2).
  - Command system (`Command` enum) targets `SceneDocument` — asset edits need either a parallel command surface or an adapter.
- **Effort**: Medium. Bridge + persistence is straightforward; UI adaptation for LocalId/relationships is the main work.

### Approach 2 — Parallel Editor State (Separate React Tree for Assets)

**Description**: Authoring mode renders a completely separate component tree (AssetAuthoringView) with its own hierarchy/inspector components purpose-built for SceneAssetDocument.

- **Pros**: Clean separation, no compromise on either document model.
- **Cons**: Doubles UI component surface; higher implementation cost; slower delivery.
- **Effort**: High.

### Approach 3 — Catalog-Only First, Authoring Deferred

**Description**: Ship only the Project Asset Browser (list/create/delete from pre-seeded JSON assets), defer authoring mode to a follow-up.

- **Pros**: Smallest slice, fastest delivery.
- **Cons**: Violates spec acceptance criteria ("user can open it, edit its entities/components, and save it"). Users can create assets but not edit them — low value.
- **Effort**: Low.

---

## Recommendation

**Approach 1 (A-full)** — Thread-local Catalog + Authoring Mode as document swap.

**Why**: The Rust substrate is complete and the thread-local pattern is already proven for scenes. The main design decisions for `propose` are:
1. **OPFS layout**: `assets/<logical_path>.asset.json` vs `assets/<asset_id>.asset.json` + catalog index.
2. **Command surface**: Do asset edits reuse the `Command` enum (adapted for LocalId) or get a new `AssetCommand` enum?
3. **Authoring mode UX**: Modal overlay, dedicated tab, or panel split — and how the user returns to scene editing.

These are design questions, not blockers. Proceed to `sddk-propose` → `sddk-spec` → `sddk-design` → `sddk-tasks`.

**Include spec + design next**: Yes. This change touches persistence conventions, command surface, and UX workflow — all three benefit from explicit spec scenarios and a design doc before tasks.

---

## Research Gate Answers

### Gate 1 — Unity Prefab Mode

**Source**: [Unity Manual — Edit prefab assets](https://docs.unity3d.com/6000.5/Documentation/Manual/EditingInPrefabMode.html)

**How it separates asset editing from scene editing**:
- Two modes: **In isolation** (hides rest of scene, shows only prefab GameObjects) and **In context** (rest of scene visible but locked).
- **Auto-Save** toggle: when enabled, edits save directly to the prefab asset; when disabled, a save dialog appears on exit.
- **Show Overrides** toggle: visualizes which properties differ from the prefab defaults.
- **Editing Environment**: a configurable scene used as background for isolated editing.
- Transform values cannot be edited in context mode — only in isolation.

**What NOT to copy (terminology pollution)**:
- "Prefab" and "Prefab Instance" → use Scene Asset / Scene Instance.
- "Prefab Variant" → deferred per ADR-0005; do not introduce now.
- "Nested Prefab" → not in scope.
- Unity's `GameObject` / `MonoBehaviour` concepts are engine-specific.
- The editing-environment-as-scene concept is useful but should not introduce a "background scene" Project concept yet.

**What TO copy (mechanism inspiration)**:
- Isolation vs context modes — for Hito 2, isolation is sufficient (context editing is a later enhancement).
- Show Overrides visualization concept → feeds into Capability 3 (Override/Resync Workbench).
- Explicit save vs auto-save toggle — useful UX pattern.

---

### Gate 2 — Godot PackedScene / Inherited Scenes

**Sources**:
- [Godot 4.x PackedScene class docs](https://docs.godotengine.org/en/4.5/classes/class_packedscene.html)
- [Godot issue #77576 — PackedScene inconsistency](https://github.com/godotengine/godot/issues/77576)
- [Godot issue #85372 — editable children signal duplication](https://github.com/godotengine/godot/issues/85372)

**How local modifications are constrained**:
- `PackedScene.instantiate(edit_state)` accepts `GenEditState`: `MODE_MAIN_INHERITED` provides local scene resources for inherited scenes; `MODE_MAIN` is for the main scene only.
- `Node.owner` determines which nodes get saved when packing — only owned sub-nodes are serialized.
- "Editable children" allows modifying inherited nodes but causes known fragility: signal connections duplicate, packing produces inconsistent output, and the feature is explicitly "editor-only."

**Known fragility issues**:
- Editable children + signal connections → "signal already connected" errors ([#85372](https://github.com/godotengine/godot/issues/85372)).
- PackedScene.pack() with editable instances produces different output than the editor's save ([#77576](https://github.com/godotengine/godot/issues/77576)).
- Inherited node changes can silently orphan local modifications.
- Saving scenes with editable instances from outside the editor is explicitly unsupported.

**What to explicitly avoid**:
- **Editable children semantics** — Godot's approach of inlining inherited content into the parent scene's save data is the root cause of its fragility. The Bevy 2D Editor's ADR-0005 model (Scene Instance = reference + patches, NOT deep clone) already avoids this.
- **Owner-based serialization** — Godot's `owner` property determines save scope; our model uses explicit `LocalId` targeting for overrides, which is cleaner.
- **Silent inheritance resolution** — Godot resolves inheritance at load time with limited visibility. Our resync is explicit and non-destructive (active/orphaned/stale/conflict states).

---

### Gate 3 — Defold Collections / Collection Factories

**Source**: [Defold — Collection factories manual](https://defold.com/manuals/collection-factory/)

**How reusable hierarchy identity / spawning works**:
- `collectionfactory.create()` spawns all game objects from a collection file and returns an **ID map table**: keys = collection-local IDs (hashed), values = runtime IDs with a `/collection[N]/` prefix.
- Example: local `/bean` → runtime `/collection0/bean`; local `/shield` → runtime `/collection0/shield`.
- The parent-child relationship exists only in the runtime scene graph — re-parenting never changes the ID.
- Dynamic Prototype option allows swapping the collection file at runtime.

**Why the vocabulary split is confusing**:
- Defold has four overlapping concepts: **GameObject** (single entity), **Collection** (hierarchy of GameObjects), **Factory** (spawns GameObjects), **Collection Factory** (spawns Collections). Users must learn when to use which, and the distinctions are subtle (a Collection can contain one GameObject; a Factory can spawn into any collection).
- **Proxy** adds a fifth concept for loading/unloading entire collections as separate "worlds."

**What to avoid copying**:
- The **four-way concept split** (GameObject/Collection/Factory/Proxy). ADR-0005 explicitly rejects this in favor of one Scene Asset concept with roles.
- The **`/collection[N]/` prefix** naming scheme — our `id_map` uses opaque `StableId` mapping, which is cleaner.
- **Defold's "blueprint vs instance" runtime distinction** — our Scene Instance is an editor concept (reference + patches), not a runtime spawning primitive.

**What IS worth keeping**:
- The **ID map return value** from `collectionfactory.create()` directly inspired our `SceneInstance.id_map: BTreeMap<LocalId, StableId>`. This is already implemented.
- **Dynamic Prototype** concept → maps to "replace asset reference" (future Capability 2 scope).

---

### Gate 4 — Bevy 0.19 BSN Status

**Sources**:
- [Bevy 0.19 release notes](https://bevy.org/news/bevy-0-19/)
- [Bevy PR #23413 — Core scene system, bsn! macro](https://github.com/bevyengine/bevy/pull/23413)
- [Bevy issue #23637 — BSN editor infrastructure roadmap](https://github.com/bevyengine/bevy/issues/23637)
- [Bevy PR #23639 — BSN scene writer](https://github.com/bevyengine/bevy/pull/23639)
- [bevy_scene docs.rs](https://docs.rs/bevy_scene/latest/bevy_scene/index.html)

**What is safe NOW (Bevy 0.19)**:
- `bsn!` and `bsn_list!` macros — fully functional for defining scenes in Rust code.
- Scene composition, patching, inheritance, templates, relationships in code.
- `spawn_scene` / `queue_spawn_scene` with dependency-aware loading.
- Handle path resolution inside `bsn!` (string literal → `AssetServer::load`).
- Our existing `bsn_codegen.rs` generates valid `bsn!` source from `BsnIr` — this is the correct primary target.

**What must remain DEFERRED**:
- **`.bsn` asset loader** — Bevy 0.19 explicitly does NOT ship a first-party `.bsn` file parser/loader. Quote: "Bevy 0.19 does not ship with a `.bsn` asset loader. We're working on it!"
- **BSN write-back** (World → `.bsn` text) — PR [#23639](https://github.com/bevyengine/bevy/pull/23639) is open but not merged as of the 0.19 release.
- **BSN asset catalog** — PR [#23648](https://github.com/bevyengine/bevy/pull/23648) is open.
- **Persistent AST** and **`SceneDocument`/`SceneAssetCatalog` APIs** — planned in [#23637](https://github.com/bevyengine/bevy/issues/23637) but not yet available upstream.
- **Default-diffing** (only emit non-default fields) — part of the write-back PR, not stable.

**Implication for this change**: The editor's SceneAssetDocument JSON remains the source of truth. `bsn!` codegen remains the primary Bevy target. Physical `.bsn` files are explicitly out of scope. This is fully aligned with ADR-0005 step 7 and ADR-0006 deferrals.

---

### Gate 5 — OPFS Project Layout Implications

**Sources**:
- [MDN — Origin Private File System](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system)
- [web.dev — The origin private file system](https://web.dev/articles/origin-private-file-system)

**Key facts**:
- OPFS supports full hierarchical directory structure via `getDirectoryHandle()`.
- Files/directories are private to the origin — not user-visible (no file explorer integration).
- Subject to browser storage quota (`navigator.storage.estimate()`).
- Synchronous access available in Web Workers only; main thread uses async Promise-based API.
- Existing project already uses async OPFS via `frontend/src/opfs-bridge.ts` with `window.opfs_save_file` / `opfs_load_file` / `opfs_list_files` / `opfs_exists` / `opfs_delete_file`.

**Layout implications for Scene Assets**:
Current layout:
```
project.json
scenes/
  <name>.scene.json
schemas/
  <type_id>.schema.json
```

Proposed extension (decision for `propose`):
```
project.json          ← add "scene_assets": ["id_...", ...] or catalog snapshot
scenes/
schemas/
assets/
  <logical_path>.asset.json   ← e.g. assets/characters/player.asset.json
```

**Design tension**: The catalog uses `logical_path` (human-readable, e.g. `characters/player`) as a normalized key. Two options:
1. **Path-based file storage**: `assets/<logical_path>.asset.json` — file path mirrors logical path. Pro: browsable, debuggable. Con: rename = file move.
2. **ID-based file storage + catalog index**: `assets/<asset_id>.asset.json` + catalog in `project.json` or separate `catalog.json`. Pro: rename is cheap (catalog-only update). Con: files not human-discoverable.

**Recommendation for propose**: Option 1 (path-based) for discoverability and simplicity, accepting that rename triggers a file move. The catalog index in `project.json` provides the authoritative `asset_id` → `logical_path` → role mapping.

---

## Risks

1. **HierarchyPanel/InspectorPanel incompatibility**: `SceneAssetEntity` uses `LocalId` (not `StableId`) and stores hierarchy in `relationships` (not `parent` field). Existing UI components expect `SceneDocument.entities` shape. **Mitigation**: Adapter layer or purpose-built asset hierarchy component in `propose`/`design`.

2. **Command surface divergence**: The `Command` enum targets `SceneDocument` with `StableId`. Scene Asset edits target `SceneAssetEntity` with `LocalId`. Reusing the same enum risks type confusion; a new `AssetCommand` enum risks duplicating apply/inverse logic. **Mitigation**: Design decision in `sddk-design`.

3. **Authoring mode state management**: Swapping between SceneDocument and SceneAssetDocument in thread-locals is conceptually clean but the preview world (`rebuild_preview_world`) currently only knows about `SceneDocument`. Authoring mode needs either a separate preview path or a SceneAssetDocument→SceneDocument projection for preview. **Mitigation**: Design decision in `sddk-design`; likely project SceneAssetDocument to a temporary SceneDocument for preview rendering.

4. **Catalog persistence atomicity**: If `project.json` is updated but the asset file write fails (or vice versa), the catalog and files diverge. **Mitigation**: Write asset file first, then update catalog; on load, detect orphan entries.

5. **Domain language regression**: New UI copy might accidentally use "prefab", "template", or "blueprint" instead of "Scene Asset". **Mitigation**: Acceptance criterion explicitly checks terminology; code review gate.

---

## Ready for Proposal

**Yes.** Proceed to `sddk-propose` with Path A-full (spec + design + tasks).

**What the orchestrator should tell the user**:
> Exploration complete. The Rust Scene Asset substrate is fully built and tested but has zero WASM bridge and zero frontend. This change is primarily bridge + persistence + UI work, not new domain modeling. Three design decisions need resolution in propose/design: (1) OPFS path layout for assets, (2) authoring-mode UI strategy, (3) command surface for asset edits. All five research gates answered. Recommend A-full path with spec + design before tasks.

---

## Suggested Artifact Path / Topic Key

- **Artifact path**: `docs/sddk/project-asset-browser-and-scene-asset-authoring/`
- **Topic key**: `architecture/scene-asset-authoring`
- **Engram scope**: `project`

---

## Metrics Placeholders

| Metric | Value |
|--------|-------|
| `files_read` | 14 |
| `external_sources_cited` | 11 |
| `research_gates_answered` | 5/5 |
| `approaches_compared` | 3 |
| `estimated_implementation_effort` | Medium (bridge + persistence + UI adaptation) |
