//! Override status and identity tests.
//! Covers scenarios S5, S8, S10.

use editor_core::StableId;
use editor_core::scene_asset::{LocalId, SceneAssetEntity};
use editor_core::scene_instance::{OverridePatch, OverrideStatus};
use std::any::TypeId;

#[test]
fn s5_override_status_is_closed_enum() {
    // Construct an OverridePatch with status Active, serialize to JSON,
    // deserialize back, assert status equals Active (lowercase snake_case string).
    let patch = OverridePatch {
        target_local_id: LocalId("weapon".into()),
        field_path: vec!["Sprite2D".into(), "asset".into()],
        value: serde_json::json!("cannon.png"),
        status: OverrideStatus::Active,
    };

    let json = serde_json::to_string(&patch).expect("serialize OverridePatch");
    let roundtripped: OverridePatch =
        serde_json::from_str(&json).expect("deserialize OverridePatch");

    assert_eq!(
        roundtripped.status,
        OverrideStatus::Active,
        "OverrideStatus::Active should round-trip correctly"
    );

    // Assert the enum has exactly 4 variants by exhaustive match.
    // If a new variant is added to OverrideStatus, this test will fail to compile
    // unless this match is updated — providing a compile-time exhaustiveness check.
    match patch.status {
        OverrideStatus::Active
        | OverrideStatus::Orphaned
        | OverrideStatus::Stale
        | OverrideStatus::Conflict => {
            // Exhaustiveness confirmed: only the 4 expected variants exist.
        }
    }

    // Also verify serde output uses snake_case lowercase.
    assert!(
        json.contains("\"active\""),
        "serde should output snake_case lowercase: expected 'active', got: {}",
        json
    );
}

#[test]
fn s8_local_path_and_name_independent_of_local_id() {
    // Construct a SceneAssetEntity with local_id, local_path, and name.
    let entity = SceneAssetEntity {
        local_id: LocalId("abc".into()),
        local_path: "root/weapon".into(),
        name: "Weapon".into(),
        components: vec![],
    };

    // Serialize and deserialize.
    let json = serde_json::to_string(&entity).expect("serialize SceneAssetEntity");
    let mut roundtripped: SceneAssetEntity =
        serde_json::from_str(&json).expect("deserialize SceneAssetEntity");

    // Verify initial state.
    assert_eq!(roundtripped.local_id.as_str(), "abc");
    assert_eq!(roundtripped.local_path, "root/weapon");
    assert_eq!(roundtripped.name, "Weapon");

    // Mutate the name field — local_id and local_path must stay unchanged.
    roundtripped.name = "Cannon".to_string();

    assert_eq!(
        roundtripped.local_id.as_str(),
        "abc",
        "local_id must NOT change when name is mutated"
    );
    assert_eq!(
        roundtripped.local_path, "root/weapon",
        "local_path must NOT change when name is mutated"
    );
    assert_eq!(
        roundtripped.name, "Cannon",
        "name should have changed to 'Cannon'"
    );
}

#[test]
fn s10_local_id_and_stable_id_are_distinct_types() {
    // Use std::any::TypeId to assert LocalId and StableId are distinct types.
    // This is a runtime assertion that confirms the type-system guarantee.
    let local_id_type = TypeId::of::<LocalId>();
    let stable_id_type = TypeId::of::<StableId>();

    assert_ne!(
        local_id_type, stable_id_type,
        "LocalId and StableId must be distinct types (TypeId check)"
    );

    // Additionally demonstrate compile-time safety via function signatures.
    // These helper functions accept only their specific type.
    fn accepts_local_id(_: LocalId) {}
    fn accepts_stable_id(_: StableId) {}

    let lid = LocalId("root".into());
    let sid = StableId::new("ent_a");

    // Each function accepts only its own type — this compiles.
    accepts_local_id(lid.clone());
    accepts_stable_id(sid.clone());

    // The following would NOT compile (commented out to allow the test to compile):
    // accepts_stable_id(lid);   // Error: expected StableId, found LocalId
    // accepts_local_id(sid);   // Error: expected LocalId, found StableId
    //
    // This compile-time isolation is the type-system guarantee spec S10 requires.

    // Extra runtime confirmation: they remain distinct even when cloned.
    let lid2 = lid.clone();
    let sid2 = sid.clone();
    assert_ne!(TypeId::of::<LocalId>(), TypeId::of::<StableId>());
    assert_eq!(lid2, lid);
    assert_eq!(sid2, sid);
}
