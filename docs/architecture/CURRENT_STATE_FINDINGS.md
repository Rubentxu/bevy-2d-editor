# Current-State Findings — Baseline for Hardening

## Purpose

Record the concrete observations that justify the program. This is not a complete static-analysis report; it is a prioritized architecture baseline.

## Strong foundations to preserve

- Semantic editor-owned model distinct from Bevy runtime identity.
- Stable IDs and separate Scene Asset local IDs.
- Reversible typed command surfaces.
- TransactionKernel/ChangeSet direction.
- GraphKernel + dialect concept.
- Extensive ADR/spec/test discipline.
- Existing UI workflows across Scene Assets, Logic, Runtime, Validation, World and import/export.
- Architecture target already documented in `docs/architecture/02-target-architecture.md`.

## Critical boundary debt

### F-001 Application dependency direction

Current `crates/editor-application/Cargo.toml` includes concrete `editor-storage-web` and target-specific `editor-bevy` dependency paths. This contradicts the intended adapter→application direction.

### F-002 Model contains runtime/service registries

`crates/editor-model/src/ports.rs` contains registration/access patterns for ProjectStore, EditorSession, ExtensionRegistry and ImporterRegistry.

These are composition concerns and create temporal coupling.

### F-003 Multiple canonical-session paths

`editor-wasm` owns an application session while the same/session-related services are also registered through model-level registries. Remaining Bevy thread-local state further spreads ownership.

### F-004 Frontend raw bridge

`frontend/src/engine-bridge.ts` exposes a large set of operations through `window` globals and uses broad `any` typing.

### F-005 UI bridge bypass

`HierarchyPanel` directly constructs a reparent command envelope and calls bridge dispatch rather than receiving a semantic reparent capability.

### F-006 Kernel + legacy mutation routes

`editor-bevy` retains runtime selectable kernel/legacy dispatch, creating duplicated semantic risk after migration is mature.

### F-007 `editor-bevy` responsibility breadth

The crate still contains pure authoring commands, persistence-oriented behavior, graph/domain helpers, import/export and Bevy runtime behavior. It is not yet primarily an adapter.

### F-008 Frontend coordination concentration

`App`, `AppShell`, `useSceneHandlers` and mode-controller contracts remain very broad even after useful decomposition work.

### F-009 Hierarchy asymptotic risk

Parent/depth lookup performs repeated scene-array searches during row rendering. This is incompatible with an unexamined 10k-entity target.

### F-010 GraphKernel performance/ISP questions

Dialects may scan edges for incoming/outgoing and use boxed iterators. Mutation interface can force unsupported operations. These require measurement before optimization.

### F-011 Architecture CI reliability

The dedicated architecture workflow configuration can fail in Node cache setup before running the architecture checker. A broken gate is not an enforcement mechanism.

### F-012 Fitness coverage gap

Existing checks focus on purity regexes and selected assertions, but do not fully validate intended Cargo dependency direction.

### F-013 Transitional/incomplete capability surfaces

Some importer/extension/WASM paths include transition stubs or simplified flows. Stable vs experimental product contracts should be explicit before v1.

## Smell classification

| Finding | SOLID | Connascence | Smell |
|---|---|---|---|
| F-001 | DIP | Type/Meaning | dependency inversion violation |
| F-002/F-003 | DIP/SRP | Timing/Execution | service locator, ambient state |
| F-004/F-005 | ISP/DIP | Name/Meaning | stringly bridge, layer bypass |
| F-006 | SRP | Algorithm | duplicated mutation pipeline |
| F-007 | SRP | Type/Algorithm | god crate/modular monolith |
| F-008 | SRP/ISP | Name/Position | god orchestration contracts |
| F-009 | — | Algorithm | hidden N²-ish lookup pattern |
| F-010 | ISP/LSP | Values/Algorithm | overly broad contract |
| F-011/F-012 | — | — | false sense of enforcement |

## Program implication

The project does not need a rewrite. The highest-leverage strategy is to strengthen the existing semantic core by correcting system boundaries and then optimize UX/performance with real corpus evidence.

