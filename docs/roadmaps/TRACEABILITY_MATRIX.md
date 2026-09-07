# Traceability Matrix — Existing Roadmaps to Converged Program

| Existing intent | Existing source | Converged destination | Decision |
|---|---|---|---|
| CI + architecture fitness | v0.87 / application stabilization | H0 | Keep + strengthen |
| editor-model extraction | v0.87 | H1/H2 cleanup | Keep direction, finish boundaries |
| EditorSession migration | v0.87 / architecture docs | H2 | Keep, converge globals |
| ProjectStore port | v0.87 | H1/H9 | Keep, invert dependency + fault tests |
| TransactionKernel | v0.87/v0.89 | H4 | Keep, make sole path |
| typed backend | v0.87/application stabilization | H3 | Keep, elevate priority |
| hierarchy performance | v0.88 | H7 | Keep + measurable 10k budget |
| direct manipulation/world/recipes | v0.88 | Current feature hardening | Preserve; no broad new scope |
| ChangeWorkbench | v0.89 | H4 + existing feature hardening | Preserve |
| runtime causality/apply-back | v0.89 | H4/H7 feature hardening | Preserve |
| agent runtime | v0.90 | Post-v1 | Defer |
| semantic agent retrieval | v0.91 | Post-v1 | Defer |
| extension SDK/importers | v0.92/v0.93 | H4/H9 contract hardening | Preserve delivered functionality |
| shell integrity | UI overhaul | H6/H7 regression requirements | Mostly delivered; retain unresolved UX |
| hierarchy/inspector v2 | UI overhaul | H7 v3 hardening | Keep + performance/a11y |
| validation/search/runtime/logic UX | UI overhaul | H7 feature strengthening | Keep, consolidate |
| AI panel autonomy | AI-native roadmap | Post-v1 | Defer expansion; preserve safe proposal UX |
| v1 data/perf/a11y/release gates | v1 stabilization | H9/H10 | Keep + make evidence explicit |

## Supersession rule

When status conflicts:

1. measured implementation/evidence;
2. `docs/roadmaps/MASTER_ROADMAP.md`;
3. active hardening roadmap;
4. durable specs/ADRs;
5. historical roadmaps.

Historical docs remain useful for why a capability exists, but they do not reorder the active v1 plan.

