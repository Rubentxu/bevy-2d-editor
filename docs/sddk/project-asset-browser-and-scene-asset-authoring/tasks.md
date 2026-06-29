# Tasks: Project Asset Browser + Scene Asset Authoring

> Change: `project-asset-browser-and-scene-asset-authoring` · Phase: sddk-tasks · Path: A-full
> Source: [`./proposal.md`](./proposal.md) · [`./spec.md`](./spec.md) · [`./design.md`](./design.md)
> ADRs: [0005](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md) · [0006](../../adr/0006-authoring-first-roadmap-after-bsn-migration.md) · [0007](../../adr/0007-separate-asset-command-surface.md) · [0008](../../adr/0008-path-based-scene-asset-opfs-layout.md)
> Scope guard: out = Scene Instance placement, Override/Resync Workbench, Validation Center, live Bevy preview, `.bsn` I/O, CRDT/plugins, hierarchical drag-drop reparenting in asset.

---

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~1500–1900 total (split across 3 PRs); PR1 ~250, PR2 ~600, PR3 ~500 |
| 400-line budget risk | High (per PR — each slice is reviewable but borders the budget; PR2 = `asset_command.rs` + bridge) |
| Chained PRs recommended | Yes — 3-PR chain (design §13 ADR candidates + proposal `Chained PR Recommendation`) |
| Suggested split | PR1 Rust persistence+catalog OPFS → PR2 Rust WASM bridge+`AssetCommand` → PR3 Frontend Browser+Authoring+E2E |
| Delivery strategy | `ask-on-risk` (per launch plan) |
| Chain strategy | `stacked-to-main` (each PR merges to main in order; PR2 base = main@PR1, PR3 base = main@PR2) |
| All 21 spec scenarios traced | yes (mapping in Phase 4 verification task) |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | OPFS path layout + `ProjectMetadata.scene_assets` + `SCENE_ASSET_CATALOG`/`SCENE_ASSET_DOC` thread-locals + `load_project` orphan detection | PR1 | base = `main`; pure Rust; gate = `cargo test -p editor-core` (no WASM bridge yet) |
| 2 | New `asset_command.rs` (enum + processor + log) + all `#[wasm_bindgen]` functions + body-first/catalog-second `save_scene_asset` | PR2 | base = `main@PR1`; pure Rust; gate = `cargo check --target wasm32-unknown-unknown` + `cargo test` |
| 3 | Engine bridge, services/hooks, `ProjectAssetBrowser`, `AssetAuthoringView`, `AssetUnsavedChangesDialog`, `App.tsx` `editorMode` flag, Playwright E2E | PR3 | base = `main@PR2`; gate = `just check` + `npx playwright test project-asset-browser.spec.ts` |

### Entropy Note (per entropy-sdd)

`AssetCommand` introduces a deliberate parallel module (ADR-0007); connascence with `Command` is **Name + Type only** (similar surface names), bounded to one `set_field_path_vec` helper shape. `ProjectMetadata.scene_assets` is purely additive (`#[serde(default)]`); H(`Δ_existing`) ≈ 0. Frontend panels are greenfield (no coupling to existing `HierarchyPanel`/`InspectorPanel` per design Approach). Predicted DQS drop = low; acceptable per proposal entropy envelope.

---

## Phase 1: Rust persistence + catalog holder (PR1, base `main`)

- [ ] 1.1 Wire ADR-0005/0006/0007/0008 in module doc comments of `crates/editor-core/src/persistence.rs` and `crates/editor-core/src/lib.rs`.
- [ ] 1.2 RED: write failing test for `asset_path("characters/player")` → `"assets/characters/player.asset.json"` (spec S18) in `crates/editor-core/src/persistence.rs` `#[cfg(test)]`.
- [ ] 1.3 GREEN: add `pub const ASSETS_DIR: &str = "assets"` + `pub fn asset_path(lp: &str) -> String` to `persistence.rs`.
- [ ] 1.4 RED: write test parsing a `project.json` without `scene_assets` (spec S17) — `Vec` defaults empty, no warning.
- [ ] 1.5 GREEN: extend `ProjectMetadata` with `#[serde(default)] scene_assets: Vec<SceneAssetCatalogEntry>`; update `Default` impl.
- [ ] 1.6 Add `SCENE_ASSET_CATALOG` + `SCENE_ASSET_DOC` thread-locals + `with_asset_catalog[_mut]` helpers in `crates/editor-core/src/lib.rs` (mirror `SCENE_REGISTRY`).
- [ ] 1.7 Extend `load_project` in `lib.rs` to rebuild catalog from `project.scene_assets`; for each entry, check `js_exists(asset_path(lp))` and push `CatalogWarning{code:"orphaned_index", asset_id, logical_path}` (spec S16) — **keep entry, never silent delete**.
- [ ] 1.8 Add `crates/editor-core/tests/asset_persistence.rs` covering S4, S5, S6, S7, S8, S17, S18 (create/dup-reject/rename/duplicate/delete + back-compat + path shape).
- [ ] 1.9 Add `crates/editor-core/tests/asset_load.rs` covering S16 (orphan→warning) and S19 (catalog survives across calls without `project.json` write).

## Phase 2: Core command + WASM bridge (PR2, base `main@PR1`)

- [ ] 2.1 RED: write `AssetCommand` deserialization test asserting `tag="type"`, `rename_all="PascalCase"`, `SetComponentValue.field_path: Vec<String>` (spec S14 + D2).
- [ ] 2.2 GREEN: define `AssetCommand` enum (AddEntity/RemoveEntity/RenameEntity/AddComponent/RemoveComponent/SetComponentValue/Batch) + `AssetCommandError` in new `crates/editor-core/src/asset_command.rs`; reference ADR-0007 in module doc.
- [ ] 2.3 RED: write tests for `AddEntity` apply/inverse (S13), `SetComponentValue` field path set (S14), and `dispatch_asset_command` does NOT mutate `SCENE_DOC` (S10).
- [ ] 2.4 GREEN: implement `AssetProcessor::apply` + mechanical inverse generation table per design §5; add `set_field_path_vec` helper (sibling of `processor::set_field_path`).
- [ ] 2.5 RED: write `RenameEntity` inverse test asserting swap — `old_name = prior_actual_name`, `new_name = prior_requested_name`; round-trip undo/redo restores both values.
- [ ] 2.6 GREEN: implement `RenameEntity` inverse — capture entity.name pre-mutation as `old_name`, swap to make forward/inverse pairs mechanical.
- [ ] 2.7 RED: write `RemoveEntity` test capturing full entity; add `ponytail:` comment marking dangling-relationship refs as **deferred debt to Validation Center** (Capability 4) — never silently cleaned in this change.
- [ ] 2.8 GREEN: implement `RemoveEntity` inverse = `AddEntity` with captured `{local_id, name, local_path, components}`.
- [ ] 2.9 Implement `AssetOperationLog` (new_const/record/undo/redo/can_undo/can_redo/`is_dirty()`) in `asset_command.rs`; add `ASSET_OPERATION_LOG` thread-local + `with_asset_log[_mut]` in `lib.rs`.
- [ ] 2.10 Implement `dispatch_asset_command` / `undo_asset` / `redo_asset` / `get_asset_log_state` `#[wasm_bindgen]` fns in `lib.rs` (mirror `dispatch_command`); does NOT call `mark_dirty()`.
- [ ] 2.11 Implement `create_scene_asset` / `rename_scene_asset` (file move) / `duplicate_scene_asset` (`_2` collision suffix) / `delete_scene_asset` / `list_scene_assets(role_filter)` WASM fns in `lib.rs`; reference ADR-0008 in `asset_path` doc.
- [ ] 2.12 Implement `open_scene_asset` (loads body into `SCENE_ASSET_DOC`, resets log) / `close_scene_asset` / `get_asset_document_json` / `get_scene_asset_catalog_json` WASM fns.
- [ ] 2.13 Implement `save_scene_asset`: (1) serialize doc, (2) write body file, (3) bump `current_version` in catalog entry, (4) write `project.json`, (5) clear `ASSET_OPERATION_LOG` dirty (spec S15).
- [ ] 2.14 Add `crates/editor-core/tests/asset_command.rs` covering S10, S13, S14, S15.
- [ ] 2.15 Extend `crates/editor-core/tests/scene_asset_catalog.rs` with role-filter + default-all tests (spec S2, S3).
- [ ] 2.16 REFACTOR: extract shared `set_field_path_vec` helper shape doc between `processor.rs` and `asset_command.rs` (one comment, no logic unification per ADR-0007).

## Phase 3: Frontend + E2E (PR3, base `main@PR2`)

- [ ] 3.1 Append `window.*` bindings for every new WASM fn from design §6 to `frontend/src/engine-bridge.ts` (append-only block, mirror PR2 multi-scene block).
- [ ] 3.2 Create `frontend/src/services/scene-assets.ts` with typed wrappers; copy uses **Scene Asset** vocabulary (S21 guard: no prefab/EntityTemplate/blueprint/archetype strings).
- [ ] 3.3 Create `frontend/src/hooks/useSceneAssets.ts`: catalog state, `assetDoc`, `dispatch`/`undo`/`redo`/`save`, `dirty` from `get_asset_log_state`.
- [ ] 3.4 Create `frontend/src/components/ProjectAssetBrowser.tsx`: role-filter `<select>` (default `all`, S3), empty-state message (S1), row per entry with create/rename/duplicate/delete/open actions.
- [ ] 3.5 Create `frontend/src/components/AssetAuthoringView.tsx`: entity list + read-only relationships display + component editor; dispatches `AssetCommand`; save button.
- [ ] 3.6 Create `frontend/src/components/AssetUnsavedChangesDialog.tsx` per design D4 (title `Unsaved Scene Asset Changes`; testids `asset-unsaved-save-btn` / `…-discard-btn` / `…-cancel-btn`).
- [ ] 3.7 Modify `frontend/src/App.tsx`: add `editorMode: 'scene' | 'asset-authoring'` + `activeAssetLogicalPath`; render `<AssetAuthoringView/>` replacing canvas area in authoring mode; "Back to Scene" → check `dirty`, render dialog if true.
- [ ] 3.8 Create `frontend/tests/project-asset-browser.spec.ts` covering S1 (empty state), S3 (default filter), S9 (reload survival: create→reload→asset present), S11 (back-to-scene restores previous scene), S12 (dirty-guard dialog blocks), S20 (no Bevy preview of asset).
- [ ] 3.9 Add S21 terminology-guard spec block: Playwright DOM scan asserts no `/prefab|EntityTemplate|Entity Template|template|blueprint|archetype/i` (word-boundary, case-insensitive) in asset UI text.

## Phase 4: Verification

- [ ] 4.1 `cargo check -p editor-core --target wasm32-unknown-unknown` (PR2 gate).
- [ ] 4.2 `cargo test -p editor-core` (PR1+PR2 gates).
- [ ] 4.3 `just check` + `just test` (PR3 gate).
- [ ] 4.4 `cd frontend && npx playwright test project-asset-browser.spec.ts` (PR3 gate).
- [ ] 4.5 Verify 21-scenario coverage: S1–S9, S10–S21 traceable to tests in tasks 1.8, 1.9, 2.1, 2.3, 2.5, 2.7, 2.14, 2.15, 3.8, 3.9.
