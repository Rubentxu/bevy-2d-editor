//! Graph Kernel — Pure-Rust Dialect-Agnostic Substrate.
//!
//! Provides a dialect-agnostic substrate for graph-shaped data in the editor
//! model. Dialects adapt domain-specific data (e.g. `LogicGraphAsset`,
//! `SceneAssetDocument`) into a uniform `Graph` view; the kernel runs pure
//! graph algorithms over any dialect.
//!
//! All kernel operations are pure, allocation-free on the dialect side, and
//! return owned `Vec<NodeIndex>` so lifetimes do not leak across the dialect
//! boundary.
//!
//! See ADR-0053 for the canonical design.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// Dialects — one file per dialect. Each dialect adapts a domain-specific
// graph shape to the `Graph` trait.
pub mod changeset_dialect;
pub mod scene_asset_dialect;
pub mod world_dialect;
pub use changeset_dialect::ChangeSetDialect;
pub use scene_asset_dialect::SceneAssetDialect;
pub use world_dialect::WorldGraphDialect;

/// Opaque kernel-owned node index. Stable for the lifetime of a `Graph` view.
///
/// Dialects translate their own ID types (e.g. `NodeId`, `SceneAssetLocalId`)
/// into `NodeIndex` at binding time. The kernel treats the index as opaque: it
/// does not assume any relationship between index and dialect-specific ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeIndex(pub u32);

/// Opaque kernel-owned edge index. Stable for the lifetime of a `Graph` view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EdgeIndex(pub u32);

/// Errors reported by the graph kernel.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphKernelError {
    /// The graph has at least one cycle; the participating nodes are listed.
    #[error("graph contains a cycle involving {participating:?} nodes")]
    Cycle {
        /// Nodes that lie on a cycle (no node has its in-degree reduced to 0).
        participating: Vec<NodeIndex>,
    },
    /// A `NodeIndex` was out of range for the underlying graph.
    #[error("node index {idx:?} out of range (graph has {total} nodes)")]
    NodeIndexOutOfRange {
        /// The offending index.
        idx: NodeIndex,
        /// Number of nodes in the graph.
        total: usize,
    },
    /// An `EdgeIndex` was out of range for the underlying graph.
    #[error("edge index {idx:?} out of range (graph has {total} edges)")]
    EdgeIndexOutOfRange {
        /// The offending index.
        idx: EdgeIndex,
        /// Number of edges in the graph.
        total: usize,
    },
}

/// The dialect contract: any graph-shaped data structure implements `Graph`.
///
/// The kernel reads the graph through this trait. Dialects are responsible
/// for the translation from their internal ID types to `NodeIndex` /
/// `EdgeIndex`. The kernel trusts the dialect: returning `None` from a method
/// means "not present" and stops walking that branch.
pub trait Graph {
    /// Per-node data exposed by the dialect.
    type NodeData: Clone;
    /// Per-edge data exposed by the dialect.
    type EdgeData: Clone;
    /// Dialect-specific error type. Most dialects use `Infallible` for v1.
    type Error: std::error::Error;

    /// Total node count.
    fn node_count(&self) -> usize;
    /// Total edge count.
    fn edge_count(&self) -> usize;
    /// Look up a node by index.
    fn node(&self, idx: NodeIndex) -> Option<&Self::NodeData>;
    /// Look up an edge by index.
    fn edge(&self, idx: EdgeIndex) -> Option<&Self::EdgeData>;
    /// Return the `[source, target]` endpoints of an edge.
    fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)>;
    /// Iterate edges whose source is `node`.
    fn outgoing(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_>;
    /// Iterate edges whose target is `node`.
    fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_>;
}

// ============================================================================
// Test helpers — only compiled under `cfg(test)`.
// ============================================================================

#[cfg(test)]
mod test_graphs {
    use super::{EdgeIndex, Graph, NodeIndex};
    use std::convert::Infallible;

    /// Minimal graph used in kernel tests. Nodes are unit; edges are `(src, dst)`.
    pub(super) struct SimpleGraph {
        pub(super) nodes: Vec<()>,
        pub(super) edges: Vec<(u32, u32)>,
    }

    impl Graph for SimpleGraph {
        type NodeData = ();
        type EdgeData = (u32, u32);
        type Error = Infallible;

        fn node_count(&self) -> usize {
            self.nodes.len()
        }
        fn edge_count(&self) -> usize {
            self.edges.len()
        }
        fn node(&self, idx: NodeIndex) -> Option<&()> {
            self.nodes.get(idx.0 as usize)
        }
        fn edge(&self, idx: EdgeIndex) -> Option<&(u32, u32)> {
            self.edges.get(idx.0 as usize)
        }
        fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
            let (a, b) = self.edges.get(idx.0 as usize)?;
            Some((NodeIndex(*a), NodeIndex(*b)))
        }
        fn outgoing(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
            Box::new(
                self.edges
                    .iter()
                    .enumerate()
                    .filter_map(move |(i, (a, _))| {
                        if *a == node.0 {
                            Some(EdgeIndex(i as u32))
                        } else {
                            None
                        }
                    }),
            )
        }
        fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
            Box::new(
                self.edges
                    .iter()
                    .enumerate()
                    .filter_map(move |(i, (_, b))| {
                        if *b == node.0 {
                            Some(EdgeIndex(i as u32))
                        } else {
                            None
                        }
                    }),
            )
        }
    }

    pub(super) fn empty() -> SimpleGraph {
        SimpleGraph {
            nodes: vec![],
            edges: vec![],
        }
    }

    pub(super) fn linear_chain(n: u32) -> SimpleGraph {
        let nodes: Vec<()> = (0..n).map(|_| ()).collect();
        let edges: Vec<(u32, u32)> = (0..n.saturating_sub(1)).map(|i| (i, i + 1)).collect();
        SimpleGraph { nodes, edges }
    }

    pub(super) fn diamond() -> SimpleGraph {
        SimpleGraph {
            nodes: vec![(), (), (), ()],
            edges: vec![(0, 1), (0, 2), (1, 3), (2, 3)],
        }
    }

    pub(super) fn triangle() -> SimpleGraph {
        SimpleGraph {
            nodes: vec![(), (), ()],
            edges: vec![(0, 1), (1, 2), (2, 0)],
        }
    }

    pub(super) fn two_roots() -> SimpleGraph {
        SimpleGraph {
            nodes: vec![(), (), ()],
            edges: vec![(0, 2), (1, 2)],
        }
    }
}

// ============================================================================
// Traversal operations.
// ============================================================================

/// Return nodes with no incoming edges. Order is iteration order of
/// `0..node_count`, which is deterministic.
///
/// Cost: `O(V * avg_in_degree)`.
pub fn roots<G: Graph + ?Sized>(g: &G) -> Vec<NodeIndex> {
    (0..g.node_count())
        .map(|i| NodeIndex(i as u32))
        .filter(|&idx| g.incoming(idx).count() == 0)
        .collect()
}

/// Return nodes with no outgoing edges. Order is iteration order of
/// `0..node_count`, which is deterministic.
pub fn leaves<G: Graph + ?Sized>(g: &G) -> Vec<NodeIndex> {
    (0..g.node_count())
        .map(|i| NodeIndex(i as u32))
        .filter(|&idx| g.outgoing(idx).count() == 0)
        .collect()
}

/// Return `root` plus every node reachable from `root` via outgoing edges.
/// Order is BFS; ties broken by insertion order (deterministic).
///
/// The result includes `root` itself; callers wanting strict descendants
/// should drop the head.
pub fn descendants<G: Graph + ?Sized>(g: &G, root: NodeIndex) -> Vec<NodeIndex> {
    let mut out = Vec::new();
    let mut visited: BTreeSet<NodeIndex> = BTreeSet::new();
    let mut queue: std::collections::VecDeque<NodeIndex> = std::collections::VecDeque::new();
    queue.push_back(root);
    while let Some(node) = queue.pop_front() {
        if !visited.insert(node) {
            continue;
        }
        out.push(node);
        for edge in g.outgoing(node) {
            if let Some((_, target)) = g.edge_endpoints(edge) {
                if !visited.contains(&target) {
                    queue.push_back(target);
                }
            }
        }
    }
    out
}

/// Return `leaf` plus every node that can reach `leaf` via incoming edges.
/// Order is BFS; ties broken by insertion order (deterministic).
pub fn ancestors<G: Graph + ?Sized>(g: &G, leaf: NodeIndex) -> Vec<NodeIndex> {
    let mut out = Vec::new();
    let mut visited: BTreeSet<NodeIndex> = BTreeSet::new();
    let mut queue: std::collections::VecDeque<NodeIndex> = std::collections::VecDeque::new();
    queue.push_back(leaf);
    while let Some(node) = queue.pop_front() {
        if !visited.insert(node) {
            continue;
        }
        out.push(node);
        for edge in g.incoming(node) {
            if let Some((source, _)) = g.edge_endpoints(edge) {
                if !visited.contains(&source) {
                    queue.push_back(source);
                }
            }
        }
    }
    out
}

/// `reachable_from(g, root)` is an alias for `descendants(g, root)`. Kept as
/// a named function for documentation parity with the spec.
pub fn reachable_from<G: Graph + ?Sized>(g: &G, root: NodeIndex) -> Vec<NodeIndex> {
    descendants(g, root)
}

/// Kahn's topological sort. Returns nodes in an order where every edge's
/// source appears before its target. Returns `Err` if the graph has a cycle.
///
/// Order is deterministic for a given dialect (queue polls in insertion order).
pub fn topological_sort<G: Graph + ?Sized>(g: &G) -> Result<Vec<NodeIndex>, GraphKernelError> {
    let mut in_degree: Vec<usize> = (0..g.node_count())
        .map(|i| g.incoming(NodeIndex(i as u32)).count())
        .collect();
    let mut queue: std::collections::VecDeque<NodeIndex> = (0..g.node_count())
        .map(|i| NodeIndex(i as u32))
        .filter(|&i| in_degree[i.0 as usize] == 0)
        .collect();
    let mut out = Vec::with_capacity(g.node_count());
    while let Some(node) = queue.pop_front() {
        out.push(node);
        for edge in g.outgoing(node) {
            if let Some((_, target)) = g.edge_endpoints(edge) {
                let idx = target.0 as usize;
                in_degree[idx] -= 1;
                if in_degree[idx] == 0 {
                    queue.push_back(target);
                }
            }
        }
    }
    if out.len() != g.node_count() {
        let participating = (0..g.node_count())
            .map(|i| NodeIndex(i as u32))
            .filter(|&i| in_degree[i.0 as usize] > 0)
            .collect();
        Err(GraphKernelError::Cycle { participating })
    } else {
        Ok(out)
    }
}

/// Returns `true` iff the graph contains at least one cycle. Short-circuits
/// via Kahn's algorithm.
pub fn has_cycle<G: Graph + ?Sized>(g: &G) -> bool {
    topological_sort(g).is_err()
}

// ============================================================================
// First dialect: LogicGraphDialect.
// ============================================================================

/// Adapter that lets `LogicGraphAsset` be read as a `Graph`.
///
/// Dialects are cheap to construct: they pre-compute index maps at binding
/// time. Dialects borrow the underlying asset; they are not owned.
pub struct LogicGraphDialect<'a> {
    asset: &'a crate::logic_graph::LogicGraphAsset,
    node_index: BTreeMap<&'a crate::logic_graph::NodeId, NodeIndex>,
}

impl<'a> LogicGraphDialect<'a> {
    /// Build a dialect view over `asset`. The dialect borrows `asset` for
    /// its lifetime.
    pub fn new(asset: &'a crate::logic_graph::LogicGraphAsset) -> Self {
        let node_index = asset
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (&n.node_id, NodeIndex(i as u32)))
            .collect();
        Self { asset, node_index }
    }

    /// Resolve a `NodeId` to its `NodeIndex` inside this dialect view.
    pub fn node_index_of(&self, id: &crate::logic_graph::NodeId) -> Option<NodeIndex> {
        self.node_index.get(id).copied()
    }
}

impl<'a> Graph for LogicGraphDialect<'a> {
    type NodeData = crate::logic_graph::LogicNode;
    type EdgeData = crate::logic_graph::LogicEdge;
    type Error = std::convert::Infallible;

    fn node_count(&self) -> usize {
        self.asset.nodes.len()
    }
    fn edge_count(&self) -> usize {
        self.asset.edges.len()
    }
    fn node(&self, idx: NodeIndex) -> Option<&crate::logic_graph::LogicNode> {
        self.asset.nodes.get(idx.0 as usize)
    }
    fn edge(&self, idx: EdgeIndex) -> Option<&crate::logic_graph::LogicEdge> {
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
        Box::new(
            self.asset
                .edges
                .iter()
                .enumerate()
                .filter_map(move |(i, e)| {
                    if Some(&e.from_node) == source_id.as_ref() {
                        Some(EdgeIndex(i as u32))
                    } else {
                        None
                    }
                }),
        )
    }
    fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let target_id = self.node(node).map(|n| n.node_id.clone());
        Box::new(
            self.asset
                .edges
                .iter()
                .enumerate()
                .filter_map(move |(i, e)| {
                    if Some(&e.to_node) == target_id.as_ref() {
                        Some(EdgeIndex(i as u32))
                    } else {
                        None
                    }
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic_graph::{
        LogicEdge, LogicGraphAsset, LogicNode, LogicNodeRole, NodeId, NodeTypeId, PortId,
    };

    // ----------------------------------------------------------------------
    // Tests for the test helper graph (kernel algorithms).
    // ----------------------------------------------------------------------

    #[test]
    fn empty_graph_returns_no_roots() {
        let g = test_graphs::empty();
        assert!(roots(&g).is_empty());
        assert!(leaves(&g).is_empty());
        assert!(!has_cycle(&g));
    }

    #[test]
    fn single_node_is_both_root_and_leaf() {
        let g = test_graphs::linear_chain(1);
        let r = roots(&g);
        let l = leaves(&g);
        assert_eq!(r, vec![NodeIndex(0)]);
        assert_eq!(l, vec![NodeIndex(0)]);
    }

    #[test]
    fn linear_chain_returns_root_and_leaf() {
        let g = test_graphs::linear_chain(4);
        let r = roots(&g);
        assert_eq!(r, vec![NodeIndex(0)]);
        let l = leaves(&g);
        assert_eq!(l, vec![NodeIndex(3)]);
    }

    #[test]
    fn two_roots_returns_both() {
        let g = test_graphs::two_roots();
        let r = roots(&g);
        assert_eq!(r, vec![NodeIndex(0), NodeIndex(1)]);
    }

    #[test]
    fn diamond_no_cycle() {
        let g = test_graphs::diamond();
        assert!(!has_cycle(&g));
        let sorted = topological_sort(&g).unwrap();
        assert_eq!(sorted.len(), 4);
        assert_eq!(sorted[0], NodeIndex(0));
    }

    #[test]
    fn triangle_creates_cycle() {
        let g = test_graphs::triangle();
        assert!(has_cycle(&g));
        assert!(matches!(
            topological_sort(&g),
            Err(GraphKernelError::Cycle { .. })
        ));
    }

    #[test]
    fn descendants_bfs_includes_root() {
        let g = test_graphs::diamond();
        let desc = descendants(&g, NodeIndex(0));
        assert_eq!(
            desc,
            vec![NodeIndex(0), NodeIndex(1), NodeIndex(2), NodeIndex(3)]
        );
    }

    #[test]
    fn ancestors_bfs_includes_leaf() {
        let g = test_graphs::diamond();
        let anc = ancestors(&g, NodeIndex(3));
        // Diamond: 0->1, 0->2, 1->3, 2->3. BFS from 3: 3, then 1 and 2, then 0.
        assert_eq!(
            anc,
            vec![NodeIndex(3), NodeIndex(1), NodeIndex(2), NodeIndex(0)]
        );
    }

    #[test]
    fn reachable_from_is_descendants() {
        let g = test_graphs::diamond();
        assert_eq!(
            reachable_from(&g, NodeIndex(0)),
            descendants(&g, NodeIndex(0))
        );
    }

    // ----------------------------------------------------------------------
    // Tests for the LogicGraphDialect.
    // ----------------------------------------------------------------------

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
        let desc = descendants(&d, d.node_index_of(&NodeId::new("a")).unwrap());
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

    #[test]
    fn dialect_kernel_roots_and_leaves() {
        let mut g = LogicGraphAsset::default();
        g.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
            sample_node("c", LogicNodeRole::Actuator),
        ];
        g.edges = vec![sample_edge("a", "b"), sample_edge("b", "c")];
        let d = LogicGraphDialect::new(&g);
        let r = roots(&d);
        assert_eq!(r, vec![d.node_index_of(&NodeId::new("a")).unwrap()]);
        let l = leaves(&d);
        assert_eq!(l, vec![d.node_index_of(&NodeId::new("c")).unwrap()]);
    }
}
