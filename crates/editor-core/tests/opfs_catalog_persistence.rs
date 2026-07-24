//! Integration tests for OPFS catalog persistence ordering (opfs-catalog-flake-fix).
//!
//! Covers two scenarios from spec `opfs-catalog-persistence/spec.md`:
//!  - Scenario "metadata write failure rolls back in-memory entry"
//!  - Scenario "load_project sees persisted entries"
//!
//! The wasm-side fault-injection test for `update_project_metadata_for_asset`
//! lives in the Playwright suite (ADR-0019).

use editor_core::scene_asset::SceneAssetRole;
use editor_core::scene_asset_catalog::{SceneAssetCatalog, SceneAssetCatalogEntry};
use editor_core::ProjectMetadata;

fn entry(
    asset_id: &str,
    logical_path: &str,
    role: SceneAssetRole,
    version: u32,
) -> SceneAssetCatalogEntry {
    SceneAssetCatalogEntry {
        asset_id: asset_id.to_string(),
        logical_path: logical_path.to_string(),
        role,
        current_version: version,
        tags: vec![],
        created_at: 1000,
        updated_at: 1000,
        // ADR-0026: no preview by default in tests.
        preview_resource: None,
    }
}

/// Spec scenario: metadata write failure rolls back in-memory entry.
///
/// The WASM create/duplicate helpers unregister on metadata failure. If
/// `unregister` regresses (e.g., leaves a dangling path_index entry), this
/// catches it before the WASM layer can observe it.
#[test]
fn create_then_failed_metadata_rolls_back_in_memory_catalog() {
    let mut catalog = SceneAssetCatalog::new();
    let e = entry("id_create_1", "actors/player", SceneAssetRole::Actor, 1);
    catalog.register(e.clone()).expect("register should succeed");

    assert_eq!(catalog.get("id_create_1"), Some(&e));
    assert_eq!(catalog.resolve_path("actors/player"), Some("id_create_1"));

    // The WASM code calls `with_asset_catalog_mut(|c| { let _ = c.unregister(id); })`.
    let removed = catalog.unregister("id_create_1").expect("rollback unregister");
    assert_eq!(removed.asset_id, "id_create_1");

    assert_eq!(catalog.get("id_create_1"), None);
    assert_eq!(catalog.resolve_path("actors/player"), None);
    assert!(catalog.list_all().is_empty());
    assert!(catalog.list_by_role(SceneAssetRole::Actor).is_empty());
    assert!(catalog.validate_invariants().is_empty());
}

/// Spec scenario: load_project sees persisted entries.
///
/// `update_project_metadata_for_asset` writes the catalog snapshot into
/// `project.json` under `scene_assets`. `load_project` reads that field
/// back into a fresh `SceneAssetCatalog`. Verifies the ProjectMetadata
/// round-trip contract that both sides rely on.
#[test]
fn project_metadata_round_trip_preserves_scene_asset_entries() {
    let mut pm = ProjectMetadata::default();
    pm.scene_assets.push(entry("id_load_1", "actors/player", SceneAssetRole::Actor, 1));
    pm.scene_assets.push(entry("id_load_2", "ui/menu", SceneAssetRole::Ui, 3));

    let written = serde_json::to_string(&pm).expect("serialize");
    let loaded: ProjectMetadata = serde_json::from_str(&written).expect("deserialize");

    assert_eq!(loaded.scene_assets.len(), 2);
    let mut catalog = SceneAssetCatalog::new();
    for e in &loaded.scene_assets {
        catalog.register(e.clone()).expect("rebuild register");
    }
    assert_eq!(catalog.resolve_path("actors/player"), Some("id_load_1"));
    assert_eq!(catalog.resolve_path("ui/menu"), Some("id_load_2"));
    assert_eq!(catalog.list_all().len(), 2);
}