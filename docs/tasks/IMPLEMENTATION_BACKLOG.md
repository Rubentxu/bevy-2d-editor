# Implementation Backlog

Priority is dependency-driven. IDs are suggested epic identifiers for issue creation.

## P0 — Repository trust

### ARCH-001 CI workflows
**Done when:** required PR checks execute fmt/clippy/tests/WASM/frontend/smoke architecture gates.

### ARCH-002 Golden compatibility corpus
Representative JSON/BSN/project fixtures committed and exercised.

### ARCH-003 Architecture fitness scripts
Dependency/global-state/untyped-bridge growth checks fail CI.

### GOV-001 Documentation/issue hygiene
Close or reconcile stale issues and contradictions between CONTRIBUTING/roadmap/repo automation.

## P1 — Hexagonal foundation

### ARCH-010 `editor-model`
Move pure value types with legacy re-exports.

### ARCH-011 `editor-application`
Introduce application crate, ports and use-case modules.

### ARCH-012 `EditorSession`
Migrate scattered globals by bounded context.

### ARCH-013 `Clock` + `IdGenerator`
Deterministic tests and collision-safe production IDs.

### ARCH-014 `ProjectStore`
In-memory + OPFS contract implementations.

### ARCH-015 Transaction Kernel
Shared history/batch/rollback mechanics.

### ARCH-016 ChangeSet v1
Origin, affected resources, semantic summary, effects.

### FE-010 Typed EditorBackend
Scene vertical slice then asset/logic/runtime/code.

### FE-011 App shell decomposition
Move orchestration to feature modules/controllers.

## P2 — 2D production workflow

### UX-020 viewport direct manipulation
### UX-021 snapping/guides
### UX-022 align/distribute
### WORLD-020 WorldDocument
### WORLD-021 World canvas/navigation
### WORLD-022 topology validation
### WORKFLOW-020 Scope of Change
### RECIPE-020 Recipe runtime
### RECIPE-021 initial recipe pack
### STORE-020 filesystem-backed project mode
### STORE-021 deterministic format/migration UX

## P3 — Change/runtime workbench

### CHANGE-030 Change Workbench shell
### CHANGE-031 semantic diff engine
### CHANGE-032 checkpoints/rollback UX
### RUNTIME-030 causality model/events
### RUNTIME-031 causality inspector UI
### RUNTIME-032 authorability metadata
### RUNTIME-033 runtime apply-back

## P4 — Agent runtime

### AI-040 editor-protocol tools
### AI-041 agent-runtime/Rig foundation
### AI-042 Scene specialist
### AI-043 validation/runtime specialist
### AI-044 compatibility propose endpoint
### AI-045 policy/approval enforcement
### AI-046 mock provider/tool integration tests

## P5 — Semantic agents

### AI-050 semantic index
### AI-051 typed retrieval
### AI-052 specialist agents
### AI-053 post-apply verification
### AI-054 bounded background maintenance

## P6 — Ecosystem

### SDK-060 extension registry
### SDK-061 capability permissions/versioning
### IMPORT-060 ExternalSource model ✅ DONE (v0.93 PR1 — external_source.rs + importer.rs)
### IMPORT-061 Aseprite pipeline ✅ DONE (v0.93 PR2 — aseprite.rs + builtin)
### IMPORT-062 LDtk pipeline ✅ DONE (v0.93 PR3 — ldtk.rs + builtin)
### IMPORT-063 Tiled pipeline ✅ DONE (v0.93 PR4 — tiled.rs + builtin)

## P7 — v1 hardening

### PERF-070 large-project benchmarks
### DATA-070 migration support matrix
### DATA-071 crash/recovery tests
### A11Y-070 critical workflow audit
### DOC-070 onboarding/tutorial game
### REL-070 reproducible release workflow
