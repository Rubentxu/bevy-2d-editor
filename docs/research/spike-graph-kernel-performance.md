# Spike R-05 — GraphKernel Performance and Contracts

## Baseline

Benchmark current dialects and algorithms before changing APIs.

## Hypotheses

H1: repeated full edge scans dominate graph queries.  
H2: boxed iterators are measurable but secondary.  
H3: current simplicity is sufficient for actual v1 graph sizes.

All three are valid possible conclusions.

## Implement reference benchmark cases

- roots/leaves;
- descendants/ancestors;
- topological sort;
- cycle check on edge insertion;
- node remove/reindex;
- query chains.

## Candidate experiments

A. transient adjacency map at dialect construction.  
B. GAT-based iterator API.  
C. persistent indexes in mutable dialect.  
D. no change.

## Correctness guard

Property tests compare candidate output to frozen reference implementation.

## Output

Select minimum implementation that satisfies budget and preserves API clarity.

