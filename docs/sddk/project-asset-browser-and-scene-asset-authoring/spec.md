# Spec: Project Asset Browser + Scene Asset Authoring

> Change: `project-asset-browser-and-scene-asset-authoring` · Phase: sddk-spec · Path: A-full
> Source proposal: [`./proposal.md`](./proposal.md) · Source explore: [`./explore-report.md`](./explore-report.md)
> Authoritative references: [ADR-0005 §Identity/§Roles/§Versioning](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md), [ADR-0006 §Normative Rules](../../adr/0006-authoring-first-roadmap-after-bsn-migration.md), [Roadmap Capability 1](../../specs/post-bsn-authoring-roadmap.md), [Spec: scene-asset-catalog](../scene-asset-catalog/spec.md), [Spec: scene-asset-document](../scene-asset-document/spec.md), [Spec: opfs-persistence](../opfs-persistence/spec.md).

---

## §1. Capability Delta Table

| Capability | Status | Summary of change |
|------------|--------|-------------------|
| `project-asset-browser` | **NEW** | Panel + WASM catalog CRUD + filter by role + OPFS persistence of `SceneAssetDocument` bodies. |
| `scene-asset-authoring` | **NEW** | Isolated authoring mode + `AssetCommand` surface + per-asset operation log + back-to-scene. |
| `opfs-persistence` | **MODIFIED** | `ProjectMetadata` gains `scene_assets: Vec<SceneAssetCatalogEntry>`; adds `ASSETS_DIR` + `asset_path()`; body-first/catalog-second write order; load-time orphan detection. |
| `scene-asset-catalog` | **MODIFIED** | Thread-local catalog + document holders; rebuild on `load_project` from `ProjectMetadata.scene_assets`; persistence-through-`ProjectMetadata`. |

---

## §2. NEW Capability: `project-asset-browser`

### Requirement: asset-list-and-filter

The system MUST expose Project-level Scene Assets through a UI surface that lists catalog entries, supports filtering by role, and surfaces an empty state when no assets exist.

#### Scenario: S1 — Empty Project Asset Browser shows the empty state
- GIVEN `ProjectMetadata.scene_assets` is empty
- WHEN the Project Asset Browser panel is rendered
- THEN the list area shows the empty-state message
- AND no `Scene Asset` row is visible
- AND the `Create Scene Asset` action remains enabled.

#### Scenario: S2 — Filter by role returns only matching entries
- GIVEN a catalog with 2 `Actor`, 1 `Level`, 1 `Ui` entries
- WHEN the role filter is set to `actor`
- THEN the list shows exactly 2 rows
- AND every visible row has role `actor`.

#### Scenario: S3 — Role filter `All` is the default and shows everything
- GIVEN a non-empty catalog
- WHEN the panel mounts without an explicit filter
- THEN the filter is `all`
- AND every catalog entry is visible.

### Requirement: asset-catalog-crud

The system MUST provide catalog CRUD (`create`, `rename`, `duplicate`, `delete`) as dedicated WASM functions, never as `AssetCommand`s. CRUD operations MUST round-trip across page reload.

#### Scenario: S4 — Create a new Scene Asset persists a file and a catalog entry
- GIVEN a Project with empty `scene_assets`
- WHEN `create_scene_asset(name="Player", role="actor")` is invoked
- THEN a normalized `logical_path` of `player` is assigned
- AND a unique `asset_id` is minted
- AND the file `assets/player.asset.json` exists in OPFS
- AND `project.json` contains a `SceneAssetCatalogEntry` with matching `asset_id`, `logical_path`, `role`, `current_version: 1`.

#### Scenario: S5 — Create with a duplicate `logical_path` is rejected
- GIVEN an existing asset with `logical_path: "player"`
- WHEN `create_scene_asset(name="Player", role="actor")` is invoked
- THEN the call returns a typed error (`DuplicateLogicalPath`)
- AND no file is written
- AND the catalog is unchanged.

#### Scenario: S6 — Rename moves the file and updates the catalog
- GIVEN an existing asset at `assets/player.asset.json` with `logical_path: "player"`
- WHEN `rename_scene_asset(asset_id, "characters/player")` succeeds
- THEN the file `assets/characters/player.asset.json` exists
- AND the file `assets/player.asset.json` no longer exists
- AND the catalog entry's `logical_path` is `characters/player` and `current_version` increased by 1.

#### Scenario: S7 — Duplicate creates a new asset with a unique id and copies the body
- GIVEN an existing asset A with `logical_path: "player"`
- WHEN `duplicate_scene_asset(A.asset_id)` is invoked with suggested name `player_copy`
- THEN a new asset B is created with a distinct `asset_id`
- AND B's `logical_path` is `player_copy` (or `player_copy_2` if collision)
- AND B's body file exists in OPFS
- AND B's body is byte-equal to A's body at the moment of duplication.

#### Scenario: S8 — Delete removes the file and the catalog entry
- GIVEN an existing asset A
- WHEN `delete_scene_asset(A.asset_id)` is invoked
- THEN the file `assets/<A.logical_path>.asset.json` no longer exists
- AND the catalog no longer contains A's `asset_id`
- AND `resolve_path(A.logical_path)` returns `None`.

### Requirement: asset-reload-survival

Catalog entries and their bodies MUST survive page reload via `load_project`.

#### Scenario: S9 — Catalog and bodies survive reload
- GIVEN 3 created assets persisted to OPFS
- WHEN the page is reloaded and `load_project` runs
- THEN `list_scene_assets(None)` returns exactly 3 entries
- AND `open_scene_asset(<each id>)` returns the same body that was saved
- AND `logical_path`, `role`, `current_version` are unchanged.

---

## §3. NEW Capability: `scene-asset-authoring`

### Requirement: authoring-mode-isolation

Opening a Scene Asset MUST enter `editorMode = "asset-authoring"`. In that mode the active `SceneDocument` MUST NOT be mutated. The previously active scene MUST be restorable.

#### Scenario: S10 — Opening an asset does not mutate the active SceneDocument
- GIVEN a loaded `SceneDocument` with entity `E1`
- AND an opened Scene Asset with entity `A1`
- WHEN `dispatch_asset_command(AddEntity("a2"))` is applied
- THEN the Scene Asset contains `A1` and `A2`
- AND the `SceneDocument` still contains exactly `E1` and no asset-side entities
- AND no `AssetCommand` appears in the scene's `Command` operation log.

#### Scenario: S11 — Back-to-scene restores the previously active scene
- GIVEN a previously active `SceneDocument` (id=`scene_a`)
- AND the editor in `asset-authoring` mode editing asset X
- WHEN the user activates `Back to Scene`
- THEN `editorMode` returns to `"scene"`
- AND `get_current_scene_id()` returns `scene_a`
- AND `scene_a` is unchanged.

#### Scenario: S12 — Dirty-guard blocks leaving authoring mode with unsaved edits
- GIVEN authoring mode with an unsaved `AssetCommand` (dirty bit set)
- WHEN the user attempts `Back to Scene`
- THEN a confirmation dialog appears naming the unsaved changes
- AND the mode remains `asset-authoring` until the user explicitly discards, saves, or cancels.

### Requirement: asset-command-surface

A new `AssetCommand` enum and per-asset operation log MUST own entity and component mutations against `SceneAssetDocument`. Catalog CRUD MUST NOT be `AssetCommand`s (it uses dedicated WASM functions — see S4–S8).

#### Scenario: S13 — `AddEntity` applies and inverts
- GIVEN an empty Scene Asset
- WHEN `dispatch_asset_command(AddEntity { local_id: "a1", name: "A", components: [] })` is applied
- THEN the document has 1 entity
- AND `undo_asset()` reduces the entity count to 0
- AND `redo_asset()` restores the entity.

#### Scenario: S14 — `SetComponentValue` targets a `LocalId` and a field path
- GIVEN a Scene Asset with entity `LocalId("a1")` and a `Transform2D` component
- WHEN `dispatch_asset_command(SetComponentValue { local_id: "a1", field_path: ["translation"], value: {x:1,y:2} })` is applied
- THEN the entity's `Transform2D.values["translation"]` equals `{x:1,y:2}`
- AND `undo_asset()` restores the prior value.

### Requirement: dirty-flag-and-save

The asset log MUST expose `dirty` state; `save_scene_asset` MUST persist the body before updating `project.json`.

#### Scenario: S15 — Save writes body first, then catalog (atomicity)
- GIVEN an open asset with a pending dirty `AssetCommand`
- WHEN `save_scene_asset()` is called
- THEN the asset body file in OPFS reflects the new state
- AND only after the body write succeeds, `project.json` is rewritten with the updated `current_version`
- AND the operation log's `dirty` flag clears.

#### Scenario: S16 — `load_project` flags orphan catalog entries
- GIVEN `project.json` lists asset A but the file `assets/<A.logical_path>.asset.json` is missing
- WHEN `load_project` runs
- THEN the catalog still contains A (no silent delete)
- AND a `CatalogWarning` with code `orphaned_index` and `asset_id = A.asset_id` is emitted.

---

## §4. MODIFIED Capability: `opfs-persistence`

### Requirement: asset-persistence-layout

`ProjectMetadata` MUST gain `scene_assets: Vec<SceneAssetCatalogEntry>` (with `#[serde(default)]` for back-compat). Assets MUST live at `assets/<logical_path>.asset.json`.

#### Scenario: S17 — `ProjectMetadata` with old shape still loads
- GIVEN a `project.json` written before this change (no `scene_assets` field)
- WHEN `load_project` parses it
- THEN parsing succeeds
- AND `scene_assets` defaults to an empty `Vec`
- AND no warning is emitted for the missing field.

#### Scenario: S18 — `asset_path` produces the expected OPFS path
- GIVEN `logical_path = "characters/player"`
- WHEN `asset_path("characters/player")` is called
- THEN the result equals `"assets/characters/player.asset.json"`.

---

## §5. MODIFIED Capability: `scene-asset-catalog`

### Requirement: catalog-thread-local-state

A thread-local catalog holder MUST mirror the `SCENE_REGISTRY` pattern and be rebuilt from `ProjectMetadata.scene_assets` on `load_project`.

#### Scenario: S19 — Catalog survives across WASM calls within a session
- GIVEN a `create_scene_asset` call
- AND a subsequent `list_scene_assets(None)` call
- WHEN both run in the same session
- THEN the second call returns the entry created by the first
- AND no `project.json` save happens between the two calls (catalog is in-memory until `save_*`).

---

## §6. Deferred: live Bevy preview in authoring mode (Decision 7)

A `SceneAssetDocument → SceneDocument` projection for `rebuild_preview_world` carries the Godot-editability fragility risk and is **out of scope for this change**. The first cut renders the entity/component editor only (no live preview). A follow-up change MAY add a one-way projection; it MUST be one-way only and MUST NOT write back to the `SceneAssetDocument`.

#### Scenario: S20 — Authoring mode renders the editor only, no Bevy preview
- GIVEN authoring mode is active for asset X
- WHEN the editor mounts
- THEN the entity list + component editor are visible
- AND no live Bevy preview of asset X is rendered
- AND no `rebuild_preview_world` call uses the `SceneAssetDocument`.

---

## §7. Terminology Guard

UI copy MUST NOT contain the forbidden terms `prefab`, `EntityTemplate`, `Entity Template`, `template` (as a noun for a reusable composition), `blueprint`, or `archetype` (referring to reusable content).

#### Scenario: S21 — DOM contains no forbidden terminology
- GIVEN the Project Asset Browser and Asset Authoring View are visible
- WHEN a Playwright DOM scan runs over the visible text
- THEN no rendered string matches the forbidden-terms regex (case-insensitive, word-boundary).

---

## §8. Non-Goals / Out of Scope

- Scene Instance placement (Capability 2), Override/Resync Workbench (Capability 3), Validation Center (Capability 4).
- Physical `.bsn` import/export/write-back (ADR-0005 step 7 — Bevy loader not stable).
- Scene Asset Variants, nested assets, plugin-provided asset types.
- Bidirectional UI adapter reusing `HierarchyPanel`/`InspectorPanel` for asset editing (deferred; first cut ships purpose-built `AssetAuthoringView`).
- Live Bevy preview of the edited Scene Asset (deferred — see §6).
- Full hierarchy drag-drop reparenting *inside* an asset (read-only display only in first cut).
- Collaboration, CRDT, multi-tab sync.

---

## §9. Acceptance Criteria Checklist

- [ ] Project Asset Browser creates, lists (with role filter), renames, duplicates, deletes, and opens Scene Assets.
- [ ] Each asset has stable `asset_id`, normalized `logical_path`, role, and monotonic `version`.
- [ ] Asset body file at `assets/<logical_path>.asset.json` plus a `SceneAssetCatalogEntry` in `project.json` survive page reload.
- [ ] Opening an asset switches `editorMode` to `asset-authoring` and never mutates the active `SceneDocument`.
- [ ] `Back to Scene` restores the previously active scene; dirty-guard prevents silent loss.
- [ ] `AssetCommand` covers at minimum `AddEntity`, `RemoveEntity`, `RenameEntity`, `AddComponent`, `RemoveComponent`, `SetComponentValue`; `apply` and `inverse` are tested; operation log supports `undo`/`redo`.
- [ ] Catalog CRUD is implemented as dedicated WASM functions, not as `AssetCommand`s.
- [ ] Save is body-first, catalog-second; orphan catalog entries produce a `CatalogWarning`, never a silent delete.
- [ ] Old `project.json` files (no `scene_assets`) parse cleanly via `#[serde(default)]`.
- [ ] Rust authoritative for `logical_path`/role/`asset_id`/orphan validation; frontend soft pre-validation only; Validation Center deferred.
- [ ] No forbidden terminology in UI copy (Playwright DOM scan green).
- [ ] Live Bevy preview of edited Scene Asset is NOT in this change (deferred).

---

## §10. Verification Plan

| Category | Test file / type | Scenarios covered |
|----------|------------------|-------------------|
| Rust unit (persistence) | `crates/editor-core/tests/asset_persistence.rs` | S4, S5, S6, S7, S8, S9, S16, S17, S18 |
| Rust unit (catalog) | extend `crates/editor-core/tests/scene_asset_catalog.rs` | S2, S3, S9, S19 |
| Rust unit (command) | `crates/editor-core/tests/asset_command.rs` | S10, S13, S14, S15 |
| Rust unit (load/orphan) | `crates/editor-core/tests/asset_load.rs` | S16, S17 |
| Playwright E2E | `frontend/tests/project-asset-browser.spec.ts` | S1, S3, S11, S12, S20, S21 |
| Playwright roundtrip | `frontend/tests/project-asset-browser.spec.ts` (reload flow) | S9 |
| Build gate | `cargo check -p editor-core --target wasm32-unknown-unknown`, `cargo test -p editor-core` | all Rust scenarios |

Gate: `just check` and `just test` must stay green. The PR1/PR2/PR3 chain in the proposal (Rust persistence → Rust bridge + `AssetCommand` → Frontend) keeps each step independently reviewable.

---

## §11. Open Questions (design-owned, non-blocking)

1. **`AssetCommand` JSON wire format for `field_path`** — design picks `Vec<String>` vs dotted-string. Spec requires the value to be addressable; format is design.
2. **`mint_asset_id` format** — already exists; the exact `id_<…>` string format remains a design detail (see `scene-asset-catalog/spec.md` §5 Q2).
3. **Dialog copy for the dirty-guard** — UX wording, not behavior. Spec fixes the *block*; design fixes the *copy*.
4. **One-way `SceneAssetDocument → SceneDocument` projection shape** — only relevant if/when live preview is added in a follow-up. Out of scope here.

---

## §12. Result Contract

| Field | Value |
|-------|-------|
| **status** | `success` |
| **executive_summary** | Locks observable behavior for the two new capabilities (`project-asset-browser`, `scene-asset-authoring`) and the two modified capabilities (`opfs-persistence`, `scene-asset-catalog`). 21 Given-When-Then scenarios cover CRUD, role filter, reload survival, authoring-mode isolation, the `AssetCommand` apply/inverse cycle, dirty-guard, body-first/catalog-second atomicity, orphan detection, back-compat with old `project.json`, the deferred-preview boundary, and the terminology guard. Preview in authoring mode is explicitly deferred (Decision 7). |
| **capabilities** | new = 2 (`project-asset-browser`, `scene-asset-authoring`); modified = 2 (`opfs-persistence`, `scene-asset-catalog`) |
| **scenarios_total** | 21 (≥ 14 required) |
| **coverage** | happy_paths: covered · edge_cases: covered (orphan, back-compat, dirty-guard, rename-file-move) · error_states: covered (duplicate path, missing file, OPFS rejection) |
| **next_recommended** | `sddk-design` (sequential, not parallel) — design resolves the 4 non-blocking open questions and the `AssetCommand` JSON wire format |
| **risks** | None new beyond the proposal; preview-projection risk is **deferred**, not removed |
