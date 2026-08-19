# SPEC-AUTHOR-003 — AutoLayer Rule Graph

**Status:** Proposed  
**Depends on:** SPEC-GRAPH-001

## Goal

Evolve AutoLayer into a graph-based rule system without making simple painting unnecessarily complex.

## Rule concepts

```text
NeighborhoodInput
Condition
Terrain/TagMatch
Random/WeightedChoice
VariantSelector
OutputTile
DecorationSpawn
```

## Moldable authoring

Simple users see presets/forms. Advanced users can open the same rule as a graph.

## Runtime

Compile rules to dense lookup/evaluation structures; do not naïvely evaluate the visual graph for every tile.

## Determinism

Weighted/random rules use explicit project/paint seed where deterministic output is required.

## Diagnostics

Unreachable rule, conflicting output, missing tile, illegal recursion/dependency, excessive evaluation cost.

## UAT

Create terrain preset -> inspect graph -> paint -> undo -> repaint same seed -> validate deterministic result -> save/reload.
