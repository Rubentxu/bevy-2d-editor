# UAT — Performance & Accessibility

## Reference environment

Record for every benchmark run:

- CPU;
- RAM;
- browser version;
- OS;
- build profile;
- commit SHA.

CI numbers and developer workstation numbers must not be mixed without labeling.

## UAT-PERF-001 — 10k hierarchy load

1. Load generated 10,000-entity scene with representative hierarchy depth.
2. Measure index build and first useful hierarchy render.

Initial targets:

- index p95 < 100 ms;
- no unbounded main-thread stall;
- memory remains within documented budget.

Thresholds may be calibrated after baseline checkpoint.

## UAT-PERF-002 — 10k search

1. Search common prefix.
2. Search unique deep leaf.
3. Clear search.

Expected initial target: interaction update p95 < 50 ms after indexes exist.

## UAT-PERF-003 — Selection/Inspector latency

On 10k scene, select entities at top/middle/bottom, including multi-select.

Expected initial target: visible inspector response p95 < 100 ms.

## UAT-PERF-004 — Large LogicGraph

Run query/traversal/topological/cycle operations on benchmark corpus.

Expected: versioned budgets; no accidental O(V×E) regression without review.

## UAT-PERF-005 — Project hydration

Measure cold start/hydration on project corpus with large asset catalog and resources metadata.

Expected: readiness contract fires only after required state is usable; latency budget is tracked release-to-release.

## UAT-A11Y-001 — Keyboard critical project flow

Keyboard-only user can:

- create/select/rename/delete entity;
- navigate hierarchy;
- edit inspector field;
- save;
- open command palette;
- reach validation problem;
- start/stop play.

Mouse-only drag features must have an alternate command where they are critical to workflow.

## UAT-A11Y-002 — Screen reader semantics smoke

Verify primary tree, dialogs, tabs, menu/palette and form fields expose appropriate roles/names/states.

## UAT-A11Y-003 — Focus recovery

After dialogs, delete, document switch and validation navigation, focus returns/moves to a predictable useful target.

## UAT-A11Y-004 — Contrast/focus tokens

Touched critical surfaces pass agreed contrast and visible-focus requirements for both supported themes.

