# ADR Evolution Index — Approved Addendum

Merge these entries into the repository's existing `docs/adr/README.md`; this file intentionally has a different name to avoid overwriting the historical index by accident.

| Number | Title | Status | Relationship |
|---|---|---|---|
| ADR-0030 | Compile-Time Hexagonal Crate Boundaries | Accepted | New |
| ADR-0031 | Explicit EditorSession Replaces Domain-Level Global State | Accepted | New |
| ADR-0032 | Shared Transaction Kernel and ChangeSet, with Domain-Specific Commands | Accepted | Evolves shared mechanics; preserves ADR-0007 domain split |
| ADR-0033 | ProjectStore Port with OPFS and Filesystem Adapters | Accepted | Amends ADR-0008 |
| ADR-0034 | Typed EditorBackend Contract Replaces Global Window Bridge | Accepted | New |
| ADR-0035 | Clock and IdGenerator Are Explicit Application Ports | Accepted | New |
| ADR-0036 | Bevy Runtime Preview Is an Ephemeral Projection Adapter | Accepted | Reinforces editor-owned state direction |
| ADR-0037 | World Workspace Is a First-Class Product Context | Accepted | Extends level workflow |
| ADR-0038 | Workflow and Gameplay Recipes Compile Intent into Typed Changes | Accepted | Extends Logic Bricks/application workflows |
| ADR-0039 | Change Workbench Is the Unified Review and Approval Surface | Accepted | Generalizes AI proposal review |
| ADR-0040 | Editor Extension SDK Is Capability-First and Transactional | Accepted | New |
| ADR-0041 | External Authoring Sources Use Provenance-Aware Import/Reimport Pipelines | Accepted + Implemented (v0.93) | New |
| ADR-0042 | Runtime Apply-Back Is Explicit, Scoped and Authorable-Field Only | Accepted | Extends play/runtime preview |
| ADR-0043 | Agent Runtime Uses Replaceable Orchestration Behind Typed Editor Capabilities | Accepted | Refines ADR-0027/0028 |
| ADR-0044 | CI and Architecture Fitness Gates Are Release-Critical | Accepted | Operational architecture |
| ADR-0045 | Project Format Is Git-Friendly, Deterministic and Explicitly Migrated | Accepted | Complements ADR-0033 |
| ADR-0046 | Semantic Editor Model Is the Authoritative Source of Truth | Accepted | Supersedes ADR-0001 source-of-truth semantics |

> **Renumbering note:** ADR-0046 was originally numbered ADR-0029 in the evolution pack. It was renumbered to avoid collision with the repository's existing ADR-0029 (Frontend Performance Budget Contract). The pack therefore covers ADR-0030 through ADR-0046.

## Existing status edits to apply

- ADR-0001 → `Superseded in source-of-truth semantics by ADR-0046`.
- ADR-0008 → `Accepted; amended by ADR-0033`.
- ADR-0027 → `Accepted; refined by ADR-0043`.
- ADR-0028 → `Accepted; extended by v0.87 Architecture Foundation and the new master roadmap`.
