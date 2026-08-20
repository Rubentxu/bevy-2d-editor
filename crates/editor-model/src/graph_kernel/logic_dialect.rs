//! LogicGraphDialect — adapts `LogicGraphAsset` to the kernel `Graph` and `GraphMut` traits.
//!
//! Two variants are provided:
//! - `LogicGraphDialect<'a>` — read-only view over `&'a LogicGraphAsset`
//! - `LogicGraphDialectMut<'a>` — mutable view over `&'a mut LogicGraphAsset`

use std::collections::BTreeMap;
use std::convert::Infallible;

use crate::graph_kernel::{EdgeIndex, Graph, GraphKernelError, GraphMut, GraphMutStrictness, NodeIndex};
use crate::logic_graph::{LogicEdge, LogicGraphAsset, LogicNode, NodeId};

/// Adapter that lets `LogicGraphAsset` be read as a `Graph`.
///
/// Dialects are cheap to construct: they pre-compute index maps at binding
/// time. Dialects borrow the underlying asset; they are not owned.
pub struct LogicGraphDialect<'a> {
    asset: &'a LogicGraphAsset,
    node_index: BTreeMap<&'a NodeId, NodeIndex>,
}

impl<'a> LogicGraphDialect<'a> {
    /// Build a dialect view over `asset`. The dialect borrows `asset` for
    /// its lifetime.
    pub fn new(asset: &'a LogicGraphAsset) -> Self {
        let node_index = asset
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (&n.node_id, NodeIndex(i as u32)))
            .collect();
        Self { asset, node_index }
    }

    /// Resolve a `NodeId` to its `NodeIndex` inside this dialect view.
    pub fn node_index_of(&self, id: &NodeId) -> Option<NodeIndex> {
        self.node_index.get(id).copied()
    }
}

impl<'a> Graph for LogicGraphDialect<'a> {
    type NodeData = LogicNode;
    type EdgeData = LogicEdge;
    type Error = Infallible;

    fn node_count(&self) -> usize {
        self.asset.nodes.len()
    }
    fn edge_count(&self) -> usize {
        self.asset.edges.len()
    }
    fn node(&self, idx: NodeIndex) -> Option<&LogicNode> {
        self.asset.nodes.get(idx.0 as usize)
    }
    fn edge(&self, idx: EdgeIndex) -> Option<&LogicEdge> {
        self.asset.edges.get(idx.0 as usize)
    }
    fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        let e = self.edge(idx)?;
        Some((
            *self.node_index.get(&e.from_node)?,
            *self.node_index.get(&e.to_node)?,
        ))
    }
    fn outgoing(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let source_id = self.node(node).map(|n| n.node_id.clone());
        Box::new(self.asset.edges.iter().enumerate().filter_map(move |(i, e)| {
            if Some(&e.from_node) == source_id.as_ref() {
                Some(EdgeIndex(i as u32))
            } else {
                None
            }
        }))
    }
    fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let target_id = self.node(node).map(|n| n.node_id.clone());
        Box::new(self.asset.edges.iter().enumerate().filter_map(move |(i, e)| {
            if Some(&e.to_node) == target_id.as_ref() {
                Some(EdgeIndex(i as u32))
            } else {
                None
            }
        }))
    }
}

// ============================================================================
// LogicGraphDialectMut — mutable dialect.
// ============================================================================

/// Mutable adapter that owns `&'a mut LogicGraphAsset` and implements `GraphMut`.
///
/// This dialect rejects self-loops and duplicate edges (CyclicNoSelfLoop strictness),
/// but allows cycles in the graph (e.g. flip-flop bricks where actuators can feed
/// back into sensors).
pub struct LogicGraphDialectMut<'a> {
    /// The owned mutable reference to the asset.
    asset: &'a mut LogicGraphAsset,
    /// Maps stable NodeId to NodeIndex. Rebuilt on every mutation.
    /// Uses owned NodeId to avoid lifetime issues with the mutable borrow.
    node_index: BTreeMap<NodeId, NodeIndex>,
    /// Maps (from_node_id, to_node_id) to EdgeIndex. Rebuilt on every edge mutation.
    edge_index: BTreeMap<(NodeId, NodeId), EdgeIndex>,
}

impl<'a> LogicGraphDialectMut<'a> {
    /// Build a mutable dialect over `asset`. The dialect borrows `asset` for
    /// its lifetime.
    pub fn new(asset: &'a mut LogicGraphAsset) -> Self {
        let node_index: BTreeMap<NodeId, NodeIndex> = asset
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.node_id.clone(), NodeIndex(i as u32)))
            .collect();
        let edge_index: BTreeMap<(NodeId, NodeId), EdgeIndex> = asset
            .edges
            .iter()
            .enumerate()
            .filter_map(|(i, e)| Some(((e.from_node.clone(), e.to_node.clone()), EdgeIndex(i as u32))))
            .collect();
        Self {
            asset,
            node_index,
            edge_index,
        }
    }

    /// Resolve a `NodeId` to its `NodeIndex` inside this dialect view.
    pub fn node_index_of(&self, id: &NodeId) -> Option<NodeIndex> {
        self.node_index.get(id).copied()
    }

    /// Rebuild the node index from the current asset.nodes vec.
    fn rebuild_node_index(&mut self) {
        self.node_index = self.asset.nodes.iter()
            .enumerate()
            .map(|(i, n)| (n.node_id.clone(), NodeIndex(i as u32)))
            .collect();
    }

    /// Rebuild the edge index from the current asset.edges vec.
    fn rebuild_edge_index(&mut self) {
        self.edge_index = self.asset.edges.iter()
            .enumerate()
            .filter_map(|(i, e)| Some(((e.from_node.clone(), e.to_node.clone()), EdgeIndex(i as u32))))
            .collect();
    }
}

impl<'a> Graph for LogicGraphDialectMut<'a> {
    type NodeData = LogicNode;
    type EdgeData = LogicEdge;
    type Error = Infallible;

    fn node_count(&self) -> usize {
        self.asset.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.asset.edges.len()
    }

    fn node(&self, idx: NodeIndex) -> Option<&LogicNode> {
        self.asset.nodes.get(idx.0 as usize)
    }

    fn edge(&self, idx: EdgeIndex) -> Option<&LogicEdge> {
        self.asset.edges.get(idx.0 as usize)
    }

    fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        let e = self.edge(idx)?;
        Some((
            *self.node_index.get(&e.from_node)?,
            *self.node_index.get(&e.to_node)?,
        ))
    }

    fn outgoing(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let source_id = self.node(node).map(|n| n.node_id.clone());
        Box::new(self.asset.edges.iter().enumerate().filter_map(move |(i, e)| {
            if Some(&e.from_node) == source_id.as_ref() {
                Some(EdgeIndex(i as u32))
            } else {
                None
            }
        }))
    }

    fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let target_id = self.node(node).map(|n| n.node_id.clone());
        Box::new(self.asset.edges.iter().enumerate().filter_map(move |(i, e)| {
            if Some(&e.to_node) == target_id.as_ref() {
                Some(EdgeIndex(i as u32))
            } else {
                None
            }
        }))
    }
}

impl<'a> GraphMut for LogicGraphDialectMut<'a> {
    fn strictness(&self) -> GraphMutStrictness {
        GraphMutStrictness::CyclicNoSelfLoop
    }

    fn add_node(&mut self, data: Self::NodeData) -> NodeIndex {
        let idx = NodeIndex(self.asset.nodes.len() as u32);
        self.asset.nodes.push(data);
        self.rebuild_node_index();
        idx
    }

    fn add_edge(
        &mut self,
        src: NodeIndex,
        dst: NodeIndex,
        data: Self::EdgeData,
    ) -> Result<EdgeIndex, GraphKernelError> {
        if src.0 as usize >= self.asset.nodes.len()
            || dst.0 as usize >= self.asset.nodes.len()
        {
            return Err(GraphKernelError::NodeIndexOutOfRange {
                idx: if src.0 as usize >= self.asset.nodes.len() { src } else { dst },
                total: self.asset.nodes.len(),
            });
        }

        // CyclicNoSelfLoop: reject self-loops.
        if src == dst {
            return Err(GraphKernelError::SelfLoop { node: src });
        }

        // CyclicNoSelfLoop: reject duplicate edges.
        let from_id = self.asset.nodes.get(src.0 as usize).map(|n| n.node_id.clone());
        let to_id = self.asset.nodes.get(dst.0 as usize).map(|n| n.node_id.clone());
        if let (Some(fid), Some(tid)) = (&from_id, &to_id) {
            if self.edge_index.contains_key(&(fid.clone(), tid.clone())) {
                return Err(GraphKernelError::DuplicateEdge { src, dst });
            }
        }

        let idx = EdgeIndex(self.asset.edges.len() as u32);
        self.asset.edges.push(data);
        self.rebuild_edge_index();
        Ok(idx)
    }

    fn remove_node(&mut self, idx: NodeIndex) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.asset.nodes.len() {
            return Err(GraphKernelError::NodeIndexOutOfRange {
                idx,
                total: self.asset.nodes.len(),
            });
        }
        let removed_id = self.asset.nodes[idx.0 as usize].node_id.clone();
        self.asset.edges.retain(|e| e.from_node != removed_id && e.to_node != removed_id);
        self.asset.nodes.remove(idx.0 as usize);
        self.rebuild_node_index();
        self.rebuild_edge_index();
        Ok(())
    }

    fn remove_edge(&mut self, idx: EdgeIndex) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.asset.edges.len() {
            return Err(GraphKernelError::EdgeIndexOutOfRange {
                idx,
                total: self.asset.edges.len(),
            });
        }
        self.asset.edges.remove(idx.0 as usize);
        self.rebuild_edge_index();
        Ok(())
    }

    fn update_node(&mut self, idx: NodeIndex, data: Self::NodeData) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.asset.nodes.len() {
            return Err(GraphKernelError::NodeIndexOutOfRange {
                idx,
                total: self.asset.nodes.len(),
            });
        }
        self.asset.nodes[idx.0 as usize] = data;
        self.rebuild_node_index();
        Ok(())
    }

    fn update_edge(&mut self, idx: EdgeIndex, data: Self::EdgeData) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.asset.edges.len() {
            return Err(GraphKernelError::EdgeIndexOutOfRange {
                idx,
                total: self.asset.edges.len(),
            });
        }
        self.asset.edges[idx.0 as usize] = data;
        self.rebuild_edge_index();
        Ok(())
    }
}

// ============================================================================
// Tests for LogicGraphDialect and LogicGraphDialectMut.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_kernel::{has_cycle, topological_sort};
    use crate::logic_graph::{LogicNodeRole, NodeTypeId, PortId};

    fn sample_node(id: &str, role: LogicNodeRole) -> LogicNode {
        LogicNode {
            node_id: NodeId::new(id),
            role,
            node_type_id: NodeTypeId::new("sensor.generic"),
            field_values: serde_json::Value::Null,
            controller_id: None,
        }
    }

    fn sample_edge(from: &str, to: &str) -> LogicEdge {
        LogicEdge {
            from_node: NodeId::new(from),
            from_port: PortId::new("out"),
            to_node: NodeId::new(to),
            to_port: PortId::new("in"),
        }
    }

    // --- LogicGraphDialect (read-only) tests ---

    #[test]
    fn dialect_translates_node_id_to_node_index() {
        let mut g = LogicGraphAsset::default();
        g.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
        ];
        let d = LogicGraphDialect::new(&g);
        assert_eq!(d.node_count(), 2);
        assert_eq!(
            d.node(d.node_index_of(&NodeId::new("a")).unwrap())
                .unwrap()
                .node_id
                .as_str(),
            "a"
        );
        assert_eq!(d.node_index_of(&NodeId::new("missing")), None);
    }

    #[test]
    fn dialect_outgoing_and_incoming() {
        let mut g = LogicGraphAsset::default();
        g.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
            sample_node("c", LogicNodeRole::Actuator),
        ];
        g.edges = vec![sample_edge("a", "b"), sample_edge("b", "c")];
        let d = LogicGraphDialect::new(&g);
        let out_a: Vec<EdgeIndex> = d
            .outgoing(d.node_index_of(&NodeId::new("a")).unwrap())
            .collect();
        assert_eq!(out_a, vec![EdgeIndex(0)]);
        let in_b: Vec<EdgeIndex> = d
            .incoming(d.node_index_of(&NodeId::new("b")).unwrap())
            .collect();
        assert_eq!(in_b, vec![EdgeIndex(0)]);
        let out_b: Vec<EdgeIndex> = d
            .outgoing(d.node_index_of(&NodeId::new("b")).unwrap())
            .collect();
        assert_eq!(out_b, vec![EdgeIndex(1)]);
    }

    #[test]
    fn dialect_kernel_descendants() {
        let mut g = LogicGraphAsset::default();
        g.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
        ];
        g.edges = vec![sample_edge("a", "b")];
        let d = LogicGraphDialect::new(&g);
        let desc = crate::graph_kernel::descendants(
            &d,
            d.node_index_of(&NodeId::new("a")).unwrap(),
        );
        assert_eq!(desc.len(), 2);
        assert_eq!(desc[0], d.node_index_of(&NodeId::new("a")).unwrap());
    }

    #[test]
    fn dialect_kernel_topological_sort() {
        let mut g = LogicGraphAsset::default();
        g.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
            sample_node("c", LogicNodeRole::Actuator),
        ];
        g.edges = vec![sample_edge("a", "b"), sample_edge("b", "c")];
        let d = LogicGraphDialect::new(&g);
        let sorted = topological_sort(&d).unwrap();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], d.node_index_of(&NodeId::new("a")).unwrap());
        assert_eq!(sorted[2], d.node_index_of(&NodeId::new("c")).unwrap());
    }

    #[test]
    fn dialect_kernel_cycle_detection() {
        let mut g = LogicGraphAsset::default();
        g.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
            sample_node("c", LogicNodeRole::Actuator),
        ];
        g.edges = vec![
            sample_edge("a", "b"),
            sample_edge("b", "c"),
            sample_edge("c", "a"),
        ];
        let d = LogicGraphDialect::new(&g);
        assert!(has_cycle(&d));
    }

    // --- LogicGraphDialectMut (mutable) tests ---

    #[test]
    fn add_node_increments_count() {
        let mut asset = LogicGraphAsset::default();
        {
            let mut d = LogicGraphDialectMut::new(&mut asset);
            assert_eq!(d.node_count(), 0);
            d.add_node(sample_node("a", LogicNodeRole::Sensor));
            assert_eq!(d.node_count(), 1);
            d.add_node(sample_node("b", LogicNodeRole::Controller));
            assert_eq!(d.node_count(), 2);
        }
        // Asset was mutated.
        assert_eq!(asset.nodes.len(), 2);
    }

    #[test]
    fn add_node_returns_valid_index() {
        let mut asset = LogicGraphAsset::default();
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let idx0 = d.add_node(sample_node("a", LogicNodeRole::Sensor));
        let idx1 = d.add_node(sample_node("b", LogicNodeRole::Controller));
        assert_eq!(idx0, NodeIndex(0));
        assert_eq!(idx1, NodeIndex(1));
    }

    #[test]
    fn add_edge_no_self_loop() {
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
        ];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let a_idx = d.node_index_of(&NodeId::new("a")).unwrap();
        let result = d.add_edge(a_idx, a_idx, sample_edge("a", "a"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GraphKernelError::SelfLoop { .. }));
    }

    #[test]
    fn add_edge_rejects_self_loop() {
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![sample_node("a", LogicNodeRole::Sensor)];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let a_idx = d.node_index_of(&NodeId::new("a")).unwrap();
        let result = d.add_edge(a_idx, a_idx, sample_edge("a", "a"));
        assert!(matches!(result.unwrap_err(), GraphKernelError::SelfLoop { node } if node == a_idx));
    }

    #[test]
    fn add_edge_rejects_duplicate() {
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
        ];
        // NOTE: do NOT pre-populate edges; we use add_edge for the first insertion.
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let a_idx = d.node_index_of(&NodeId::new("a")).unwrap();
        let b_idx = d.node_index_of(&NodeId::new("b")).unwrap();
        // First add should succeed.
        let first = d.add_edge(a_idx, b_idx, sample_edge("a", "b"));
        assert!(first.is_ok());
        // Second add of same edge should fail with DuplicateEdge.
        let second = d.add_edge(a_idx, b_idx, sample_edge("a", "b"));
        assert!(matches!(second.unwrap_err(), GraphKernelError::DuplicateEdge { src, dst }
            if src == a_idx && dst == b_idx));
    }

    #[test]
    fn add_edge_allows_cycle() {
        // Logic graphs allow cycles (e.g. flip-flop bricks).
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
            sample_node("c", LogicNodeRole::Actuator),
        ];
        asset.edges = vec![sample_edge("a", "b"), sample_edge("b", "c")];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let a_idx = d.node_index_of(&NodeId::new("a")).unwrap();
        let c_idx = d.node_index_of(&NodeId::new("c")).unwrap();
        // Adding c->a closes the cycle; CyclicNoSelfLoop allows it.
        let result = d.add_edge(c_idx, a_idx, sample_edge("c", "a"));
        assert!(result.is_ok());
        // The resulting graph has a cycle.
        assert!(has_cycle(&d));
    }

    #[test]
    fn remove_node_cascades_edges() {
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
            sample_node("c", LogicNodeRole::Actuator),
        ];
        asset.edges = vec![sample_edge("a", "b"), sample_edge("b", "c")];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let b_idx = d.node_index_of(&NodeId::new("b")).unwrap();
        d.remove_node(b_idx).unwrap();
        assert_eq!(d.node_count(), 2);
        assert_eq!(d.edge_count(), 0); // Both edges removed via cascade.
    }

    #[test]
    fn remove_node_returns_error_for_out_of_range() {
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![sample_node("a", LogicNodeRole::Sensor)];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let result = d.remove_node(NodeIndex(99));
        assert!(matches!(
            result.unwrap_err(),
            GraphKernelError::NodeIndexOutOfRange { idx, total } if idx == NodeIndex(99) && total == 1
        ));
    }

    #[test]
    fn remove_edge_returns_error_for_out_of_range() {
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![sample_node("a", LogicNodeRole::Sensor)];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let result = d.remove_edge(EdgeIndex(99));
        assert!(matches!(
            result.unwrap_err(),
            GraphKernelError::EdgeIndexOutOfRange { idx, total } if idx == EdgeIndex(99) && total == 0
        ));
    }

    #[test]
    fn update_node_replaces_data() {
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![sample_node("a", LogicNodeRole::Sensor)];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let a_idx = d.node_index_of(&NodeId::new("a")).unwrap();
        let mut new_node = sample_node("a", LogicNodeRole::Actuator);
        new_node.node_type_id = NodeTypeId::new("actuator.generic");
        d.update_node(a_idx, new_node).unwrap();
        assert_eq!(
            d.node(a_idx).unwrap().node_type_id.as_str(),
            "actuator.generic"
        );
    }

    #[test]
    fn update_edge_replaces_data() {
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
        ];
        asset.edges = vec![sample_edge("a", "b")];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let edge_idx = EdgeIndex(0);
        let mut new_edge = sample_edge("a", "b");
        new_edge.from_port = PortId::new("out2");
        d.update_edge(edge_idx, new_edge).unwrap();
        assert_eq!(d.edge(edge_idx).unwrap().from_port.as_str(), "out2");
    }

    #[test]
    fn index_stability_after_unrelated_mutation() {
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
            sample_node("c", LogicNodeRole::Actuator),
        ];
        asset.edges = vec![sample_edge("a", "b"), sample_edge("b", "c")];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let a_idx = d.node_index_of(&NodeId::new("a")).unwrap();
        let b_idx = d.node_index_of(&NodeId::new("b")).unwrap();
        // Add an unrelated node.
        d.add_node(sample_node("d", LogicNodeRole::Sensor));
        // Original indices are stable.
        assert_eq!(d.node_index_of(&NodeId::new("a")), Some(a_idx));
        assert_eq!(d.node_index_of(&NodeId::new("b")), Some(b_idx));
    }

    #[test]
    fn strictness_is_cyclic_no_self_loop() {
        let mut asset = LogicGraphAsset::default();
        let d = LogicGraphDialectMut::new(&mut asset);
        assert_eq!(d.strictness(), GraphMutStrictness::CyclicNoSelfLoop);
    }

    #[test]
    fn cycle_through_kernel_topological_sort_after_mutation() {
        // After adding a cycle via add_edge, has_cycle should return true.
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
        ];
        asset.edges = vec![sample_edge("a", "b")];
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let a_idx = d.node_index_of(&NodeId::new("a")).unwrap();
        let b_idx = d.node_index_of(&NodeId::new("b")).unwrap();
        // Add b->a to create a cycle.
        d.add_edge(b_idx, a_idx, sample_edge("b", "a")).unwrap();
        assert!(has_cycle(&d));
        let sorted = topological_sort(&d);
        assert!(sorted.is_err());
    }

    #[test]
    fn existing_read_only_dialect_unchanged() {
        // The read-only LogicGraphDialect should still work after we added LogicGraphDialectMut.
        let mut asset = LogicGraphAsset::default();
        asset.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
        ];
        asset.edges = vec![sample_edge("a", "b")];
        // Read-only view.
        let d = LogicGraphDialect::new(&asset);
        assert_eq!(d.node_count(), 2);
        assert_eq!(d.edge_count(), 1);
        let sorted = topological_sort(&d).unwrap();
        assert_eq!(sorted.len(), 2);
    }
}
