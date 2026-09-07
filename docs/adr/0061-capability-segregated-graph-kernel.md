# ADR-0061 — Capability-Segregated GraphKernel

Status: Proposed pending benchmark spike

## Context

GraphKernel is a strong reusable abstraction, but the current mutation trait can require dialects to implement operations that are not semantically supported. Traversal contracts may also impose allocations/scans.

## Decision

Preserve the graph-kernel/dialect architecture but allow mutation capabilities to be segregated when evidence supports it.

Candidate capability split:

```text
GraphRead
GraphTopologyMut
GraphNodeDataMut
GraphEdgeDataMut
```

Adjacency/index representation and iterator strategy are selected by benchmark, not aesthetics.

## Invariant

Dialect-specific stable IDs remain outside algorithm assumptions, and deterministic ordering remains part of the contract.

## Rejected

- replace GraphKernel with a third-party graph crate solely to reduce code;
- prematurely optimize without real corpus evidence.

