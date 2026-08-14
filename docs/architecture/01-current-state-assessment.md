# Current-State Architecture Assessment

## Summary

The current codebase has a **strong domain vocabulary and mutation model**, but the physical code boundaries have not kept pace with capability growth. It behaves as a modular monolith whose internal design is often good, while dependency direction and global state remain weakly enforced.

## Strengths to preserve

### Editor-owned domain model

Persistent editor identity is separated from ephemeral Bevy `Entity` identity. This is essential for undo/redo, references, migrations, AI operations and deterministic project state.

### Semantic reversible commands

Commands capture intent rather than raw diffs. Batches can be treated atomically and inverse commands are mechanically generated. This is an excellent base for human actions, agents and automation.

### Domain-specific identity separation

Scene editing and Scene Asset authoring deliberately use different identities and command surfaces. This prevents accidental `StableId`/`LocalId` mixing.

### BSN alignment without surrendering editor semantics

The project models reusable scene composition in a Bevy-aligned way while still retaining editor metadata and explicit instance/override semantics.

### Logic Bricks extension point

The `NodeEvaluator` trait and registry demonstrate the right OCP direction: behavior is extended through compiled evaluators rather than a growing central VM switch.

### Documentation discipline

The repository has extensive ADR/spec/roadmap history and a well-defined domain glossary. That history should be retained, not rewritten.

## Architectural debt

### 1. Crate boundary collapse

`editor-core` currently includes domain types, application use cases, Bevy runtime integration, WASM bindings, browser-specific APIs and global state. A developer can accidentally import infrastructure into core logic because Cargo does not prevent it.

### 2. Global mutable session state

Multiple `thread_local!` stores coordinate active documents, catalogs, caches, logs, validation and runtime state. This makes production behavior easy to call but obscures dependencies and blocks clean parallelism/multi-session evolution.

### 3. Central command switches

The semantic command idea is strong, but large `enum` + `match validate/apply` processors grow with every capability. The shared mechanics should be extracted without collapsing domain-specific command types.

### 4. Frontend coordination growth

`App.tsx` coordinates selection, modes, dialogs, assets, AI, logic, navigation, polling and dock behavior. `engine-bridge.ts` manually exports a large untyped global API via `window`.

### 5. Persistence coupling

OPFS is currently close to the project worldview. The product needs a persistence port so browser-only, native filesystem and Git-friendly modes can coexist.

### 6. CI/documentation drift

The repository documents a trunk-based, always-green workflow, but architecture and quality gates need to exist as actual automated workflows, not only developer expectations.

## SOLID assessment

### SRP
Strong inside some domain modules, weak at system boundaries (`lib.rs`, frontend root coordination, evaluator/runtime mixtures).

### OCP
Good in Logic Bricks, weaker in central command processors and bridges that require modification per capability.

### LSP
Generally healthy because inheritance is limited; trait contracts should remain behaviorally narrow.

### ISP
Needs improvement around broad bridge/API surfaces. Prefer capability interfaces (`SceneApi`, `AssetApi`, `RuntimeApi`) over a monolithic editor interface.

### DIP
The primary target for improvement. Application logic must depend on ports rather than OPFS, Bevy, browser globals or process/thread-local registries.

## Conclusion

The architecture does **not** need a rewrite. It needs a strangler-style extraction that turns existing good concepts into compile-time boundaries while preserving serialized compatibility and user workflows throughout the migration.
