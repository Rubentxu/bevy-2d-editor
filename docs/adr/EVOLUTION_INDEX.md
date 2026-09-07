# ADR Evolution Index — Approved Addendum

Merge these entries into the repository's existing `docs/adr/README.md`; this file intentionally has a different name to avoid overwriting the historical index by accident.

| Number | Title | Status | Relationship |
|---|---|---|---|
| ADR-0030 | Compile-Time Hexagonal Crate Boundaries | Accepted (v0.94.0) | **Superseded by ADR-0056** (enforcement layer added) |
| ADR-0031 | Explicit EditorSession Replaces Domain-Level Global State | Accepted | **Superseded by ADR-0057** (composition root ownership) |
| ADR-0032 | Shared Transaction Kernel and ChangeSet, with Domain-Specific Commands | Accepted | Evolves shared mechanics; preserves ADR-0007 domain split; **extended by ADR-0059** |
| ADR-0033 | ProjectStore Port with OPFS and Filesystem Adapters | Accepted | Amends ADR-0008 |
| ADR-0034 | Typed EditorBackend Contract Replaces Global Window Bridge | Accepted | **Superseded by ADR-0058** (capability split + fitness rule + codegen spike) |
| ADR-0035 | Clock and IdGenerator Are Explicit Application Ports | Accepted | New |
| ADR-0036 | Bevy Runtime Preview Is an Ephemeral Projection Adapter | Accepted | Reinforces editor-owned state direction |
| ADR-0037 | World Workspace Is a First-Class Product Context | Accepted | Extends level workflow |
| ADR-0038 | Workflow and Gameplay Recipes Compile Intent into Typed Changes | Accepted | Extends Logic Bricks/application workflows |
| ADR-0039 | Change Workbench Is the Unified Review and Approval Surface | Accepted | Generalizes AI proposal review |
| ADR-0040 | Editor Extension SDK Is Capability-First and Transactional | Accepted | New |
| ADR-0041 | External Authoring Sources Use Provenance-Aware Import/Reimport Pipelines | Accepted + Implemented (v0.93) | New |
| ADR-0042 | Runtime Apply-Back Is Explicit, Scoped and Authorable-Field Only | Accepted | Extends play/runtime preview |
| ADR-0043 | Agent Runtime Uses Replaceable Orchestration Behind Typed Editor Capabilities | Accepted | Refines ADR-0027/0028; **parked post-v1 per MASTER_ROADMAP** |
| ADR-0044 | CI and Architecture Fitness Gates Are Release-Critical | Accepted | Operational architecture; **extended by `docs/specs/quality-fitness-gates.md`** |
| ADR-0045 | Project Format Is Git-Friendly, Deterministic and Explicitly Migrated | Accepted | Complements ADR-0033 |
| ADR-0046 | Semantic Editor Model Is the Authoritative Source of Truth | Accepted | Supersedes ADR-0001 source-of-truth semantics |
| ADR-0047 | Logic Graph Model Split — Pure Types in editor-model, Bevy Adapter in editor-core | Accepted + Implemented (v0.87) | New |
| ADR-0048 | ProjectStore v1 Is a Synchronous Port | Accepted | New |
| ADR-0049 | Dual Dispatch Gate for TransactionKernel Adoption | Accepted (v0.89) | **Superseded by ADR-0059** (single-path end-state planned) |
| ADR-0050 | ApplyBackPolicy Lives in editor-application | Accepted | New |
| ADR-0051 | ChangeWorkbenchPanel Lives in Bottom-Dock | Accepted | New |
| ADR-0052 | Runtime Causality — RebuildCause + LogicActivationRing + CausalityEdge | Accepted | New |
| ADR-0053 | Graph Kernel — A Pure-Rust Dialect-Agnostic Substrate | Accepted + Implemented (v0.101–v0.103) | **Extended by ADR-0061** (capability segregation) |
| ADR-0054 | Rig Agent Runtime Foundation — Transport Neutrality Addendum | Accepted | Extends ADR-0027/0043; **parked post-v1** |
| ADR-0055 | Work-Unit Commit Discipline — Trunk-Based Split Policy | Accepted | New |
| ADR-0056 | Enforce Hexagonal Dependency Direction | Accepted (2026-09-07) | Supersedes ADR-0030 |
| ADR-0057 | Single WASM Composition Root | Accepted (2026-09-07) | Supersedes ADR-0031 |
| ADR-0058 | Typed EditorBackend Capability API | Accepted (2026-09-07) | Supersedes ADR-0034 |
| ADR-0059 | Single Transaction/Mutation Dispatch Path | Accepted (2026-09-07) | Supersedes ADR-0049 (builds on ADR-0032) |
| ADR-0060 | Active Document + Orthogonal Workspace State | Accepted (2026-09-07) | New |
| ADR-0061 | Capability-Segregated GraphKernel | Accepted (2026-09-07) — pending benchmark spike | Extends ADR-0053 |
| ADR-0062 | BSN Anti-Corruption Layer | Accepted (2026-09-07) | New |
| ADR-0063 | Frontend Feature Slices and No Direct Bridge from UI | Accepted (2026-09-07) | New |

> **Renumbering note (preserved):** ADR-0046 was originally numbered ADR-0029 in the evolution pack. It was renumbered to avoid collision with the repository's existing ADR-0029 (Frontend Performance Budget Contract). The pack therefore covers ADR-0030 through ADR-0046.

> **Hardening Pack note (2026-09-07):** ADR-0056 through ADR-0063 land the `docs/bevy-2d-editor-hardening-pack/` program into `docs/adr/` as a single 8-record supersede/extend batch. Four supersede (0030/0031/0034/0049), one extends (0053→0061), three are net-new (0060/0062/0063). ADR-0058 adds a codegen spike documented at `docs/research/spike-typed-backend-bindings.md` (R-02). ADR-0061 is accepted pending the H8 benchmark spike.

## Existing status edits to apply

- ADR-0001 → `Superseded in source-of-truth semantics by ADR-0046`.
- ADR-0008 → `Accepted; amended by ADR-0033`.
- ADR-0027 → `Accepted; refined by ADR-0043`.
- ADR-0028 → `Accepted; extended by v0.87 Architecture Foundation and the converged pre-v1 roadmap`.
- ADR-0030 → `Accepted; superseded by ADR-0056 (enforcement layer added)`.
- ADR-0031 → `Accepted; superseded by ADR-0057 (composition root ownership)`.
- ADR-0034 → `Accepted; superseded by ADR-0058 (capability split + fitness rule + codegen spike)`.
- ADR-0049 → `Accepted; superseded by ADR-0059 (single-path end-state planned)`.