# SPEC-AUTHOR-001 — Scene Asset Variants and Provenance

**Status:** Proposed  
**ADR:** 0064

## Model

```text
SceneAsset Base -> VariantOf -> SceneAsset Variant -> InstanceOf -> Instance -> local overrides
```

A variant is reusable and has explicit lineage.

## Invariants

- lineage acyclic;
- stable IDs preserved;
- base changes never silently erase overrides;
- effective resolution deterministic;
- provenance queryable for every authorable field;
- stale/conflict/orphaned overrides explicit.

## Effective value result

```text
value
source_subject
source_kind
override_state
revision
```

Source kinds: BaseDefinition, VariantDefinition, InstanceOverride, RuntimeTemporary, AppliedBack, Imported.

Override states: Inherited, Active, Stale, Conflict, Orphaned.

## UX actions

- revert local;
- apply to variant;
- apply to base with impact review;
- compare lineage;
- show dependents.

## ChangeSet

Promotion/revert operations are cross-resource ChangeSets where necessary.

## UAT

Create base -> variant -> instances -> override -> base change -> provenance/conflict -> resolve -> save/reload -> undo/redo.
