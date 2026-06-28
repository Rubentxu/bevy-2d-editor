//! Integration tests for the scene_asset_catalog module.
//! Covers all 11 scenarios from design.md §Testing Strategy.

use editor_core::scene_asset::SceneAssetRole;
use editor_core::scene_asset_catalog::{
    normalize_logical_path, validate_logical_path, CatalogError,
    SceneAssetCatalog, SceneAssetCatalogEntry, mint_asset_id,
};

fn entry(
    asset_id: &str,
    logical_path: &str,
    role: SceneAssetRole,
    version: u32,
    tags: Vec<&str>,
    created_at: u64,
    updated_at: u64,
) -> SceneAssetCatalogEntry {
    SceneAssetCatalogEntry {
        asset_id: asset_id.to_string(),
        logical_path: logical_path.to_string(),
        role,
        current_version: version,
        tags: tags.into_iter().map(String::from).collect(),
        created_at,
        updated_at,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// S1 — Empty catalog
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn register_valid_entry_populates_all_indices() {
    let mut catalog = SceneAssetCatalog::new();

    let e = entry("id_1", "assets/player", SceneAssetRole::Actor, 1, vec![], 1000, 1000);
    catalog.register(e.clone()).expect("register should succeed");

    // by id
    assert_eq!(catalog.get("id_1"), Some(&e));
    // by path (normalized)
    assert_eq!(catalog.resolve_path("assets/player"), Some("id_1"));
    // list_all
    let all = catalog.list_all();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].asset_id, "id_1");
}

#[test]
fn register_duplicate_asset_id_returns_error() {
    let mut catalog = SceneAssetCatalog::new();

    let e1 = entry("id_1", "assets/a", SceneAssetRole::Actor, 1, vec![], 1000, 1000);
    let e2 = entry("id_1", "assets/b", SceneAssetRole::Actor, 1, vec![], 1000, 1000);

    catalog.register(e1).expect("first register should succeed");
    let err = catalog.register(e2).expect_err("second register should fail");
    assert!(matches!(err, CatalogError::DuplicateAssetId { id } if id == "id_1"));
    assert_eq!(catalog.list_all().len(), 1);
}

#[test]
fn register_duplicate_normalized_path_returns_error() {
    let mut catalog = SceneAssetCatalog::new();

    // "Assets/Player/" normalizes to "assets/player"
    let e1 = entry("id_1", "Assets/Player/", SceneAssetRole::Actor, 1, vec![], 1000, 1000);
    catalog.register(e1).expect("first register should succeed");

    // Already-normalized form should conflict
    let e2 = entry("id_2", "assets/player", SceneAssetRole::Actor, 1, vec![], 1000, 1000);
    let err = catalog.register(e2).expect_err("should fail with DuplicateLogicalPath");
    assert!(matches!(err, CatalogError::DuplicateLogicalPath { path } if path == "assets/player"));
}

// ─────────────────────────────────────────────────────────────────────────────
// S5 — unregister
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn unregister_existing_returns_entry_and_cleans_indices() {
    let mut catalog = SceneAssetCatalog::new();

    let e = entry("id_1", "assets/player", SceneAssetRole::Actor, 1, vec![], 1000, 1000);
    catalog.register(e.clone()).expect("register should succeed");

    let removed = catalog.unregister("id_1").expect("unregister should succeed");
    assert_eq!(removed.asset_id, "id_1");

    assert_eq!(catalog.get("id_1"), None);
    assert_eq!(catalog.resolve_path("assets/player"), None);
    assert!(catalog.list_all().is_empty());

    // second unregister fails
    let err = catalog.unregister("id_1").expect_err("second unregister should fail");
    assert!(matches!(err, CatalogError::NotFound { id } if id == "id_1"));
}

#[test]
fn unregister_missing_returns_not_found() {
    let mut catalog = SceneAssetCatalog::new();
    let err = catalog.unregister("id_nonexistent").expect_err("should fail");
    assert!(matches!(err, CatalogError::NotFound { id } if id == "id_nonexistent"));
}

// ─────────────────────────────────────────────────────────────────────────────
// S2 / S6 — lookups and list by role
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn resolve_path_and_get_lookups() {
    let mut catalog = SceneAssetCatalog::new();

    let e = entry("id_1", "assets/player", SceneAssetRole::Actor, 1, vec![], 1000, 1000);
    catalog.register(e).expect("register should succeed");

    assert_eq!(catalog.get("id_1").map(|e| e.logical_path.as_str()), Some("assets/player"));
    assert_eq!(catalog.resolve_path("assets/player"), Some("id_1"));
    // path lookup on unregistered key
    assert_eq!(catalog.resolve_path("assets/nonexistent"), None);
    // id lookup on unregistered key
    assert_eq!(catalog.get("id_2"), None);
}

#[test]
fn list_by_role_filters_correctly() {
    let mut catalog = SceneAssetCatalog::new();

    let e1 = entry("id_1", "assets/player", SceneAssetRole::Actor, 1, vec![], 1000, 1000);
    let e2 = entry("id_2", "assets/enemy", SceneAssetRole::Actor, 1, vec![], 1000, 1000);
    let e3 = entry("id_3", "assets/menu", SceneAssetRole::Ui, 1, vec![], 1000, 1000);

    catalog.register(e1).expect("register should succeed");
    catalog.register(e2).expect("register should succeed");
    catalog.register(e3).expect("register should succeed");

    let actors = catalog.list_by_role(SceneAssetRole::Actor);
    assert_eq!(actors.len(), 2);
    assert!(actors.iter().all(|e| e.role == SceneAssetRole::Actor));

    let ui = catalog.list_by_role(SceneAssetRole::Ui);
    assert_eq!(ui.len(), 1);
    assert_eq!(ui[0].asset_id, "id_3");

    let level = catalog.list_by_role(SceneAssetRole::Level);
    assert!(level.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// S7 — broken_references
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn broken_references_returns_missing_in_input_order() {
    let mut catalog = SceneAssetCatalog::new();

    catalog.register(entry("id_1", "assets/a", SceneAssetRole::Actor, 1, vec![], 1000, 1000)).unwrap();
    catalog.register(entry("id_2", "assets/b", SceneAssetRole::Actor, 1, vec![], 1000, 1000)).unwrap();

    let broken = catalog.broken_references(["id_1", "id_missing", "id_2", "id_also_missing"]);
    assert_eq!(broken, vec!["id_missing", "id_also_missing"]);

    // All present returns empty
    let none = catalog.broken_references(["id_1", "id_2"]);
    assert!(none.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// S9 — serde round-trip
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn serde_roundtrip_preserves_entries() {
    let mut catalog = SceneAssetCatalog::new();

    catalog.register(entry("id_1", "assets/player", SceneAssetRole::Actor, 1, vec!["enemy", "boss"], 1000, 1000)).unwrap();
    catalog.register(entry("id_2", "assets/menu", SceneAssetRole::Ui, 3, vec![], 2000, 2000)).unwrap();
    catalog.register(entry("id_3", "assets/level1", SceneAssetRole::Level, 7, vec!["menu"], 3000, 3000)).unwrap();

    let json = serde_json::to_string(&catalog).expect("serialize should succeed");
    let roundtripped: SceneAssetCatalog = serde_json::from_str(&json).expect("deserialize should succeed");

    assert_eq!(roundtripped.list_all().len(), 3);

    // Resolve paths
    assert_eq!(roundtripped.resolve_path("assets/player"), Some("id_1"));
    assert_eq!(roundtripped.resolve_path("assets/menu"), Some("id_2"));
    assert_eq!(roundtripped.resolve_path("assets/level1"), Some("id_3"));

    // List by role
    assert_eq!(roundtripped.list_by_role(SceneAssetRole::Actor).len(), 1);
    assert_eq!(roundtripped.list_by_role(SceneAssetRole::Ui).len(), 1);
    assert_eq!(roundtripped.list_by_role(SceneAssetRole::Level).len(), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// S8 — path validation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn normalize_and_validate_logical_path() {
    // normalize_logical_path
    assert_eq!(normalize_logical_path("Assets/Player/"), "assets/player");
    assert_eq!(normalize_logical_path("foo//bar"), "foo/bar");
    assert_eq!(normalize_logical_path("/leading/"), "leading");
    assert_eq!(normalize_logical_path(""), "");

    // validate_logical_path — empty
    let err = validate_logical_path("").expect_err("empty should fail");
    assert!(matches!(err, CatalogError::InvalidPath { reason } if reason == "empty"));

    let err = validate_logical_path("   ").expect_err("whitespace should fail");
    assert!(matches!(err, CatalogError::InvalidPath { reason } if reason == "empty"));

    // validate_logical_path — path traversal
    let err = validate_logical_path("foo/../bar").expect_err(".. should fail");
    assert!(matches!(err, CatalogError::InvalidPath { reason } if reason == "path traversal not allowed"));

    let err = validate_logical_path("foo/./bar").expect_err(". should fail");
    assert!(matches!(err, CatalogError::InvalidPath { reason } if reason == "path traversal not allowed"));

    // valid paths
    validate_logical_path("assets/player").expect("valid path should pass");
    validate_logical_path("a/b/c").expect("valid path should pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// S10 — version update
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn update_version_validates_monotonic() {
    let mut catalog = SceneAssetCatalog::new();

    catalog.register(entry("id_1", "assets/player", SceneAssetRole::Actor, 1, vec![], 1000, 1000)).unwrap();

    // Advance version
    catalog.update_version("id_1", 2).expect("version bump should succeed");
    assert_eq!(catalog.get("id_1").unwrap().current_version, 2);
    assert!(catalog.get("id_1").unwrap().updated_at > 1000);

    // Same version fails
    let err = catalog.update_version("id_1", 2).expect_err("same version should fail");
    assert!(matches!(err, CatalogError::InvalidVersion { current: 2, new: 2 }));

    // Downgrade fails
    let err = catalog.update_version("id_1", 1).expect_err("downgrade should fail");
    assert!(matches!(err, CatalogError::InvalidVersion { current: 2, new: 1 }));

    // Missing asset
    let err = catalog.update_version("id_99", 2).expect_err("missing asset should fail");
    assert!(matches!(err, CatalogError::NotFound { id } if id == "id_99"));
}

// ─────────────────────────────────────────────────────────────────────────────
// mint_asset_id uniqueness
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mint_asset_id_produces_distinct_ids() {
    let ids: Vec<String> = (0..100).map(|_| mint_asset_id()).collect();
    // All should be unique
    let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(unique_count, 100);
    // All should start with "id_"
    assert!(ids.iter().all(|id| id.starts_with("id_")));
}
