# Proposal: Scene Asset Catalog

> Change: `scene-asset-catalog` · Phase: propose · Status: draft

## Intent

Fase 0 shipped `SceneAssetDocument` and `SceneInstance` as inert types with no discovery layer. Fase 2 adds the missing **metadata index**: a `SceneAssetCatalog` that registers every Scene Asset in a Project by `asset_id`, `logical_path`, and `role`, enabling id/path lookups, role-filtered listing, and broken-reference detection. Without it there is no way to resolve a `SceneInstance.asset_ref`, list all actors, or flag a dangling reference. This spike delivers pure Rust types + lookup helpers + serde round-trip tests — no frontend, no commands, no OPFS I/O. Body content stays in `SceneAssetDocument`; the catalog holds metadata only.

## Scope

### In Scope
- New module `crates/editor-core/src/scene_asset_catalog.rs`: `SceneAssetCatalog`, `SceneAssetCatalogEntry`, `CatalogError`, `CatalogWarning`.
- Three synchronized indices (`asset_id → entry`, `logical_path → asset_id`, `role → asset_id set`).
- Lifecycle: `register` / `unregister` / `update_version`.
- Lookups: `get` / `resolve_path` / `list_all` / `list_by_role`.
- Validation: `broken_references` / `validate_invariants`.
- ID minting: `mint_asset_id` + path normalize/validate helpers.
- 11 tests in `crates/editor-core/tests/scene_asset_catalog.rs`.
- `pub mod` + `pub use` wiring in `lib.rs`.

### Out of Scope
- OPFS persistence (`catalog.json` I/O, `load_catalog`/`save_catalog`).
- Commands / undo / `processor.rs` integration.
- Frontend (React, hooks, panels).
- Scene Instance override resolution / resync (Fase 3).
- `bsn!` codegen changes (Fase 1).
- `EntityTemplate` → Scene Asset migration.
- `SceneAssetDocument` body I/O (catalog is metadata-only).

## Capabilities

> CONTRACT with sddk-spec. `openspec/specs/` contains only `entity-reparent-dnd`; no catalog or scene-asset capability exists yet.

### New Capabilities
- `scene-asset-catalog`: Project-level metadata index of Scene Assets — registration lifecycle, lookup by id/path/role, broken-reference detection, invariant validation. Owns the catalog data model and behavior contract.

### Modified Capabilities
- None. This change is purely additive (new module + types). It reads `SceneAssetDocument` fields but alters no existing requirement.

## Approach

A standalone module with three `BTreeMap` indices kept in sync by the three mutating methods. Entries are a **metadata projection** (`asset_id`, `logical_path`, `role`, `current_version`, `tags`, `created_at`, `updated_at`); the catalog never holds document bodies. `SceneAssetRole` is reused from Fase 0 (`scene_asset.rs`). `mint_asset_id` uses a zero-dependency fallback (`id_<unix_ms>_<8hex>`) because `uuid` is not in `Cargo.toml`. The catalog derives `Serialize`/`Deserialize` for a future `catalog.json` but performs no I/O in this phase.

## Public API

```rust
pub struct SceneAssetCatalogEntry {
    pub asset_id: String,
    pub logical_path: String,
    pub role: SceneAssetRole,
    pub current_version: u32,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(thiserror::Error)]
pub enum CatalogError {
    DuplicateAssetId { id: String },
    DuplicateLogicalPath { path: String },
    NotFound { id: String },
    InvalidPath { reason: String },
}

pub struct CatalogWarning {
    pub code: String,
    pub message: String,
    pub asset_id: Option<String>,
    pub logical_path: Option<String>,
}

pub struct SceneAssetCatalog { /* private fields */ }

impl SceneAssetCatalog {
    pub fn new() -> Self;
    pub fn from_entries(entries: Vec<SceneAssetCatalogEntry>) -> Self;
    pub fn register(&mut self, entry: SceneAssetCatalogEntry) -> Result<(), CatalogError>;
    pub fn unregister(&mut self, asset_id: &str) -> Result<SceneAssetCatalogEntry, CatalogError>;
    pub fn update_version(&mut self, asset_id: &str, new_version: u32) -> Result<(), CatalogError>;
    pub fn get(&self, asset_id: &str) -> Option<&SceneAssetCatalogEntry>;
    pub fn resolve_path(&self, path: &str) -> Option<&str>;                      // → asset_id
    pub fn list_all(&self) -> Vec<&SceneAssetCatalogEntry>;
    pub fn list_by_role(&self, role: SceneAssetRole) -> Vec<&SceneAssetCatalogEntry>;
    pub fn broken_references<I, S>(&self, references: I) -> Vec<String>
    where I: IntoIterator<Item = S>, S: AsRef<str>;
    pub fn validate_invariants(&self) -> Vec<CatalogWarning>;
}

pub fn mint_asset_id() -> String;

// private helpers
fn normalize_logical_path(path: &str) -> String;
fn validate_logical_path(path: &str) -> Result<(), CatalogError>;
fn entry_matches_role(entry: &SceneAssetCatalogEntry, role: &SceneAssetRole) -> bool;
fn role_key(role: SceneAssetRole) -> &'static str;   // string discriminant for role_index
```

## Internal Data Layout

```rust
struct SceneAssetCatalog {
    entries: BTreeMap<String, SceneAssetCatalogEntry>,   // asset_id → entry
    path_index: BTreeMap<String, String>,                 // logical_path → asset_id
    role_index: BTreeMap<&'static str, BTreeSet<String>>, // role_key → asset_id set
}
```

> **Note:** `SceneAssetRole` (Fase 0) derives `Eq` but not `Ord`/`Hash`, so it cannot key a `BTreeMap`/`HashMap`. The spike constraint forbids modifying `scene_asset.rs`. `role_index` therefore keys on a `&'static str` discriminant produced by a local `role_key()` match. (A future cleanup may add `PartialOrd, Ord` to `SceneAssetRole` — additive, safe — and switch the key to the typed enum.)

All three indices stay in sync; mutations go through `register`/`unregister`/`update_version` only.

## Derive Decisions

| Type | Derives |
|------|---------|
| `SceneAssetCatalog` | `Debug, Clone, Default, Serialize, Deserialize` |
| `SceneAssetCatalogEntry` | `Debug, Clone, PartialEq, Serialize, Deserialize` |
| `CatalogError` | `Debug, Clone, PartialEq, Eq, thiserror::Error` (`#[serde(rename_all = "snake_case")]` not needed — not serialized) |
| `CatalogWarning` | `Debug, Clone, PartialEq, Serialize, Deserialize` |

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/scene_asset_catalog.rs` | New | Catalog types, indices, methods, helpers. |
| `crates/editor-core/src/lib.rs` | Modified | `pub mod scene_asset_catalog;` + `pub use` re-exports. **Only existing source file touched.** |
| `crates/editor-core/tests/scene_asset_catalog.rs` | New | 11 behavior + serde tests. |

## Design Tensions (resolved; surfaced for spec scenarios)

1. **`mint_asset_id` format.** `uuid` is not a dependency. Recommend fallback `id_<unix_ms>_<8_hex_random>` (opaque, time-sortable, zero new deps). *Decision: fallback now; `uuid` v7 is a future option if cross-project uniqueness is ever required.*
2. **`broken_references` input semantics.** `SceneInstance.asset_ref` is `AssetReference` (logical path), but the catalog's primary key is `asset_id`. The method takes generic `AsRef<str>` keys and returns those absent from the catalog's `asset_id` set. *Decision: method treats input as `asset_id`s; path→id resolution is caller-side (via `resolve_path`).* Spec should cover a scenario showing the caller resolving paths first.
3. **`tags` deduplication on `register`.** *Decision: dedupe order-preserving on insert.*
4. **`role_index` key type** (see Internal Data Layout note). *Decision: `&'static str` discriminant to respect the no-edit-`scene_asset.rs` constraint.*

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Index desync (path/role drift from entries) | Low | All mutations funnel through 3 methods; `validate_invariants` is a defensive checker. |
| `mint_asset_id` collision (fallback format) | Low | 8 hex random (~4.3B space) + unix_ms prefix; acceptable for a single-project WASM editor. |
| `SceneAssetRole` lacks `Ord` (forces string-keyed role_index) | Med | `role_key()` discriminant; flag for a future additive derive cleanup. |
| `CatalogError` not serializable (frontend can't read it) | Low | Frontend out of scope this phase; revisit when commands land. |

## Rollback Plan

Delete `crates/editor-core/src/scene_asset_catalog.rs` and `crates/editor-core/tests/scene_asset_catalog.rs`, revert the two lines in `lib.rs` (`pub mod` + `pub use`). No data migration (no persistence). `git revert` of the single commit is sufficient.

## Dependencies
- `thiserror` (already a dep) for `CatalogError`.
- `std::collections::{BTreeMap, BTreeSet}`, `std::time::SystemTime` (std).
- **No new external crates.**

## Success Criteria
- [ ] `cargo test -p editor-core` passes with 11 new catalog tests.
- [ ] `cargo build -p editor-core --target wasm32-unknown-unknown` succeeds.
- [ ] Only `lib.rs` is modified among existing source files.
- [ ] Serde round-trip: register 3 entries → serialize → deserialize → `list_all()` matches.
- [ ] `broken_references(["known_id", "missing_id"])` returns `["missing_id"]`.
- [ ] `mint_asset_id()` produces 100 distinct ids.
- [ ] `normalize_logical_path("Assets\\Player\\")` == `"assets/player"`.

## Token Budget
~1100 words. Under implementation budget.
