# Specification — GraphKernel Hardening

## Goal

Preserve the dialect-agnostic graph substrate while improving contract precision, asymptotic behavior and mutation capability segregation.

## Invariants

- deterministic output order;
- dialect-specific stable IDs never leak as assumptions into algorithms;
- algorithms do not depend on Bevy/browser;
- DAG validation rejects cycles, self-loops and duplicates according to dialect policy;
- cyclic logic graphs preserve allowed feedback semantics.

## Problems to investigate

### G1 Edge traversal complexity

Current dialect implementations may scan all edges for each `incoming`/`outgoing` call. Measure actual algorithms on representative graph sizes.

Candidate solutions:

- adjacency indices built at dialect binding time;
- persistent adjacency in domain graph model;
- compact CSR-like transient index for query-heavy workloads.

### G2 Boxed iterators

Measure allocations/dynamic dispatch caused by boxed iterator returns.

Candidate:

- GAT iterator associated types;
- slice-based adjacency references;
- visitor callback only if it improves ergonomics/perf.

Do not adopt GATs purely for stylistic reasons.

### G3 Mutation ISP

Split `GraphMut` if dialects are forced to fake unsupported operations.

Candidate traits:

```text
GraphRead
GraphTopologyMut
GraphNodeDataMut
GraphEdgeDataMut
```

### G4 Edge endpoint duplication

Ensure there is one canonical source of topology. Do not accept contradictory `src/dst` parameters and endpoint-bearing edge data without validation.

### G5 Correctness properties

Property-based tests should verify:

- topo sort order validity;
- cycle insertion rejection for DAG;
- remove-node removes incident edges according to dialect contract;
- index rebuild remains consistent;
- traversals equal a simple reference implementation.

## Benchmarks

At minimum:

- chains;
- wide trees;
- dense DAGs;
- cyclic logic graphs;
- real project fixture graphs.

Record allocations where feasible.

## Decision outcome

The spike may conclude that the current simple implementation is sufficient for v1.0. That is a valid result if budgets pass.

