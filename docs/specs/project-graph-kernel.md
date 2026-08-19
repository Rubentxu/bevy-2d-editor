# SPEC-GRAPH-001 — Project Graph Kernel and Runtime

**Status:** Proposed  
**ADRs:** 0055, 0060

## Goals

Provide reusable, typed, efficient graph infrastructure for project relationships and runtime invalidation.

## Core API

Illustrative:

```rust
pub trait GraphDialect {
    type NodeKind;
    type EdgeKind;
    fn validate(&self, graph: &GraphDocument<Self>) -> ValidationReport;
}

pub struct GraphDocument<D: GraphDialect> {
    pub id: GraphId,
    pub revision: GraphRevision,
    pub nodes: Vec<GraphNode<D>>,
    pub edges: Vec<GraphEdge<D>>,
}
```

## Required algorithms

- deterministic adjacency/reverse adjacency;
- reachability;
- shortest explanatory path;
- transitive dependency closure;
- cycle detection;
- topological sort for DAG dialects;
- strongly connected components for diagnostics;
- subgraph extraction;
- incremental diff application.

## Initial edge taxonomy

```text
Contains
InstanceOf
VariantOf
UsesAsset
UsesSource
LogicBinding
ReferencesEntity
DependsOn
ProducedFrom
OverrideOf
ProjectedFrom
ValidatedBy
WorldLink
```

Avoid a generic free-form string edge model for core relations.

## Materialization

Project graph is rebuildable. It may be fully rebuilt, incrementally updated by GraphDiff, or optionally cached. It is never sole truth.

## Runtime indexes

```text
StableId -> NodeSlot
NodeSlot -> adjacency range
NodeSlot -> reverse adjacency range
```

Prefer dense structures for hot traversal.

## Query API

```text
dependencies(subject, depth?)
dependents(subject, depth?)
impact(change_set)
path(from, to)
instances_of(scene_asset)
variant_lineage(scene_asset)
runtime_projection(stable_id)
logic_affecting(stable_id)
unused_assets()
```

## Performance corpus

Benchmark 1k/5k, 10k/50k and algorithm-only 100k/500k node/edge classes. Establish empirical budgets before enforcing limits.

## Properties

- deterministic output;
- incremental result equals rebuild;
- reverse edge consistency;
- dialect-specific cycle handling;
- no dangling references after validated mutations.
