# ADR-0053: Graph Kernel — A Pure-Rust Dialect-Agnostic Substrate

## Status

Proposed — 2026-08-20 — v0.87+ (M1 backlog: GRAPH-001)

## Context

The editor model already carries several graph-shaped data structures. Each is implemented independently today, with its own ID types, its own traversal helpers, and its own dialect-specific validation:

| Concrete graph | ID type | Edge type | Lives in |
|---|---|---|---|
| `LogicGraphAsset` | `NodeId` + `PortId` | `LogicEdge` (typed by port) | `editor-model/src/logic_graph.rs` |
| `SceneAssetEntity` hierarchy + `SceneAssetRelationship` | `SceneAssetLocalId` | `SceneAssetRelationship` (typed by `RelationshipKind`) | `editor-model/src/scene_asset.rs` |
| `ChangeSet<O>` | `u64` (op index) | (none today — implicit linear order) | `editor-model/src/transaction.rs` |
| `ComponentInstance` overrides | `StableId` + `ComponentTypeId` | `ComponentOverride` (keyed by component + field path) | `editor-bevy/src/scene_instance_overrides.rs` |
| `WorldDocument` | `WorldInstanceId` | (planned, ADR-0037) | `editor-model/src/world_document.rs` |

Without a kernel, every consumer re-implements:

- **Roots / leaves** — find nodes with no incoming edges (`rebuild_preview_world`, `process_play_mode_request`).
- **Descendants / ancestors** — walk a node's transitive closure (`instance_projection`, `validate_overrides`).
- **Topological sort** — order ops for apply-back (`history_scopes`, `apply_atomic`).
- **Cycle detection** — guard the editor against cycles in overrides, scene hierarchies, and logic graphs.
- **Reachability** — which nodes are reachable from a root.

These are graph-algorithm primitives. Each dialect today ships a bespoke, often partial, implementation. The result is:

1. **Divergent semantics**: top-sort in `apply_atomic` treats cycle as error; `validate_overrides` silently ignores cycles. Two graph algorithms in the same crate disagree.
2. **Cost of new dialects**: every new graph-shaped domain (OverrideGraph, WorldGraph, ExternalSourceGraph per ADR-0041) re-invents the wheel.
3. **Test asymmetry**: the kernel has no testable surface; each dialect algorithm has its own ad-hoc test.
4. **Forward-compat risk**: ADR-0046 mandates deterministic orderings; without a kernel, ordering is implicit per dialect.

The M1 backlog entry `GRAPH-001 | graph kernel IDs/types/dialects | ARCH-020 | pure Rust tests` names this without prescribing a shape. This ADR picks the shape.

## Decision

Introduce a pure-Rust graph kernel in `editor-model/src/graph_kernel.rs` with three concerns kept strictly separate:

### 1. Opaque node and edge IDs (kernel-owned)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeIndex(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EdgeIndex(pub u32);
```

The kernel assigns `NodeIndex` / `EdgeIndex` at dialect-binding time (zero-based, stable for the lifetime of a `GraphView`). Dialects keep their own ID types (`NodeId`, `SceneAssetLocalId`, etc.) and translation happens via the `Graph` trait.

### 2. `trait Graph` — the dialect contract

A free-standing trait (no lifetimes, no `&self` aliasing) that any dialect can implement. The kernel reads the graph through this trait; dialects map their data into `NodeIndex` / `EdgeIndex` on demand.

```rust
pub trait Graph {
    type NodeData: Clone;
    type EdgeData: Clone;
    type Error: std::error::Error;

    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
    fn node(&self, idx: NodeIndex) -> Option<&Self::NodeData>;
    fn edge(&self, idx: EdgeIndex) -> Option<&Self::EdgeData>;
    fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)>;
    fn outgoing(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_>;
    fn incoming(&self, node: NodeIndex) -> Box<dyn Iterator<Item = EdgeIndex> + '_>;
}
```

### 3. Kernel operations — pure functions over `&dyn Graph`

```rust
pub fn roots<G: Graph + ?Sized>(g: &G) -> Vec<NodeIndex>;
pub fn leaves<G: Graph + ?Sized>(g: &G) -> Vec<NodeIndex>;
pub fn descendants<G: Graph + ?Sized>(g: &G, root: NodeIndex) -> Vec<NodeIndex>;
pub fn ancestors<G: Graph + ?Sized>(g: &G, leaf: NodeIndex) -> Vec<NodeIndex>;
pub fn topological_sort<G: Graph + ?Sized>(g: &G) -> Result<Vec<NodeIndex>, GraphKernelError>;
pub fn has_cycle<G: Graph + ?Sized>(g: &G) -> bool;
pub fn reachable_from<G: Graph + ?Sized>(g: &G, root: NodeIndex) -> Vec<NodeIndex>;
```

All kernel operations are pure (no `&mut self`, no side effects). All return owned vectors — no iterator lifetimes leak across the dialect boundary.

### 4. Dialect — the binding layer

A `Dialect` is a struct that holds a reference to the dialect-specific data plus the node/edge index translation tables. It implements `Graph` by mapping `NodeIndex` → `&NodeData` and `EdgeIndex` → `&EdgeData` and `Endpoints`.

```rust
pub struct LogicGraphDialect<'a> {
    asset: &'a LogicGraphAsset,
    node_index: BTreeMap<&'a NodeId, NodeIndex>,
    edge_index: BTreeMap<(NodeId, PortId, NodeId, PortId), EdgeIndex>,
}

impl<'a> Graph for LogicGraphDialect<'a> { /* ... */ }
```

The dialect is an adapter — it lives in `editor-model` (no Bevy, no WASM) and is side-effect-free. It is constructed on demand by the kernel consumer.

### 5. Initial dialect set (the seed)

Four dialect implementations, in order of priority:

1. **`LogicGraphDialect`** — `LogicGraphAsset` + `LogicEdge`. Replaces the bespoke traversal in `LogicGraphCatalog::find_reachable`, `validate_logic_graph`, and the runtime dispatch.
2. **`SceneAssetDialect`** — `SceneAssetEntity` + `SceneAssetRelationship`. Replaces the bespoke DFS in `instance_projection` and the validation in `validate_overrides`.
3. **`ChangeSetDialect`** — `ChangeSet<O>` exposed as a DAG (a future-proofing move: when op dependencies are added, the kernel handles the cycle detection; today, v1 of this dialect is a no-op wrapper).
4. **`OverrideGraphDialect`** — `ComponentOverride` keyed by `(StableId, ComponentTypeId, FieldPath)`. Replaces the bespoke cycle-detection in `validate_overrides`.

Each dialect ships with a `#[cfg(test)] module` of pure-Rust tests that build a small synthetic graph and verify the kernel operations against the expected output.

### 6. Failure-tolerance contract

The kernel never panics. When a dialect returns `None` for a node or edge that an algorithm is asking about, the algorithm treats it as "not present" and stops walking that branch. This is forgiving but explicit: dialect implementations are responsible for the `Option` translation; the kernel trusts the dialect.

### 7. Out of scope for this ADR

- **Persistent graph storage** (BSN / RON / JSON). The kernel is in-memory; dialects get their data from the existing `AssetDocument` types.
- **Graph mutation**. The kernel is read-only. Mutation is the dialect's responsibility; the canonical pattern is "dialect rebuilds the index map when the underlying doc changes".
- **GraphQL/SPARQL/Cypher queries**. The kernel is the substrate those queries would compile to; the query language itself is out of scope (handled by GRAPH-010 or later).
- **Performance optimisation** (e.g. CSR/CSC adjacency storage). The kernel uses `BTreeMap` lookups today; if a profiler flags a hot path, swap in a pre-indexed adjacency list.

## Considered options

### A. Adopt `petgraph` directly
Rejected: `petgraph` is a full-featured graph library, but it forces dialects to rebuild their data into `Graph<N, E>` (lossy translation) and hides the dialect semantics behind generic nodes. The kernel we want is the dialect boundary, not the graph storage.

### B. Make `LogicGraphAsset` itself generic over a kernel-supplied trait
Rejected: backwards-incompatible. Existing `LogicGraphAsset` callers would all need to dispatch through the trait, and the change is large for the kernel's first seed.

### C. Define a dialect trait only (no kernel operations)
Rejected: the goal is shared semantics. Without a kernel, dialects still disagree on cycle handling, root definition, etc. The dialect trait alone is just a uniform interface; the kernel is what makes the algorithms shared.

### D. Pick one dialect first, ship it, then add the kernel
Considered, rejected for v1. The risk of "we shipped LogicGraphDialect and now its API is whatever was convenient for LogicGraph" is real. The kernel trait is generic from line 1.

## Consequences

- **`editor-model` stays pure Rust** (no Bevy, no WASM), per ADR-0030. The kernel is `&'static`-bound, no `Rc`/`Arc` needed.
- **Existing algorithms become one-liners**:
  - `apply_atomic`'s "no cycles in op deps" check → `if has_cycle(&dialect) { return Err(…) }`.
  - `validate_overrides`'s "is X reachable from Y" → `descendants(&dialect, x).contains(&y)`.
  - `LogicGraphCatalog::find_reachable` → `reachable_from(&dialect, root)`.
- **Deterministic orderings** (ADR-0045) become a kernel property: `topological_sort` returns `NodeIndex` in `BTreeMap` iteration order, which is total and reproducible.
- **Test surface grows**: each dialect ships 4-6 unit tests. Total: ~24 new tests in `editor-model`.
- **Risk R1**: the kernel is a thin layer over generic algorithms. If a dialect adds a new traversal pattern that the kernel doesn't expose, the dialect ends up with a side algorithm — the temptation is real. **Mitigation**: a trait extension point (`Graph::custom_query`) is intentionally NOT provided. New operations go to the kernel.
- **Risk R2**: `Box<dyn Iterator>` in the trait adds a small allocation overhead. **Mitigation**: profiled in `editor-bevy` later; if hot, swap to a `Vec<EdgeIndex>` return (one allocation per call, but no `Box`).

## References

- ADR-0005 — Scene Asset: BSN-aligned reusable scene model
- ADR-0009 — Component Override: ECS/BSN replacement for override patch
- ADR-0030 — Compile-Time Hexagonal Crate Boundaries
- ADR-0037 — World Workspace is a first-class product context
- ADR-0041 — External Authoring Sources use Provenance-Aware Import/Reimport Pipelines
- ADR-0046 — Semantic Editor Model is the Authoritative Source of Truth
- v0.87 cycle spec — `cycles/v0.87-architecture-foundation/`
- M1 backlog — `BEVY_NATIVE_IMPLEMENTATION_BACKLOG.md` row 25 (GRAPH-001)
