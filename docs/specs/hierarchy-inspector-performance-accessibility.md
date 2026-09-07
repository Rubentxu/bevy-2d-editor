# Specification — Hierarchy & Inspector v3 Hardening

## Goals

- scale to large scenes;
- behave as professional editor controls;
- preserve context during search;
- support keyboard users;
- reduce visual action noise;
- connect validation and override workflows to the fields/entities they affect.

## Hierarchy data model

Build once per scene revision:

```text
entityById
childrenByParent
parentById
depthById
visibleFlattenedTree
```

Normal render must not perform repeated linear parent lookup for every row.

## Search

Search is tree-aware:

- matching nodes remain visible;
- ancestors remain visible as context;
- optional descendants can be shown according to filter mode;
- matched text is highlighted;
- clearing search restores expansion state.

## Tree interaction

Required behavior:

- expand/collapse;
- multi-select;
- range select;
- inline rename;
- drag/drop reparent;
- keyboard reparent alternative or command-palette action;
- contextual actions;
- stable selection after refresh where entity still exists.

## Accessibility

Implement WAI-ARIA tree semantics where applicable:

- `role=tree`;
- `role=treeitem`;
- `aria-level`;
- `aria-expanded`;
- roving tab index;
- arrows navigate/collapse/expand;
- Enter/Space activates/selects;
- F2 rename;
- Delete invokes delete flow;
- visible focus indicator.

## Virtualization

Required when benchmark evidence shows DOM/render budget violation. Choice of library is a spike outcome.

Virtualization must preserve:

- keyboard navigation;
- drag/drop;
- scroll-to-selection;
- row measurement stability.

## Visual semantics

Rows must prioritize:

1. entity name/type;
2. warning/error state;
3. instance/logic/override metadata;
4. actions on hover/context menu.

Avoid persistent text buttons such as `Open Logic` on every row when an icon/context action is sufficient.

## Inspector

Keep and strengthen:

- multi-edit;
- Mixed values;
- schema-driven field editors;
- instance/override metadata;
- runtime diagnostics;
- logic binding.

Add progressively:

- component search/filter;
- reset/revert field;
- copy/paste component;
- per-field validation message/quick fix;
- persistent section collapse;
- vector sub-field mixed state;
- provenance indicator for overridden/runtime-applied values.

## Performance budgets

Budgets are finalized after baseline spike. Initial acceptance targets on reference CI/browser hardware:

- 10k entity data index build: p95 < 100 ms;
- hierarchy search update: p95 < 50 ms after index exists;
- selection-to-inspector visible update: p95 < 100 ms;
- scroll remains subjectively continuous and automated long-task count stays bounded;
- no 10k-row DOM requirement if virtualization is enabled.

Exact thresholds may be adjusted through a recorded checkpoint using baseline evidence, never silently.

