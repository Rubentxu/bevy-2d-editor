//! Integration tests for scene asset persistence (PR1 tasks 1.8).
//! Covers spec scenarios: S4, S5, S6, S7, S8, S17, S18.
//!
//! These tests run entirely in-memory without WASM/OPFS, using the
//! pure Rust persistence and catalog functions directly.

use editor_core::scene_asset::SceneAssetRole;
use editor_model::scene_asset_catalog::{
    SceneAssetCatalog, SceneAssetCatalogEntry, mint_asset_id, random_hex_8,
};
use editor_core::test_helpers::FakeClock;
use editor_core::{ASSETS_DIR, ProjectMetadata, asset_path};

// ─────────────────────────────────────────────────────────────────────────
// Helper: make a catalog entry
// ─────────────────────────────────────────────────────────────────────────

fn entry(
    asset_id: &str,
    logical_path: &str,
    role: SceneAssetRole,
    version: u32,
) -> SceneAssetCatalogEntry {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    SceneAssetCatalogEntry {
        asset_id: asset_id.to_string(),
        logical_path: logical_path.to_string(),
        role,
        current_version: version,
        tags: vec![],
        created_at: now,
        updated_at: now,
        // ADR-0026: no preview by default in tests.
        preview_resource: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// S18 — asset_path produces the expected OPFS path
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn asset_path_simple() {
    assert_eq!(asset_path("player"), "assets/player.asset.json");
}

#[test]
fn asset_path_nested() {
    assert_eq!(
        asset_path("characters/player"),
        "assets/characters/player.asset.json"
    );
}

#[test]
fn asset_path_deeply_nested() {
    assert_eq!(
        asset_path("ui/menus/title_screen"),
        "assets/ui/menus/title_screen.asset.json"
    );
}

#[test]
fn assets_dir_constant() {
    assert_eq!(ASSETS_DIR, "assets");
}

// ─────────────────────────────────────────────────────────────────────────
// S17 — ProjectMetadata with old shape (no scene_assets) still loads
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn project_metadata_without_scene_assets_deserializes() {
    // GIVEN old project.json JSON without scene_assets field
    let json = r#"{"version":"0.1","name":"Old","scenes":["s1"]}"#;
    // WHEN deserializing
    let pm: ProjectMetadata = serde_json::from_str(json).unwrap();
    // THEN scene_assets defaults to empty Vec
    assert_eq!(pm.name, "Old");
    assert_eq!(pm.scenes, vec!["s1"]);
    assert!(pm.scene_assets.is_empty());
}

#[test]
fn project_metadata_with_scene_assets_roundtrip() {
    let pm = ProjectMetadata {
        version: "0.1".to_string(),
        name: "Test".to_string(),
        scenes: vec![],
        schemas: vec![],
        active_scene: None,
        scene_assets: vec![
            entry("id_1", "player", SceneAssetRole::Actor, 1),
            entry("id_2", "menu", SceneAssetRole::Ui, 1),
        ],
    };
    let json = serde_json::to_string(&pm).unwrap();
    let rt: ProjectMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.scene_assets.len(), 2);
    assert_eq!(rt.scene_assets[0].logical_path, "player");
    assert_eq!(rt.scene_assets[1].logical_path, "menu");
}

// ─────────────────────────────────────────────────────────────────────────
// S4 — Create scene asset persists file and catalog entry
// (catalog registration with unique asset_id and logical_path)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn catalog_register_creates_entry() {
    let mut catalog = SceneAssetCatalog::new();
    let e = entry("id_1", "player", SceneAssetRole::Actor, 1);

    catalog
        .register(e.clone())
        .expect("register should succeed");

    assert_eq!(catalog.get("id_1"), Some(&e));
    assert_eq!(catalog.resolve_path("player"), Some("id_1"));
    assert_eq!(catalog.list_all().len(), 1);
}

#[test]
fn catalog_register_normalizes_path() {
    let mut catalog = SceneAssetCatalog::new();
    // Mixed case gets normalized to lowercase and leading/trailing slashes stripped
    let e = entry("id_1", "Assets/Player/", SceneAssetRole::Actor, 1);

    catalog
        .register(e.clone())
        .expect("register should succeed");

    // Resolves the normalized path (lowercase, stripped slashes)
    assert_eq!(catalog.resolve_path("assets/player"), Some("id_1"));
    assert_eq!(catalog.resolve_path("Assets/Player/"), Some("id_1"));
    // Original non-normalized forms that don't match fail
    assert_eq!(catalog.resolve_path("player"), None);
}

// ─────────────────────────────────────────────────────────────────────────
// S5 — Create with duplicate logical_path is rejected
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn catalog_register_duplicate_path_rejected() {
    let mut catalog = SceneAssetCatalog::new();
    let e1 = entry("id_1", "player", SceneAssetRole::Actor, 1);
    catalog.register(e1).expect("first should succeed");

    let e2 = entry("id_2", "player", SceneAssetRole::Actor, 1);
    let err = catalog
        .register(e2)
        .expect_err("duplicate path should fail");
    assert!(matches!(
        err,
        editor_model::scene_asset_catalog::CatalogError::DuplicateLogicalPath { path }
        if path == "player"
    ));
}

#[test]
fn catalog_register_duplicate_asset_id_rejected() {
    let mut catalog = SceneAssetCatalog::new();
    let e1 = entry("id_1", "player", SceneAssetRole::Actor, 1);
    catalog.register(e1).expect("first should succeed");

    let e2 = entry("id_1", "enemy", SceneAssetRole::Actor, 1);
    let err = catalog.register(e2).expect_err("duplicate id should fail");
    assert!(matches!(
        err,
        editor_model::scene_asset_catalog::CatalogError::DuplicateAssetId { id }
        if id == "id_1"
    ));
}

// ─────────────────────────────────────────────────────────────────────────
// S6 — Rename updates catalog entry's logical_path and bumps version
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn catalog_update_version_bumps() {
    let mut catalog = SceneAssetCatalog::new();
    let e = entry("id_1", "player", SceneAssetRole::Actor, 1);
    catalog.register(e).expect("register should succeed");

    catalog
        .update_version("id_1", 2)
        .expect("version bump should succeed");

    let updated = catalog.get("id_1").expect("entry should exist");
    assert_eq!(updated.current_version, 2);
    assert!(updated.updated_at > updated.created_at);
}

#[test]
fn catalog_update_version_rejects_downgrade() {
    let mut catalog = SceneAssetCatalog::new();
    let e = entry("id_1", "player", SceneAssetRole::Actor, 2);
    catalog.register(e).expect("register should succeed");

    let err = catalog
        .update_version("id_1", 1)
        .expect_err("downgrade should fail");
    assert!(matches!(
        err,
        editor_model::scene_asset_catalog::CatalogError::InvalidVersion { current: 2, new: 1 }
    ));
}

#[test]
fn catalog_unregister_then_register_moves_path() {
    let mut catalog = SceneAssetCatalog::new();
    let e1 = entry("id_1", "player", SceneAssetRole::Actor, 1);
    catalog.register(e1).expect("register should succeed");

    // Unregister and re-register with new path (simulates rename)
    catalog
        .unregister("id_1")
        .expect("unregister should succeed");
    let e2 = entry("id_1", "characters/player", SceneAssetRole::Actor, 2);
    catalog.register(e2).expect("re-register should succeed");

    assert_eq!(catalog.resolve_path("characters/player"), Some("id_1"));
    assert_eq!(catalog.resolve_path("player"), None); // old path gone
}

// ─────────────────────────────────────────────────────────────────────────
// S7 — Duplicate creates new asset with distinct asset_id, copies body
// (catalog: register new entry with minted id, same logical_path collision
// handled by suffix)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn catalog_mint_asset_id_produces_unique_ids() {
    let clock = FakeClock::new();
    let ids: Vec<String> = (0..50)
        .map(|i| {
            clock.advance(1);
            mint_asset_id(&clock, &random_hex_8())
        })
        .collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), 50); // all unique
    assert!(ids.iter().all(|id| id.starts_with("id_")));
}

#[test]
fn catalog_duplicate_entry_with_unique_id() {
    let mut catalog = SceneAssetCatalog::new();
    let e1 = entry("id_1", "player", SceneAssetRole::Actor, 1);
    catalog.register(e1).expect("register should succeed");

    // Simulate duplicate: new entry with different id
    let new_id = mint_asset_id(&FakeClock::new(), &random_hex_8());
    let e2 = entry(&new_id, "player", SceneAssetRole::Actor, 1);
    let err = catalog.register(e2).expect_err("same path should fail");

    // The error is DuplicateLogicalPath
    assert!(matches!(
        err,
        editor_model::scene_asset_catalog::CatalogError::DuplicateLogicalPath { .. }
    ));
}

// ─────────────────────────────────────────────────────────────────────────
// S8 — Delete removes file reference and catalog entry
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn catalog_unregister_removes_entry() {
    let mut catalog = SceneAssetCatalog::new();
    let e = entry("id_1", "player", SceneAssetRole::Actor, 1);
    catalog.register(e).expect("register should succeed");

    catalog
        .unregister("id_1")
        .expect("unregister should succeed");

    assert!(catalog.get("id_1").is_none());
    assert!(catalog.resolve_path("player").is_none());
    assert!(catalog.list_all().is_empty());
}

#[test]
fn catalog_unregister_missing_returns_not_found() {
    let mut catalog = SceneAssetCatalog::new();
    let err = catalog
        .unregister("nonexistent")
        .expect_err("missing should fail");
    assert!(matches!(
        err,
        editor_model::scene_asset_catalog::CatalogError::NotFound { id }
        if id == "nonexistent"
    ));
}

// ─────────────────────────────────────────────────────────────────────────
// S9 — Catalog persists across serde round-trips (survival)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn catalog_serde_roundtrip_preserves_entries() {
    let mut catalog = SceneAssetCatalog::new();
    catalog
        .register(entry("id_1", "player", SceneAssetRole::Actor, 3))
        .unwrap();
    catalog
        .register(entry("id_2", "menu", SceneAssetRole::Ui, 1))
        .unwrap();

    let json = serde_json::to_string(&catalog).unwrap();
    let rt: SceneAssetCatalog = serde_json::from_str(&json).unwrap();

    assert_eq!(rt.list_all().len(), 2);
    assert_eq!(rt.resolve_path("player"), Some("id_1"));
    assert_eq!(rt.resolve_path("menu"), Some("id_2"));
}
