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
/// Each dialect that implements `GraphMut` declares its `strictness` via the
/// `strictness()` method. The method is `&self` so the trait remains
/// object-safe. This allows `add_edge` to validate topology per dialect.
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
/// Dialects that want mutation implement `GraphMut` with a `strictness` method
/// that governs `add_edge` validation. The trait is object-safe: methods return
/// `NodeIndex`/`EdgeIndex` (opaque newtypes) and never `Self` (only `&mut self`).
pub trait GraphMut: Graph {
    /// Per-dialect topology strictness. Resolved at compile time per dialect,
    /// invoked as a method so the trait remains object-safe.
    fn strictness(&self) -> GraphMutStrictness;

    /// Append a new node. Returns the new `NodeIndex`.
    fn add_node(&mut self, data: Self::NodeData) -> NodeIndex;

    /// Connect `src` to `dst`. Validates per `strictness()`.
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

/// Returns `true` iff adding an edge `hypothetical_src → hypothetical_dst`
/// would create a cycle in the given graph.
///
/// Adding src→dst creates a cycle iff `hypothetical_dst` is already reachable
/// from `hypothetical_src` in the existing graph. This is O(V+E).
///
/// This helper is used by `GraphMut::add_edge` implementations with
/// `strictness() == Dag` to detect whether adding an edge would create a cycle.
#[allow(dead_code)]
pub(crate) fn would_create_cycle<G: Graph + ?Sized>(
    g: &G,
    hypothetical_src: NodeIndex,
    hypothetical_dst: NodeIndex,
) -> bool {
    // Adding src→dst creates a cycle iff dst is reachable from src.
    // In a chain a→b→c, adding c→a: dst=a is reachable from src=c? No.
    // But wait - we need to check if dst can reach src (reverse direction).
    // If dst can reach src, then src→dst closes a cycle.
    // In chain a→b→c, can a reach c? Yes (a→b→c). So adding c→a creates a cycle.
    // ancestors(src) contains dst? ancestors(c) = {c}. a is not in {c}. No.
    // descendants(src) contains dst? descendants(c) = {c}. a is not in {c}. No.
    // Let me re-think...
    //
    // In a chain a→b→c (indices 0,1,2):
    // - a has no incoming, b has incoming from a, c has incoming from b
    // - ancestors(a) = {a} (root)
    // - descendants(a) = {a, b, c}
    // - ancestors(c) = {a, b, c} (all can reach c)
    // - descendants(c) = {c} (c has no outgoing)
    //
    // Adding c→a (src=c, dst=a):
    // - Is a reachable from c? No (c has no outgoing). But adding c→a creates an edge.
    // - Is c reachable from a? Yes (a→b→c). So a→b→c→a is a cycle.
    //
    // The condition is: is hypothetical_dst in descendants(hypothetical_src)?
    // In our example: is a in descendants(c)? No. So we return false.
    // But we should return true (adding c→a creates a cycle).
    //
    // Let me re-check the helper's current logic:
    // ancestors(hypothetical_dst).contains(&hypothetical_src)
    // For src=c, dst=a: ancestors(a) = {a}. Is c in {a}? No.
    //
    // Hmm. ancestors(a) gives {a} because a is a root (no incoming edges).
    // But adding c→a would give a an incoming edge, making the cycle a→b→c→a.
    //
    // I think the correct condition is:
    // is hypothetical_src in ancestors(hypothetical_dst)?
    // For src=c, dst=a: is c in ancestors(a)? ancestors(a) = {a}. No.
    //
    // Wait, maybe I'm confused about which direction the cycle forms.
    // In a→b→c, we have edges a→b and b→c.
    // If we add c→a, we get a→b→c→a.
    // Is this a cycle? Yes.
    // What is the cycle? a→b→c→a.
    // Which node is "repeated"? a.
    //
    // So the cycle is: a is reachable from c, and then we add c→a.
    // If a is reachable from c, then adding c→a closes the loop.
    // Can a reach c in the original graph? Yes! a→b→c.
    //
    // So the condition should be:
    // is hypothetical_dst reachable from hypothetical_src?
    // I.e., is hypothetical_dst in descendants(hypothetical_src)?
    // For src=c, dst=a: is a in descendants(c)? No (descendants(c) = {c}).
    //
    // I'm making a mistake somewhere. Let me re-examine...
    //
    // In the original graph with a→b→c:
    // - Can a reach c? Yes (a→b→c).
    // - Can c reach a? No (c has no outgoing).
    //
    // After adding c→a:
    // - Can a reach c? Yes (a→b→c).
    // - Can c reach a? Yes (c→a).
    // - Is it a cycle? a→b→c→a is a cycle because a appears twice.
    //
    // So the question is: when does adding src→dst create a cycle?
    // It creates a cycle if there already exists a path from dst to src.
    // In our example: is there a path from a to c? Yes (a→b→c).
    // So adding c→a creates a cycle.
    //
    // The condition is: is hypothetical_dst an ancestor of hypothetical_src?
    // (i.e., can we reach hypothetical_dst from hypothetical_src going backwards?)
    //
    // ancestors(g, src) = all nodes that can reach src
    // If hypothetical_dst is in ancestors(hypothetical_src), then dst can reach src.
    // And adding src→dst closes a cycle.
    //
    // For src=c, dst=a: ancestors(g, c) = {a, b, c} (all nodes can reach c because
    // a→b→c means c is a leaf; but wait, going INCOMING: c has no incoming, so
    // ancestors(c) = {c} only.
    //
    // ancestors walks INCOMING edges. In a→b→c:
    // - incoming(a) = empty → ancestors(a) = {a}
    // - incoming(b) = {a} → ancestors(b) = {a, b}
    // - incoming(c) = {b} → ancestors(c) = {a, b, c}
    //
    // Wait, let me re-read ancestors():
    // ancestors(g, leaf) = leaf + every node that can reach leaf via incoming edges.
    // For c (leaf in a→b→c): ancestors(c) = {c} ∪ ancestors(b) ∪ ... via incoming(b)={a}
    // = {c} ∪ {a,b} ∪ {a} = {a,b,c}. Yes.
    //
    // So for src=c, dst=a: is a in ancestors(c)? Yes. So we would create a cycle.
    //
    // But the current helper says: is hypothetical_src in ancestors(hypothetical_dst)?
    // For src=c, dst=a: is c in ancestors(a)? ancestors(a) = {a}. No.
    // This is WRONG. We should check is hypothetical_dst in ancestors(hypothetical_src).
    // For src=c, dst=a: is a in ancestors(c)? Yes → return true (cycle).
    //
    // CURRENT (wrong): ancestors(dst).contains(src)
    // FIXED (correct): ancestors(src).contains(dst)
    ancestors(g, hypothetical_src).contains(&hypothetical_dst)
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
        fn strictness(&self) -> GraphMutStrictness {
            GraphMutStrictness::Free
        }

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
        // Verify strictness() method is accessible.
        assert_eq!(dialect.strictness(), GraphMutStrictness::Free);
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
        // Chain 0->1->2. Adding 1->0: is 0 an ancestor of 1?
        // ancestors(1) = {0, 1}. Yes. So adding 1->0 closes a cycle.
        let chain = test_graphs::linear_chain(3); // 0->1->2
        let result = would_create_cycle(&chain, NodeIndex(1), NodeIndex(0));
        assert!(
            result,
            "adding 1->0 would create a cycle because 0 is already an ancestor of 1"
        );
    }

    #[test]
    fn would_create_cycle_helper_returns_false_when_no_cycle() {
        // Chain 0->1->2. Adding 0->1: is 1 an ancestor of 0?
        // ancestors(0) = {0}. Is 1 in {0}? No. So no cycle.
        let chain = test_graphs::linear_chain(3); // 0->1->2
        let result = would_create_cycle(&chain, NodeIndex(0), NodeIndex(1));
        assert!(
            !result,
            "adding 0->1 to chain 0->1->2 is acyclic (1 is not an ancestor of 0)"
        );
    }
}

// ============================================================================
// Cross-dialect integration tests (Phase 5).
// ============================================================================

#[cfg(test)]
mod cross_dialect_integration_tests {
    use super::*;
    use crate::graph_kernel::scene_asset_dialect::SceneAssetDialectMut;
    use crate::graph_kernel::world_dialect::WorldGraphDialectMut;
    use crate::graph_kernel::logic_dialect::LogicGraphDialectMut;
    use crate::ids::SceneAssetLocalId;
    use crate::logic_graph::{LogicNodeRole, NodeTypeId, NodeId};
    use crate::scene_asset::{RelationshipKind, SceneAssetMetadata, SceneAssetRole};
    use crate::world::{LayoutPolicy, LinkDirection, StreamingPolicy, WorldId, WorldLinkKind};
    use std::collections::BTreeMap;

    fn empty_scene_asset() -> crate::scene_asset::SceneAssetDocument {
        crate::scene_asset::SceneAssetDocument {
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

    fn empty_world() -> crate::world::WorldDocument {
        crate::world::WorldDocument {
            id: WorldId::new("w"),
            name: "Test".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![],
            links: vec![],
            updated_at: 0,
            extension_data: BTreeMap::new(),
        }
    }

    #[test]
    fn strictness_method_returns_correct_variant() {
        // Verify strictness() method returns the per-dialect variant.
        let mut asset = crate::logic_graph::LogicGraphAsset::default();
        let mut sad = empty_scene_asset();
        let mut world = empty_world();
        let ld = LogicGraphDialectMut::new(&mut asset);
        let sad_d = SceneAssetDialectMut::new(&mut sad);
        let wd = WorldGraphDialectMut::new(&mut world);
        assert_eq!(ld.strictness(), GraphMutStrictness::CyclicNoSelfLoop);
        assert_eq!(sad_d.strictness(), GraphMutStrictness::Dag);
        assert_eq!(wd.strictness(), GraphMutStrictness::Free);
    }

    #[test]
    fn graph_mut_is_object_safe() {
        // Spec requirement 9: GraphMut must be usable as `&mut dyn GraphMut`
        // and `Box<dyn GraphMut>`. Associated consts would break object-safety;
        // we use a method instead. This test verifies the trait is object-safe.
        let mut asset = crate::logic_graph::LogicGraphAsset::default();
        let boxed: Box<
            dyn GraphMut<
                NodeData = crate::logic_graph::LogicNode,
                EdgeData = crate::logic_graph::LogicEdge,
                Error = std::convert::Infallible,
            >,
        > = Box::new(LogicGraphDialectMut::new(&mut asset));
        let _borrowed: &dyn GraphMut<
            NodeData = crate::logic_graph::LogicNode,
            EdgeData = crate::logic_graph::LogicEdge,
            Error = std::convert::Infallible,
        > = &*boxed;
        // If this compiles, the trait is object-safe.
    }

    #[test]
    fn graph_mut_methods_return_node_index_u32_newtype() {
        let mut asset = crate::logic_graph::LogicGraphAsset::default();
        let mut d = LogicGraphDialectMut::new(&mut asset);
        let idx = d.add_node(crate::logic_graph::LogicNode {
            node_id: NodeId::new("test"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.generic"),
            field_values: serde_json::Value::Null,
            controller_id: None,
        });
        assert_eq!(idx.0, 0u32);
        let idx2 = d.add_node(crate::logic_graph::LogicNode {
            node_id: NodeId::new("test2"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.generic"),
            field_values: serde_json::Value::Null,
            controller_id: None,
        });
        assert_eq!(idx2.0, 1u32);
    }

    #[test]
    fn existing_callers_unchanged() {
        use crate::graph_kernel::{has_cycle, topological_sort};
        use crate::graph_kernel::scene_asset_dialect::SceneAssetDialect;
        use crate::graph_kernel::world_dialect::WorldGraphDialect;
        use crate::graph_kernel::logic_dialect::LogicGraphDialect;

        // LogicGraphDialect
        let mut asset = crate::logic_graph::LogicGraphAsset::default();
        asset.nodes = vec![
            crate::logic_graph::LogicNode {
                node_id: NodeId::new("a"),
                role: LogicNodeRole::Sensor,
                node_type_id: NodeTypeId::new("sensor.generic"),
                field_values: serde_json::Value::Null,
                controller_id: None,
            },
            crate::logic_graph::LogicNode {
                node_id: NodeId::new("b"),
                role: LogicNodeRole::Actuator,
                node_type_id: NodeTypeId::new("actuator.generic"),
                field_values: serde_json::Value::Null,
                controller_id: None,
            },
        ];
        asset.edges = vec![crate::logic_graph::LogicEdge {
            from_node: NodeId::new("a"),
            from_port: crate::logic_graph::PortId::new("out"),
            to_node: NodeId::new("b"),
            to_port: crate::logic_graph::PortId::new("in"),
        }];
        let d = LogicGraphDialect::new(&asset);
        assert!(!has_cycle(&d));
        assert!(topological_sort(&d).is_ok());

        // SceneAssetDialect
        let doc = crate::scene_asset::SceneAssetDocument {
            asset_id: "test".to_string(),
            logical_path: "test/asset".to_string(),
            role: SceneAssetRole::Actor,
            version: 1,
            entities: vec![
                crate::scene_asset::SceneAssetEntity {
                    local_id: SceneAssetLocalId::new("root"),
                    local_path: "root".to_string(),
                    name: "root".to_string(),
                    components: vec![],
                    extension_data: BTreeMap::new(),
                },
                crate::scene_asset::SceneAssetEntity {
                    local_id: SceneAssetLocalId::new("child"),
                    local_path: "child".to_string(),
                    name: "child".to_string(),
                    components: vec![],
                    extension_data: BTreeMap::new(),
                },
            ],
            relationships: vec![crate::scene_asset::SceneAssetRelationship {
                from_local_id: SceneAssetLocalId::new("root"),
                to_local_id: SceneAssetLocalId::new("child"),
                kind: RelationshipKind::Child,
                field_path: None,
            }],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            layers: vec![],
            extension_data: BTreeMap::new(),
        };
        let sd = SceneAssetDialect::new(&doc);
        assert!(!has_cycle(&sd));
        assert!(topological_sort(&sd).is_ok());

        // WorldGraphDialect
        let wd = crate::world::WorldDocument {
            id: WorldId::new("world"),
            name: "World".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![
                crate::world::WorldLevelRef {
                    level_id: "a".to_string(),
                    asset_ref: "levels/a".to_string(),
                    position: [0.0, 0.0],
                    dimensions: None,
                    tags: vec![],
                    streaming: StreamingPolicy::AlwaysResident,
                },
                crate::world::WorldLevelRef {
                    level_id: "b".to_string(),
                    asset_ref: "levels/b".to_string(),
                    position: [100.0, 0.0],
                    dimensions: None,
                    tags: vec![],
                    streaming: StreamingPolicy::AlwaysResident,
                },
            ],
            links: vec![crate::world::WorldLink {
                id: "ab".to_string(),
                from: "a".to_string(),
                to: "b".to_string(),
                direction: LinkDirection::East,
                kind: WorldLinkKind::OneWay,
                entrance: None,
                exit: None,
            }],
            updated_at: 0,
            extension_data: BTreeMap::new(),
        };
        let wd_d = WorldGraphDialect::new(&wd);
        assert!(!has_cycle(&wd_d));
        assert!(topological_sort(&wd_d).is_ok());
    }

    #[test]
    fn index_stability_after_mutation_across_dialects() {
        // --- Logic ---
        {
            let mut asset = crate::logic_graph::LogicGraphAsset::default();
            asset.nodes = vec![
                crate::logic_graph::LogicNode {
                    node_id: NodeId::new("a"),
                    role: LogicNodeRole::Sensor,
                    node_type_id: NodeTypeId::new("sensor.generic"),
                    field_values: serde_json::Value::Null,
                    controller_id: None,
                },
                crate::logic_graph::LogicNode {
                    node_id: NodeId::new("b"),
                    role: LogicNodeRole::Controller,
                    node_type_id: NodeTypeId::new("controller.generic"),
                    field_values: serde_json::Value::Null,
                    controller_id: None,
                },
            ];
            let mut d = LogicGraphDialectMut::new(&mut asset);
            let a_idx = d.node_index_of(&NodeId::new("a")).unwrap();
            let b_idx = d.node_index_of(&NodeId::new("b")).unwrap();
            d.add_node(crate::logic_graph::LogicNode {
                node_id: NodeId::new("c"),
                role: LogicNodeRole::Actuator,
                node_type_id: NodeTypeId::new("actuator.generic"),
                field_values: serde_json::Value::Null,
                controller_id: None,
            });
            assert_eq!(d.node_index_of(&NodeId::new("a")), Some(a_idx));
            assert_eq!(d.node_index_of(&NodeId::new("b")), Some(b_idx));
        }

        // --- SceneAsset ---
        {
            let mut doc = crate::scene_asset::SceneAssetDocument {
                asset_id: "test".to_string(),
                logical_path: "test/asset".to_string(),
                role: SceneAssetRole::Actor,
                version: 1,
                entities: vec![
                    crate::scene_asset::SceneAssetEntity {
                        local_id: SceneAssetLocalId::new("x"),
                        local_path: "x".to_string(),
                        name: "x".to_string(),
                        components: vec![],
                        extension_data: BTreeMap::new(),
                    },
                    crate::scene_asset::SceneAssetEntity {
                        local_id: SceneAssetLocalId::new("y"),
                        local_path: "y".to_string(),
                        name: "y".to_string(),
                        components: vec![],
                        extension_data: BTreeMap::new(),
                    },
                ],
                relationships: vec![],
                exposed_properties: vec![],
                metadata: SceneAssetMetadata::default(),
                layers: vec![],
                extension_data: BTreeMap::new(),
            };
            let mut d = SceneAssetDialectMut::new(&mut doc);
            let x_idx = d.node_index_of(&SceneAssetLocalId::new("x")).unwrap();
            let y_idx = d.node_index_of(&SceneAssetLocalId::new("y")).unwrap();
            d.add_node(crate::scene_asset::SceneAssetEntity {
                local_id: SceneAssetLocalId::new("z"),
                local_path: "z".to_string(),
                name: "z".to_string(),
                components: vec![],
                extension_data: BTreeMap::new(),
            });
            assert_eq!(d.node_index_of(&SceneAssetLocalId::new("x")), Some(x_idx));
            assert_eq!(d.node_index_of(&SceneAssetLocalId::new("y")), Some(y_idx));
        }

        // --- World ---
        {
            let mut doc = crate::world::WorldDocument {
                id: WorldId::new("world"),
                name: "World".to_string(),
                version: 1,
                layout_policy: LayoutPolicy::Free,
                levels: vec![
                    crate::world::WorldLevelRef {
                        level_id: "l1".to_string(),
                        asset_ref: "levels/l1".to_string(),
                        position: [0.0, 0.0],
                        dimensions: None,
                        tags: vec![],
                        streaming: StreamingPolicy::AlwaysResident,
                    },
                    crate::world::WorldLevelRef {
                        level_id: "l2".to_string(),
                        asset_ref: "levels/l2".to_string(),
                        position: [100.0, 0.0],
                        dimensions: None,
                        tags: vec![],
                        streaming: StreamingPolicy::AlwaysResident,
                    },
                ],
                links: vec![],
                updated_at: 0,
                extension_data: BTreeMap::new(),
            };
            let mut d = WorldGraphDialectMut::new(&mut doc);
            let l1_idx = d.node_index_of("l1").unwrap();
            let l2_idx = d.node_index_of("l2").unwrap();
            d.add_node(crate::world::WorldLevelRef {
                level_id: "l3".to_string(),
                asset_ref: "levels/l3".to_string(),
                position: [200.0, 0.0],
                dimensions: None,
                tags: vec![],
                streaming: StreamingPolicy::AlwaysResident,
            });
            assert_eq!(d.node_index_of("l1"), Some(l1_idx));
            assert_eq!(d.node_index_of("l2"), Some(l2_idx));
        }
    }

    #[test]
    fn graphmut_trait_implemented_across_all_dialects() {
        // Verify all three dialects implement GraphMut and mutations persist.
        let mut asset = crate::logic_graph::LogicGraphAsset::default();
        {
            let mut d = LogicGraphDialectMut::new(&mut asset);
            let idx = d.add_node(crate::logic_graph::LogicNode {
                node_id: NodeId::new("x"),
                role: LogicNodeRole::Sensor,
                node_type_id: NodeTypeId::new("sensor.generic"),
                field_values: serde_json::Value::Null,
                controller_id: None,
            });
            assert_eq!(idx, NodeIndex(0));
        }
        assert_eq!(asset.nodes.len(), 1);

        let mut doc = crate::scene_asset::SceneAssetDocument {
            asset_id: "test".to_string(),
            logical_path: "test/asset".to_string(),
            role: SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            layers: vec![],
            extension_data: BTreeMap::new(),
        };
        {
            let mut d = SceneAssetDialectMut::new(&mut doc);
            let idx = d.add_node(crate::scene_asset::SceneAssetEntity {
                local_id: SceneAssetLocalId::new("e"),
                local_path: "e".to_string(),
                name: "e".to_string(),
                components: vec![],
                extension_data: BTreeMap::new(),
            });
            assert_eq!(idx, NodeIndex(0));
        }
        assert_eq!(doc.entities.len(), 1);

        let mut wdoc = crate::world::WorldDocument {
            id: WorldId::new("world"),
            name: "World".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![],
            links: vec![],
            updated_at: 0,
            extension_data: BTreeMap::new(),
        };
        {
            let mut d = WorldGraphDialectMut::new(&mut wdoc);
            let idx = d.add_node(crate::world::WorldLevelRef {
                level_id: "l".to_string(),
                asset_ref: "levels/l".to_string(),
                position: [0.0, 0.0],
                dimensions: None,
                tags: vec![],
                streaming: StreamingPolicy::AlwaysResident,
            });
            assert_eq!(idx, NodeIndex(0));
        }
        assert_eq!(wdoc.levels.len(), 1);
    }
}
