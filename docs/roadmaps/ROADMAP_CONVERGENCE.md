# Roadmap Convergence — Pre-v1.0

## Why converge

The repository accumulated several roadmap programs as capabilities were delivered:

- architecture foundation;
- production authoring;
- change/runtime workbench;
- agent runtime;
- semantic agents;
- ecosystem/importers;
- application stabilization;
- UI workflow overhaul;
- AI-native editor;
- v1 stabilization.

Most older documents are valuable history but should no longer compete as active execution plans.

## Active execution sources

1. `docs/roadmaps/MASTER_ROADMAP.md` — canonical ordering and product gates.
2. `docs/roadmaps/v1.0-architecture-ux-hardening.md` — active engineering program.
3. `docs/roadmaps/v1.0-stabilization.md` — release/product proof after hardening slices land.
4. Specs/ADRs/UAT referenced by those roadmaps.

## Historical/reference status

| Roadmap | New status | What survives |
|---|---|---|
| v0.87 architecture foundation | Historical | principles, delivered extraction history |
| v0.88 production authoring | Historical | capability/product rationale |
| v0.89 change/runtime | Historical | ChangeWorkbench/runtime causality intent |
| v0.90 agent runtime | Parked post-v1 | only requirements compatible with typed capabilities/ChangeSet |
| v0.91 semantic agents | Parked post-v1 | retrieval/runtime-aware direction |
| v0.92 ecosystem/importers | Delivered historical | compatibility and provenance requirements |
| application stabilization | Superseded active plan | release-health lessons, deterministic test cohorts |
| UI workflow overhaul | Superseded active plan | hierarchy/inspector/search/runtime UX requirements not fully hardened |
| AI-native editor | Parked post-v1 | product vision, typed operations, reviewability |

## What is intentionally deferred

Until v1 product gates pass:

- new autonomous agent runtime;
- background agent jobs;
- new large editor modes not required by canonical game proof;
- marketplace;
- collaborative editing;
- generic 3D parity;
- speculative framework work.

## Pull-forward exception

A deferred item may be pulled forward only when it directly blocks a v1 gate. The change must state:

- blocking evidence;
- smallest required scope;
- why an existing capability cannot solve it;
- UAT added to the v1 gate.

