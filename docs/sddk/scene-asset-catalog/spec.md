# Spec: Scene Asset Catalog

> Change: `scene-asset-catalog` · Phase: sddk-spec · Path: A-lite

## §1. Spec Metadata

- **Change:** `scene-asset-catalog`
- **Phase:** spec
- **Source proposal:** [`docs/sddk/scene-asset-catalog/proposal.md`](../scene-asset-catalog/proposal.md)
- **Source explore:** [`docs/sddk/scene-asset-catalog/explore-report.md`](../scene-asset-catalog/explore-report.md)
- **Authoritative references:**
  - [ADR-0005 — Scene Asset as the BSN-Aligned Reusable Scene Model](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md) (§Scene Asset Catalog)
  - [Spec: SceneAssetDocument + SceneInstance + BSN IR (Fase 0)](../scene-asset-document/spec.md)
  - `crates/editor-core/src/scene_asset.rs` (`SceneAssetRole`, `SceneAssetDocument` projection source)
  - Bevy issue #23637 — BSN editor infrastructure roadmap (asset catalog = item 4)
  - Bevy PR #23648 — runtime `bsn_asset_catalog` (diverged by design — not a dependency)

---

## §2. Capability: `scene-asset-catalog`

A Project-level metadata index of every `SceneAssetDocument` registered in the editor. The catalog holds a **projection** of document-level fields (`asset_id`, `logical_path`, `role`, `current_version`) plus catalog-specific fields (`tags`, `created_at`, `updated_at`). It is the discovery layer that resolves `SceneInstance.asset_ref` (path → entry) and flags dangling references. Bodies stay in `SceneAssetDocument`; the catalog never holds them.

### Requirement: catalog-lifecycle

`register` / `unregister` / `update_version` keep three synchronized indices (`asset_id → entry`, `logical_path → asset_id`, `role → asset_id set`) and reject invalid state.

#### Scenario: S1 — Empty catalog has zero entries and zero warnings

**Given** a fresh `SceneAssetCatalog::new()`

**When** `list_all()` and `validate_invariants()` are called

**Then** `list_all()` returns an empty `Vec`
- AND `validate_invariants()` returns an empty `Vec<CatalogWarning>`
- AND `broken_references(["anything"])` returns a `Vec` containing `"anything"` (catalog is empty, everything is missing)

#### Scenario: S2 — Register an entry and look it up by id and path

**Given** a `SceneAssetCatalogEntry { asset_id: "id_1", logical_path: "assets/player", role: SceneAssetRole::Actor, current_version: 1, tags: vec![], created_at: "...", updated_at: "..." }`

**When** `register(entry)` returns `Ok(())`

**Then** `get("id_1")` returns `Some(&entry)` (entry fields all preserved)
- AND `resolve_path("assets/player")` returns `Some("id_1")`
- AND `list_all().len() == 1`

#### Scenario: S3 — Duplicate `asset_id` fails with `DuplicateAssetId`

**Given** two `SceneAssetCatalogEntry` values sharing `asset_id: "id_1"` and different `logical_path` values (`"assets/a"` and `"assets/b"`)

**When** the first `register` succeeds and the second `register` is called

**Then** the second call returns `Err(CatalogError::DuplicateAssetId { id: "id_1".to_string() })`
- AND the catalog still contains exactly 1 entry
- AND `get("id_1").logical_path == "assets/a"` (first write wins)

#### Scenario: S4 — Logical paths are normalized before duplicate check

**Given** a `SceneAssetCatalogEntry` with `logical_path: "Assets/Player/"`

**When** `register(entry)` succeeds

**Then** `resolve_path("assets/player")` returns the entry's `asset_id` (the path was normalized to `"assets/player"`)
- AND registering a second entry with `logical_path: "assets/player"` (already-normalized form) returns `Err(CatalogError::DuplicateLogicalPath { path: "assets/player".to_string() })`

#### Scenario: S5 — `unregister` returns the entry and clears indices

**Given** a registered entry with `asset_id: "id_1"` and `logical_path: "assets/player"`

**When** `unregister("id_1")` is called

**Then** it returns `Ok(entry)` whose fields match the registered entry
- AND `get("id_1")` returns `None`
- AND `resolve_path("assets/player")` returns `None`
- AND `list_all().is_empty()`
- AND calling `unregister("id_1")` again returns `Err(CatalogError::NotFound { id: "id_1".to_string() })`

#### Scenario: S6 — `list_by_role` filters correctly

**Given** a catalog with 3 entries: 2 with `role: SceneAssetRole::Actor`, 1 with `role: SceneAssetRole::Ui`

**When** `list_by_role(SceneAssetRole::Actor)` is called

**Then** the returned `Vec` has exactly 2 entries
- AND all returned entries have `role == SceneAssetRole::Actor`
- AND `list_by_role(SceneAssetRole::Level)` returns an empty `Vec`

### Requirement: catalog-validation

`broken_references` and `validate_invariants` surface structural problems as data (never as panics).

#### Scenario: S7 — `broken_references` identifies missing `asset_id`s

**Given** a catalog containing entries `id_1` and `id_2`

**When** `broken_references(["id_1", "id_missing", "id_2"])` is called

**Then** it returns `vec!["id_missing".to_string()]`
- AND the returned order matches the input's filter order
- AND calling `broken_references(["id_1", "id_2"])` (all present) returns an empty `Vec`

#### Scenario: S8 — Invalid `logical_path` is rejected

**Given** a `SceneAssetCatalogEntry` with `logical_path: ""` (empty) or with `logical_path: "   "` (whitespace only)

**When** `register(entry)` is called

**Then** it returns `Err(CatalogError::InvalidPath { reason: "..." })` (the `reason` is a non-empty human-readable string)
- AND no entry is added to the catalog
- AND `validate_invariants()` does NOT report a phantom entry for the rejected path

### Requirement: catalog-persistence-shape

`SceneAssetCatalog` and `SceneAssetCatalogEntry` round-trip through `serde_json` without losing fields.

#### Scenario: S9 — Catalog serde round-trip preserves all entries

**Given** a catalog with 3 entries covering mixed roles (`Actor`, `Ui`, `Level`), mixed tags (`["enemy", "boss"]`, `[]`, `["menu"]`), mixed versions (1, 3, 7), and distinct `created_at`/`updated_at` strings

**When** the catalog is serialized to JSON via `serde_json::to_string` and deserialized back via `serde_json::from_str`

**Then** `list_all()` on the deserialized catalog returns 3 entries
- AND for every entry, `asset_id`, `logical_path`, `role`, `current_version`, `tags`, `created_at`, `updated_at` all equal the original
- AND `resolve_path(<each original path>)` returns the same `asset_id` for each original
- AND `list_by_role(<each original role>)` returns the same count as before

### Requirement: catalog-versioning

`update_version` advances `current_version` and refreshes `updated_at`.

#### Scenario: S10 — `update_version` increments and updates `updated_at`

**Given** a registered entry with `current_version: 1`, `created_at: "T0"`, `updated_at: "T0"`

**When** `update_version("id_1", 2)` is called and the returned time `T1` satisfies `T1 > T0`

**Then** `get("id_1").current_version == 2`
- AND `get("id_1").updated_at` strictly compares greater than the original `"T0"` (lexicographic or numeric per the type's ordering convention — see §5 Q1)
- AND `get("id_1").created_at` is unchanged (still `"T0"`)
- AND calling `update_version("id_1", 1)` (no-op or downgrade) returns `Err(CatalogError::InvalidVersion { from: 1, to: 1 })` — versions MUST be monotonic non-decreasing

---

## §3. Out-of-Scope Behaviors

The following are NOT part of this change:

1. **OPFS persistence** — no `catalog.json` I/O, no `load_catalog`/`save_catalog` functions, no `assets/` directory body I/O.
2. **Commands / undo / `processor.rs` integration** — no `Command` enum variants; `register`/`unregister`/`update_version` are direct method calls, not command types.
3. **Frontend** — no React components, no hooks, no inspector changes, no broken-reference badge.
4. **Scene Instance override resolution** — Fase 3 territory (resync, rebind, orphaned/stale handling).
5. **`bsn!` codegen changes** — Fase 1 owns `bsn_codegen.rs`; this change does not touch it.
6. **`EntityTemplate` → `SceneAssetDocument` migration** — separate change; catalog indexes docs that already exist.
7. **`SceneAssetDocument` body I/O** — catalog holds metadata only; individual `<asset_id>.asset.json` files are saved/loaded in a future change.
8. **Sync with Bevy runtime's `bsn_asset_catalog` (PR #23648)** — different concept, different lifecycle (see explore-report §1).
9. **`asset_id` minting format specification** — `mint_asset_id()` exists but its exact format (`uuid v7` vs fallback `id_<unix_ms>_<8hex>`) is the design decision in §5 Q2.

---

## §4. Acceptance Criteria

1. New module `crates/editor-core/src/scene_asset_catalog.rs` exposes `SceneAssetCatalog`, `SceneAssetCatalogEntry`, `CatalogError`, `CatalogWarning`, `mint_asset_id()`.
2. `lib.rs` adds exactly two lines: `pub mod scene_asset_catalog;` and `pub use scene_asset_catalog::{SceneAssetCatalog, SceneAssetCatalogEntry, CatalogError, CatalogWarning, mint_asset_id};`.
3. **No existing source file other than `lib.rs` is modified** (catalog is purely additive).
4. `crates/editor-core/src/scene_asset.rs` is unchanged (verified by `git diff --stat`).
5. All 10 scenarios (S1–S10) have passing tests in `crates/editor-core/tests/scene_asset_catalog.rs`.
6. `cargo build -p editor-core --target wasm32-unknown-unknown` succeeds.
7. `cargo test -p editor-core` passes 11 new catalog tests with zero regressions.
8. `validate_invariants()` is callable at any time and never panics; it returns `Vec<CatalogWarning>`.

---

## §5. Open Questions for Design

1. **`role_index` keying strategy** — `SceneAssetRole` (Fase 0, `scene_asset.rs:43`) derives `Eq` but not `Ord`/`Hash`. The spike constraint forbids editing `scene_asset.rs`. Proposal §Internal Data Layout chose `&'static str` discriminant (`role_key()` match). **Confirm:** is a `&'static str` key the right call for `role_index`, or should the design add an additive `Hash + Ord` derive to `SceneAssetRole` and key directly on the enum? Adding the derives is non-breaking but still edits Fase 0 code.
2. **`CatalogWarning` code strings** — `validate_invariants()` emits warnings with `code: String` (e.g. `"duplicate_in_role"`, `"orphaned_index"`). **Confirm:** the exact code vocabulary — must remain stable for future Phase persistence to index them, so the design must lock the strings before implementation.
3. **`from_entries` fail-fast vs collect-all** — `SceneAssetCatalog::from_entries(Vec<SceneAssetCatalogEntry>)` on a list containing duplicates: should it return the first `Err` it hits, or collect all errors into a `Vec<CatalogError>` and return them together? **Confirm:** the error-reporting style for batch construction; tests in S4 assume fail-fast, but collect-all is friendlier for migration scenarios (e.g. loading an old `catalog.json` with hand-edited duplicates).
