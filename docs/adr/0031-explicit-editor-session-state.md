# ADR-0031: Explicit EditorSession Replaces Domain-Level Global State

## Status

Accepted — 2026-08-14


## Context

The current implementation uses multiple `thread_local!` stores for scene, asset, logic, validation, cache and runtime request state. This is workable in a single browser session but hides dependencies and complicates testing, multi-project sessions and agent/background work.

## Decision

Introduce `EditorSession` as the explicit application-level owner of mutable editing state. Use cases receive a session/context or focused service references.

WASM may retain one target-specific `thread_local! RefCell<EditorRuntime>` in the composition root during the browser phase. No domain/application module may declare new ambient mutable stores.

## Rules

- caches have named owners and invalidation methods;
- active document selection is part of session state;
- operation histories are scoped explicitly;
- test code creates isolated sessions;
- `ProcessorContext::from_globals()` is transitional and must be removed.

## Consequences

This enables deterministic tests, future multiple projects/tabs, background work queues and more explicit concurrency decisions.

## Amendment (2026-08-16)

**EditorSession sub-states use editor-model domain types** — `SceneSessionState.document` now holds `Option<SceneDocument>` (real type from `editor_model::document`). `AssetSessionState` and `LogicSessionState` similarly use `SceneAssetDocument` and `LogicGraphAsset` from `editor_model`. `OperationLog` remains in `editor-core` (future migration to `editor_model` is tracked separately).

**editor-core modules become state-parameterized** — `processor::apply` now uses `ProcessorContext::empty()` instead of `ProcessorContext::from_globals()`. The deprecated `from_globals()` will be removed once all callers migrate to explicit context passing. `ProcessorContext::with_asset_body()` provides explicit asset body injection for `ReplaceInstanceAsset` commands.

**No new ambient stores** — The PR2a migration keeps the existing `thread_local!` stores in `editor-core` but no new ambient stores are introduced. Further migration of stores (SCENE_DOC, OPERATION_LOG, etc.) to `EditorSession` is progress toward the ADR's end-state of zero ambient mutable state.

**Status**: Ratified by owner — 2026-08-16.
