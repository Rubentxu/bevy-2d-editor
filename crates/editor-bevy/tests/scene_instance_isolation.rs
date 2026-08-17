//! Tests for scene instance isolation (E8).
//!
//! Covers: E8 scenario.
//!
//! E8: Two instances of the same asset must have isolated id_maps.
//! The namespaced `inst_<iid>_<lid>` minting ensures no collision between instances.

use editor_bevy::{
    StableId,
    command::Command,
    document::SceneDocument,
    processor,
    scene_asset::{AssetReference, LocalId},
    scene_instance::SceneInstance,
};
use std::collections::BTreeMap;

// Helper: empty SceneDocument
fn empty_doc() -> SceneDocument {
    SceneDocument {
        version: "0.1".to_string(),
        scene_id: "test".to_string(),
        name: "Test".to_string(),
        entities: vec![],
        instances: BTreeMap::new(),
    }
}

/// E8: Two instances of the same asset have isolated id_maps.
#[test]
fn e8_two_instances_isolated_id_maps() {
    let mut doc = empty_doc();

    // Place first instance of "player_asset"
    let instance_id_1 = StableId::new("inst_first");
    let id_map_1: BTreeMap<LocalId, StableId> = vec![
        (LocalId::new("root"), StableId::new("inst_first_root")),
        (LocalId::new("weapon"), StableId::new("inst_first_weapon")),
    ]
    .into_iter()
    .collect();

    let cmd1 = Command::PlaceInstance {
        instance_components: vec![],

        instance_id: instance_id_1.clone(),
        asset_ref: AssetReference::new("characters/player"),
        asset_version: 1,
        id_map: id_map_1.clone(),
        component_overrides: vec![],
        orphaned_component_overrides: vec![],
    };

    processor::apply(&mut doc, &cmd1).expect("first place should succeed");

    // Place second instance of the SAME asset
    let instance_id_2 = StableId::new("inst_second");
    let id_map_2: BTreeMap<LocalId, StableId> = vec![
        (LocalId::new("root"), StableId::new("inst_second_root")),
        (LocalId::new("weapon"), StableId::new("inst_second_weapon")),
    ]
    .into_iter()
    .collect();

    let cmd2 = Command::PlaceInstance {
        instance_components: vec![],

        instance_id: instance_id_2.clone(),
        asset_ref: AssetReference::new("characters/player"), // Same asset!
        asset_version: 1,
        id_map: id_map_2.clone(),
        component_overrides: vec![],
        orphaned_component_overrides: vec![],
    };

    processor::apply(&mut doc, &cmd2).expect("second place should succeed");

    // Verify both instances exist
    assert_eq!(doc.instances.len(), 2);

    // Verify instance 1 has its own id_map
    let inst_1 = doc
        .instances
        .get(&instance_id_1)
        .expect("instance 1 should exist");
    assert_eq!(
        inst_1.id_map.get(&LocalId::new("root")).unwrap().as_str(),
        "inst_first_root"
    );
    assert_eq!(
        inst_1.id_map.get(&LocalId::new("weapon")).unwrap().as_str(),
        "inst_first_weapon"
    );

    // Verify instance 2 has its own id_map (isolated)
    let inst_2 = doc
        .instances
        .get(&instance_id_2)
        .expect("instance 2 should exist");
    assert_eq!(
        inst_2.id_map.get(&LocalId::new("root")).unwrap().as_str(),
        "inst_second_root"
    );
    assert_eq!(
        inst_2.id_map.get(&LocalId::new("weapon")).unwrap().as_str(),
        "inst_second_weapon"
    );

    // Verify no collision between id_maps
    assert_ne!(
        inst_1.id_map.get(&LocalId::new("root")),
        inst_2.id_map.get(&LocalId::new("root")),
        "id_maps should be isolated - no collision"
    );
}

/// E8: Inverse operations are independent for two instances.
#[test]
fn e8_inverse_independence() {
    let mut doc = empty_doc();

    // Place two instances
    let cmd1 = Command::PlaceInstance {
        instance_components: vec![],

        instance_id: StableId::new("inst_a"),
        asset_ref: AssetReference::new("shared_asset"),
        asset_version: 1,
        id_map: vec![(LocalId::new("root"), StableId::new("inst_a_root"))]
            .into_iter()
            .collect(),
        component_overrides: vec![],
        orphaned_component_overrides: vec![],
    };

    let cmd2 = Command::PlaceInstance {
        instance_components: vec![],

        instance_id: StableId::new("inst_b"),
        asset_ref: AssetReference::new("shared_asset"),
        asset_version: 1,
        id_map: vec![(LocalId::new("root"), StableId::new("inst_b_root"))]
            .into_iter()
            .collect(),
        component_overrides: vec![],
        orphaned_component_overrides: vec![],
    };

    processor::apply(&mut doc, &cmd1).expect("first apply should succeed");
    processor::apply(&mut doc, &cmd2).expect("second apply should succeed");

    assert_eq!(doc.instances.len(), 2);

    // Remove first instance - second should be unaffected
    let remove_cmd1 = Command::RemoveInstance {
        instance_id: StableId::new("inst_a"),
    };

    let inverse1 = processor::apply(&mut doc, &remove_cmd1).expect("remove first should succeed");

    assert_eq!(doc.instances.len(), 1);
    assert!(
        doc.instances.get(&StableId::new("inst_b")).is_some(),
        "inst_b should remain"
    );

    // Apply inverse of remove (PlaceInstance) - should restore only inst_a
    processor::apply(&mut doc, &inverse1).expect("inverse should succeed");

    assert_eq!(doc.instances.len(), 2);

    // Verify inst_b is still intact
    let inst_b = doc
        .instances
        .get(&StableId::new("inst_b"))
        .expect("inst_b should exist");
    assert_eq!(
        inst_b.id_map.get(&LocalId::new("root")).unwrap().as_str(),
        "inst_b_root",
        "inst_b should be unaffected by inst_a operations"
    );
}

/// E8: Three instances of same asset maintain isolation.
#[test]
fn e8_three_instances_maintain_isolation() {
    let mut doc = empty_doc();

    // Place three instances of the same asset
    for i in 1..=3 {
        let instance_id = StableId::new(format!("inst_{}", i));
        let id_map: BTreeMap<LocalId, StableId> = vec![(
            LocalId::new("root"),
            StableId::new(format!("inst_{}_root", i)),
        )]
        .into_iter()
        .collect();

        let cmd = Command::PlaceInstance {
            instance_components: vec![],

            instance_id,
            asset_ref: AssetReference::new("shared"),
            asset_version: 1,
            id_map,
            component_overrides: vec![],
            orphaned_component_overrides: vec![],
        };

        processor::apply(&mut doc, &cmd).expect(&format!("instance {} should succeed", i));
    }

    assert_eq!(doc.instances.len(), 3);

    // Verify each has isolated id_map
    for i in 1..=3 {
        let instance_id = StableId::new(format!("inst_{}", i));
        let inst = doc
            .instances
            .get(&instance_id)
            .expect(&format!("inst_{} should exist", i));
        let expected_root = format!("inst_{}_root", i);
        assert_eq!(
            inst.id_map.get(&LocalId::new("root")).unwrap().as_str(),
            expected_root,
            "inst_{} should have isolated id_map",
            i
        );
    }
}
