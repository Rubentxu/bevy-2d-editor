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
//!
//! Two variants are provided:
//! - `SceneAssetDialect<'a>` — read-only view over `&'a SceneAssetDocument`
//! - `SceneAssetDialectMut<'a>` — mutable view over `&'a mut SceneAssetDocument`

use std::collections::BTreeMap;

use crate::ids::SceneAssetLocalId;
use crate::scene_asset::{
    RelationshipKind, SceneAssetDocument, SceneAssetEntity, SceneAssetRelationship,
};

use super::{EdgeIndex, Graph, GraphKernelError, GraphMut, GraphMutStrictness, NodeIndex};

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

// ============================================================================
// SceneAssetDialectMut — mutable dialect.
// ============================================================================

/// Mutable adapter that owns `&'a mut SceneAssetDocument` and implements `GraphMut`.
///
/// This dialect enforces DAG semantics (Dag strictness): no cycles, no self-loops,
/// no duplicate edges. Scene asset hierarchies must be strict trees.
pub struct SceneAssetDialectMut<'a> {
    /// The owned mutable reference to the document.
    doc: &'a mut SceneAssetDocument,
    /// Maps stable local ID to NodeIndex. Rebuilt on every mutation.
    entity_index: BTreeMap<SceneAssetLocalId, NodeIndex>,
    /// Maps (from_local_id, to_local_id) to EdgeIndex for Child relationships.
    rel_index: BTreeMap<(SceneAssetLocalId, SceneAssetLocalId), EdgeIndex>,
}

impl<'a> SceneAssetDialectMut<'a> {
    /// Build a mutable dialect over `doc`. The dialect borrows `doc` for
    /// its lifetime.
    pub fn new(doc: &'a mut SceneAssetDocument) -> Self {
        let entity_index: BTreeMap<SceneAssetLocalId, NodeIndex> = doc
            .entities
            .iter()
            .enumerate()
            .map(|(i, e)| (e.local_id.clone(), NodeIndex(i as u32)))
            .collect();

        let rel_index: BTreeMap<(SceneAssetLocalId, SceneAssetLocalId), EdgeIndex> = doc
            .relationships
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                if matches!(r.kind, RelationshipKind::Child) {
                    Some(((r.from_local_id.clone(), r.to_local_id.clone()), EdgeIndex(i as u32)))
                } else {
                    None
                }
            })
            .collect();

        Self {
            doc,
            entity_index,
            rel_index,
        }
    }

    /// Resolve a `SceneAssetLocalId` to its `NodeIndex` inside this dialect view.
    pub fn node_index_of(&self, id: &SceneAssetLocalId) -> Option<NodeIndex> {
        self.entity_index.get(id).copied()
    }

    /// Rebuild the entity index from the current doc.entities vec.
    fn rebuild_entity_index(&mut self) {
        self.entity_index = self.doc.entities.iter()
            .enumerate()
            .map(|(i, e)| (e.local_id.clone(), NodeIndex(i as u32)))
            .collect();
    }

    /// Rebuild the relationship index from the current doc.relationships vec (Child only).
    fn rebuild_rel_index(&mut self) {
        self.rel_index = self.doc.relationships.iter()
            .enumerate()
            .filter_map(|(i, r)| {
                if matches!(r.kind, RelationshipKind::Child) {
                    Some(((r.from_local_id.clone(), r.to_local_id.clone()), EdgeIndex(i as u32)))
                } else {
                    None
                }
            })
            .collect();
    }
}

impl<'a> Graph for SceneAssetDialectMut<'a> {
    type NodeData = SceneAssetEntity;
    type EdgeData = SceneAssetRelationship;
    type Error = std::convert::Infallible;

    fn node_count(&self) -> usize {
        self.doc.entities.len()
    }

    fn edge_count(&self) -> usize {
        // Only Child relationships count as edges.
        self.doc.relationships.iter()
            .filter(|r| matches!(r.kind, RelationshipKind::Child))
            .count()
    }

    fn node(&self, idx: NodeIndex) -> Option<&SceneAssetEntity> {
        self.doc.entities.get(idx.0 as usize)
    }

    fn edge(&self, idx: EdgeIndex) -> Option<&SceneAssetRelationship> {
        self.doc.relationships.get(idx.0 as usize)
    }

    fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        let r = self.edge(idx)?;
        if !matches!(r.kind, RelationshipKind::Child) {
            return None;
        }
        Some((
            *self.entity_index.get(&r.from_local_id)?,
            *self.entity_index.get(&r.to_local_id)?,
        ))
    }

    fn outgoing(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let source_id = self.node(node).map(|e| e.local_id.clone());
        Box::new(
            self.doc.relationships.iter()
                .enumerate()
                .filter_map(move |(i, r)| {
                    if !matches!(r.kind, RelationshipKind::Child) {
                        return None;
                    }
                    if Some(&r.from_local_id) == source_id.as_ref() {
                        Some(EdgeIndex(i as u32))
                    } else {
                        None
                    }
                }),
        )
    }

    fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let target_id = self.node(node).map(|e| e.local_id.clone());
        Box::new(
            self.doc.relationships.iter()
                .enumerate()
                .filter_map(move |(i, r)| {
                    if !matches!(r.kind, RelationshipKind::Child) {
                        return None;
                    }
                    if Some(&r.to_local_id) == target_id.as_ref() {
                        Some(EdgeIndex(i as u32))
                    } else {
                        None
                    }
                }),
        )
    }
}

impl<'a> GraphMut for SceneAssetDialectMut<'a> {
    fn strictness(&self) -> GraphMutStrictness {
        GraphMutStrictness::Dag
    }

    fn add_node(&mut self, data: Self::NodeData) -> NodeIndex {
        let idx = NodeIndex(self.doc.entities.len() as u32);
        self.doc.entities.push(data);
        self.rebuild_entity_index();
        idx
    }

    fn add_edge(
        &mut self,
        src: NodeIndex,
        dst: NodeIndex,
        data: Self::EdgeData,
    ) -> Result<EdgeIndex, GraphKernelError> {
        if src.0 as usize >= self.doc.entities.len()
            || dst.0 as usize >= self.doc.entities.len()
        {
            return Err(GraphKernelError::NodeIndexOutOfRange {
                idx: if src.0 as usize >= self.doc.entities.len() { src } else { dst },
                total: self.doc.entities.len(),
            });
        }

        // Dag strictness: reject self-loops.
        if src == dst {
            return Err(GraphKernelError::SelfLoop { node: src });
        }

        let from_id = self.doc.entities.get(src.0 as usize).map(|e| e.local_id.clone());
        let to_id = self.doc.entities.get(dst.0 as usize).map(|e| e.local_id.clone());

        // Dag strictness: reject duplicate Child edges.
        if let (Some(fid), Some(tid)) = (&from_id, &to_id) {
            if self.rel_index.contains_key(&(fid.clone(), tid.clone())) {
                return Err(GraphKernelError::DuplicateEdge { src, dst });
            }
        }

        // Dag strictness: reject cycles.
        // Check if adding src→dst would create a cycle.
        // This happens if src is reachable from dst (dst →+ src exists).
        // We use the would_create_cycle helper.
        if super::would_create_cycle(self, src, dst) {
            // Compute participating nodes: ancestors of src plus src itself.
            let participating = super::ancestors(self, src);
            return Err(GraphKernelError::WouldCreateCycle { participating });
        }

        let idx = EdgeIndex(self.doc.relationships.len() as u32);
        self.doc.relationships.push(data);
        self.rebuild_rel_index();
        Ok(idx)
    }

    fn remove_node(&mut self, idx: NodeIndex) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.doc.entities.len() {
            return Err(GraphKernelError::NodeIndexOutOfRange {
                idx,
                total: self.doc.entities.len(),
            });
        }
        let removed_id = self.doc.entities[idx.0 as usize].local_id.clone();
        // Cascade: remove Child relationships where this entity is parent or child.
        self.doc.relationships.retain(|r| {
            !(matches!(r.kind, RelationshipKind::Child)
                && (r.from_local_id == removed_id || r.to_local_id == removed_id))
        });
        self.doc.entities.remove(idx.0 as usize);
        self.rebuild_entity_index();
        self.rebuild_rel_index();
        Ok(())
    }

    fn remove_edge(&mut self, idx: EdgeIndex) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.doc.relationships.len() {
            return Err(GraphKernelError::EdgeIndexOutOfRange {
                idx,
                total: self.doc.relationships.len(),
            });
        }
        self.doc.relationships.remove(idx.0 as usize);
        self.rebuild_rel_index();
        Ok(())
    }

    fn update_node(&mut self, _idx: NodeIndex, _data: Self::NodeData) -> Result<(), GraphKernelError> {
        // SceneAssetEntity is immutable by stable_id; update_node is unsupported.
        // Use AssetCommand::UpdateEntityComponents instead.
        Err(GraphKernelError::NodeIndexOutOfRange {
            idx: NodeIndex(0),
            total: 0,
        })
    }

    fn update_edge(&mut self, idx: EdgeIndex, data: Self::EdgeData) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.doc.relationships.len() {
            return Err(GraphKernelError::EdgeIndexOutOfRange {
                idx,
                total: self.doc.relationships.len(),
            });
        }
        self.doc.relationships[idx.0 as usize] = data;
        self.rebuild_rel_index();
        Ok(())
    }
}

// ============================================================================
// Tests for SceneAssetDialectMut.
// ============================================================================

#[cfg(test)]
mod scene_asset_dialect_mut_tests {
    use super::*;
    use crate::graph_kernel::{descendants, has_cycle, topological_sort};
    use crate::scene_asset::{SceneAssetMetadata, SceneAssetRole};
    use std::collections::BTreeMap;

    fn empty_doc() -> SceneAssetDocument {
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

    #[test]
    fn add_node_increments_count() {
        let mut doc = empty_doc();
        let mut d = SceneAssetDialectMut::new(&mut doc);
        assert_eq!(d.node_count(), 0);
        d.add_node(entity("a"));
        assert_eq!(d.node_count(), 1);
        d.add_node(entity("b"));
        assert_eq!(d.node_count(), 2);
        assert_eq!(doc.entities.len(), 2);
    }

    #[test]
    fn add_edge_no_self_loop() {
        let mut doc = empty_doc();
        doc.entities = vec![entity("a"), entity("b")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let a_idx = d.node_index_of(&SceneAssetLocalId::new("a")).unwrap();
        // This tests the positive case: valid child edge.
        let result = d.add_edge(a_idx, a_idx, child_edge("a", "a"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GraphKernelError::SelfLoop { .. }));
    }

    #[test]
    fn add_edge_rejects_self_loop() {
        let mut doc = empty_doc();
        doc.entities = vec![entity("a")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let a_idx = d.node_index_of(&SceneAssetLocalId::new("a")).unwrap();
        let result = d.add_edge(a_idx, a_idx, child_edge("a", "a"));
        assert!(matches!(result.unwrap_err(), GraphKernelError::SelfLoop { node } if node == a_idx));
    }

    #[test]
    fn add_edge_rejects_duplicate() {
        let mut doc = empty_doc();
        doc.entities = vec![entity("a"), entity("b")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let a_idx = d.node_index_of(&SceneAssetLocalId::new("a")).unwrap();
        let b_idx = d.node_index_of(&SceneAssetLocalId::new("b")).unwrap();
        // First add succeeds.
        let first = d.add_edge(a_idx, b_idx, child_edge("a", "b"));
        assert!(first.is_ok());
        // Second add of same edge fails.
        let second = d.add_edge(a_idx, b_idx, child_edge("a", "b"));
        assert!(matches!(second.unwrap_err(), GraphKernelError::DuplicateEdge { src, dst }
            if src == a_idx && dst == b_idx));
    }

    #[test]
    fn add_edge_rejects_cycle() {
        // a -> b -> c; adding c -> a should be rejected.
        let mut doc = empty_doc();
        doc.entities = vec![entity("a"), entity("b"), entity("c")];
        doc.relationships = vec![child_edge("a", "b"), child_edge("b", "c")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let a_idx = d.node_index_of(&SceneAssetLocalId::new("a")).unwrap();
        let c_idx = d.node_index_of(&SceneAssetLocalId::new("c")).unwrap();
        // Adding c->a would close the cycle.
        let result = d.add_edge(c_idx, a_idx, child_edge("c", "a"));
        assert!(matches!(result.unwrap_err(), GraphKernelError::WouldCreateCycle { .. }));
    }

    #[test]
    fn add_edge_dag_valid_passes() {
        // a -> b -> c (valid DAG).
        let mut doc = empty_doc();
        doc.entities = vec![entity("a"), entity("b"), entity("c")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let a_idx = d.node_index_of(&SceneAssetLocalId::new("a")).unwrap();
        let b_idx = d.node_index_of(&SceneAssetLocalId::new("b")).unwrap();
        let c_idx = d.node_index_of(&SceneAssetLocalId::new("c")).unwrap();
        assert!(d.add_edge(a_idx, b_idx, child_edge("a", "b")).is_ok());
        assert!(d.add_edge(b_idx, c_idx, child_edge("b", "c")).is_ok());
        assert!(!has_cycle(&d));
        let sorted = topological_sort(&d).unwrap();
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn remove_node_cascades_child_relationships() {
        // a -> b -> c; remove b should remove both edges.
        let mut doc = empty_doc();
        doc.entities = vec![entity("a"), entity("b"), entity("c")];
        doc.relationships = vec![child_edge("a", "b"), child_edge("b", "c")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let b_idx = d.node_index_of(&SceneAssetLocalId::new("b")).unwrap();
        d.remove_node(b_idx).unwrap();
        assert_eq!(d.node_count(), 2);
        assert_eq!(d.edge_count(), 0); // Both Child edges removed via cascade.
    }

    #[test]
    fn remove_node_cascades_parent_relationships() {
        // a -> b -> c; remove b should also remove a->b (where b is child).
        let mut doc = empty_doc();
        doc.entities = vec![entity("a"), entity("b"), entity("c")];
        doc.relationships = vec![child_edge("a", "b"), child_edge("b", "c")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let b_idx = d.node_index_of(&SceneAssetLocalId::new("b")).unwrap();
        d.remove_node(b_idx).unwrap();
        // Only entity 'a' and 'c' remain; no Child edges.
        assert_eq!(d.node_count(), 2);
        assert_eq!(d.edge_count(), 0);
    }

    #[test]
    fn remove_node_returns_error_for_out_of_range() {
        let mut doc = empty_doc();
        doc.entities = vec![entity("a")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let result = d.remove_node(NodeIndex(99));
        assert!(matches!(
            result.unwrap_err(),
            GraphKernelError::NodeIndexOutOfRange { idx, total }
            if idx == NodeIndex(99) && total == 1
        ));
    }

    #[test]
    fn update_node_returns_error() {
        // update_node is unsupported for SceneAssetEntity.
        let mut doc = empty_doc();
        doc.entities = vec![entity("a")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let a_idx = d.node_index_of(&SceneAssetLocalId::new("a")).unwrap();
        let result = d.update_node(a_idx, entity("a_updated"));
        // Returns a sentinel error (NodeIndexOutOfRange with total=0).
        assert!(result.is_err());
    }

    #[test]
    fn update_edge_replaces_data() {
        let mut doc = empty_doc();
        doc.entities = vec![entity("a"), entity("b")];
        doc.relationships = vec![child_edge("a", "b")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let edge_idx = EdgeIndex(0);
        let mut new_edge = child_edge("a", "b");
        new_edge.field_path = Some(vec!["transform".to_string(), "position".to_string()]);
        d.update_edge(edge_idx, new_edge).unwrap();
        assert!(d.edge(edge_idx).unwrap().field_path.is_some());
    }

    #[test]
    fn strictness_is_dag() {
        let mut doc = empty_doc();
        let d = SceneAssetDialectMut::new(&mut doc);
        assert_eq!(d.strictness(), GraphMutStrictness::Dag);
    }

    #[test]
    fn hierarchy_integrity_after_mutation_via_kernel_has_cycle() {
        // After adding a valid DAG edge, has_cycle should return false and topological sort should succeed.
        let mut doc = empty_doc();
        doc.entities = vec![entity("a"), entity("b"), entity("c")];
        let mut d = SceneAssetDialectMut::new(&mut doc);
        let a_idx = d.node_index_of(&SceneAssetLocalId::new("a")).unwrap();
        let b_idx = d.node_index_of(&SceneAssetLocalId::new("b")).unwrap();
        let c_idx = d.node_index_of(&SceneAssetLocalId::new("c")).unwrap();
        // Add a->b and b->c (valid DAG).
        d.add_edge(a_idx, b_idx, child_edge("a", "b")).unwrap();
        d.add_edge(b_idx, c_idx, child_edge("b", "c")).unwrap();
        assert!(!has_cycle(&d));
        let sorted = topological_sort(&d).unwrap();
        assert_eq!(sorted.len(), 3);
    }
}
