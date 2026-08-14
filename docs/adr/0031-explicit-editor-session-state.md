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
