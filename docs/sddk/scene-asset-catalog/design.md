# Design: Scene Asset Catalog

> Change: `scene-asset-catalog` · Phase: design · Mode: C2 · Source of truth: ADR-0005

## Technical Approach

A standalone metadata-index module (`scene_asset_catalog.rs`) that registers `SceneAssetCatalogEntry` projections under three synchronized `BTreeMap` indices keyed by `asset_id`, normalized `logical_path`, and a role discriminant. All mutations funnel through `register` / `unregister` / `update_version`; lookups are read-only borrows. The catalog holds metadata only — document bodies stay in `SceneAssetDocument`. This maps directly to the proposal's Approach (three-index layer, no I/O, no commands) and to ADR-0005 §Detailed Rules (asset_id + logical_path dual identity, role soft-policy, monotonic version).

`SceneAssetRole` is reused from Fase 0 (`scene_asset.rs:41-50`); it derives `PartialEq, Eq` but **not** `Ord` / `Hash`, so `role_index` keys on a `&'static str` discriminant via `role_key()`. The spike constraint forbids editing `scene_asset.rs`.

## Architecture Decisions

### Decision: Three `BTreeMap` indices kept in sync

**Choice**: `entries`, `path_index`, `role_index` as `BTreeMap`s.
**Alternatives**: Single `Vec<entry>` with linear scan; `HashMap`.
**Rationale**: `BTreeMap` gives deterministic iteration order (stable test output, deterministic `catalog.json` serialization) and O(log n) lookup. A `Vec` makes lookups O(n) and serialization order insertion-dependent (fragile). `HashMap` lacks deterministic ordering.

### Decision: `&'static str` role discriminant

**Choice**: `role_key(role: &SceneAssetRole) -> &'static str` maps enum → `"actor"`, `"fragment"`, etc.
**Alternatives**: Add `Ord, Hash` to `SceneAssetRole` (edits `scene_asset.rs`); store role as `String` on entry.
**Rationale**: The spike forbids editing `scene_asset.rs`. `&'static str` is cheap, `Ord`+`Hash`, and the match is exhaustive (compiler-checked). A future additive derive cleanup (`#[derive(Ord, PartialOrd, Hash)]`) can switch the key to the typed enum without breaking the public API.

### Decision: `u64` unix-millis timestamps (no `chrono`)

**Choice**: `created_at: u64`, `updated_at: u64` (unix millis).
**Alternatives**: `String` ISO-8601 (proposal draft); `chrono::DateTime`.
**Rationale**: `chrono` is not in `Cargo.toml` and the spike adds no new deps. Fase 0 used `Option<String>` to avoid parsing; `u64` is cleaner for comparison and sorting. Serialization is trivial via serde.

### Decision: `from_entries` is fallible (fail-fast)

**Choice**: `from_entries(entries) -> Result<Self, CatalogError>`, stops on first conflict.
**Alternatives**: Collect all errors (batch validation); silently drop conflicts.
**Rationale**: Fail-fast surfaces the first conflict clearly. Batch validation adds complexity for no spike value. Silent drops hide data corruption.

## Module Placement & Wiring

**New file**: `crates/editor-core/src/scene_asset_catalog.rs`
**Modified**: `crates/editor-core/src/lib.rs` — add one line after `pub mod scene_instance;` (line 18):

```rust
pub mod scene_asset_catalog;
```

Re-exports (`pub use`) are optional for this spike; tests import via `editor_core::scene_asset_catalog::*`. The module-level doc comment cites ADR-0005.

### Imports

```rust
use crate::scene_asset::{SceneAssetRole, SceneAssetMetadata};
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
```

> `SceneAssetMetadata` is imported to document the future `from_document` projection path (`SceneAssetDocument.metadata → entry.tags/timestamps`). That conversion is out of scope; if the import is unused, drop it at implementation time to avoid a lint.

## Public Types

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneAssetCatalog {
    entries: BTreeMap<String, SceneAssetCatalogEntry>,   // asset_id → entry
    path_index: BTreeMap<String, String>,                 // normalized_path → asset_id
    role_index: BTreeMap<&'static str, BTreeSet<String>>, // role_key → asset_id set
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetCatalogEntry {
    pub asset_id: String,
    pub logical_path: String,
    pub role: SceneAssetRole,
    pub current_version: u32,
    pub tags: Vec<String>,
    pub created_at: u64,   // unix millis — no chrono dep
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatalogError {
    #[error("duplicate asset_id '{id}'")]
    DuplicateAssetId { id: String },
    #[error("duplicate logical_path '{path}'")]
    DuplicateLogicalPath { path: String },
    #[error("asset_id '{id}' not found")]
    NotFound { id: String },
    #[error("invalid logical path: {reason}")]
    InvalidPath { reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogWarning {
    pub code: String,
    pub message: String,
    pub asset_id: Option<String>,
    pub logical_path: Option<String>,
}
```

## Public API

```rust
impl SceneAssetCatalog {
    pub fn new() -> Self;
    pub fn from_entries(entries: Vec<SceneAssetCatalogEntry>) -> Result<Self, CatalogError>;
    pub fn register(&mut self, entry: SceneAssetCatalogEntry) -> Result<(), CatalogError>;
    pub fn unregister(&mut self, asset_id: &str) -> Result<SceneAssetCatalogEntry, CatalogError>;
    pub fn update_version(&mut self, asset_id: &str, new_version: u32) -> Result<(), CatalogError>;
    pub fn get(&self, asset_id: &str) -> Option<&SceneAssetCatalogEntry>;
    pub fn resolve_path(&self, path: &str) -> Option<&str>;  // → asset_id
    pub fn list_all(&self) -> Vec<&SceneAssetCatalogEntry>;
    pub fn list_by_role(&self, role: SceneAssetRole) -> Vec<&SceneAssetCatalogEntry>;
    pub fn broken_references<I, S>(&self, references: I) -> Vec<String>
    where I: IntoIterator<Item = S>, S: AsRef<str>;
    pub fn validate_invariants(&self) -> Vec<CatalogWarning>;
}

pub fn mint_asset_id() -> String;
pub fn normalize_logical_path(path: &str) -> String;
pub fn validate_logical_path(path: &str) -> Result<(), CatalogError>;
```

## Private Helpers

```rust
fn role_key(role: &SceneAssetRole) -> &'static str {
    match role {
        SceneAssetRole::Actor   => "actor",
        SceneAssetRole::Fragment => "fragment",
        SceneAssetRole::Screen  => "screen",
        SceneAssetRole::Level   => "level",
        SceneAssetRole::Ui      => "ui",
        SceneAssetRole::Effect  => "effect",
    }
}
fn dedupe_tags(tags: Vec<String>) -> Vec<String>;   // order-preserving
fn current_unix_millis() -> u64;                     // SystemTime on both targets
fn random_hex_8() -> String;                         // cfg-gated: js_sys on wasm32
```

## Behavior Contracts

**`normalize_logical_path`**: strip whitespace → `to_ascii_lowercase` (non-ASCII unchanged) → replace `\` with `/` → collapse `//` → `/` → strip leading/trailing `/`. Empty → `""`.

**`validate_logical_path`**: empty → `InvalidPath { reason: "empty" }`; contains `..` or `.` segment → `InvalidPath { reason: "path traversal not allowed" }`; else `Ok`.

**`register`**: (1) validate path → Err if invalid; (2) normalize path, store on entry; (3) duplicate `asset_id` check → `DuplicateAssetId`; (4) duplicate path check → `DuplicateLogicalPath`; (5) dedupe tags; (6) insert into all three indices; (7) Ok.

**`unregister`**: (1) find entry → `NotFound` if missing; (2) remove from all indices; (3) return entry.

**`update_version`**: (1) find entry → `NotFound`; (2) set `current_version`, refresh `updated_at`; (3) Ok.

**`broken_references`**: for each input key, if `entries.contains_key(key)` is false, collect. Returns input-order, deduped list of missing keys. Input is treated as `asset_id`s; path→id resolution is caller-side via `resolve_path`.

**`validate_invariants`** (defensive, returns warnings not errors): for each entry — path not normalized → `"non_normalized_path"`; `asset_id` not `id_`-prefixed → `"malformed_asset_id"`; tags not deduped → `"duplicate_tag"`.

**`mint_asset_id`**: `format!("id_{}_{}", current_unix_millis(), random_hex_8())`.
- `random_hex_8()` — wasm32: derive from `js_sys::Math::random()` (dep gated to wasm32 in `Cargo.toml`); native: `SystemTime` nanos + `AtomicU64` counter.

**`from_entries`**: fold `register` over input; first Err propagates; else `Ok(catalog)`.

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Integration | 10 spec scenarios + 1 mint-uniqueness test | `crates/editor-core/tests/scene_asset_catalog.rs` |

Tests to write (11 total):
1. `register_valid_entry_populates_all_indices`
2. `register_duplicate_asset_id_returns_error`
3. `register_duplicate_normalized_path_returns_error`
4. `unregister_existing_returns_entry_and_cleans_indices`
5. `unregister_missing_returns_not_found`
6. `resolve_path_and_get_lookups`
7. `list_by_role_filters_correctly`
8. `broken_references_returns_missing_in_input_order`
9. `serde_roundtrip_preserves_entries`
10. `normalize_and_validate_logical_path`
11. `mint_asset_id_produces_distinct_ids`

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/editor-core/src/scene_asset_catalog.rs` | Create | Catalog types, 3 indices, methods, helpers. |
| `crates/editor-core/src/lib.rs` | Modify | Add `pub mod scene_asset_catalog;` (1 line, after line 18). |
| `crates/editor-core/tests/scene_asset_catalog.rs` | Create | 11 integration tests. |

## Migration / Rollout

No migration required. No persistence in this phase. Rollback: delete the two new files, revert the one line in `lib.rs`.

## Open Questions

- [ ] `current_unix_millis()` on `wasm32-unknown-unknown`: `SystemTime::now()` works in Rust std on wasm32, but the random suffix needs `js_sys::Math::random()` (only available behind the wasm32 cfg in `Cargo.toml`). Verify cfg-gated `random_hex_8` compiles on both targets.
- [ ] `broken_references` ordering: spec S7 (companion, not yet written) should pin input-order deduped. **Recommendation**: input-order, deduped.
- [ ] `mint_asset_id` entropy: 32-bit random suffix is sufficient for editor-side uniqueness, not cryptographic. Acceptable for single-project WASM.

## ADR Candidates

No strong ADR candidates — the design directly implements ADR-0005 §Detailed Rules. The closest is the **`&'static str` role discriminant** (surprising: why not key on the enum?), but it fails the "hard to reverse" bar: it's an internal field, there's no existing data to migrate, and the additive `Ord` derive on `SceneAssetRole` self-resolves it. Not worth a new ADR.
