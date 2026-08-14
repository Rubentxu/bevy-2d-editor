# Current → Target Architecture Mapping

This mapping is a migration aid, not a requirement to move whole files unchanged. Large files should usually be split by responsibility while preserving behavior.

| Current area | Target owner | Notes |
|---|---|---|
| `crates/editor-core/src/document.rs` | `editor-model::scene` | move pure IDs/entities/documents |
| `scene_asset.rs` | `editor-model::scene_asset` | pure definition model |
| `scene_instance.rs`, overrides value types | `editor-model::scene_asset` | pure values/invariants; application resync orchestration separate |
| `logic_graph.rs` | `editor-model::logic` | graph values only |
| `schema.rs` | split `editor-model::schema` + application registry | remove global registry ownership from model |
| `command.rs` | `editor-application::scene::commands` | command semantics stay domain-specific |
| `processor.rs` | `editor-application::scene` | use explicit dependencies/context, no globals |
| `asset_command.rs` | `editor-application::assets` | preserve LocalId-specific commands |
| `logic_command.rs` | `editor-application::logic` | preserve graph-specific commands |
| operation logs | `editor-application::transactions` | shared mechanics via Transaction Kernel |
| `state.rs`, `*_state.rs` | `editor-application::session` + composition root | remove ambient globals |
| `persistence.rs` path/model helpers | split model/project format + storage adapter | OPFS calls remain adapter-only |
| WASM functions in `lib.rs` | `editor-wasm` feature modules | composition/bindings only |
| `preview_runtime.rs` | `editor-bevy::preview` | runtime projection and observations |
| Bevy anchor/dynamic-scene adapters | `editor-bevy` / BSN adapters | outward representation |
| `logic_evaluator.rs` model descriptors | split model/application/runtime | evaluator trait/registry separated from Bevy sensors/runtime state |
| `source_files.rs`, `asset_files.rs` | application ports + storage adapters | semantic metadata vs bytes/storage |
| `frontend/src/engine-bridge.ts` | `frontend/src/backend/wasmBackend.ts` + feature APIs | typed implementation, no globals |
| `frontend/src/App.tsx` | `app/AppShell.tsx` + feature controllers | composition only |
| `useAIAssistant.ts` | `features/agents/` | later consumes agent/change APIs |
| `useDockPrefs.ts` | UI shell | legitimate presentation state; remains TS |
| `ai-proxy` orchestration logic | split proxy transport + `agent-runtime` | proxy remains HTTP/policy boundary |

## Extraction order heuristic

Move **values first**, then **ports**, then **use cases**, then **adapters**. Moving adapter code before a port exists only relocates coupling.

## Temporary compatibility patterns

Use short-lived re-exports:

```rust
// legacy editor-core path during migration only
pub use editor_model::scene::{SceneDocument, Entity, StableId};
```

Every compatibility re-export receives a removal issue/version target so the migration layer cannot become permanent architecture.
