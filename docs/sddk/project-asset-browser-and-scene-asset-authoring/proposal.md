# Proposal: Project Asset Browser + Scene Asset Authoring

> Change: `project-asset-browser-and-scene-asset-authoring` · Phase: propose · Path: A-full
> Source explore: [`explore-report.md`](./explore-report.md)
> Normative refs: [ADR-0005](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md) §Identity/§Roles/§Versioning, [ADR-0006](../../adr/0006-authoring-first-roadmap-after-bsn-migration.md) §Normative Rules, [Roadmap Capability 1](../../specs/post-bsn-authoring-roadmap.md)
> Base: `main` @ `1a459cb` · No branch, no code edits in this phase.

## Result Contract

| Field | Value |
|-------|-------|
| **status** | `success` |
| **executive_summary** | Exposes the complete-but-invisible Scene Asset Rust substrate as two real product capabilities: a Project Asset Browser (catalog CRUD over WASM + OPFS) and an isolated Scene Asset Authoring Mode. Work is bridge + persistence conventions + UI — no new domain modeling. Six explicit decisions locked below; one open question (asset hierarchy editing depth) flagged for spec. |
| **context_quality** | `C1` → proposal locks the C1 gaps (OPFS layout, command surface, UI mode). |
| **taxonomy** | `frontend-authoring-workflow` (dominant), `rust-wasm-bridge-surface`, `opfs-persistence-layout`, `asset-command-surface`, `domain-language-preservation` |
| **lenses_used** | scope-control, architecture-boundary, persistence-layout, command-surface, UX-workflow, testability |
| **risk_level** | Medium |
| **next_recommended** | `sddk-spec` (then `sddk-design`) — sequential, NOT parallel |
| **capabilities** | new = 2 (`project-asset-browser`, `scene-asset-authoring`); modified = 2 (`opfs-persistence`, `scene-asset-catalog`) |

---

## Intent

ADR-0005 shipped a complete, tested Scene Asset substrate (`SceneAssetDocument`, `SceneAssetCatalog`, `SceneInstance`, `OverridePatch`, `BsnIr`, `bsn_codegen`, `scene_instance_overrides`) — but **none of it is reachable by users**. There are zero `#[wasm_bindgen]` functions and zero frontend references for Scene Assets (explore §WASM bridge, §Frontend). Reusable content is Rust-only.

This change converts that substrate into two visible authoring workflows (Roadmap Capability 1):
1. **Project Asset Browser** — create / list / filter-by-role / rename / duplicate / delete / open Scene Assets, surviving page reload.
2. **Scene Asset Authoring Mode** — open a Scene Asset, edit its entities/components in isolation (without mutating the active SceneDocument), save it back.

User value: a designer can build a reusable actor/fragment/screen/level once and have it persist as a Project asset — the foundation every later Hito 2 capability (instance placement, override workbench, validation) depends on.

## Scope

### In Scope
- OPFS layout + `ProjectMetadata` extension for Scene Asset persistence.
- Thread-local Scene Asset catalog + document holders (mirrors `SCENE_REGISTRY`/`SCENE_DOC`).
- WASM bridge: catalog CRUD, asset body load/save, `AssetCommand` dispatch + undo/redo.
- **Project Asset Browser** React panel (list, role filter, create/rename/duplicate/delete/open).
- **Scene Asset Authoring Mode** (editor-mode flag + document swap + purpose-built entity/component editing view).
- Rust integration tests (persistence roundtrip, catalog CRUD, `AssetCommand` apply/inverse).
- Playwright E2E (create/list/open/edit/save/delete + reload + terminology guard).

### Out of Scope (explicit)
- Scene Instance placement (Capability 2), Override/Resync Workbench (Capability 3), Validation Center (Capability 4).
- Physical `.bsn` import/export/write-back (ADR-0005 step 7 — Bevy loader not stable).
- Scene Asset Variants / inheritance / nested assets.
- Bidirectional UI adapter reusing `HierarchyPanel`/`InspectorPanel` for assets (deferred — see Decision 5).
- Full hierarchy/relationship drag-drop reparenting *inside* an asset (read-only display only).
- Runtime preview rendering of a Scene Asset (preview projection is a design-detail; minimum cut may render nothing or a flat entity list — Decision 7).
- Collaboration / CRDT / plugin-provided asset types.

## Capabilities

> CONTRACT with `sddk-spec`. Existing capability names taken from `docs/sddk/` folders. Research done: `openspec/` does not exist (removed); spec store = `docs/sddk/<change>/spec.md` + `docs/specs/`.

### New Capabilities
- `project-asset-browser`: Project Asset Browser panel + WASM catalog CRUD + OPFS Scene Asset persistence. Requirements: create/list/filter-by-role/rename/duplicate/delete/open; reload-survival; stable `asset_id`, normalized `logical_path`, role, `version`; terminology guard (no prefab/EntityTemplate in UI copy).
- `scene-asset-authoring`: isolated Scene Asset Authoring Mode. Requirements: open asset into an isolated editor mode without mutating the active SceneDocument; edit entities (add/remove/rename) and component field values via a reversible `AssetCommand` surface with its own operation log; save back to OPFS; explicit "back to scene" restoration.

### Modified Capabilities
- `opfs-persistence`: extend `ProjectMetadata` with a Scene Asset catalog index; add `ASSETS_DIR` and `asset_path()`; define the `assets/<logical_path>.asset.json` physical layout; load-time catalog↔file consistency check.
- `scene-asset-catalog`: add a thread-local catalog holder (mirror of `SCENE_REGISTRY`) so catalog state lives across `#[wasm_bindgen]` calls; persistence-on-save behavior (catalog snapshot written through `ProjectMetadata`).

## Approach

**Approach 1 (explore, chosen): Thread-local catalog + Authoring Mode as document swap.**

- Rust: `SCENE_ASSET_CATALOG` + `SCENE_ASSET_DOC` thread-locals (same pattern as `SCENE_REGISTRY`/`SCENE_DOC`). A new `AssetCommand` enum owns entity/component mutations on `SceneAssetDocument`. Catalog CRUD (create/rename/duplicate/delete) = dedicated WASM functions mirroring `scene_create`/`scene_rename`.
- Persistence: additive — `ASSETS_DIR` + `ProjectMetadata.scene_assets`. No change to scene/schema paths.
- Frontend: new `ProjectAssetBrowser` panel + a purpose-built `AssetAuthoringView`. App gains an `editorMode: 'scene' | 'asset-authoring'` flag; opening an asset swaps the active document in thread-locals and switches mode.

**Rejected:**
- *Approach 2 (parallel React tree, separate asset hierarchy/inspector)* — High effort, doubles UI surface; deferred full-feature asset panels to later.
- *Approach 3 (catalog-only first, authoring deferred)* — violates acceptance criterion ("user can open it, edit its entities/components, and save it"); low value.
- *Bidirectional UI adapter* (project `SceneAssetDocument` ↔ `SceneDocument` shape for reuse of `HierarchyPanel`/`InspectorPanel`) — rejected as a **first cut**: `LocalId`↔`StableId` display+edit-back mapping is the exact Godot editable-children fragility class ADR-0005 avoids. Honest separation now; adapter reconsidered once asset panels prove their own needs.

## Explicit Decisions

1. **OPFS path convention** — `assets/<logical_path>.asset.json` (**path-based, not ID-based**). The normalized catalog `logical_path` (e.g. `characters/player`) IS the physical path. Rename = file move + catalog index update (acceptable cost; wins debuggability and OPFS browsability). Catalog index = `ProjectMetadata.scene_assets` (no separate `catalog.json`).
2. **ProjectMetadata / catalog persistence shape** — `ProjectMetadata` gains `scene_assets: Vec<SceneAssetCatalogEntry>` (the serialized catalog: `asset_id`, `logical_path`, `role`, `version`, metadata). On load: reconstruct the in-memory catalog from this list; asset *bodies* load lazily on `open_scene_asset`. Write order on save: **asset body file first, then `project.json` catalog** (catalog never references a file that doesn't exist yet); orphan catalog entries (file missing) detected + surfaced, never silently deleted.
3. **UI mode shape** — `editorMode: 'scene' | 'asset-authoring'` flag in `App.tsx`. Authoring mode is **isolation-style** (Unity Prefab Mode "in isolation", not "in context"): the active scene is hidden, only the Scene Asset is editable. **One asset editable at a time** (thread-local holder — acceptable for Hito 2). Explicit "Back to Scene" affordance restores the previously-active scene tab. Not a modal overlay (too confining), not a side-by-side split (premature).
4. **Command surface shape** — **new `AssetCommand` enum + its own processor + its own operation log** (separate module `asset_command.rs`, separate from `command.rs`). Target: `SceneAssetEntity` / `LocalId`. Justification per ADR-0006 ("define why a new command surface is required"): `SceneAssetDocument` is a *different document* from `SceneDocument` — `LocalId` identity + `relationships`-based hierarchy + exposed properties. Routing it through `Command` would require a value adapter that is itself a StableId/LocalId bug surface. Exposed surface: `dispatch_asset_command` / `undo_asset` / `redo_asset` / `get_asset_log_state`. **Catalog CRUD is NOT commands** — it uses dedicated WASM functions (`create_scene_asset`, …), mirroring `scene_create`/`scene_rename`.
5. **Validation boundaries** — Rust is authoritative for: `logical_path` normalization + uniqueness (`validate_logical_path`), `asset_id` uniqueness (catalog), role validity (`validate_role`), path safety (reject `..`/absolute), and load-time catalog↔file orphan detection. Frontend does only soft pre-validation (non-empty name, role-required-at-create). Project-wide issue aggregation is the **Validation Center (Capability 4, deferred)** — this change surfaces only create/save **blocking** errors, not a health panel.
6. **Frontend shape** — purpose-built minimal `AssetAuthoringView` (entity list + component editor, relationships shown read-only) for the first cut; purpose-built `ProjectAssetBrowser`. Reuse of `HierarchyPanel`/`InspectorPanel` deferred (see Approach). Services/hooks: `services/scene-assets.ts`, `hooks/useSceneAssets.ts`, `hooks/useEditorMode.ts` (or fold into `App`).
7. **Preview in authoring mode** — **open question for spec/design**, not locked here. Minimum cut: authoring mode renders the entity/component editor with **no live Bevy preview** (or a flat projection). A `SceneAssetDocument → SceneDocument` projection for `rebuild_preview_world` is a real design decision with the Godot-fragility risk; spec should state whether preview is in-scope for Capability 1 or deferred. Evidence to resolve: cost of a safe one-way projection vs. value of live preview during asset authoring.

## Integration Plan by Area

### Rust persistence (`crates/editor-core/src/persistence.rs`)
- Add `ASSETS_DIR = "assets"` and `asset_path(logical_path) -> String` → `assets/{logical_path}.asset.json`.
- Extend `ProjectMetadata` with `scene_assets: Vec<SceneAssetCatalogEntry>` (serde `#[serde(default)]` for back-compat with old `project.json`).
- `save_scene_asset(asset_id)`: serialize `SceneAssetDocument` → write file → update `scene_assets` index → (caller) persist `project.json`.
- `load_project`: reconstruct `SCENE_ASSET_CATALOG` from `scene_assets`; detect orphan entries (file missing) → return warnings, keep entry.
- Atomicity rule documented + tested: body-first, catalog-second.

### Rust catalog + bridge (`crates/editor-core/src/scene_asset_catalog.rs`, `lib.rs`, new `asset_command.rs`)
- `scene_asset_catalog.rs`: thread-local `SCENE_ASSET_CATALOG: RefCell<Option<SceneAssetCatalog>>` + `SCENE_ASSET_DOC: RefCell<Option<SceneAssetDocument>>` (mirror `SCENE_REGISTRY`).
- `lib.rs` `#[wasm_bindgen]`: `create_scene_asset(name, role)`, `rename_scene_asset(asset_id, new_logical_path)`, `duplicate_scene_asset(asset_id)`, `delete_scene_asset(asset_id)`, `list_scene_assets(role_filter: Option<String>) -> String`, `open_scene_asset(asset_id) -> String`, `close_scene_asset()`, `get_scene_asset_catalog_json() -> String`, `save_scene_asset()`, `get_asset_document_json() -> String`, `dispatch_asset_command(cmd_json)`, `undo_asset()`, `redo_asset()`, `get_asset_log_state() -> String`.
- `asset_command.rs` (new): `AssetCommand` enum (AddEntity / RemoveEntity / RenameEntity / AddComponent / RemoveComponent / SetComponentValue — minimum for acceptance), `AssetProcessor::apply`/`inverse`, per-asset `AssetOperationLog`.
- `load_project` signature unchanged; behavior extended.

### Frontend (`frontend/src/`)
- `engine-bridge.ts`: add `window.*` bindings for every new WASM function.
- `services/scene-assets.ts`: typed wrappers (catalog CRUD, open/close, command dispatch).
- `hooks/useSceneAssets.ts`: catalog state + actions + dirty tracking; `hooks/useEditorMode.ts` (or inline): `'scene' | 'asset-authoring'`.
- `components/ProjectAssetBrowser.tsx`: list + role filter + create/rename/duplicate/delete/open actions.
- `components/AssetAuthoringView.tsx`: entity list (+ read-only relationships) + component editor; dispatches `AssetCommand`; save action.
- `App.tsx`: `editorMode` state; render `ProjectAssetBrowser` (e.g. docked panel / sidebar entry) and switch main area to `AssetAuthoringView` when in authoring mode; "Back to Scene" affordance.
- `tests/project-asset-browser.spec.ts`: Playwright E2E.

### Tests
- Rust: `crates/editor-core/tests/asset_persistence.rs` (path roundtrip, orphan detection, catalog↔file consistency); `crates/editor-core/tests/asset_command.rs` (`AssetCommand` apply/inverse + log). Run via `just test` / `wasm-pack`.
- Frontend: Playwright — create → list → open → edit entity/component → save → reload → asset present; delete; terminology scan (assert no "prefab"/"EntityTemplate"/"template" text in DOM).

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Catalog↔file divergence on partial write (file ok, `project.json` fails). | Medium | Body-first/catalog-second write order; load-time orphan detection + warnings (never silent delete). |
| `AssetCommand` duplicates apply/inverse logic vs `Command`. | Medium | Separate module keeps scene pipeline untouched; share field-path helpers only; unification is a future refactor, not now. |
| `LocalId`↔`StableId` confusion if preview projection is added. | Medium | Decision 7 keeps preview as an open question; if added, projection is **one-way** (asset→temp SceneDocument), never edit-back. |
| Authoring-mode dirty state lost on accidental "Back to Scene". | Medium | Dirty-guard dialog (reuse `UnsavedChangesDialog` pattern) before leaving authoring mode with unsaved edits. |
| UI copy regresses to prefab/template vocabulary. | Low | Terminology acceptance criterion + Playwright DOM scan; CONTEXT.md terms enforced. |
| `ProjectMetadata` back-compat (old `project.json` without `scene_assets`). | Low | `#[serde(default)]`; existing projects load with empty asset catalog. |

## Rollback Plan

This change is additive at the Rust level (new module + thread-locals + `#[serde(default)]` field) and additive at the frontend level (new components/hooks; `App.tsx` mode flag). Revert:
1. Remove the new WASM functions + `asset_command.rs` + thread-locals from `scene_asset_catalog.rs`/`lib.rs`.
2. Revert `ProjectMetadata` field (old `project.json` files unaffected due to `#[serde(default)]`; orphan `assets/` dir left in OPFS, harmless).
3. Delete frontend components/hooks/bridge bindings; revert `App.tsx` mode flag.
4. Drop the test files.
No data migration needed — Scene Asset files are editor-owned JSON; removing the feature returns the editor to the v0.20.0 "asset substrate invisible" state.

## Dependencies
- Shipped + tested substrate: `SceneAssetDocument`, `SceneAssetCatalog` (incl. `validate_logical_path`, `mint_asset_id`, `validate_role`), `scene_asset.rs`, OPFS bridge (`opfs_save_file`/`opfs_load_file`/`opfs_list_files`/`opfs_exists`/`opfs_delete_file`).
- ADR-0005 §Identity/§Roles/§Versioning; ADR-0006 §Normative Rules (source of truth, terminology, command-pipeline rule).
- `thiserror`/`serde`/`serde_json` (already workspace deps).

## Chained PR Recommendation

**Yes — 3-PR chain** (each scoped to stay reviewable; triggers `chained-pr` policy at >400 LOC):

1. **PR1 — Rust persistence + catalog holder** (`persistence.rs`, `scene_asset_catalog.rs` thread-locals, `ProjectMetadata`). Pure Rust; gate = `just test` (persistence + catalog roundtrip). No UI.
2. **PR2 — Rust WASM bridge + `AssetCommand`** (`lib.rs` functions, new `asset_command.rs`, integration tests). Pure Rust; gate = `just test`. Depends on PR1.
3. **PR3 — Frontend** (`engine-bridge.ts`, services/hooks, `ProjectAssetBrowser`, `AssetAuthoringView`, `App.tsx` mode, Playwright E2E). Depends on PR2.

PR1 and PR2 land before any frontend work; each is independently green on `just check`/`just test`.

## Acceptance Criteria (proposal level)
- [ ] User can create a Scene Asset from scratch (name + role) via the Project Asset Browser.
- [ ] User can open it, edit entities/components, and save — without the active SceneDocument being mutated.
- [ ] Asset appears in the Project Asset Browser after page reload.
- [ ] Asset carries stable `asset_id`, normalized `logical_path`, role, and `version`.
- [ ] Filter by role works; rename/duplicate/delete work; delete removes file + catalog entry.
- [ ] "Back to Scene" restores the previously-active scene; dirty-guard prevents silent loss.
- [ ] No UI copy uses EntityTemplate/prefab/template/blueprint (Playwright DOM scan).
- [ ] Rust tests cover create/list/open/save/delete + OPFS roundtrip + catalog↔file orphan detection.
- [ ] `AssetCommand` apply/inverse + operation-log tests green.
- [ ] `just check` + `just test` green on wasm32.

## Next Recommended Phase

**`sddk-spec`, then `sddk-design` — sequential, NOT parallel.**

Why sequential: design's biggest open questions (the `AssetCommand` surface contract, the persistence atomicity guarantees, the preview-projection decision #7) are *exactly* what spec must lock as observable behaviors first. Running design in parallel risks it diverging from spec's requirement boundaries. Spec writes `docs/sddk/<change>/spec.md` (Given/When/Then for both new capabilities + the two deltas), then design resolves #7 and the `AssetCommand`/adapter internals. Both precede `sddk-tasks`.

## Open Question for Spec
- **Authoring-mode preview scope (Decision 7)**: is live Bevy preview of the edited Scene Asset in-scope for Capability 1, or deferred? Spec must state the boundary. Resolving evidence: cost/risk of a one-way `SceneAssetDocument → SceneDocument` projection for `rebuild_preview_world` vs. the authoring value of live preview. Recommend: **defer live preview to a follow-up**; Capability 1 ships editor-only rendering (entity/component tree) to avoid the projection risk in the first cut.

## Artifact References & Topic Key
- Artifact path: `docs/sddk/project-asset-browser-and-scene-asset-authoring/proposal.md`
- Source explore: `docs/sddk/project-asset-browser-and-scene-asset-authoring/explore-report.md`
- Topic key: `architecture/scene-asset-authoring` (Engram scope: `project`)
- Entropy envelope (qualitative, no CogniCode): introducing a *new* command surface (`AssetCommand`) is a deliberate **boundary seam** — acceptable entropy increase because it isolates a genuinely different document model (`LocalId`/relationships) rather than forcing connascence with the `StableId`-based `Command`. Persistence change is purely **additive** (`#[serde(default)]`), zero connascence impact on existing scene/schema flows. UI is greenfield (no coupling to existing panels in the first cut) — lowest-entropy option per Approach decision.

---

## Standard Envelope

- **status**: `success`
- **executive_summary**: Proposal locks 6 decisions (path-based OPFS layout, `ProjectMetadata.scene_assets` index, isolation authoring mode with one asset at a time, new `AssetCommand` surface, Rust-authoritative validation, purpose-built asset UI). One open question (authoring-mode live preview) flagged for spec. 2 new + 2 modified capabilities. Work = bridge + persistence + UI; 3-PR chain recommended.
- **capabilities**: new = 2 (`project-asset-browser`, `scene-asset-authoring`); modified = 2 (`opfs-persistence`, `scene-asset-catalog`)
- **context_quality**: `C1` (gaps locked here)
- **risk_level**: Medium
- **taxonomy**: `frontend-authoring-workflow`, `rust-wasm-bridge-surface`, `opfs-persistence-layout`, `asset-command-surface`, `domain-language-preservation`
- **next_recommended**: `sddk-spec` (then `sddk-design`, sequential)
- **decisions_made**:
  1. OPFS = `assets/<logical_path>.asset.json` (path-based); catalog index in `ProjectMetadata.scene_assets` (no separate catalog.json).
  2. `ProjectMetadata.scene_assets: Vec<SceneAssetCatalogEntry>`; bodies lazy-loaded on open; save = body-first/catalog-second; orphan detection on load.
  3. `editorMode: 'scene' | 'asset-authoring'`; isolation style; one asset at a time; explicit back-to-scene.
  4. New `AssetCommand` enum + processor + own operation log in `asset_command.rs` (justified per ADR-0006); catalog CRUD = dedicated WASM functions, not commands.
  5. Validation: Rust authoritative (path/role/uniqueness/orphan); frontend soft only; project-wide panel deferred to Validation Center.
  6. Purpose-built `AssetAuthoringView` + `ProjectAssetBrowser`; bidirectional UI adapter deferred.
- **open_questions_for_spec**:
  1. Authoring-mode live preview in-scope for Capability 1 or deferred? (Recommend defer; one-way projection risk.)
- **engram_save_topic_key**: `sddk/project-asset-browser-and-scene-asset-authoring/propose`
- **capture_prompt**: false
