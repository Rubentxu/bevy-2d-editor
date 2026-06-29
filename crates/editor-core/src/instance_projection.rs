//! Instance projection — maps Scene Asset documents to preview entities.
//!
//! Design decision D5: `root_local_ids` gate that returns only top-level
//! LocalIds (entities with no incoming Child relationships).

use std::collections::BTreeMap;

use crate::scene_asset::{LocalId, SceneAssetDocument, SceneAssetRelationship, RelationshipKind};

/// A projected entity in the preview world, derived from a Scene Asset.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewEntity {
    /// The StableId assigned during instance placement.
    pub stable_id: crate::document::StableId,
    /// The LocalId from the source asset.
    pub local_id: LocalId,
    /// Component values from the asset entity.
    pub component_values: Vec<crate::document::ComponentInstance>,
}

/// Returns all root LocalIds from a SceneAssetDocument.
///
/// A root is an entity with NO incoming Child relationships
/// from any other entity in the asset.
///
/// Design decision D5 gate:
///
/// - empty asset → 0 roots
/// - single entity with no children → 1 root
/// - multiple entities with clear hierarchy → count of top-level entities
///
/// # Arguments
///
/// * `asset` - The SceneAssetDocument to extract roots from
///
/// # Returns
///
/// Vec of LocalId representing root entities
pub fn root_local_ids(asset: &SceneAssetDocument) -> Vec<LocalId> {
    if asset.entities.is_empty() {
        return Vec::new();
    }

    // Build a set of all entities that have an incoming Child relationship
    let mut has_incoming_child: BTreeMap<LocalId, bool> = BTreeMap::new();
    for entity in &asset.entities {
        has_incoming_child.insert(entity.local_id.clone(), false);
    }

    for rel in &asset.relationships {
        if matches!(rel.kind, RelationshipKind::Child) {
            has_incoming_child.insert(rel.to_local_id.clone(), true);
        }
    }

    // Roots are entities with no incoming Child relationships
    asset
        .entities
        .iter()
        .filter(|e| {
            !has_incoming_child
                .get(&e.local_id)
                .copied()
                .unwrap_or(false)
        })
        .map(|e| e.local_id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_asset::{SceneAssetEntity, SceneAssetRole};

    fn make_asset(
        entities: Vec<SceneAssetEntity>,
        relationships: Vec<SceneAssetRelationship>,
    ) -> SceneAssetDocument {
        SceneAssetDocument {
            asset_id: "test-asset".to_string(),
            logical_path: "assets/test.bsn".to_string(),
            role: SceneAssetRole::Actor,
            version: 1,
            entities,
            relationships,
            exposed_properties: vec![],
            metadata: Default::default(),
        }
    }

    #[test]
    fn root_local_ids_empty_asset_returns_empty() {
        let asset = make_asset(vec![], vec![]);
        let roots = root_local_ids(&asset);
        assert!(roots.is_empty(), "Empty asset should have 0 roots");
    }

    #[test]
    fn root_local_ids_single_entity_returns_that_entity() {
        let entity = SceneAssetEntity {
            local_id: LocalId("root".to_string()),
            local_path: "root".to_string(),
            name: "Single Entity".to_string(),
            components: vec![],
        };
        let asset = make_asset(vec![entity], vec![]);
        let roots = root_local_ids(&asset);
        assert_eq!(roots.len(), 1, "Single entity should have 1 root");
        assert_eq!(roots[0].as_str(), "root");
    }

    #[test]
    fn root_local_ids_multi_entity_hierarchy_returns_only_toplevel() {
        // root -> child (root has outgoing Child to child, child has incoming)
        let root = SceneAssetEntity {
            local_id: LocalId("root".to_string()),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![],
        };
        let child = SceneAssetEntity {
            local_id: LocalId("child".to_string()),
            local_path: "root/child".to_string(),
            name: "Child".to_string(),
            components: vec![],
        };
        let relationships = vec![SceneAssetRelationship {
            from_local_id: LocalId("root".to_string()),
            to_local_id: LocalId("child".to_string()),
            kind: RelationshipKind::Child,
            field_path: None,
        }];
        let asset = make_asset(vec![root, child], relationships);

        let roots = root_local_ids(&asset);
        assert_eq!(roots.len(), 1, "Hierarchy should have 1 root");
        assert_eq!(roots[0].as_str(), "root");
    }

    #[test]
    fn root_local_ids_two_roots_no_relationships() {
        // Two unrelated entities - both are roots
        let entity1 = SceneAssetEntity {
            local_id: LocalId("entity1".to_string()),
            local_path: "entity1".to_string(),
            name: "Entity 1".to_string(),
            components: vec![],
        };
        let entity2 = SceneAssetEntity {
            local_id: LocalId("entity2".to_string()),
            local_path: "entity2".to_string(),
            name: "Entity 2".to_string(),
            components: vec![],
        };
        let asset = make_asset(vec![entity1, entity2], vec![]);

        let roots = root_local_ids(&asset);
        assert_eq!(
            roots.len(),
            2,
            "Two unrelated entities should both be roots"
        );
    }

    #[test]
    fn root_local_ids_deep_hierarchy_only_top_level_returned() {
        // root -> child1 -> grandchild (only root is a root)
        let root = SceneAssetEntity {
            local_id: LocalId("root".to_string()),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![],
        };
        let child1 = SceneAssetEntity {
            local_id: LocalId("child1".to_string()),
            local_path: "root/child1".to_string(),
            name: "Child1".to_string(),
            components: vec![],
        };
        let grandchild = SceneAssetEntity {
            local_id: LocalId("grandchild".to_string()),
            local_path: "root/child1/grandchild".to_string(),
            name: "Grandchild".to_string(),
            components: vec![],
        };
        let relationships = vec![
            SceneAssetRelationship {
                from_local_id: LocalId("root".to_string()),
                to_local_id: LocalId("child1".to_string()),
                kind: RelationshipKind::Child,
                field_path: None,
            },
            SceneAssetRelationship {
                from_local_id: LocalId("child1".to_string()),
                to_local_id: LocalId("grandchild".to_string()),
                kind: RelationshipKind::Child,
                field_path: None,
            },
        ];
        let asset = make_asset(vec![root, child1, grandchild], relationships);

        let roots = root_local_ids(&asset);
        assert_eq!(roots.len(), 1, "Deep hierarchy should have only 1 root");
        assert_eq!(roots[0].as_str(), "root");
    }

    #[test]
    fn root_local_ids_multiple_roots_with_relationships() {
        // Two separate trees - each has its own root
        let root1 = SceneAssetEntity {
            local_id: LocalId("root1".to_string()),
            local_path: "root1".to_string(),
            name: "Root1".to_string(),
            components: vec![],
        };
        let child1 = SceneAssetEntity {
            local_id: LocalId("child1".to_string()),
            local_path: "root1/child1".to_string(),
            name: "Child1".to_string(),
            components: vec![],
        };
        let root2 = SceneAssetEntity {
            local_id: LocalId("root2".to_string()),
            local_path: "root2".to_string(),
            name: "Root2".to_string(),
            components: vec![],
        };
        let relationships = vec![SceneAssetRelationship {
            from_local_id: LocalId("root1".to_string()),
            to_local_id: LocalId("child1".to_string()),
            kind: RelationshipKind::Child,
            field_path: None,
        }];
        let asset = make_asset(vec![root1, child1, root2], relationships);

        let roots = root_local_ids(&asset);
        assert_eq!(roots.len(), 2, "Two separate trees should have 2 roots");
        assert!(roots.iter().any(|r| r.as_str() == "root1"));
        assert!(roots.iter().any(|r| r.as_str() == "root2"));
    }
}
