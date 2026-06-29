# Design: Project Asset Browser + Scene Asset Authoring

> Change: `project-asset-browser-and-scene-asset-authoring` · Phase: sddk-design · Path: A-full
> Source proposal: [`./proposal.md`](./proposal.md) · Source spec: [`./spec.md`](./spec.md) · Source explore: [`./explore-report.md`](./explore-report.md)
> Authoritative refs: [ADR-0005 §Identity/§Roles/§Versioning](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md), [ADR-0006 §Normative Rules](../../adr/0006-authoring-first-roadmap-after-bsn-migration.md), [CONTEXT.md](../../../CONTEXT.md)
> Base: `main` @ `1a459cb` · No code edits in this phase (design artifact only).
> Verified against actual source: `persistence.rs`, `scene_asset_catalog.rs`, `scene_asset.rs`, `command.rs`, `processor.rs`, `operation_log.rs`, `lib.rs`, `document.rs`, `frontend/src/App.tsx`, `engine-bridge.ts`, `opfs-bridge.ts`, `components/UnsavedChangesDialog.tsx`.

---

## §1. Result Contract

| Field | Value |
|-------|-------|
| **status** | `success` |
| **executive_summary** | Turns the 21 spec scenarios into an implementable architecture: new `asset_command.rs` mirroring the proven `Command`/`OperationLog`/`processor` seam but keyed on `LocalId`; three new thread-locals (`SCENE_ASSET_CATALOG`/`SCENE_ASSET_DOC`/`ASSET_OPERATION_LOG`) mirroring `SCENE_REGISTRY`/`SCENE_DOC`; additive `ProjectMetadata.scene_assets` + `assets/<logical_path>.asset.json` layout with body-first/catalog-second write order; purpose-built `ProjectAssetBrowser` + `AssetAuthoringView` under a new `editorMode` flag; and a dirty-guard reusing the `UnsavedChangesDialog` shape. All four spec-owned design questions resolved decisively with codebase evidence. |
| **approach** | Thread-local asset catalog + document + log holders; isolation authoring mode via separate `SCENE_ASSET_DOC` (never writes `SCENE_DOC`); body-first OPFS persistence. |
| **key_decisions** | 8 (see §3) |
| **files_affected** | 1 new Rust module, 3 modified Rust files, 7 new frontend files, 4 modified frontend files, 4 new test files → **12 new, 7 modified, 0 deleted** |
| **testing_strategy** | Unit (Rust persistence/catalog/command/load) + Playwright E2E; strict TDD seams defined per §8 |
| **adr_candidates** | 2 (see §13) |
| **spec_questions_resolved** | 4/4 (see §12) |
| **next_recommended** | `sddk-tasks` |
| **risks** | See §10 |

---

## §2. Architecture Overview

The change is **bridge + persistence + UI**. No new domain modeling — the substrate (`SceneAssetDocument`, `SceneAssetCatalog`, `SceneInstance`, `BsnIr`) is complete and tested. The design adds the missing runtime holders, the WASM surface, the OPFS layout, and the React views.

```
                  ┌─────────────────────── Frontend (React) ───────────────────────┐
  ProjectAssetBrowser ──open──▶ AssetAuthoringView (editorMode='asset-authoring')
        │ list/create/                 │ dispatch AssetCommand / undo / redo / save
        │ rename/dup/delete            ▼
        │                      services/scene-assets.ts ──▶ window.* bindings
        │                              │                        │
        ▼                              ▼                        ▼
  useSceneAssets ◀──────────── engine-bridge.ts ◀──── wasm_bindgen surface (lib.rs)
                                                                      │
        ┌────────────── Rust thread-locals (lib.rs) ─────────────────┤
        SCENE_ASSET_CATALOG  SCENE_ASSET_DOC  ASSET_OPERATION_LOG     │
        (mirror of              (active body)  (per-asset undo log)    │
         SCENE_REGISTRY/SCENE_DOC)                                     │
              │                  │                │                    │
              ▼                  ▼                ▼                    │
        SceneAssetCatalog   asset_command::*   AssetOperationLog       │
        (3-index BTreeMap)  AssetProcessor                            │
              │                                                        │
              ▼                                                        ▼
        ProjectMetadata.scene_assets ◀──▶ assets/<logical_path>.asset.json (OPFS)
```

**Two independence guarantees** (spec S10, S20): (1) `dispatch_asset_command` mutates only `SCENE_ASSET_DOC`; `SCENE_DOC`/`SCENE_REGISTRY`/`OPERATION_LOG` are never touched. (2) `rebuild_preview_world` reads only `SCENE_DOC`; in authoring mode the canvas area is replaced by `AssetAuthoringView`, so no asset document ever reaches the Bevy preview.

---

## §3. Architecture Decisions

### Decision D1 — New `AssetCommand` surface, separate module + separate log

**Choice**: A new `asset_command.rs` owning `AssetCommand`, `AssetCommandError`, `AssetCommandResult`, `AssetProcessor`, and `AssetOperationLog` (parallel to `command.rs`/`operation_log.rs`/`processor.rs`), keyed on `LocalId`.
**Alternatives considered**: (a) Reuse `Command` via a `StableId`↔`LocalId` adapter; (b) generic `OperationLog<C, D>`.
**Rationale**: ADR-0006 §Normative Rules ("define why a new command surface is required") — `SceneAssetDocument` is a different document (`LocalId` identity, `relationships`-based hierarchy, no `parent` field). Routing through `Command` needs a value adapter that is itself a `StableId`/`LocalId` bug surface (the exact Godot editable-children fragility class ADR-0005 rejects). A concrete parallel type is cheaper than a generic refactor and keeps the scene pipeline untouched. Entropy note: this is a *deliberate boundary seam*, acceptable per proposal entropy envelope.

### Decision D2 — `field_path: Vec<String>` (resolves spec Q1)

**Choice**: `Vec<String>`, JSON-serialized as an array.
**Alternatives considered**: dotted `String` (what scene `Command::SetComponentField` uses).
**Rationale**: (1) Spec S14 pins the observable shape to `field_path: ["translation"]`. (2) The asset module *already* uses `Vec<String>` for `ExposedProperty.field_path` and `SceneAssetRelationship.field_path` (`scene_asset.rs:101,93`) — module-local consistency wins over mirroring the scene command. (3) `Vec<String>` is unambiguous for field names containing `.` and maps 1:1 to Bevy component field addressing. The divergence from scene `Command`'s dotted string is intentional and contained inside the asset module.

### Decision D3 — Keep existing `mint_asset_id()` format (resolves spec Q2)

**Choice**: Use the existing `mint_asset_id()` unchanged (`id_<unix_millis>_<8hex>`).
**Alternatives considered**: revise to ULID/UUID/`asset_<…>`.
**Rationale**: `mint_asset_id()` exists and is tested (`scene_asset_catalog.rs:232`); `validate_invariants()` already enforces `starts_with("id_")` (`:203`) and existing tests assert that invariant. The ID is opaque, stable, and collision-resistant enough (millis + per-platform entropy: `js_sys` on WASM, atomic counter elsewhere). Revising would break the documented invariant and existing assertions for zero benefit. The `asset_` prefix is *not* required by any spec scenario.

### Decision D4 — Dirty-guard behavior + copy (resolves spec Q3)

**Choice**: Intercept "Back to Scene" when `ASSET_OPERATION_LOG` is dirty; show a dedicated `AssetUnsavedChangesDialog` (reuses the `UnsavedChangesDialog` overlay/action shape, scene-agnostic copy).
**Alternatives considered**: (a) reuse `UnsavedChangesDialog` verbatim — rejected, its copy hardcodes "Scene {sourceName}"; (b) auto-save — rejected, violates explicit-save intent and spec S15 ordering.
**Rationale**: spec S12 fixes the *block* ("dialog appears naming the unsaved changes … mode remains asset-authoring until discard/save/cancel"); design fixes the *copy*.

**Exact behavior**: "Back to Scene" handler reads `get_asset_log_state().dirty`. If `dirty===true`, render dialog and abort the mode switch. Three actions:
- **Save and Leave** → `save_scene_asset()` (body-first/catalog-second) → on success clear log dirty + `close_scene_asset()` + `editorMode='scene'`.
- **Discard and Leave** → `close_scene_asset()` (drops `SCENE_ASSET_DOC` + resets `ASSET_OPERATION_LOG`) + `editorMode='scene'`. No file write.
- **Cancel** → dismiss dialog; stay in authoring mode.

**Default copy (English, exact domain terms)**:
- Title: `Unsaved Scene Asset Changes`
- Body: `Scene Asset **{logicalPath}** has {unsavedCount} unsaved edit(s). Save before leaving authoring mode?`
- Buttons: `Save and Leave` (primary, `data-testid="asset-unsaved-save-btn"`), `Discard and Leave` (danger, `data-testid="asset-unsaved-discard-btn"`), `Cancel` (`data-testid="asset-unsaved-cancel-btn"`).

### Decision D5 — Defer `SceneAssetDocument → SceneDocument` projection (resolves spec Q4)

**Choice**: Projection is **out of scope** for this change (spec §6/S20). The future seam is reserved.
**Alternatives considered**: ship a one-way projection now for live preview.
**Rationale**: a projection feeding `rebuild_preview_world` is the Godot-editability fragility surface in disguise (asset `LocalId` ↔ scene `StableId`). Spec §6 explicitly defers it and mandates "one-way only, MUST NOT write back".
**Future seam definition**: when added, it MUST be a pure function `project_asset_to_scene_document(&SceneAssetDocument) -> SceneDocument` in a **new** module `asset_projection.rs`, producing ephemeral `StableId`s (e.g. `format!("preview_{}", local_id)`), never persisted, never edit-back. It will be consumed only by a follow-up preview change, registered behind `editorMode === 'asset-authoring' && previewEnabled`. This change lands *no* file for it — only this documented contract.

### Decision D6 — Path-based OPFS layout; catalog index in `ProjectMetadata`

**Choice**: `assets/<logical_path>.asset.json`; `ProjectMetadata.scene_assets: Vec<SceneAssetCatalogEntry>` (no separate `catalog.json`).
**Alternatives considered**: ID-based filenames + `catalog.json`.
**Rationale**: proposal Decision 1 — debuggability + OPFS browsability outweigh rename-as-file-move cost; the catalog is already small and serializes cleanly (`SceneAssetCatalogEntry` derives `Serialize`).

### Decision D7 — Isolation authoring mode, one asset at a time

**Choice**: `editorMode: 'scene' | 'asset-authoring'`; opening an asset swaps `SCENE_ASSET_DOC` (leaving `SCENE_DOC` intact) and switches mode; canvas area replaced by `AssetAuthoringView`.
**Alternatives considered**: side-by-side split; modal overlay.
**Rationale**: proposal Decision 3; single thread-local holder = one asset at a time (acceptable for Hito 2).

### Decision D8 — Catalog CRUD ≠ commands; dedicated WASM functions

**Choice**: create/rename/duplicate/delete are dedicated `#[wasm_bindgen]` functions mirroring `scene_create`/`scene_rename`; only entity/component mutations are `AssetCommand`s.
**Alternatives considered**: model CRUD as commands.
**Rationale**: spec §3 Requirement `asset-command-surface` ("Catalog CRUD MUST NOT be AssetCommands"); CRUD operates on the catalog + OPFS, not on the document body, so it has no meaningful per-body inverse in the same log.

---

## §4. Data Model Changes

### `crates/editor-core/src/persistence.rs`

```rust
/// Subdirectory containing SceneAssetDocument bodies.
pub const ASSETS_DIR: &str = "assets";

/// Resolve OPFS path for a Scene Asset body: `assets/<logical_path>.asset.json`.
/// `logical_path` MUST be already-normalized (segments joined by '/').
pub fn asset_path(logical_path: &str) -> String {
    format!("{}/{}.asset.json", ASSETS_DIR, logical_path)
}
```
`ProjectMetadata` gains one additive field (back-compat via `#[serde(default)]`, matching the existing `schemas`/`active_scene` precedent at `persistence.rs:27-33`):

```rust
#[serde(default)]
pub scene_assets: Vec<SceneAssetCatalogEntry>,
```
`Default::default()` sets it to `Vec::new()` (satisfies S17). Re-export `SceneAssetCatalogEntry` into `persistence`'s scope via the existing `scene_asset_catalog` `pub use`.

### Thread-local holders (added in `lib.rs`, mirroring `SCENE_REGISTRY`/`SCENE_DOC`/`OPERATION_LOG` at `lib.rs:84-87,163-168`)

```rust
thread_local! {
    static SCENE_ASSET_CATALOG: RefCell<Option<SceneAssetCatalog>>
        = const { RefCell::new(None) };
    static SCENE_ASSET_DOC: RefCell<Option<SceneAssetDocument>>
        = const { RefCell::new(None) };
    static ASSET_OPERATION_LOG: RefCell<AssetOperationLog>
        = const { RefCell::new(AssetOperationLog::new_const()) };
}
// + with_asset_catalog / with_asset_catalog_mut helpers (clone of with_registry[_mut])
```
Catalog is rebuilt from `ProjectMetadata.scene_assets` via the existing `SceneAssetCatalog::from_entries(...)` (`scene_asset_catalog.rs:63`) on `load_project`.

---

## §5. Rust Module Design — `asset_command.rs` (new)

### Enum (mirrors `Command` serde convention `#[serde(tag="type", rename_all="PascalCase")]`, `command.rs:17-18`)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum AssetCommand {
    AddEntity {
        local_id: String,        // LocalId inner; minted client-side or by caller
        name: String,
        local_path: String,
        #[serde(default)]
        components: Vec<ComponentInstance>,
    },
    RemoveEntity { local_id: String },
    RenameEntity {
        local_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_name: Option<String>,
        new_name: String,
    },
    AddComponent {
        local_id: String,
        type_id: String,
        #[serde(default)]
        values: serde_json::Value,
    },
    RemoveComponent { local_id: String, type_id: String },
    SetComponentValue {            // spec S14 variant name
        local_id: String,
        type_id: String,
        field_path: Vec<String>,   // D2: array, not dotted
        value: serde_json::Value,
    },
    Batch { label: String, commands: Vec<AssetCommand> }, // parity w/ Command::Batch
}
```
`local_id` is carried as `String` at the wire boundary for trivial JSON; the processor wraps/unwraps `LocalId`. Field names are snake_case to match the existing `Command` convention.

### Result + error (mirror `CommandResult`/`CommandError`)

```rust
#[derive(Debug, Error)]
pub enum AssetCommandError {
    #[error("entity not found: {0}")] EntityNotFound(String),
    #[error("duplicate local_id: {0}")] DuplicateLocalId(String),
    #[error("component not found: {0}")] ComponentNotFound(String),
    #[error("field not found: {0:?}")] FieldNotFound(Vec<String>),
    #[error("batch failed at {index}: {source}")] BatchFailed { index: usize, #[source] source: Box<AssetCommandError> },
    #[error("json error: {0}")] JsonError(String),
}
pub struct AssetCommandResult { pub inverse: AssetCommand, pub snapshot: SceneAssetDocument }
```

### Inverse generation table (processor: validate-then-mutate, mechanical inverse — same contract as `processor.rs:179`)

| Forward | Inverse (captured pre-state) |
|---------|------------------------------|
| `AddEntity` | `RemoveEntity { local_id }` |
| `RemoveEntity` | `AddEntity { local_id, name, local_path, components }` (full captured entity). **Note:** relationships referencing the removed entity are NOT mutated (relationships are read-only in this cut, §11); dangling refs are a future Validation Center concern. |
| `RenameEntity` | `RenameEntity { old_name: actual_old, new_name: actual_old_swapped }` |
| `AddComponent` | `RemoveComponent { local_id, type_id }` |
| `RemoveComponent` | `AddComponent` with captured values (or self-inverse if absent — matches `processor.rs:246-253`) |
| `SetComponentValue` | `SetComponentValue` with old value at `field_path` |
| `Batch` | `Batch { label:"inverse", commands: reversed_inverses }` with rollback-on-failure (matches `processor.rs:304-327`) |

`field_path` navigation uses a `Vec<String>`-based `set_field_path_vec` (sibling of `processor::set_field_path` but no `.`-splitting).

### `AssetOperationLog` (concrete, mirrors `OperationLog` shape, `operation_log.rs:44-158`)

`struct AssetOperationLog { entries: Vec<AssetLogEntry>, cursor: isize, max_size }` with `new_const()`, `record(envelope, inverse)`, `undo(&mut SceneAssetDocument)`, `redo(...)`, `can_undo`/`can_redo`/`get_log_size`/`get_cursor`/`clear`, plus `is_dirty()` (dirty = `cursor < entries.len()-1` after any `record` since last save-clear). `dirty` is the seam for spec S12/S15.

### lib.rs integration

`mod asset_command;` added (`lib.rs:11` block). `dispatch_asset_command` mirrors `dispatch_command` (`lib.rs:264`): parse envelope → `SCENE_ASSET_DOC.with` → `asset_command::apply` → `ASSET_OPERATION_LOG.record` → return `AssetCommandResult` JSON. It does **not** call `mark_dirty()` (no scene preview involvement — satisfies S20). `undo_asset`/`redo_asset` mirror `undo`/`redo`. Save order in `save_scene_asset`: **(1)** serialize `SCENE_ASSET_DOC`, **(2)** `js_save_file(asset_path(logical_path))`, **(3)** on success update `ProjectMetadata.scene_assets` + bump `current_version` + write `project.json`, **(4)** clear `ASSET_OPERATION_LOG` dirty. This matches the existing `save_scene` body-first→metadata order (`lib.rs:1219-1241`) and satisfies spec S15.

---

## §6. WASM API Contract

| Function | Args | Returns | Error behavior |
|----------|------|---------|----------------|
| `create_scene_asset` | `name: String, role: String` | `String` (JSON `SceneAssetCatalogEntry`) | `Err(JsValue)` with `DuplicateLogicalPath` / `InvalidPath` message (S5). Normalizes `logical_path`; writes body + updates catalog. |
| `rename_scene_asset` | `asset_id: String, new_logical_path: String` | `String` (new entry JSON) | `DuplicateLogicalPath` / `NotFound`. File-move + catalog update + `current_version += 1` (S6). |
| `duplicate_scene_asset` | `asset_id: String, suggested_name: String` | `String` (new entry JSON) | collision → `…_2` suffix (S7). New `asset_id` via `mint_asset_id()`; byte-equal body copy. |
| `delete_scene_asset` | `asset_id: String` | `()` | `NotFound`. Deletes body file + catalog entry (S8). |
| `list_scene_assets` | `role_filter: Option<String>` | `String` (JSON `Vec<SceneAssetCatalogEntry>`) | `None` → all; `"actor"` → role-filtered (S2/S3). |
| `open_scene_asset` | `asset_id: String` | `String` (body JSON) | loads body into `SCENE_ASSET_DOC`, resets `ASSET_OPERATION_LOG`. `Err` if body missing. |
| `close_scene_asset` | — | `()` | drops `SCENE_ASSET_DOC`, resets log (no write). |
| `get_asset_document_json` | — | `String` (body JSON) | `Err` if none open. |
| `get_scene_asset_catalog_json` | — | `String` (JSON `Vec<entry>`) | never errors (empty list). |
| `dispatch_asset_command` | `cmd_json: String` (envelope) | `String` (JSON `AssetCommandResult`) | `AssetCommandError` message. Mutates only `SCENE_ASSET_DOC`. |
| `undo_asset` / `redo_asset` | — | `String` (snapshot JSON) | `OperationLogError` message. |
| `get_asset_log_state` | — | `String` (`{size,can_undo,can_redo,cursor,dirty}`) | never errors. `dirty` is the S12 seam. |
| `save_scene_asset` | — | `String` (path written) | body-first/catalog-second (S15); clears dirty on success. |

All are `#[wasm_bindgen]`; catalog mutation fns are `async` (OPFS) like `save_scene`. Error contract: `Result<_, JsValue>` with `JsValue::from_str(&err.to_string())`, identical to existing bridge functions.

---

## §7. Frontend Design

### `editorMode` (App.tsx)
New state `editorMode: 'scene' | 'asset-authoring'` + `activeAssetLogicalPath: string | null`. Render branch:
- `'scene'` → existing layout.
- `'asset-authoring'` → `<AssetAuthoringView …/>` replaces the `.canvas-container` (canvas stays mounted, untouched — S20). `SceneTabs` hidden or disabled; a "Back to Scene" button in `TopBar`/`AssetAuthoringView` triggers the D4 dirty-guard flow.

### `components/ProjectAssetBrowser.tsx` (new)
Props: `entries`, `roleFilter`, `onCreate(name,role)`, `onRename(id,newPath)`, `onDuplicate(id)`, `onDelete(id)`, `onOpen(id)`. Renders: role-filter `<select>` (default `all`, S3), empty-state message when `entries.length===0` (S1), a row per entry (name, role badge, actions). All visible copy uses **Scene Asset** terminology (S21).

### `components/AssetAuthoringView.tsx` (new)
Purpose-built entity list (+ read-only relationships display) + component editor. Dispatches `AssetCommand` via `services/scene-assets.ts`; undo/redo/save buttons bound to `useSceneAssets`. Exposed relationships are **read-only** (no drag-drop reparenting — §11).

### `services/scene-assets.ts` (new)
Typed wrappers over `window.*` bindings (one per §6 function), returning parsed JSON. Mirrors `services/scenes.ts` style.

### `hooks/useSceneAssets.ts` (new)
Catalog state (`entries`, `refresh`), actions (create/rename/duplicate/delete/open/close), asset-doc state (`assetDoc`, `dispatch(cmd)`, `undo`, `redo`, `save`), and `dirty` derived from `get_asset_log_state`. Mirrors `useScenes`/`useSceneState` composition.

### `engine-bridge.ts` (modify)
Append `window.*` bindings for every §6 function in the existing `(window as any).x = (...) => wasm.x(...)` block (`engine-bridge.ts:101-110`).

### `components/AssetUnsavedChangesDialog.tsx` (new)
Per D4 — same overlay/`data-testid` shape as `UnsavedChangesDialog` (`UnsavedChangesDialog.tsx:1-41`), scene-agnostic copy.

---

## §8. OPFS Persistence Algorithm

| Operation | Steps (OPFS = async `js_*` helpers, `lib.rs:697-761`) |
|-----------|-------------------------------------------------------|
| **create** | normalize `logical_path` → `validate_logical_path` → check uniqueness via catalog → `mint_asset_id()` → build empty `SceneAssetDocument{version:1}` → write `asset_path(lp)` body → `register` into `SCENE_ASSET_CATALOG` → write `project.json` (body-first). On any failure after body write: best-effort delete body + do not register. |
| **rename** | validate new path uniqueness → read old body → write new `asset_path(new)` → delete old `asset_path(old)` → `catalog.register`/`unregister`+re-add (or extend `update_version`) with `current_version+1` → write `project.json`. Rename = file-move (D6/S6). |
| **duplicate** | mint new `asset_id` → derive unique `logical_path` (suffix `_2`, `_3`…) → write new body byte-equal to source (S7) → `register` → write `project.json`. |
| **delete** | `catalog.unregister(asset_id)` → delete `asset_path(lp)` → write `project.json`. Missing file is not fatal (idempotent). |
| **save** | serialize `SCENE_ASSET_DOC` → write body → bump `current_version` in catalog entry → write `project.json` → clear log dirty (S15). |
| **load_project** (extend existing `lib.rs:1145`) | after scenes load: `let catalog = SceneAssetCatalog::from_entries(project.scene_assets)?` → for each entry, `js_exists(asset_path(lp))`: if missing push `CatalogWarning{ code:"orphaned_index", asset_id, logical_path }` and **keep** the entry (S16, never silent delete) → store catalog in `SCENE_ASSET_CATALOG` → return warnings (surfaced to UI; full aggregation deferred to Validation Center). |
| **orphan detection** | only on `load_project`; bodies load lazily on `open_scene_asset`. |

---

## §9. Test Design (strict TDD seams; scenario coverage)

| PR | Test file | Scenarios | Key seams asserted |
|----|-----------|-----------|--------------------|
| 1 | `tests/asset_persistence.rs` | S4,S5,S6,S7,S8,S17,S18 | `asset_path`, `ProjectMetadata` back-compat (S17 = old json parses), create/rename/dup/delete roundtrip, path-shape (S18) |
| 1 | `tests/asset_load.rs` | S16,S19 | `load_project` orphan→`CatalogWarning{orphaned_index}` (S16); catalog survives across calls without `project.json` write (S19) |
| 2 | `tests/asset_command.rs` | S10,S13,S14,S15 | `AssetProcessor::apply`/inverse per variant; `undo`/`redo`; `SCENE_DOC` untouched by `dispatch_asset_command` (S10); save clears dirty (S15) |
| 2 | extend `tests/scene_asset_catalog.rs` | S2,S3,S9 | `list_by_role` filter, default-all, reload-survival |
| 3 | `frontend/tests/project-asset-browser.spec.ts` | S1,S3,S11,S12,S20,S21,S9 | empty state (S1); default filter all (S3); back-to-scene restores (S11); dirty-guard dialog (S12); no Bevy preview of asset (S20); forbidden-terms DOM scan (S21); reload survival (S9) |

**TDD seam rule**: every processor/catalog/persistence function is pure (takes `&mut doc`/`&mut catalog`, returns `Result`), so unit tests drive it without WASM. WASM fns are thin wrappers tested via the existing `window.*` Playwright path. `dirty` is a first-class read (`get_asset_log_state`) so S12/S15 are assertable without UI.

Build gates: `cargo check -p editor-core --target wasm32-unknown-unknown`; `cargo test -p editor-core`; `just check` + `just test`.

---

## §10. Risks / Mitigations

| Risk | L | Mitigation |
|------|---|-----------|
| Catalog↔file divergence on partial write | M | Body-first/catalog-second (§8); load-time orphan detection (S16) never silent-deletes. |
| `AssetCommand` logic duplication vs `Command` | M | Separate module keeps scene pipeline untouched; share only the `set_field_path_vec` helper shape; unification is a *future* refactor, not now (D1). |
| UI copy regresses to prefab/template | L | S21 Playwright DOM scan; CONTEXT.md terms enforced; copy fixed in D4. |
| `ProjectMetadata` back-compat | L | `#[serde(default)]` (D6); S17 test. |
| `RemoveEntity` leaves dangling relationship refs | L | Relationships read-only in this cut (§11); deferred to Validation Center. |
| Authoring dirty state lost on accidental back-to-scene | M | D4 dirty-guard. |

---

## §11. Out of Scope / Future Seams

- Scene Instance placement (Cap 2), Override/Resync Workbench (Cap 3), Validation Center (Cap 4).
- Physical `.bsn` import/export/write-back (ADR-0005 step 7).
- Scene Asset Variants / nested assets / plugin asset types.
- Bidirectional UI adapter reusing `HierarchyPanel`/`InspectorPanel` (purpose-built view ships first).
- **Live Bevy preview of the edited asset** — deferred; future seam = `asset_projection.rs::project_asset_to_scene_document`, one-way, never edit-back (D5).
- Relationship drag-drop reparenting inside an asset (read-only display only).
- Collaboration / CRDT / multi-tab sync.

---

## §12. Spec Question Resolutions (§11 of spec.md)

| Q | Decision | Evidence/Why |
|---|----------|--------------|
| **Q1** `field_path` wire format | **`Vec<String>`** (D2) | Spec S14 pins `["translation"]`; asset module already uses `Vec<String>` (`scene_asset.rs:101,93`); unambiguous for dotted field names. |
| **Q2** `mint_asset_id` format | **Keep existing** `id_<millis>_<hex>` (D3) | Already implemented+tested; `validate_invariants` enforces `id_` prefix; opaque/stable; revision adds zero value and breaks the invariant. |
| **Q3** Dirty-guard copy | **Fixed in D4** (English copy + behavior + testids) | Spec S12 fixes the block; design fixes the copy. |
| **Q4** Asset→Scene projection | **Deferred** (D5) | Spec §6/S20 defer it; future seam documented as one-way pure fn in `asset_projection.rs`. |

All 21 spec scenarios are covered by §9 test mapping; no design decision contradicts spec, ADR-0005/0006, or CONTEXT.md terminology.

---

## §13. ADR Candidates

- **`AssetCommand` as a separate command surface** (D1) — hard to reverse (second parallel log/processor), surprising without ADR-0006 context, real trade-off (duplication vs type-safety). → **ADR-007**.
- **Path-based OPFS asset layout + catalog-in-`ProjectMetadata`** (D6) — hard to reverse (persisted file paths), surprising (rename = file move), real trade-off (debuggability vs rename cost). → **ADR-008**.

(The orchestrator creates ADR-007/008 files in MCW Step 1.4.)

---

## §14. Open Questions

None blocking. (All four spec-owned questions resolved in §12.) Implementation-detail choices deferred to `sddk-tasks`: exact `AssetAuthoringView` styling, `useSceneAssets` polling vs event-driven refresh, and whether `editorMode` lives in `App.tsx` or a dedicated `useEditorMode` hook (proposal Decision 6 left this open — either is acceptable; recommend inline in `App.tsx` to match the existing single-component state style).

---

## §15. Standard Envelope

- **status**: `success`
- **executive_summary**: Design converts 21 spec scenarios into implementable architecture: new `asset_command.rs` (enum + processor + `AssetOperationLog`) keyed on `LocalId`, three new thread-locals mirroring the scene holders, additive `ProjectMetadata.scene_assets` + `assets/<logical_path>.asset.json` with body-first/catalog-second persistence, purpose-built `ProjectAssetBrowser`/`AssetAuthoringView` under a new `editorMode`, and a dirty-guard with fixed English copy. All 4 spec design questions resolved with codebase evidence; projection deferred with a documented one-way seam.
- **summary.approach**: Thread-local asset holders + isolation authoring mode + body-first OPFS persistence; new `AssetCommand` surface (separate from `Command`).
- **summary.key_decisions**: 8
- **summary.files_affected**: 12 new, 7 modified, 0 deleted
- **summary.testing_strategy**: Rust unit (persistence/catalog/command/load) + Playwright E2E; strict TDD seams per §9
- **summary.adr_candidates**: 2 (ADR-007 AssetCommand surface; ADR-008 path-based OPFS layout)
- **open_questions**: None blocking (minor UI-state placement deferred to tasks)
- **next_recommended**: `sddk-tasks`
- **risks**: catalog↔file divergence (mitigated: body-first + orphan detection); AssetCommand duplication (mitigated: separate module, future unification); dirty-state loss (mitigated: D4 guard)
- **engram_save_topic_key**: `sddk/project-asset-browser-and-scene-asset-authoring/design`
- **capture_prompt**: false
