//! ChangeSetDialect — adapts `ChangeSet<O>` to the kernel `Graph` trait.
//!
//! The dialect treats each op in `ChangeSet.ops` as a node. For each entry
//! `op_dependencies[i] = [j]`, the dialect exposes a directed edge
//! `(depends_on, op_idx) = (j, i)`. The kernel's topological sort returns
//! `depends_on` before `op_idx`, which matches the apply order we want:
//! ops that produce data run before ops that consume it.
//!
//! Edges are enumerated in row-major order: `op_dependencies[0]` first, then
//! `op_dependencies[1]`, etc. This ordering is deterministic and matches the
//! flat `EdgeIndex` space the kernel expects.
//!
//! Cycles are prevented at build time by `ChangeSet::add_op_dependency`. The
//! dialect therefore never observes a cycle through the public API; the
//! kernel's `has_cycle` is still available for tests and for callers that
//! bypass the builder.
//!
//! See GRAPH-003 spec for the design.

use std::convert::Infallible;

use crate::transaction::ChangeSet;

use super::{EdgeIndex, Graph, NodeIndex};

/// Adapter that lets `ChangeSet<O>` be read as a `Graph`.
///
/// Dialects are cheap to construct: they borrow the change-set for their
/// lifetime. They are not owned.
pub struct ChangeSetDialect<'a, O> {
    cs: &'a ChangeSet<O>,
    /// Cached row-prefix sizes: `prefix[i] = sum(deps[0..i].len())`.
    /// Used to translate `(op_idx, dep_index_within_row)` to a flat `EdgeIndex`.
    prefix: Vec<usize>,
}

impl<'a, O> ChangeSetDialect<'a, O> {
    /// Build a dialect view over `cs`. The dialect borrows `cs` for its lifetime.
    pub fn new(cs: &'a ChangeSet<O>) -> Self {
        let prefix = compute_prefix(cs.op_dependencies());
        Self { cs, prefix }
    }

    /// Borrow the underlying `ChangeSet<O>`.
    pub fn change_set(&self) -> &ChangeSet<O> {
        self.cs
    }

    /// Resolve an op index to its `NodeIndex` (always `NodeIndex(i)`).
    pub fn node_index_of(&self, op_index: usize) -> Option<NodeIndex> {
        if op_index < self.cs.ops.len() {
            Some(NodeIndex(op_index as u32))
        } else {
            None
        }
    }

    /// Look up the `(op_idx, dep_index_within_row)` pair for a flat `EdgeIndex`.
    pub fn edge_origins(&self, idx: EdgeIndex) -> Option<(usize, usize)> {
        let target = idx.0 as usize;
        let mut prefix = 0usize;
        for (op_idx, deps) in self.cs.op_dependencies().iter().enumerate() {
            let next = prefix + deps.len();
            if target < next {
                return Some((op_idx, target - prefix));
            }
            prefix = next;
        }
        None
    }
}

fn compute_prefix(deps: &[Vec<usize>]) -> Vec<usize> {
    let mut prefix = Vec::with_capacity(deps.len() + 1);
    let mut acc = 0usize;
    prefix.push(0);
    for row in deps {
        acc += row.len();
        prefix.push(acc);
    }
    prefix
}

impl<'a, O: Clone> Graph for ChangeSetDialect<'a, O> {
    type NodeData = O;
    type EdgeData = ();
    type Error = Infallible;

    fn node_count(&self) -> usize {
        self.cs.ops.len()
    }
    fn edge_count(&self) -> usize {
        self.cs.op_dependencies().iter().map(|d| d.len()).sum()
    }
    fn node(&self, idx: NodeIndex) -> Option<&O> {
        self.cs.ops.get(idx.0 as usize)
    }
    fn edge(&self, _idx: EdgeIndex) -> Option<&()> {
        Some(&())
    }
    fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        let (op_idx, dep_index) = self.edge_origins(idx)?;
        let dep = *self.cs.op_dependencies().get(op_idx)?.get(dep_index)?;
        // Edge is (depends_on, op_idx) — depends_on runs first, then op_idx.
        Some((NodeIndex(dep as u32), NodeIndex(op_idx as u32)))
    }
    fn outgoing(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        // Edges where this node is the DEPENDED-ON (source / "from").
        let target_dep = node.0 as usize;
        let mut out: Vec<EdgeIndex> = Vec::new();
        let mut prefix = 0usize;
        for deps in self.cs.op_dependencies() {
            for (i, &dep) in deps.iter().enumerate() {
                if dep == target_dep {
                    out.push(EdgeIndex((prefix + i) as u32));
                }
            }
            prefix += deps.len();
        }
        Box::new(out.into_iter())
    }
    fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_> {
        // Edges where this node is the DEPENDENT (target / "to").
        let target_op = node.0 as usize;
        let prefix = self.prefix.get(target_op).copied().unwrap_or(0);
        let deps = self
            .cs
            .op_dependencies()
            .get(target_op)
            .cloned()
            .unwrap_or_default();
        Box::new(
            deps.into_iter()
                .enumerate()
                .map(move |(i, _)| EdgeIndex((prefix + i) as u32)),
        )
    }
}

#[cfg(test)]
mod dialect_tests {
    use super::*;
    use crate::graph_kernel::{descendants, has_cycle, roots, topological_sort};
    use crate::transaction::{ChangeOrigin, ChangeSet, ChangeSetError};

    fn empty_cs() -> ChangeSet<String> {
        ChangeSet::new(
            "empty".to_string(),
            ChangeOrigin::Human,
            "t".to_string(),
            "t".to_string(),
        )
    }

    fn cs_with_n_ops(n: usize) -> ChangeSet<String> {
        let mut cs = empty_cs();
        for i in 0..n {
            cs.push_op(format!("op-{i}"));
        }
        cs
    }

    #[test]
    fn dialect_empty_changeset_has_no_nodes() {
        let cs = empty_cs();
        let d = ChangeSetDialect::new(&cs);
        assert_eq!(d.node_count(), 0);
        assert_eq!(d.edge_count(), 0);
        assert!(roots(&d).is_empty());
        assert!(has_cycle(&d) == false);
    }

    #[test]
    fn dialect_single_op_is_both_root_and_leaf() {
        let cs = cs_with_n_ops(1);
        let d = ChangeSetDialect::new(&cs);
        assert_eq!(d.node_count(), 1);
        let r = roots(&d);
        assert_eq!(r.len(), 1);
        assert_eq!(d.node(r[0]).unwrap(), "op-0");
    }

    #[test]
    fn dialect_parallel_no_deps_are_all_roots() {
        let cs = cs_with_n_ops(4);
        let d = ChangeSetDialect::new(&cs);
        let r = roots(&d);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn dialect_chain_deps_topological_sort() {
        let mut cs = cs_with_n_ops(3);
        // op 0 depends on op 1; op 1 depends on op 2.
        // Apply order: op 2 first, then op 1, then op 0.
        assert!(cs.add_op_dependency(0, 1).is_ok());
        assert!(cs.add_op_dependency(1, 2).is_ok());
        let d = ChangeSetDialect::new(&cs);
        let sorted = topological_sort(&d).unwrap();
        let labels: Vec<String> = sorted.iter().map(|i| d.node(*i).unwrap().clone()).collect();
        assert_eq!(labels, vec!["op-2", "op-1", "op-0"]);
    }

    #[test]
    fn dialect_diamond_deps_no_cycle() {
        let mut cs = cs_with_n_ops(4);
        // 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3 (diamond)
        assert!(cs.add_op_dependency(1, 0).is_ok());
        assert!(cs.add_op_dependency(2, 0).is_ok());
        assert!(cs.add_op_dependency(3, 1).is_ok());
        assert!(cs.add_op_dependency(3, 2).is_ok());
        let d = ChangeSetDialect::new(&cs);
        assert!(!has_cycle(&d));
        let sorted = topological_sort(&d).unwrap();
        let labels: Vec<String> = sorted.iter().map(|i| d.node(*i).unwrap().clone()).collect();
        assert_eq!(labels[0], "op-0");
        assert_eq!(labels[3], "op-3");
    }

    #[test]
    fn dialect_descendants_walk_follows_dependents() {
        // In the dialect, outgoing = "to dependents". So descendants(2) returns
        // the ops that transitively depend on op 2.
        let mut cs = cs_with_n_ops(3);
        assert!(cs.add_op_dependency(0, 1).is_ok());
        assert!(cs.add_op_dependency(1, 2).is_ok());
        let d = ChangeSetDialect::new(&cs);
        let desc = descendants(&d, d.node_index_of(2).unwrap());
        let labels: Vec<String> = desc.iter().map(|i| d.node(*i).unwrap().clone()).collect();
        assert_eq!(labels, vec!["op-2", "op-1", "op-0"]);
    }

    #[test]
    fn dialect_incoming_walk_returns_direct_dependencies() {
        // To get the ops that `op_idx` directly depends on, walk incoming.
        let mut cs = cs_with_n_ops(3);
        assert!(cs.add_op_dependency(0, 1).is_ok());
        assert!(cs.add_op_dependency(0, 2).is_ok());
        let d = ChangeSetDialect::new(&cs);
        let in_zero: Vec<EdgeIndex> = d.incoming(NodeIndex(0)).collect();
        // op 0 depends on op 1 and op 2; incoming(0) returns the edges (1, 0)
        // and (2, 0) — both target=0.
        assert_eq!(in_zero.len(), 2);
    }

    #[test]
    fn dialect_edge_endpoints_returns_depends_on_and_op() {
        let mut cs = cs_with_n_ops(3);
        // op 0 depends on op 1. Edge is (1, 0): source=1, target=0.
        assert!(cs.add_op_dependency(0, 1).is_ok());
        let d = ChangeSetDialect::new(&cs);
        let (src, dst) = d.edge_endpoints(EdgeIndex(0)).unwrap();
        assert_eq!(src, NodeIndex(1));
        assert_eq!(dst, NodeIndex(0));
    }

    #[test]
    fn dialect_outgoing_returns_in_order_edges() {
        let mut cs = cs_with_n_ops(2);
        // op 1 depends on op 0. Edge is (0, 1): source=0, target=1.
        assert!(cs.add_op_dependency(1, 0).is_ok());
        let d = ChangeSetDialect::new(&cs);
        // outgoing(0) returns edges where 0 is the source (depended-on).
        let out: Vec<EdgeIndex> = d.outgoing(NodeIndex(0)).collect();
        assert_eq!(out, vec![EdgeIndex(0)]);
        // incoming(1) returns edges where 1 is the target (dependent).
        let in_one: Vec<EdgeIndex> = d.incoming(NodeIndex(1)).collect();
        assert_eq!(in_one, vec![EdgeIndex(0)]);
    }

    #[test]
    fn add_op_dependency_out_of_range_returns_err() {
        let mut cs = cs_with_n_ops(2);
        let r = cs.add_op_dependency(5, 0);
        assert_eq!(
            r,
            Err(ChangeSetError::OutOfRange {
                op_idx: 5,
                ops_len: 2
            })
        );
        let r = cs.add_op_dependency(0, 5);
        assert_eq!(
            r,
            Err(ChangeSetError::OutOfRange {
                op_idx: 5,
                ops_len: 2
            })
        );
    }

    #[test]
    fn add_op_dependency_self_dependency_returns_err() {
        let mut cs = cs_with_n_ops(2);
        let r = cs.add_op_dependency(0, 0);
        assert_eq!(r, Err(ChangeSetError::SelfDependency { op_idx: 0 }));
    }

    #[test]
    fn add_op_dependency_would_create_cycle_returns_err() {
        let mut cs = cs_with_n_ops(3);
        assert!(cs.add_op_dependency(1, 0).is_ok());
        assert!(cs.add_op_dependency(2, 1).is_ok());
        // 0 <-> 2 via 0 <- 1 <- 2: adding 0 -> 2 would close a cycle.
        let r = cs.add_op_dependency(0, 2);
        assert_eq!(
            r,
            Err(ChangeSetError::WouldCreateCycle {
                op_idx: 0,
                depends_on: 2
            })
        );
    }

    #[test]
    fn add_op_dependency_valid_returns_ok() {
        let mut cs = cs_with_n_ops(3);
        assert!(cs.add_op_dependency(0, 1).is_ok());
        assert!(cs.add_op_dependency(1, 2).is_ok());
        assert_eq!(cs.op_dependencies(), &[vec![1], vec![2], vec![]]);
    }

    #[test]
    fn dialect_node_index_of_returns_node_index() {
        let cs = cs_with_n_ops(3);
        let d = ChangeSetDialect::new(&cs);
        assert_eq!(d.node_index_of(0), Some(NodeIndex(0)));
        assert_eq!(d.node_index_of(2), Some(NodeIndex(2)));
        assert_eq!(d.node_index_of(5), None);
    }
}
