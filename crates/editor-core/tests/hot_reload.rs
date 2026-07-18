//! Integration tests for hot-reload bus and drain system.
//!
//! Covers Task 1.1–1.5: HOT_RELOAD_BUS, drain system, source cache, asset cache,
//! and force-reload variant.

use editor_core::hot_reload_source_wasm;
use editor_core::source_files;

/// Test 1.1: hot_reload_source_wasm pushes Source request onto HOT_RELOAD_BUS.
#[test]
fn hot_reload_source_wasm_pushes_source_request() {
    // The HOT_RELOAD_BUS starts empty. After calling hot_reload_source_wasm,
    // it should contain exactly one entry.
    hot_reload_source_wasm("foo.rs");
    // Bus depth should be 1 (Source{file_id: "foo.rs"})
    let depth = editor_core::hot_reload_bus_depth_for_tests();
    assert_eq!(depth, 1, "HOT_RELOAD_BUS should have depth 1 after one Source request");
}

// §1.2: Drain system invalidates source cache
#[test]
fn process_drains_bus_and_invalidates_cache() {
    // Seed the cache with a source file entry
    source_files::cache_source("a.rs", "fn main() {}");

    // Seed the HOT_RELOAD_BUS with a Source request
    editor_core::hot_reload_source_wasm("a.rs");

    // Run the drain system
    editor_core::process_hot_reload_requests();

    // The cached source for "a.rs" should now be None (invalidated)
    let cached = source_files::get_cached_source("a.rs");
    assert!(
        cached.is_none(),
        "Expected None, got {:?} — cache should be invalidated after processing Source request",
        cached
    );
}

// §1.4: Asset request invalidates ASSET_BODY_CACHE and sets DIRTY_FLAG
#[test]
fn asset_request_invalidates_body_cache() {
    // Clear any prior state
    editor_core::clear_dirty_for_tests();

    // Seed ASSET_BODY_CACHE using the internal mutable accessor
    editor_core::with_asset_body_cache_mut_for_tests(|cache| {
        cache.insert("x".to_string(), editor_core::scene_asset::SceneAssetDocument {
            layers: vec![],
            asset_id: "asset-x".to_string(),
            logical_path: "x".to_string(),
            role: editor_core::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: editor_core::scene_asset::SceneAssetMetadata::default(),
        });
    });

    // Enqueue Asset{asset_id:"x"} and run drain
    editor_core::hot_reload_asset_wasm("x");
    editor_core::process_hot_reload_requests();

    // Entry should be gone from ASSET_BODY_CACHE
    let found = editor_core::with_asset_body_cache_mut_for_tests(|cache| {
        cache.contains_key("x")
    });
    assert!(!found, "ASSET_BODY_CACHE should not contain 'x' after Asset invalidation");

    // DIRTY_FLAG should be set
    assert!(editor_core::is_dirty_for_tests(), "DIRTY_FLAG should be set after Asset request");
}

// §1.5: ForceReloadAll clears all caches and sets dirty flag
#[test]
fn force_reload_emits_force_variant() {
    // Seed source cache
    source_files::cache_source("a.rs", "content");
    // Seed asset body cache
    editor_core::with_asset_body_cache_mut_for_tests(|cache| {
        cache.insert("y".to_string(), editor_core::scene_asset::SceneAssetDocument {
            layers: vec![],
            asset_id: "asset-y".to_string(),
            logical_path: "y".to_string(),
            role: editor_core::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: editor_core::scene_asset::SceneAssetMetadata::default(),
        });
    });

    // Enqueue ForceReloadAll
    editor_core::force_reload_wasm();
    editor_core::process_hot_reload_requests();

    // Source cache should be empty
    assert!(source_files::get_cached_source("a.rs").is_none(), "Source cache should be cleared");

    // Asset body cache should be empty
    let cache_len = editor_core::with_asset_body_cache_mut_for_tests(|cache| cache.len());
    assert_eq!(cache_len, 0, "ASSET_BODY_CACHE should be cleared");
}
