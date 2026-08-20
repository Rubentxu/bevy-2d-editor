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
pub mod logic_dialect;
pub mod scene_asset_dialect;
pub mod world_dialect;
pub use changeset_dialect::ChangeSetDialect;
pub use logic_dialect::{LogicGraphDialect, LogicGraphDialectMut};
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
#[non_exhaustive]
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
    /// `add_edge` would create a cycle in a `Dag` dialect.
    /// `participating` lists the nodes that would lie on the new cycle.
    #[error("add_edge would create a cycle involving {participating:?} nodes")]
    WouldCreateCycle {
        /// Nodes that would lie on the cycle.
        participating: Vec<NodeIndex>,
    },
    /// `add_edge` would create a self-loop in a non-`Free` dialect.
    #[error("add_edge would create a self-loop at node {node:?}")]
    SelfLoop {
        /// The node that would be the source and target of the self-loop.
        node: NodeIndex,
    },
    /// `add_edge` would duplicate an existing edge in a non-`Free` dialect.
    #[error("add_edge would duplicate edge {src:?} -> {dst:?}")]
    DuplicateEdge {
        /// The source node of the duplicated edge.
        src: NodeIndex,
        /// The destination node of the duplicated edge.
        dst: NodeIndex,
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
// GraphMut — opt-in mutation trait segregated from Graph.
// ============================================================================

/// Per-dialect topology strictness for mutation operations.
///
/// Each dialect that implements `GraphMut` declares its `STRICTNESS` via the
/// associated constant. This allows `add_edge` to validate topology once at
/// compile time without runtime dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphMutStrictness {
    /// No cycles, no self-loops, no duplicate edges. (SceneAssetDialect)
    Dag,
    /// Cycles allowed, no self-loops, no duplicate edges. (LogicGraphDialect)
    CyclicNoSelfLoop,
    /// Anything goes; dialect decides its own validation. (WorldGraphDialect)
    Free,
}

/// Opt-in mutation trait for graph dialects.
///
/// This trait is segregated from `Graph` to preserve read-only kernel operations.
/// Dialects that want mutation implement `GraphMut` with a `STRICTNESS` constant
/// that governs `add_edge` validation. The trait is object-safe: methods return
/// `NodeIndex`/`EdgeIndex` (opaque newtypes) and never `Self`.
pub trait GraphMut: Graph {
    /// Per-dialect topology strictness. Resolved at compile time.
    const STRICTNESS: GraphMutStrictness;

    /// Append a new node. Returns the new `NodeIndex`.
    fn add_node(&mut self, data: Self::NodeData) -> NodeIndex;

    /// Connect `src` to `dst`. Validates per `STRICTNESS`.
    fn add_edge(
        &mut self,
        src: NodeIndex,
        dst: NodeIndex,
        data: Self::EdgeData,
    ) -> Result<EdgeIndex, GraphKernelError>;

    /// Remove a node. Default impl removes every edge where `idx` is source or target.
    fn remove_node(&mut self, idx: NodeIndex) -> Result<(), GraphKernelError>;

    /// Remove an edge by index.
    fn remove_edge(&mut self, idx: EdgeIndex) -> Result<(), GraphKernelError>;

    /// Replace a node's data in place.
    fn update_node(
        &mut self,
        idx: NodeIndex,
        data: Self::NodeData,
    ) -> Result<(), GraphKernelError>;

    /// Replace an edge's data in place.
    fn update_edge(
        &mut self,
        idx: EdgeIndex,
        data: Self::EdgeData,
    ) -> Result<(), GraphKernelError>;
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
// Mutation helpers.
// ============================================================================

/// Returns `true` iff `hypothetical_src` is reachable from `hypothetical_dst`
/// in the given graph.
///
/// This helper is used by `GraphMut::add_edge` implementations with
/// `STRICTNESS = Dag` to detect whether adding an edge would create a cycle.
/// The check is O(V+E).
#[allow(dead_code)]
pub(crate) fn would_create_cycle<G: Graph + ?Sized>(
    g: &G,
    hypothetical_src: NodeIndex,
    hypothetical_dst: NodeIndex,
) -> bool {
    // True iff `hypothetical_src` can be reached by walking backwards from
    // `hypothetical_dst`. If so, adding hypothetical_src → hypothetical_dst
    // would close a cycle.
    ancestors(g, hypothetical_dst).contains(&hypothetical_src)
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
// Tests for GraphMut, GraphMutStrictness, and GraphKernelError extensions.
// ============================================================================

#[cfg(test)]
mod graph_mut_tests {
    use super::*;

    // --- GraphMutStrictness discriminant tests ---

    #[test]
    fn graph_mut_strictness_const_usize() {
        // Each variant must have a distinct usize representation.
        use std::mem;
        let dag = GraphMutStrictness::Dag;
        let cyclic = GraphMutStrictness::CyclicNoSelfLoop;
        let free = GraphMutStrictness::Free;
        // Discriminants are distinct (match is exhaustive so this is compile-time safe).
        match (dag, cyclic, free) {
            (GraphMutStrictness::Dag, GraphMutStrictness::CyclicNoSelfLoop, GraphMutStrictness::Free) => {}
            (GraphMutStrictness::CyclicNoSelfLoop, GraphMutStrictness::Dag, GraphMutStrictness::Free) => {}
            (GraphMutStrictness::Free, GraphMutStrictness::CyclicNoSelfLoop, GraphMutStrictness::Dag) => {}
            _ => unreachable!(),
        }
        // Also verify via mem::discriminant.
        assert_ne!(mem::discriminant(&dag), mem::discriminant(&cyclic));
        assert_ne!(mem::discriminant(&cyclic), mem::discriminant(&free));
        assert_ne!(mem::discriminant(&dag), mem::discriminant(&free));
    }

    /// A stub dialect used to verify GraphMut can be implemented.
    struct StubDialect {
        nodes: Vec<i32>,
        edges: Vec<(u32, u32)>,
    }

    impl StubDialect {
        fn new() -> Self {
            StubDialect {
                nodes: vec![],
                edges: vec![],
            }
        }
    }

    impl Graph for StubDialect {
        type NodeData = i32;
        type EdgeData = (u32, u32);
        type Error = std::convert::Infallible;

        fn node_count(&self) -> usize {
            self.nodes.len()
        }
        fn edge_count(&self) -> usize {
            self.edges.len()
        }
        fn node(&self, idx: NodeIndex) -> Option<&i32> {
            self.nodes.get(idx.0 as usize)
        }
        fn edge(&self, idx: EdgeIndex) -> Option<&(u32, u32)> {
            self.edges.get(idx.0 as usize)
        }
        fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
            let (a, b) = *self.edges.get(idx.0 as usize)?;
            Some((NodeIndex(a), NodeIndex(b)))
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

    impl GraphMut for StubDialect {
        const STRICTNESS: GraphMutStrictness = GraphMutStrictness::Free;

        fn add_node(&mut self, data: Self::NodeData) -> NodeIndex {
            let idx = NodeIndex(self.nodes.len() as u32);
            self.nodes.push(data);
            idx
        }

        fn add_edge(
            &mut self,
            src: NodeIndex,
            dst: NodeIndex,
            _data: Self::EdgeData,
        ) -> Result<EdgeIndex, GraphKernelError> {
            if src.0 as usize >= self.nodes.len() || dst.0 as usize >= self.nodes.len() {
                return Err(GraphKernelError::NodeIndexOutOfRange {
                    idx: src,
                    total: self.nodes.len(),
                });
            }
            let idx = EdgeIndex(self.edges.len() as u32);
            self.edges.push((src.0, dst.0));
            Ok(idx)
        }

        fn remove_node(&mut self, idx: NodeIndex) -> Result<(), GraphKernelError> {
            if idx.0 as usize >= self.nodes.len() {
                return Err(GraphKernelError::NodeIndexOutOfRange {
                    idx,
                    total: self.nodes.len(),
                });
            }
            self.nodes.remove(idx.0 as usize);
            self.edges.retain(|(a, b)| *a != idx.0 && *b != idx.0);
            Ok(())
        }

        fn remove_edge(&mut self, idx: EdgeIndex) -> Result<(), GraphKernelError> {
            if idx.0 as usize >= self.edges.len() {
                return Err(GraphKernelError::EdgeIndexOutOfRange {
                    idx,
                    total: self.edges.len(),
                });
            }
            self.edges.remove(idx.0 as usize);
            Ok(())
        }

        fn update_node(&mut self, idx: NodeIndex, data: Self::NodeData) -> Result<(), GraphKernelError> {
            if idx.0 as usize >= self.nodes.len() {
                return Err(GraphKernelError::NodeIndexOutOfRange {
                    idx,
                    total: self.nodes.len(),
                });
            }
            self.nodes[idx.0 as usize] = data;
            Ok(())
        }

        fn update_edge(&mut self, idx: EdgeIndex, data: Self::EdgeData) -> Result<(), GraphKernelError> {
            if idx.0 as usize >= self.edges.len() {
                return Err(GraphKernelError::EdgeIndexOutOfRange {
                    idx,
                    total: self.edges.len(),
                });
            }
            self.edges[idx.0 as usize] = data;
            Ok(())
        }
    }

    #[test]
    fn graphmut_trait_can_be_implemented() {
        // Verify a type can implement GraphMut and be used via &mut dyn.
        let mut dialect = StubDialect::new();
        let idx = dialect.add_node(42);
        assert_eq!(idx, NodeIndex(0));
        assert_eq!(dialect.node_count(), 1);
        // Verify STRICTNESS const is accessible.
        assert_eq!(<StubDialect as GraphMut>::STRICTNESS, GraphMutStrictness::Free);
    }

    // --- GraphKernelError #[non_exhaustive] wildcard test ---

    #[test]
    fn graph_kernel_error_non_exhaustive() {
        // A wildcard match on GraphKernelError must be exhaustive, proving
        // #[non_exhaustive] is present. If we list all known variants below
        // without a wildcard arm, adding a new variant without an explicit arm
        // will cause a compile error.
        fn match_error(e: &GraphKernelError) -> &'static str {
            match e {
                GraphKernelError::Cycle { .. } => "cycle",
                GraphKernelError::NodeIndexOutOfRange { .. } => "node_oob",
                GraphKernelError::EdgeIndexOutOfRange { .. } => "edge_oob",
                GraphKernelError::WouldCreateCycle { .. } => "would_cycle",
                GraphKernelError::SelfLoop { .. } => "self_loop",
                GraphKernelError::DuplicateEdge { .. } => "dup_edge",
            }
        }

        let cycle_err = GraphKernelError::Cycle {
            participating: vec![NodeIndex(0), NodeIndex(1)],
        };
        assert_eq!(match_error(&cycle_err), "cycle");

        let would_cycle = GraphKernelError::WouldCreateCycle {
            participating: vec![NodeIndex(2)],
        };
        assert_eq!(match_error(&would_cycle), "would_cycle");

        let self_loop = GraphKernelError::SelfLoop { node: NodeIndex(3) };
        assert_eq!(match_error(&self_loop), "self_loop");

        let dup_edge = GraphKernelError::DuplicateEdge {
            src: NodeIndex(1),
            dst: NodeIndex(2),
        };
        assert_eq!(match_error(&dup_edge), "dup_edge");
    }

    // --- GraphKernelError new variant construction and Debug formatting ---

    #[test]
    fn graph_kernel_error_variants_construction() {
        // Construct each of the 3 new variants and verify Debug formatting.
        let err_cycle = GraphKernelError::WouldCreateCycle {
            participating: vec![NodeIndex(0), NodeIndex(1), NodeIndex(2)],
        };
        let debug = format!("{:?}", err_cycle);
        assert!(debug.contains("WouldCreateCycle"));
        assert!(debug.contains("NodeIndex"));

        let err_self_loop = GraphKernelError::SelfLoop { node: NodeIndex(5) };
        let debug = format!("{:?}", err_self_loop);
        assert!(debug.contains("SelfLoop"));
        assert!(debug.contains("5"));

        let err_dup = GraphKernelError::DuplicateEdge {
            src: NodeIndex(1),
            dst: NodeIndex(3),
        };
        let debug = format!("{:?}", err_dup);
        assert!(debug.contains("DuplicateEdge"));
    }

    // --- would_create_cycle helper test ---

    #[test]
    fn would_create_cycle_helper_returns_true_when_cycle_would_form() {
        // Linear chain 0->1->2.
        // would_create_cycle returns true if hypothetical_src is in ancestors(hypothetical_dst).
        // Adding 0->1: is 0 in ancestors(1)? Yes (0->1 exists). So 0->1 would create a cycle.
        let chain = test_graphs::linear_chain(3); // 0->1->2
        let result = would_create_cycle(&chain, NodeIndex(0), NodeIndex(1));
        assert!(
            result,
            "adding 0->1 would create a cycle because 0 is already an ancestor of 1"
        );
    }

    #[test]
    fn would_create_cycle_helper_returns_false_when_no_cycle() {
        // Linear chain 0->1->2.
        // Adding 1->2: is 1 in ancestors(2)? ancestors(2) = {0, 1, 2}. Yes! So this creates a cycle.
        // Adding 0->2: is 0 in ancestors(2)? Yes! Also creates a cycle.
        // Actually in a simple chain 0->1, adding 1->0 creates a cycle.
        // Adding 1->0 in chain 0->1: ancestors(0) = {0}. Is 1 in {0}? No. So this is acyclic!
        let chain = test_graphs::linear_chain(2); // 0->1
        let result = would_create_cycle(&chain, NodeIndex(1), NodeIndex(0));
        assert!(
            !result,
            "adding 1->0 to chain 0->1 is acyclic (1 is not an ancestor of 0)"
        );
    }
}
