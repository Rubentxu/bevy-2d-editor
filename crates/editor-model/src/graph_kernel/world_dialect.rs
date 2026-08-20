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
//!
//! Two variants are provided:
//! - `WorldGraphDialect<'a>` — read-only view over `&'a WorldDocument`
//! - `WorldGraphDialectMut<'a>` — mutable view over `&'a mut WorldDocument`

use std::collections::BTreeMap;

use super::{EdgeIndex, Graph, GraphKernelError, GraphMut, GraphMutStrictness, NodeIndex};
use crate::world::{WorldDocument, WorldLevelRef, WorldLink};

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
    use crate::world::{LayoutPolicy, LinkDirection, StreamingPolicy, WorldId, WorldLinkKind};
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
            vec![
                link("ab", "a", "b"),
                link("bc", "b", "c"),
                link("ca", "c", "a"),
            ],
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
        let w = world(vec![level("a"), level("b")], vec![link("ab", "a", "b")]);
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

// ============================================================================
// WorldGraphDialectMut — mutable dialect with Free strictness.
// ============================================================================

/// Mutable adapter that owns `&'a mut WorldDocument` and implements `GraphMut`.
///
/// This dialect has `strictness() == Free`: all topology mutations are allowed.
/// Cycles, self-loops, and duplicate edges are permitted (and may be useful
/// for debugging, visualization, or temporary states).
pub struct WorldGraphDialectMut<'a> {
    /// The owned mutable reference to the world document.
    doc: &'a mut WorldDocument,
    /// Maps level_id to NodeIndex. Rebuilt on every mutation.
    level_index: BTreeMap<String, NodeIndex>,
    /// Maps (from, to) to EdgeIndex. Rebuilt on every edge mutation.
    link_index: BTreeMap<(String, String), EdgeIndex>,
}

impl<'a> WorldGraphDialectMut<'a> {
    /// Build a mutable dialect over `doc`. The dialect borrows `doc` for
    /// its lifetime.
    pub fn new(doc: &'a mut WorldDocument) -> Self {
        let level_index: BTreeMap<String, NodeIndex> = doc
            .levels
            .iter()
            .enumerate()
            .map(|(i, l)| (l.level_id.clone(), NodeIndex(i as u32)))
            .collect();

        let link_index: BTreeMap<(String, String), EdgeIndex> = doc
            .links
            .iter()
            .enumerate()
            .map(|(i, l)| ((l.from.clone(), l.to.clone()), EdgeIndex(i as u32)))
            .collect();

        Self {
            doc,
            level_index,
            link_index,
        }
    }

    /// Resolve a `level_id` to its `NodeIndex` inside this dialect view.
    pub fn node_index_of(&self, level_id: &str) -> Option<NodeIndex> {
        self.level_index.get(level_id).copied()
    }

    /// Rebuild the level index from the current doc.levels vec.
    fn rebuild_level_index(&mut self) {
        self.level_index = self
            .doc
            .levels
            .iter()
            .enumerate()
            .map(|(i, l)| (l.level_id.clone(), NodeIndex(i as u32)))
            .collect();
    }

    /// Rebuild the link index from the current doc.links vec.
    fn rebuild_link_index(&mut self) {
        self.link_index = self
            .doc
            .links
            .iter()
            .enumerate()
            .map(|(i, l)| ((l.from.clone(), l.to.clone()), EdgeIndex(i as u32)))
            .collect();
    }
}

impl<'a> Graph for WorldGraphDialectMut<'a> {
    type NodeData = WorldLevelRef;
    type EdgeData = WorldLink;
    type Error = std::convert::Infallible;

    fn node_count(&self) -> usize {
        self.doc.levels.len()
    }

    fn edge_count(&self) -> usize {
        self.doc.links.len()
    }

    fn node(&self, idx: NodeIndex) -> Option<&Self::NodeData> {
        self.doc.levels.get(idx.0 as usize)
    }

    fn edge(&self, idx: EdgeIndex) -> Option<&Self::EdgeData> {
        self.doc.links.get(idx.0 as usize)
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
            self.doc
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
            self.doc
                .links
                .iter()
                .enumerate()
                .filter(move |(_, l)| l.to == level)
                .map(|(i, _)| EdgeIndex(i as u32)),
        )
    }
}

impl<'a> GraphMut for WorldGraphDialectMut<'a> {
    fn strictness(&self) -> GraphMutStrictness {
        GraphMutStrictness::Free
    }

    fn add_node(&mut self, data: Self::NodeData) -> NodeIndex {
        let idx = NodeIndex(self.doc.levels.len() as u32);
        self.doc.levels.push(data);
        self.rebuild_level_index();
        idx
    }

    fn add_edge(
        &mut self,
        src: NodeIndex,
        dst: NodeIndex,
        data: Self::EdgeData,
    ) -> Result<EdgeIndex, GraphKernelError> {
        if src.0 as usize >= self.doc.levels.len() || dst.0 as usize >= self.doc.levels.len() {
            return Err(GraphKernelError::NodeIndexOutOfRange {
                idx: if src.0 as usize >= self.doc.levels.len() {
                    src
                } else {
                    dst
                },
                total: self.doc.levels.len(),
            });
        }
        // Free strictness: no validation — self-loop, duplicate, cycle all allowed.
        let idx = EdgeIndex(self.doc.links.len() as u32);
        self.doc.links.push(data);
        self.rebuild_link_index();
        Ok(idx)
    }

    fn remove_node(&mut self, idx: NodeIndex) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.doc.levels.len() {
            return Err(GraphKernelError::NodeIndexOutOfRange {
                idx,
                total: self.doc.levels.len(),
            });
        }
        let removed_id = self.doc.levels[idx.0 as usize].level_id.clone();
        // Cascade: remove all links where this level is from or to.
        self.doc
            .links
            .retain(|l| l.from != removed_id && l.to != removed_id);
        self.doc.levels.remove(idx.0 as usize);
        self.rebuild_level_index();
        self.rebuild_link_index();
        Ok(())
    }

    fn remove_edge(&mut self, idx: EdgeIndex) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.doc.links.len() {
            return Err(GraphKernelError::EdgeIndexOutOfRange {
                idx,
                total: self.doc.links.len(),
            });
        }
        self.doc.links.remove(idx.0 as usize);
        self.rebuild_link_index();
        Ok(())
    }

    fn update_node(
        &mut self,
        idx: NodeIndex,
        data: Self::NodeData,
    ) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.doc.levels.len() {
            return Err(GraphKernelError::NodeIndexOutOfRange {
                idx,
                total: self.doc.levels.len(),
            });
        }
        self.doc.levels[idx.0 as usize] = data;
        self.rebuild_level_index();
        Ok(())
    }

    fn update_edge(
        &mut self,
        idx: EdgeIndex,
        data: Self::EdgeData,
    ) -> Result<(), GraphKernelError> {
        if idx.0 as usize >= self.doc.links.len() {
            return Err(GraphKernelError::EdgeIndexOutOfRange {
                idx,
                total: self.doc.links.len(),
            });
        }
        self.doc.links[idx.0 as usize] = data;
        self.rebuild_link_index();
        Ok(())
    }
}

// ============================================================================
// Tests for WorldGraphDialectMut.
// ============================================================================

#[cfg(test)]
mod world_dialect_mut_tests {
    use super::*;
    use crate::world::{LayoutPolicy, LinkDirection, StreamingPolicy, WorldId, WorldLinkKind};
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
    fn add_node_increments_count() {
        let mut doc = world(vec![], vec![]);
        let mut d = WorldGraphDialectMut::new(&mut doc);
        assert_eq!(d.node_count(), 0);
        d.add_node(level("a"));
        assert_eq!(d.node_count(), 1);
        d.add_node(level("b"));
        assert_eq!(d.node_count(), 2);
        assert_eq!(doc.levels.len(), 2);
    }

    #[test]
    fn add_edge_allows_self_loop() {
        // Free strictness: self-loops are allowed.
        let mut doc = world(vec![level("a")], vec![]);
        let mut d = WorldGraphDialectMut::new(&mut doc);
        let a_idx = d.node_index_of("a").unwrap();
        let result = d.add_edge(a_idx, a_idx, link("a->a", "a", "a"));
        assert!(result.is_ok(), "Free strictness allows self-loops");
    }

    #[test]
    fn add_edge_allows_duplicate() {
        // Free strictness: duplicate edges are allowed.
        let mut doc = world(vec![level("a"), level("b")], vec![]);
        let mut d = WorldGraphDialectMut::new(&mut doc);
        let a_idx = d.node_index_of("a").unwrap();
        let b_idx = d.node_index_of("b").unwrap();
        let first = d.add_edge(a_idx, b_idx, link("ab1", "a", "b"));
        assert!(first.is_ok());
        let second = d.add_edge(a_idx, b_idx, link("ab2", "a", "b"));
        assert!(second.is_ok(), "Free strictness allows duplicate edges");
        assert_eq!(d.edge_count(), 2);
    }

    #[test]
    fn add_edge_allows_cycle() {
        // Free strictness: cycles are allowed.
        let mut doc = world(vec![level("a"), level("b"), level("c")], vec![]);
        let mut d = WorldGraphDialectMut::new(&mut doc);
        let a_idx = d.node_index_of("a").unwrap();
        let c_idx = d.node_index_of("c").unwrap();
        // Add c->a (closing a->b->c->a cycle).
        let result = d.add_edge(c_idx, a_idx, link("ca", "c", "a"));
        assert!(result.is_ok(), "Free strictness allows cycles");
    }

    #[test]
    fn remove_node_cascades_links() {
        let mut doc = world(
            vec![level("a"), level("b"), level("c")],
            vec![link("ab", "a", "b"), link("bc", "b", "c")],
        );
        let mut d = WorldGraphDialectMut::new(&mut doc);
        let b_idx = d.node_index_of("b").unwrap();
        d.remove_node(b_idx).unwrap();
        assert_eq!(d.node_count(), 2);
        assert_eq!(d.edge_count(), 0); // Both links removed via cascade.
    }

    #[test]
    fn remove_node_returns_error_for_out_of_range() {
        let mut doc = world(vec![level("a")], vec![]);
        let mut d = WorldGraphDialectMut::new(&mut doc);
        let result = d.remove_node(NodeIndex(99));
        assert!(matches!(
            result.unwrap_err(),
            GraphKernelError::NodeIndexOutOfRange { idx, total }
            if idx == NodeIndex(99) && total == 1
        ));
    }

    #[test]
    fn update_node_replaces_data() {
        let mut doc = world(vec![level("a")], vec![]);
        let mut d = WorldGraphDialectMut::new(&mut doc);
        let a_idx = d.node_index_of("a").unwrap();
        let mut new_level = level("a");
        new_level.position = [100.0, 200.0];
        d.update_node(a_idx, new_level).unwrap();
        assert_eq!(d.node(a_idx).unwrap().position, [100.0, 200.0]);
    }

    #[test]
    fn update_edge_replaces_data() {
        let mut doc = world(vec![level("a"), level("b")], vec![link("ab", "a", "b")]);
        let mut d = WorldGraphDialectMut::new(&mut doc);
        let edge_idx = EdgeIndex(0);
        let mut new_link = link("ab", "a", "b");
        new_link.direction = LinkDirection::West;
        d.update_edge(edge_idx, new_link).unwrap();
        assert_eq!(d.edge(edge_idx).unwrap().direction, LinkDirection::West);
    }

    #[test]
    fn strictness_is_free() {
        let mut doc = world(vec![], vec![]);
        let d = WorldGraphDialectMut::new(&mut doc);
        assert_eq!(d.strictness(), GraphMutStrictness::Free);
    }
}
