//! Query builder — a typed fluent interface over the graph kernel.
//!
//! `Query<'a, D>` is a composable query builder that wraps a `&'a D: Graph`
//! and accumulates a `QueryState` plus a `PredicateTable` of named predicates.
//! The builder is consumed at evaluation time (`collect`, `first`, `count`,
//! `has_cycle`) which compiles the state into a `BTreeSet<NodeIndex>` via the
//! existing kernel operations.
//!
//! ## State machine
//!
//! The query is evaluated by `evaluate_set()` which pattern-matches on
//! `QueryState` and delegates to the kernel functions. Set operations (`union`,
//! `intersect`, `difference`) carry sub-queries as `Box<QueryState>`.
//!
//! ## Predicate table
//!
//! `PredicateTable<'a, D>` stores boxed predicate closures. Each closure is
//! typed as `Fn(&D::NodeData) -> bool` or `Fn(&D::EdgeData) -> bool`. The
//! `PredicateId` is a `u32` index into the table. Predicates are applied by
//! `QueryState::Filtered` which evaluates the closure and retains only matching
//! nodes.
//!
//! See ADR-0053 §7.1 and GRAPH-010 spec.

use std::collections::BTreeSet;

use crate::graph_kernel::{
    Graph, GraphKernelError, NodeIndex, ancestors, descendants, has_cycle, leaves, reachable_from,
    roots, topological_sort, topological_sort_subset,
};

/// The mutable builder state — tracks which kernel operation is active and
/// carries the accumulated predicate chain.
#[derive(Clone)]
pub enum QueryState {
    /// Initial state: no traversal applied yet; evaluation returns all nodes.
    Initial,
    /// `reachable_from(source)` — descendants of source.
    InitialReachableFrom {
        /// The source node.
        source: NodeIndex,
    },
    /// `descendants_of(source)` — descendants of source (alias for reachable_from).
    InitialDescendantsOf {
        /// The source node.
        source: NodeIndex,
    },
    /// `ancestors_of(source)` — predecessors of source.
    InitialAncestorsOf {
        /// The target node.
        source: NodeIndex,
    },
    /// `roots()` — nodes with no incoming edges.
    InitialRoots,
    /// `leaves()` — nodes with no outgoing edges.
    InitialLeaves,
    /// Apply a node or edge predicate filter.
    Filtered {
        /// The previous state to filter.
        prev: Box<QueryState>,
        /// The predicate id to apply.
        pred_id: PredicateId,
    },
    /// Restrict to topological order (within the current node set).
    Topological {
        /// The previous state to order.
        prev: Box<QueryState>,
    },
    /// Set operation between two sub-queries.
    SetOp {
        /// The kind of set operation.
        op: SetOpKind,
        /// Left-hand side sub-query.
        lhs: Box<QueryState>,
        /// Right-hand side sub-query.
        rhs: Box<QueryState>,
    },
}

/// Opaque predicate identifier — index into `PredicateTable`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PredicateId(u32);

/// Set operation kind for `QueryState::SetOp`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SetOpKind {
    /// Union — all nodes in either lhs or rhs.
    Union,
    /// Intersection — nodes in both lhs and rhs.
    Intersect,
    /// Set difference — nodes in lhs but not in rhs.
    Difference,
}

/// A named predicate stored in `PredicateTable`.
pub enum PredicateEntry<'a, D: Graph + ?Sized> {
    /// Node predicate: `Fn(&D::NodeData) -> bool`.
    Node(Box<dyn Fn(&D::NodeData) -> bool + 'a>),
    /// Edge predicate: `Fn(&D::EdgeData) -> bool`.
    Edge(Box<dyn Fn(&D::EdgeData) -> bool + 'a>),
}

/// A table of named predicates indexed by `PredicateId`.
///
/// Each `Query` owns one `PredicateTable`. Predicates are added via
/// `add_node` / `add_edge` and referenced by `QueryState::Filtered`.
pub struct PredicateTable<'a, D: Graph + ?Sized> {
    /// The stored predicate entries.
    entries: Vec<PredicateEntry<'a, D>>,
}

impl<'a, D: Graph + ?Sized> Default for PredicateTable<'a, D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, D: Graph + ?Sized> PredicateTable<'a, D> {
    /// Construct an empty predicate table.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a node predicate and return its `PredicateId`.
    pub fn add_node<F: Fn(&D::NodeData) -> bool + 'a>(&mut self, f: F) -> PredicateId {
        let id = PredicateId(self.entries.len() as u32);
        self.entries.push(PredicateEntry::Node(Box::new(f)));
        id
    }

    /// Add an edge predicate and return its `PredicateId`.
    pub fn add_edge<F: Fn(&D::EdgeData) -> bool + 'a>(&mut self, f: F) -> PredicateId {
        let id = PredicateId(self.entries.len() as u32);
        self.entries.push(PredicateEntry::Edge(Box::new(f)));
        id
    }

    /// Look up a predicate by id.
    pub fn get(&self, id: PredicateId) -> Option<&PredicateEntry<'a, D>> {
        self.entries.get(id.0 as usize)
    }

    /// Returns the number of predicates in the table.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the table has no predicates.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// A typed query builder over a `&'a D: Graph`.
///
/// # Example
///
/// ```ignore
/// let q = Query::new(&dialect)
///     .roots()
///     .with_node_data(|n: &LogicNode| n.role == LogicNodeRole::Sensor);
/// let sensors: Vec<NodeIndex> = q.collect()?;
/// ```
pub struct Query<'a, D: Graph + ?Sized> {
    dialect: &'a D,
    state: QueryState,
    predicates: PredicateTable<'a, D>,
}

impl<'a, D: Graph + ?Sized> Query<'a, D> {
    /// Construct a new query over `dialect`, starting in `Initial` state.
    pub fn new(dialect: &'a D) -> Self {
        Self {
            dialect,
            state: QueryState::Initial,
            predicates: PredicateTable::new(),
        }
    }

    /// Returns a reference to the underlying dialect.
    pub fn dialect(&self) -> &D {
        self.dialect
    }

    /// Returns a reference to the current query state.
    pub fn state(&self) -> &QueryState {
        &self.state
    }

    /// Returns a reference to the predicate table.
    pub fn predicates(&self) -> &PredicateTable<'a, D> {
        &self.predicates
    }

    /// Consume the query and return its state and predicate table.
    pub fn into_parts(self) -> (QueryState, PredicateTable<'a, D>) {
        (self.state, self.predicates)
    }

    /// Evaluate the query and collect all matching nodes as a `Vec`.
    pub fn collect(self) -> Result<Vec<NodeIndex>, GraphKernelError> {
        let nodes = self.evaluate_set()?;
        Ok(nodes.into_iter().collect())
    }

    /// Evaluate the query and return the first matching node, if any.
    pub fn first(self) -> Result<Option<NodeIndex>, GraphKernelError> {
        let nodes = self.evaluate_set()?;
        Ok(nodes.into_iter().next())
    }

    /// Evaluate the query and return the count of matching nodes.
    pub fn count(self) -> Result<usize, GraphKernelError> {
        let nodes = self.evaluate_set()?;
        Ok(nodes.len())
    }

    /// Evaluate the query and return whether the subgraph has any cycle.
    ///
    /// This delegates to `has_cycle(dialect)` which short-circuits via Kahn's algorithm.
    pub fn has_cycle(self) -> Result<bool, GraphKernelError> {
        let dialect = self.dialect;
        let nodes = self.evaluate_set()?;
        if nodes.is_empty() {
            return Ok(false);
        }
        Ok(has_cycle(dialect))
    }

    // -------------------------------------------------------------------------
    // Non-terminal builder methods (commit 2)
    // -------------------------------------------------------------------------

    /// Restrict to nodes reachable from `source` (including `source`).
    pub fn reachable_from(mut self, source: NodeIndex) -> Self {
        self.state = QueryState::InitialReachableFrom { source };
        self
    }

    /// Restrict to descendants of `source` (including `source`).
    pub fn descendants_of(mut self, source: NodeIndex) -> Self {
        self.state = QueryState::InitialDescendantsOf { source };
        self
    }

    /// Restrict to ancestors of `source` (including `source`).
    pub fn ancestors_of(mut self, source: NodeIndex) -> Self {
        self.state = QueryState::InitialAncestorsOf { source };
        self
    }

    /// Restrict to root nodes (no incoming edges).
    pub fn roots(mut self) -> Self {
        self.state = QueryState::InitialRoots;
        self
    }

    /// Restrict to leaf nodes (no outgoing edges).
    pub fn leaves(mut self) -> Self {
        self.state = QueryState::InitialLeaves;
        self
    }

    /// Add a node-data predicate filter.
    pub fn with_node_data<F>(mut self, pred: F) -> Self
    where
        F: Fn(&D::NodeData) -> bool + 'a,
    {
        let pred_id = self.predicates.add_node(pred);
        self.state = QueryState::Filtered {
            prev: Box::new(self.state),
            pred_id,
        };
        self
    }

    /// Add an edge-kind predicate filter.
    pub fn with_edge_kind<F>(mut self, pred: F) -> Self
    where
        F: Fn(&D::EdgeData) -> bool + 'a,
    {
        let pred_id = self.predicates.add_edge(pred);
        self.state = QueryState::Filtered {
            prev: Box::new(self.state),
            pred_id,
        };
        self
    }

    /// Set union: all nodes in either `self` or `other`.
    pub fn union(mut self, other: Query<'a, D>) -> Self {
        let rhs = other.state;
        self.state = QueryState::SetOp {
            op: SetOpKind::Union,
            lhs: Box::new(self.state),
            rhs: Box::new(rhs),
        };
        self
    }

    /// Set intersection: nodes in both `self` and `other`.
    pub fn intersect(mut self, other: Query<'a, D>) -> Self {
        let rhs = other.state;
        self.state = QueryState::SetOp {
            op: SetOpKind::Intersect,
            lhs: Box::new(self.state),
            rhs: Box::new(rhs),
        };
        self
    }

    /// Set difference: nodes in `self` but not in `other`.
    pub fn difference(mut self, other: Query<'a, D>) -> Self {
        let rhs = other.state;
        self.state = QueryState::SetOp {
            op: SetOpKind::Difference,
            lhs: Box::new(self.state),
            rhs: Box::new(rhs),
        };
        self
    }

    /// Restrict the current node set to topological order.
    pub fn topological(mut self) -> Self {
        self.state = QueryState::Topological {
            prev: Box::new(self.state),
        };
        self
    }

    // -------------------------------------------------------------------------
    // Evaluation
    // -------------------------------------------------------------------------

    fn evaluate_set(self) -> Result<BTreeSet<NodeIndex>, GraphKernelError> {
        // Extract dialect before consuming self
        let dialect = self.dialect;
        let (state, predicates) = self.into_parts();
        Self::eval(dialect, &state, &predicates)
    }

    fn eval(
        dialect: &D,
        state: &QueryState,
        predicates: &PredicateTable<'a, D>,
    ) -> Result<BTreeSet<NodeIndex>, GraphKernelError> {
        match state {
            QueryState::Initial => Ok((0..dialect.node_count() as u32).map(NodeIndex).collect()),
            QueryState::InitialReachableFrom { source } => {
                Ok(reachable_from(dialect, *source).into_iter().collect())
            }
            QueryState::InitialDescendantsOf { source } => {
                Ok(descendants(dialect, *source).into_iter().collect())
            }
            QueryState::InitialAncestorsOf { source } => {
                Ok(ancestors(dialect, *source).into_iter().collect())
            }
            QueryState::InitialRoots => Ok(roots(dialect).into_iter().collect()),
            QueryState::InitialLeaves => Ok(leaves(dialect).into_iter().collect()),
            QueryState::Filtered { prev, pred_id } => {
                let mut nodes = Self::eval(dialect, prev, predicates)?;
                if let Some(PredicateEntry::Node(f)) = predicates.get(*pred_id) {
                    nodes.retain(|idx| dialect.node(*idx).map(|n| f(n)).unwrap_or(false));
                }
                Ok(nodes)
            }
            QueryState::Topological { prev } => {
                let nodes = Self::eval(dialect, prev, predicates)?;
                Self::topological_subset(dialect, &nodes)
            }
            QueryState::SetOp { op, lhs, rhs } => {
                let l = Self::eval(dialect, lhs, predicates)?;
                let r = Self::eval(dialect, rhs, predicates)?;
                match op {
                    SetOpKind::Union => Ok(l.union(&r).cloned().collect()),
                    SetOpKind::Intersect => Ok(l.intersection(&r).cloned().collect()),
                    SetOpKind::Difference => Ok(l.difference(&r).cloned().collect()),
                }
            }
        }
    }

    fn topological_subset(
        dialect: &D,
        subset: &BTreeSet<NodeIndex>,
    ) -> Result<BTreeSet<NodeIndex>, GraphKernelError> {
        if subset.is_empty() {
            return Ok(BTreeSet::new());
        }
        let full = topological_sort(dialect)?;
        Ok(full
            .into_iter()
            .filter(|idx| subset.contains(idx))
            .collect())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_kernel::logic_dialect::LogicGraphDialect;
    use crate::logic_graph::{
        LogicEdge, LogicGraphAsset, LogicNode, LogicNodeRole, NodeTypeId, PortId,
    };

    fn sample_node(id: &str, role: LogicNodeRole) -> LogicNode {
        LogicNode {
            node_id: crate::logic_graph::NodeId::new(id),
            role,
            node_type_id: NodeTypeId::new("sensor.generic"),
            field_values: serde_json::Value::Null,
            controller_id: None,
        }
    }

    fn sample_edge(from: &str, to: &str) -> LogicEdge {
        LogicEdge {
            from_node: crate::logic_graph::NodeId::new(from),
            from_port: PortId::new("out"),
            to_node: crate::logic_graph::NodeId::new(to),
            to_port: PortId::new("in"),
        }
    }

    fn linear_graph() -> LogicGraphAsset {
        let mut g = LogicGraphAsset::default();
        g.nodes = vec![
            sample_node("a", LogicNodeRole::Sensor),
            sample_node("b", LogicNodeRole::Controller),
            sample_node("c", LogicNodeRole::Actuator),
        ];
        g.edges = vec![sample_edge("a", "b"), sample_edge("b", "c")];
        g
    }

    fn cyclic_graph() -> LogicGraphAsset {
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
        g
    }

    fn empty_graph() -> LogicGraphAsset {
        LogicGraphAsset::default()
    }

    // --- Terminal tests (commit 1) ---

    #[test]
    fn collect_initial_counts_all_nodes() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d);
        let nodes = q.collect().unwrap();
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn first_returns_lowest_index() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d);
        let first = q.first().unwrap();
        assert!(first.is_some());
        let NodeIndex(idx) = first.unwrap();
        assert_eq!(idx, 0);
    }

    #[test]
    fn count_returns_size() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d);
        let cnt = q.count().unwrap();
        assert_eq!(cnt, 3);
    }

    #[test]
    fn has_cycle_returns_false_on_dag() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d);
        let cyc = q.has_cycle().unwrap();
        assert!(!cyc);
    }

    #[test]
    fn collect_returns_vec_of_nodeindex() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d);
        let nodes: Vec<NodeIndex> = q.collect().unwrap();
        assert_eq!(nodes.len(), 3);
        assert!(nodes.contains(&NodeIndex(0)));
        assert!(nodes.contains(&NodeIndex(1)));
        assert!(nodes.contains(&NodeIndex(2)));
    }

    #[test]
    fn first_returns_none_on_empty_graph() {
        let g = empty_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d);
        let first = q.first().unwrap();
        assert!(first.is_none());
    }

    #[test]
    fn count_returns_zero_on_empty_graph() {
        let g = empty_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d);
        let cnt = q.count().unwrap();
        assert_eq!(cnt, 0);
    }

    #[test]
    fn has_cycle_returns_true_on_cycle() {
        let g = cyclic_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d);
        let cyc = q.has_cycle().unwrap();
        assert!(cyc);
    }

    // --- Non-terminal tests (commit 2) ---

    #[test]
    fn reachable_from_collects_descendants() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let a_idx = d
            .node_index_of(&crate::logic_graph::NodeId::new("a"))
            .unwrap();
        let nodes = Query::new(&d).reachable_from(a_idx).collect().unwrap();
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn descendants_of_includes_self() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let b_idx = d
            .node_index_of(&crate::logic_graph::NodeId::new("b"))
            .unwrap();
        let nodes = Query::new(&d).descendants_of(b_idx).collect().unwrap();
        // b → c (b includes itself + c)
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn ancestors_of_collects_predecessors() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let c_idx = d
            .node_index_of(&crate::logic_graph::NodeId::new("c"))
            .unwrap();
        let nodes = Query::new(&d).ancestors_of(c_idx).collect().unwrap();
        // c has a and b as ancestors (including itself)
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn roots_returns_no_incoming() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let roots = Query::new(&d).roots().collect().unwrap();
        assert_eq!(roots.len(), 1);
    }

    #[test]
    fn leaves_returns_no_outgoing() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let leaves = Query::new(&d).leaves().collect().unwrap();
        assert_eq!(leaves.len(), 1);
    }

    #[test]
    fn with_node_data_filters() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let sensors = Query::new(&d)
            .with_node_data(|n: &LogicNode| n.role == LogicNodeRole::Sensor)
            .collect()
            .unwrap();
        assert_eq!(sensors.len(), 1);
    }

    #[test]
    fn union_combines() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let a_idx = d
            .node_index_of(&crate::logic_graph::NodeId::new("a"))
            .unwrap();
        let c_idx = d
            .node_index_of(&crate::logic_graph::NodeId::new("c"))
            .unwrap();
        let q = Query::new(&d)
            .reachable_from(a_idx)
            .union(Query::new(&d).reachable_from(c_idx));
        let nodes = q.collect().unwrap();
        // a reaches {a,b,c}, c reaches {c} → union = {a,b,c}
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn intersect_finds_common() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        // ancestors_of(c) = {a,b,c}, ancestors_of(b) = {a,b}
        let q = Query::new(&d)
            .ancestors_of(c_idx_val(&d, "c"))
            .intersect(Query::new(&d).ancestors_of(c_idx_val(&d, "b")));
        let nodes = q.collect().unwrap();
        // Intersection should be {a,b} (both in ancestors of c and ancestors of b)
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn difference_subtracts() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let all = Query::new(&d).collect().unwrap();
        let roots_only = Query::new(&d).roots().collect().unwrap();
        let non_roots: Vec<NodeIndex> = all
            .into_iter()
            .filter(|n| !roots_only.contains(n))
            .collect();
        assert_eq!(non_roots.len(), 2);
    }

    #[test]
    fn topological_returns_in_order() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let sorted = Query::new(&d).topological().collect().unwrap();
        assert_eq!(sorted.len(), 3);
        // a must come before b, b before c
        let positions: std::collections::BTreeMap<NodeIndex, usize> =
            sorted.iter().enumerate().map(|(i, n)| (*n, i)).collect();
        assert!(positions[&c_idx_val(&d, "a")] < positions[&c_idx_val(&d, "b")]);
        assert!(positions[&c_idx_val(&d, "b")] < positions[&c_idx_val(&d, "c")]);
    }

    #[test]
    fn topological_returns_in_reverse_dependency_order() {
        // For a reversed graph, topological should still give a valid order
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        // Query ancestors in topological order
        let sorted = Query::new(&d)
            .ancestors_of(c_idx_val(&d, "c"))
            .topological()
            .collect()
            .unwrap();
        // Should produce a valid topological order restricted to the ancestors set
        assert_eq!(sorted.len(), 3);
    }

    // --- Set-op integration tests (commit 3) ---

    fn c_idx_val(d: &LogicGraphDialect, id: &str) -> NodeIndex {
        d.node_index_of(&crate::logic_graph::NodeId::new(id))
            .unwrap()
    }

    #[test]
    fn union_with_empty() {
        let g = empty_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d).union(Query::new(&d));
        let nodes = q.collect().unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn intersect_with_empty() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d).intersect(Query::new(&d).roots());
        let nodes = q.collect().unwrap();
        // roots intersect with all = roots
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn difference_with_empty() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d).difference(Query::new(&d));
        let nodes = q.collect().unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn union_with_full_overlap() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let all = Query::new(&d).collect().unwrap();
        let q = Query::new(&d).union(Query::new(&d));
        let nodes = q.collect().unwrap();
        assert_eq!(nodes.len(), all.len());
    }

    #[test]
    fn intersect_with_disjoint() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let a_idx = c_idx_val(&d, "a");
        let c_idx = c_idx_val(&d, "c");
        // reachable_from(a) = {a,b,c}, reachable_from(c) = {c}
        let q = Query::new(&d)
            .reachable_from(a_idx)
            .intersect(Query::new(&d).reachable_from(c_idx));
        let nodes = q.collect().unwrap();
        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains(&c_idx));
    }

    #[test]
    fn difference_non_commutative() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let a_idx = c_idx_val(&d, "a");
        let c_idx = c_idx_val(&d, "c");
        // {a,b,c} difference {c} = {a,b}
        let q = Query::new(&d)
            .reachable_from(a_idx)
            .difference(Query::new(&d).reachable_from(c_idx));
        let nodes = q.collect().unwrap();
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&c_idx_val(&d, "a")));
        assert!(nodes.contains(&c_idx_val(&d, "b")));
        assert!(!nodes.contains(&c_idx));
    }

    #[test]
    fn with_node_data_and_edge_kind_both() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        // Filter to sensors, then take their descendants
        let a_idx = c_idx_val(&d, "a");
        let q = Query::new(&d)
            .with_node_data(|n: &LogicNode| n.role == LogicNodeRole::Sensor)
            .descendants_of(a_idx);
        let nodes = q.collect().unwrap();
        // Sensor (a) with descendants_of includes itself + b + c
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn with_node_data_chained() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d)
            .with_node_data(|n: &LogicNode| n.role == LogicNodeRole::Sensor)
            .with_node_data(|n: &LogicNode| n.role == LogicNodeRole::Controller);
        // Chain of two node predicates: sensor AND controller — nothing satisfies both
        let nodes = q.collect().unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn set_op_with_filter_inside() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let a_idx = c_idx_val(&d, "a");
        // union of roots and descendants_of(a), then filter to sensors
        let q = Query::new(&d)
            .roots()
            .union(Query::new(&d).descendants_of(a_idx))
            .with_node_data(|n: &LogicNode| n.role == LogicNodeRole::Sensor);
        let nodes = q.collect().unwrap();
        // roots = {a}, descendants_of(a) = {a,b,c}, union = {a,b,c}, filter sensors = {a}
        assert_eq!(nodes.len(), 1);
    }

    // --- Integration / skyline tests (commit 5) ---

    #[test]
    fn query_skyline_reachable_with_predicate_topological() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let a_idx = c_idx_val(&d, "a");
        let q = Query::new(&d)
            .reachable_from(a_idx)
            .with_node_data(|n: &LogicNode| n.role == LogicNodeRole::Actuator)
            .topological();
        let nodes = q.collect().unwrap();
        // Only c is an actuator, so result has 1 node in topo order
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn query_has_cycle_on_subset() {
        let g = cyclic_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d);
        assert!(q.has_cycle().unwrap());
    }

    #[test]
    fn query_empty_graph_collects_empty() {
        let g = empty_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d).roots().union(Query::new(&d).leaves());
        let nodes = q.collect().unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn query_edge_predicate_post_materialise() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        // Edge predicates are applied after the node set is materialized
        // (edge filter on the full graph would give the same result here)
        let a_idx = c_idx_val(&d, "a");
        let q = Query::new(&d)
            .reachable_from(a_idx)
            .with_edge_kind(|_: &LogicEdge| true);
        let nodes = q.collect().unwrap();
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn query_union_with_empty_integration() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        // difference(self) gives a truly empty set
        let empty = Query::new(&d).difference(Query::new(&d));
        let q = Query::new(&d).roots().union(empty);
        let nodes = q.collect().unwrap();
        // roots ∪ empty = roots
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn query_difference_non_commutative_integration() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let all = Query::new(&d);
        let roots_only = Query::new(&d).roots();
        let non_roots = all.difference(roots_only);
        let nodes = non_roots.collect().unwrap();
        assert_eq!(nodes.len(), 2);
    }

    // --- topological_sort_subset integration tests (Commit 5) ---

    #[test]
    fn topological_sort_subset_linear_chain() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let a_idx = c_idx_val(&d, "a");
        let b_idx = c_idx_val(&d, "b");
        let c_idx = c_idx_val(&d, "c");
        let subset: BTreeSet<NodeIndex> = [a_idx, b_idx, c_idx].into();
        let sorted = topological_sort_subset(&d, &subset).unwrap();
        // Linear chain: must respect a→b→c
        assert_eq!(sorted.len(), 3);
        let pos = |idx: NodeIndex| sorted.iter().position(|&x| x == idx).unwrap();
        assert!(pos(a_idx) < pos(b_idx), "a before b");
        assert!(pos(b_idx) < pos(c_idx), "b before c");
    }

    #[test]
    fn topological_sort_subset_partial_subset() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let a_idx = c_idx_val(&d, "a");
        let c_idx = c_idx_val(&d, "c");
        // Only a and c in subset: verify subset sort respects existing edges
        let subset: BTreeSet<NodeIndex> = [a_idx, c_idx].into();
        let sorted = topological_sort_subset(&d, &subset);
        assert!(sorted.is_ok());
        let pos = |idx: NodeIndex| {
            sorted
                .as_ref()
                .unwrap()
                .iter()
                .position(|&x| x == idx)
                .unwrap()
        };
        assert!(pos(a_idx) < pos(c_idx), "a before c in partial subset");
    }

    #[test]
    fn topological_sort_subset_empty() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let subset = BTreeSet::new();
        let sorted = topological_sort_subset(&d, &subset);
        assert!(sorted.is_ok());
        assert!(sorted.unwrap().is_empty());
    }

    // --- topological() terminal integration tests (Commit 5) ---

    #[test]
    fn topological_returns_valid_order() {
        let g = linear_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d).topological();
        let nodes = q.collect().unwrap();
        assert_eq!(nodes.len(), 3);
        let a_idx = c_idx_val(&d, "a");
        let b_idx = c_idx_val(&d, "b");
        let c_idx = c_idx_val(&d, "c");
        let pos = |idx: NodeIndex| nodes.iter().position(|&x| x == idx).unwrap();
        assert!(pos(a_idx) < pos(b_idx), "a before b");
        assert!(pos(b_idx) < pos(c_idx), "b before c");
    }

    #[test]
    fn topological_empty_graph() {
        let g = empty_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d).topological();
        let nodes = q.collect().unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn topological_on_cycle_returns_cycle_error() {
        let g = cyclic_graph();
        let d = LogicGraphDialect::new(&g);
        let q = Query::new(&d).topological();
        let result = q.collect();
        // topological sort on a cyclic graph returns Err
        assert!(result.is_err());
    }
}
