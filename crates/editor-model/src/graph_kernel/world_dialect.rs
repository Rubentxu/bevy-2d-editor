//! World Workspace Graph Dialect (GRAPH-005).
//!
//! Adapts `WorldDocument` to the GRAPH-001 `Graph` trait. Nodes are
//! `WorldLevelRef` keyed by `level_id` and ordered by `WorldDocument.levels`
//! insertion. Edges are `WorldLink` in their canonical direction `from -> to`.
//!
//! Bidirectional / Custom links are exposed as a single edge in canonical
//! direction; runtime traversal semantics are a per-call concern kept in
//! `WorldLinkKind` for the renderer and save formats. See
//! `graph-kernel-m1-graph005/spec.md` for the rationale.

use crate::world::{WorldDocument, WorldLevelRef, WorldLink};
use super::{EdgeIndex, Graph, NodeIndex};

/// Direction-aware adapter that exposes a `WorldDocument` as a directed graph.
///
/// Edges follow `from -> to` for every link, regardless of `WorldLinkKind`.
/// The kernel's topological sort therefore returns the canonical render order
/// (sources before targets) for OneWay links; Bidirectional / Custom links
/// are present in the graph but the kernel only follows the canonical direction.
///
/// Construction is O(n + m).
#[derive(Debug)]
pub struct WorldGraphDialect<'a> {
    world: &'a WorldDocument,
    /// Insertion-order index: `level_id -> NodeIndex`.
    ///
    /// Linear scan is fine for the expected world sizes (≤ 10k levels); if
    /// profiling ever pushes this past the threshold we can swap to a hashmap
    /// without changing the public surface.
    level_index: Vec<(u32, &'a str)>,
}

impl<'a> WorldGraphDialect<'a> {
    /// Build a dialect over the given world document.
    pub fn new(world: &'a WorldDocument) -> Self {
        let level_index = world
            .levels
            .iter()
            .enumerate()
            .map(|(i, l)| (i as u32, l.level_id.as_str()))
            .collect();
        Self { world, level_index }
    }

    /// Borrow the underlying world (useful for callers that want to translate
    /// back from `NodeIndex` to `WorldLevelRef`).
    pub fn world(&self) -> &'a WorldDocument {
        self.world
    }

    /// Translate a `level_id` to its `NodeIndex`. Returns `None` if the level
    /// is not in the world.
    pub fn node_index_of(&self, level_id: &str) -> Option<NodeIndex> {
        self.level_index
            .iter()
            .find(|(_, id)| *id == level_id)
            .map(|(i, _)| NodeIndex(*i))
    }
}

impl<'a> Graph for WorldGraphDialect<'a> {
    type NodeData = WorldLevelRef;
    type EdgeData = WorldLink;
    type Error = std::convert::Infallible;

    fn node_count(&self) -> usize {
        self.world.levels.len()
    }

    fn edge_count(&self) -> usize {
        self.world.links.len()
    }

    fn node(&self, idx: NodeIndex) -> Option<&Self::NodeData> {
        self.world.levels.get(idx.0 as usize)
    }

    fn edge(&self, idx: EdgeIndex) -> Option<&Self::EdgeData> {
        self.world.links.get(idx.0 as usize)
    }

    fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        let link = self.edge(idx)?;
        let src = self.node_index_of(link.from.as_str())?;
        let tgt = self.node_index_of(link.to.as_str())?;
        Some((src, tgt))
    }

    fn outgoing(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let level = self.node(node).map(|l| l.level_id.clone());
        let level = match level {
            Some(l) => l,
            None => return Box::new(std::iter::empty()),
        };
        Box::new(
            self.world
                .links
                .iter()
                .enumerate()
                .filter(move |(_, l)| l.from == level)
                .map(|(i, _)| EdgeIndex(i as u32)),
        )
    }

    fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        let level = self.node(node).map(|l| l.level_id.clone());
        let level = match level {
            Some(l) => l,
            None => return Box::new(std::iter::empty()),
        };
        Box::new(
            self.world
                .links
                .iter()
                .enumerate()
                .filter(move |(_, l)| l.to == level)
                .map(|(i, _)| EdgeIndex(i as u32)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{
        LayoutPolicy, LinkDirection, StreamingPolicy, WorldId, WorldLinkKind,
    };
    use std::collections::BTreeMap;

    fn level(id: &str) -> WorldLevelRef {
        WorldLevelRef {
            level_id: id.to_string(),
            asset_ref: format!("levels/{id}"),
            position: [0.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: StreamingPolicy::AlwaysResident,
        }
    }

    fn link(id: &str, from: &str, to: &str) -> WorldLink {
        WorldLink {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            direction: LinkDirection::East,
            kind: WorldLinkKind::OneWay,
            entrance: None,
            exit: None,
        }
    }

    fn world(levels: Vec<WorldLevelRef>, links: Vec<WorldLink>) -> WorldDocument {
        WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels,
            links,
            updated_at: 0,
            extension_data: BTreeMap::new(),
        }
    }

    #[test]
    fn empty_world_has_no_nodes_no_edges() {
        let w = world(vec![], vec![]);
        let d = WorldGraphDialect::new(&w);
        assert_eq!(d.node_count(), 0);
        assert_eq!(d.edge_count(), 0);
        assert!(super::super::roots(&d).is_empty());
        assert!(super::super::leaves(&d).is_empty());
        assert!(!super::super::has_cycle(&d));
    }

    #[test]
    fn single_node_is_own_root_and_leaf() {
        let w = world(vec![level("a")], vec![]);
        let d = WorldGraphDialect::new(&w);
        assert_eq!(d.node_count(), 1);
        let roots = super::super::roots(&d);
        let leaves = super::super::leaves(&d);
        assert_eq!(roots.len(), 1);
        assert_eq!(leaves.len(), 1);
        assert_eq!(roots[0], leaves[0]);
        assert_eq!(roots[0].0, 0);
    }

    #[test]
    fn linear_chain_returns_root_and_leaf() {
        let w = world(
            vec![level("a"), level("b"), level("c")],
            vec![link("ab", "a", "b"), link("bc", "b", "c")],
        );
        let d = WorldGraphDialect::new(&w);
        let roots = super::super::roots(&d);
        let leaves = super::super::leaves(&d);
        assert_eq!(roots, vec![NodeIndex(0)]);
        assert_eq!(leaves, vec![NodeIndex(2)]);

        let sorted = super::super::topological_sort(&d).expect("DAG");
        assert_eq!(sorted, vec![NodeIndex(0), NodeIndex(1), NodeIndex(2)]);
    }

    #[test]
    fn bidirectional_link_exposed_as_one_edge_canonical() {
        let mut l = link("ab", "a", "b");
        l.kind = WorldLinkKind::Bidirectional;
        let w = world(vec![level("a"), level("b")], vec![l]);
        let d = WorldGraphDialect::new(&w);

        let a = d.node_index_of("a").unwrap();
        let b = d.node_index_of("b").unwrap();
        let out_a: Vec<_> = d.outgoing(a).collect();
        let out_b: Vec<_> = d.outgoing(b).collect();
        let in_b: Vec<_> = d.incoming(b).collect();
        let in_a: Vec<_> = d.incoming(a).collect();
        assert_eq!(out_a.len(), 1, "a -> b present");
        assert_eq!(out_b.len(), 0, "b has no canonical outgoing edge");
        assert_eq!(in_b.len(), 1);
        assert_eq!(in_a.len(), 0);
    }

    #[test]
    fn has_cycle_detects_three_node_loop() {
        let w = world(
            vec![level("a"), level("b"), level("c")],
            vec![link("ab", "a", "b"), link("bc", "b", "c"), link("ca", "c", "a")],
        );
        let d = WorldGraphDialect::new(&w);
        assert!(super::super::has_cycle(&d));
        assert!(super::super::topological_sort(&d).is_err());
    }

    #[test]
    fn dangling_link_does_not_panic() {
        // link points to a level that does not exist
        let w = world(vec![level("a")], vec![link("az", "a", "z")]);
        let d = WorldGraphDialect::new(&w);
        let a = d.node_index_of("a").unwrap();
        let out_a: Vec<_> = d.outgoing(a).collect();
        assert_eq!(out_a.len(), 1, "link is still exposed");
        assert_eq!(d.node_count(), 1);
        assert!(!super::super::has_cycle(&d));
    }

    #[test]
    fn node_index_of_returns_none_for_unknown_level() {
        let w = world(vec![level("a")], vec![]);
        let d = WorldGraphDialect::new(&w);
        assert_eq!(d.node_index_of("a"), Some(NodeIndex(0)));
        assert_eq!(d.node_index_of("ghost"), None);
    }

    #[test]
    fn edge_endpoints_round_trip() {
        let w = world(
            vec![level("a"), level("b")],
            vec![link("ab", "a", "b")],
        );
        let d = WorldGraphDialect::new(&w);
        let a = d.node_index_of("a").unwrap();
        let b = d.node_index_of("b").unwrap();
        let outs: Vec<_> = d.outgoing(a).collect();
        assert_eq!(outs.len(), 1);
        let (src, tgt) = d.edge_endpoints(outs[0]).unwrap();
        assert_eq!(src, a);
        assert_eq!(tgt, b);
    }

    #[test]
    fn diamond_no_cycle() {
        // a -> b, a -> c, b -> d, c -> d (no cycle)
        let w = world(
            vec![level("a"), level("b"), level("c"), level("d")],
            vec![
                link("ab", "a", "b"),
                link("ac", "a", "c"),
                link("bd", "b", "d"),
                link("cd", "c", "d"),
            ],
        );
        let d = WorldGraphDialect::new(&w);
        assert!(!super::super::has_cycle(&d));
        let a = d.node_index_of("a").unwrap();
        let descendants = super::super::descendants(&d, a);
        // descendants includes the root itself per kernel semantics.
        assert_eq!(descendants.len(), 4, "a + b + c + d");
        assert!(descendants.contains(&NodeIndex(0)));
        assert!(descendants.contains(&NodeIndex(1)));
        assert!(descendants.contains(&NodeIndex(2)));
        assert!(descendants.contains(&NodeIndex(3)));
    }

    #[test]
    fn node_returns_level_ref() {
        let w = world(vec![level("alpha")], vec![]);
        let d = WorldGraphDialect::new(&w);
        let n = d.node(NodeIndex(0)).unwrap();
        assert_eq!(n.level_id, "alpha");
    }
}
