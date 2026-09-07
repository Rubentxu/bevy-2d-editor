# Spike R-04 — Large Scene Hierarchy

## Question

What minimum data/index/render strategy is needed for a responsive 10k-entity hierarchy?

## Baseline first

Measure current implementation on generated:

- flat 10k;
- balanced tree 10k;
- depth-1000 chain;
- representative scene.

Measure:

- render duration;
- search;
- select;
- reparent;
- expansion;
- memory/DOM nodes;
- long tasks.

## Step 1 hypothesis — indexing may be enough

Implement memoized:

- byId;
- children;
- depth/flattened visible tree.

Re-run benchmarks.

## Step 2 hypothesis — virtualization only if needed

Compare candidate virtualizers for:

- variable/constant row height;
- keyboard tree navigation;
- drag/drop;
- scroll-to-selected;
- testability;
- bundle size.

## Output

A decision matrix plus chosen budget. `No virtualization for v1` is valid if indexed implementation passes.

