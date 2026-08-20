//! SceneAssetDialect — adapts `SceneAssetDocument` to the kernel `Graph` trait.
//!
//! The dialect filters to `RelationshipKind::Child` edges. `RelationshipKind::Custom(_)`
//! exists in the type but is never instantiated in the codebase today; consumers that
//! need multi-kind support should add a new dialect (or pass a kind-filter parameter)
//! — see GRAPH-002 spec §7 out-of-scope.
//!
//! The dialect is the canonical owner of "scene asset as a graph" semantics. Cycle
//! detection, root finding, descendants, topological sort, and ancestors all flow
//! through this single seam.

use std::collections::BTreeMap;

use crate::ids::SceneAssetLocalId;
use crate::scene_asset::{
    RelationshipKind, SceneAssetDocument, SceneAssetEntity, SceneAssetRelationship,
};

use super::{EdgeIndex, Graph, NodeIndex};

/// Adapter that lets `SceneAssetDocument` be read as a `Graph`.
///
/// Dialects are cheap to construct: they pre-compute the index map at binding time.
/// Dialects borrow the underlying asset; they are not owned.
pub struct SceneAssetDialect<'a> {
    asset: &'a SceneAssetDocument,
    node_index: BTreeMap<&'a SceneAssetLocalId, NodeIndex>,
}

impl<'a> SceneAssetDialect<'a> {
    /// Build a dialect view over `asset`. The dialect borrows `asset` for its
    /// lifetime.
    pub fn new(asset: &'a SceneAssetDocument) -> Self {
        let node_index = asset
            .entities
            .iter()
            .enumerate()
            .map(|(i, e)| (&e.local_id, NodeIndex(i as u32)))
            .collect();
        Self { asset, node_index }
    }

    /// Resolve a `SceneAssetLocalId` to its `NodeIndex` inside this dialect view.
    pub fn node_index_of(&self, id: &SceneAssetLocalId) -> Option<NodeIndex> {
        self.node_index.get(id).copied()
    }

    /// Borrow the underlying `SceneAssetDocument`.
    pub fn asset(&self) -> &SceneAssetDocument {
        self.asset
    }
}

impl<'a> Graph for SceneAssetDialect<'a> {
    type NodeData = SceneAssetEntity;
    type EdgeData = SceneAssetRelationship;
    type Error = std::convert::Infallible;

    fn node_count(&self) -> usize {
        self.asset.entities.len()
    }
    fn edge_count(&self) -> usize {
        self.asset.relationships.len()
    }
    fn node(&self, idx: NodeIndex) -> Option<&SceneAssetEntity> {
        self.asset.entities.get(idx.0 as usize)
    }
    fn edge(&self, idx: EdgeIndex) -> Option<&SceneAssetRelationship> {
        self.asset.relationships.get(idx.0 as usize)
    }
    fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        let r = self.edge(idx)?;
        if !matches!(r.kind, RelationshipKind::Child) {
            return None;
        }
        Some((
            *self.node_index.get(&r.from_local_id)?,
            *self.node_index.get(&r.to_local_id)?,
        ))
    }
    fn outgoing(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let source_id = self.node(node).map(|e| e.local_id.clone());
        Box::new(self.asset.relationships.iter().enumerate().filter_map(move |(i, r)| {
            if !matches!(r.kind, RelationshipKind::Child) {
                return None;
            }
            if Some(&r.from_local_id) == source_id.as_ref() {
                Some(EdgeIndex(i as u32))
            } else {
                None
            }
        }))
    }
    fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let target_id = self.node(node).map(|e| e.local_id.clone());
        Box::new(self.asset.relationships.iter().enumerate().filter_map(move |(i, r)| {
            if !matches!(r.kind, RelationshipKind::Child) {
                return None;
            }
            if Some(&r.to_local_id) == target_id.as_ref() {
                Some(EdgeIndex(i as u32))
            } else {
                None
            }
        }))
    }
}

#[cfg(test)]
mod dialect_tests {
    use super::*;
    use crate::graph_kernel::{descendants, has_cycle, roots, topological_sort};
    use crate::scene_asset::{SceneAssetMetadata, SceneAssetRole};
    use std::collections::BTreeMap;

    fn empty_asset() -> SceneAssetDocument {
        SceneAssetDocument {
            asset_id: "asset".to_string(),
            logical_path: "test/asset".to_string(),
            role: SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            layers: vec![],
            extension_data: BTreeMap::new(),
        }
    }

    fn entity(id: &str) -> SceneAssetEntity {
        SceneAssetEntity {
            local_id: SceneAssetLocalId::new(id),
            local_path: id.to_string(),
            name: id.to_string(),
            components: vec![],
            extension_data: BTreeMap::new(),
        }
    }

    fn child_edge(from: &str, to: &str) -> SceneAssetRelationship {
        SceneAssetRelationship {
            from_local_id: SceneAssetLocalId::new(from),
            to_local_id: SceneAssetLocalId::new(to),
            kind: RelationshipKind::Child,
            field_path: None,
        }
    }

    fn custom_edge(from: &str, to: &str) -> SceneAssetRelationship {
        SceneAssetRelationship {
            from_local_id: SceneAssetLocalId::new(from),
            to_local_id: SceneAssetLocalId::new(to),
            kind: RelationshipKind::Custom("owns".to_string()),
            field_path: None,
        }
    }

    #[test]
    fn dialect_translates_local_id_to_node_index() {
        let mut asset = empty_asset();
        asset.entities = vec![entity("a"), entity("b")];
        let d = SceneAssetDialect::new(&asset);
        assert_eq!(d.node_count(), 2);
        assert_eq!(
            d.node(d.node_index_of(&SceneAssetLocalId::new("a")).unwrap())
                .unwrap()
                .local_id
                .as_str(),
            "a"
        );
        assert_eq!(d.node_index_of(&SceneAssetLocalId::new("missing")), None);
    }

    #[test]
    fn dialect_skips_non_child_relationships() {
        let mut asset = empty_asset();
        asset.entities = vec![entity("a"), entity("b")];
        asset.relationships = vec![custom_edge("a", "b")];
        let d = SceneAssetDialect::new(&asset);
        // Both entities are roots because the only edge is Custom, not Child.
        let r = roots(&d);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn dialect_kernel_roots_matches_hierarchy_top_level() {
        let mut asset = empty_asset();
        asset.entities = vec![entity("root"), entity("child"), entity("grandchild")];
        asset.relationships = vec![
            child_edge("root", "child"),
            child_edge("child", "grandchild"),
        ];
        let d = SceneAssetDialect::new(&asset);
        let r = roots(&d);
        assert_eq!(r.len(), 1);
        assert_eq!(d.node(r[0]).unwrap().local_id.as_str(), "root");
    }

    #[test]
    fn dialect_kernel_descendants_chain() {
        let mut asset = empty_asset();
        asset.entities = vec![entity("root"), entity("child"), entity("grandchild")];
        asset.relationships = vec![
            child_edge("root", "child"),
            child_edge("child", "grandchild"),
        ];
        let d = SceneAssetDialect::new(&asset);
        let desc = descendants(&d, d.node_index_of(&SceneAssetLocalId::new("root")).unwrap());
        assert_eq!(desc.len(), 3);
    }

    #[test]
    fn dialect_kernel_cycle_detection_returns_true() {
        let mut asset = empty_asset();
        asset.entities = vec![entity("a"), entity("b"), entity("c")];
        asset.relationships = vec![
            child_edge("a", "b"),
            child_edge("b", "c"),
            child_edge("c", "a"),
        ];
        let d = SceneAssetDialect::new(&asset);
        assert!(has_cycle(&d));
    }

    #[test]
    fn dialect_kernel_topological_sort_orders_parents_before_children() {
        let mut asset = empty_asset();
        asset.entities = vec![entity("a"), entity("b"), entity("c")];
        asset.relationships = vec![
            child_edge("a", "b"),
            child_edge("b", "c"),
        ];
        let d = SceneAssetDialect::new(&asset);
        let sorted = topological_sort(&d).unwrap();
        assert_eq!(sorted.len(), 3);
        let positions: BTreeMap<&str, usize> = (0..3)
            .map(|i| (d.node(sorted[i]).unwrap().local_id.as_str(), i))
            .collect();
        assert!(positions["a"] < positions["b"]);
        assert!(positions["b"] < positions["c"]);
    }

    #[test]
    fn dialect_outgoing_and_incoming_count_child_edges_only() {
        let mut asset = empty_asset();
        asset.entities = vec![entity("a"), entity("b"), entity("c")];
        // Mixed: child a->b, custom a->c, child b->c
        asset.relationships = vec![
            child_edge("a", "b"),
            custom_edge("a", "c"),
            child_edge("b", "c"),
        ];
        let d = SceneAssetDialect::new(&asset);
        let out_a: Vec<EdgeIndex> = d
            .outgoing(d.node_index_of(&SceneAssetLocalId::new("a")).unwrap())
            .collect();
        let out_b: Vec<EdgeIndex> = d
            .outgoing(d.node_index_of(&SceneAssetLocalId::new("b")).unwrap())
            .collect();
        let in_c: Vec<EdgeIndex> = d
            .incoming(d.node_index_of(&SceneAssetLocalId::new("c")).unwrap())
            .collect();
        // a's outgoing = [child(a,b)] only (custom filtered out).
        // Indexes in the relationships vec are 0 (child a->b), 1 (custom a->c), 2 (child b->c).
        assert_eq!(out_a, vec![EdgeIndex(0)]);
        assert_eq!(out_b, vec![EdgeIndex(2)]);
        // c's incoming = [child(b,c)] only (no edge targets c in this graph).
        assert_eq!(in_c, vec![EdgeIndex(2)]);
    }
}
