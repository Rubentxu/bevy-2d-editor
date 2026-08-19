# Risk Register

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---:|---:|---|
| R-01 | Bevy API churn causes editor-wide migration | M | H | model/application Bevy-free; runtime isolation |
| R-02 | EditorWorld becomes service-locator spaghetti | M | H | bounded resources/system ownership/fitness checks |
| R-03 | semantic/ECS duplication creates stale projections | M | H | revisions, rebuild tests, mapping invariants |
| R-04 | Graph abstraction becomes too generic | M | H | dialect schemas; only needed algorithms |
| R-05 | Graph visualization unusable at scale | H | M | moldable views/filtering/non-node-link views |
| R-06 | Logic compiler complexity delays features | M | M | staged compiler; benchmark early |
| R-07 | WASM size/performance regression | M | H | narrow Bevy features, budgets |
| R-08 | instrumentation degrades runtime | M | M | bounded/sampled modes |
| R-09 | protocol dual-path debt persists | H | M | removal tasks per capability |
| R-10 | UI reorg destabilizes users/tests | M | M | migrate behaviour first; UAT |
| R-11 | UAT becomes bureaucracy | M | M | shared schema, automation, critical subset |
| R-12 | variants become confusing | M | H | provenance/conflict UX, limited rules |
| R-13 | AI gets raw internal runtime access | M | H | typed capabilities only |
| R-14 | giant refactor blocks releases | M | H | strangler milestones; no big bang |
