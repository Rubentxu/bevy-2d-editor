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

/// Project all Scene Instances in a document into preview entities.
///
/// For each instance in `doc.instances`, this function:
/// 1. Resolves the asset reference via the provided closure
/// 2. Runs `effective_values` to compute the resolved component values
/// 3. Maps each resolved entity to its StableId via the persisted `id_map`
///
/// The returned `PreviewEntity` values carry the `stable_id` from the id_map,
/// NOT from `effective_values`' throwaway mint — this is the design D3 decision.
///
/// # Arguments
///
/// * `doc` - The SceneDocument containing instances to project
/// * `resolve` - Closure that resolves an AssetReference to a SceneAssetDocument
///
/// # Returns
///
/// Vec of PreviewEntity, one per instance entity. Empty Vec if no instances.
pub fn project_instances(
    doc: &crate::document::SceneDocument,
    resolve: &dyn Fn(&crate::scene_asset::AssetReference) -> Option<SceneAssetDocument>,
) -> Vec<PreviewEntity> {
    use crate::scene_instance_overrides::effective_values;

    let mut results = Vec::new();

    for instance in doc.instances.values() {
        // Resolve the asset
        let asset = match resolve(&instance.asset_ref) {
            Some(a) => a,
            None => continue, // Missing asset — skip (UI should mark as broken)
        };

        // Run effective_values with a throwaway mint (we use id_map StableIds instead)
        let mut throwaway_counter = 0u32;
        let mut throwaway_mint = || {
            throwaway_counter += 1;
            crate::document::StableId::new(format!("throwaway_{}", throwaway_counter))
        };

        let resolved = match effective_values(&asset, instance, &mut throwaway_mint) {
            Ok(r) => r,
            Err(_) => continue, // Empty or invalid asset — skip
        };

        // Map each resolved entity to its StableId via the persisted id_map
        for (local_id, resolved_entity) in resolved.entities {
            // Get the StableId from the persisted id_map
            let stable_id = match instance.id_map.get(&local_id) {
                Some(sid) => sid.clone(),
                None => continue, // No mapping for this local_id — skip
            };

            results.push(PreviewEntity {
                stable_id,
                local_id: local_id.clone(),
                component_values: resolved_entity.components,
            });
        }
    }

    results
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
            layers: vec![],
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
