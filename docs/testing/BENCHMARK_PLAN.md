# Benchmark Plan — v1.0

## Philosophy

Benchmarks protect workflows and asymptotic behavior, not synthetic vanity numbers.

## Corpora

### C1 Scene hierarchy

- 100 entities;
- 1,000 entities;
- 10,000 entities;
- shallow wide;
- deep chain;
- representative mixed tree.

### C2 Assets

- 100 / 1,000 / 10,000 catalog entries metadata-only;
- representative bodies/resources.

### C3 Logic graphs

- chain;
- branching DAG;
- cyclic graph;
- dense graph;
- real gameplay fixture.

### C4 Levels

- large tile grid;
- multiple layers;
- auto layer regeneration.

### C5 Project

- multi-level canonical game;
- large project hydration/search fixture.

## Metrics

Rust:

- wall time;
- allocations where practical;
- graph query complexity trend;
- serialization time;
- import/reimport time.

Frontend/browser:

- readiness time;
- hierarchy index time;
- first useful render;
- search latency;
- selection→inspector latency;
- long tasks;
- DOM row count if virtualized;
- JS/WASM bundle sizes.

## Budget management

Create versioned JSON/Markdown budget file in implementation phase. Never increase threshold in same commit as a regression without explicit rationale.

## Execution cadence

- micro/critical benchmarks per PR where cheap;
- full benchmark corpus nightly or on performance-sensitive PRs;
- all v1 budgets on release candidate.

## Regression triage

A regression triggers classification:

1. algorithmic;
2. data structure;
3. build/bundle;
4. browser rendering;
5. benchmark noise/environment;
6. intentional tradeoff.

Only #6 may update budget and requires documented decision.

