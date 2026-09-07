# ADR-0057 — Single WASM Composition Root

Status: Proposed

## Context

ProjectStore, EditorSession, extensions, importers and remaining editor state are reachable through multiple global/thread-local registration paths. This has created initialization-order coupling and makes the owner of mutable state ambiguous.

## Decision

`editor-wasm` is the only target-specific composition root. It may own one process/thread-local or `OnceLock` application container if required by browser/WASM constraints.

The container owns or references the canonical `EditorSession` and concrete adapters.

Model and application crates do not host service locators.

Bevy runtime-local transient state may use Bevy Resources when it is truly ECS runtime state, but not as a second authoritative authoring store.

## Consequences

- initialization becomes one explicit lifecycle;
- multi-session application tests become possible;
- races caused by registry registration order are removed;
- migration requires temporary adapters for old callers.

## Guardrail

A CI ratchet forbids adding new ambient global state outside an allowlist.

## Revisit

If wasm threading or worker architecture later requires per-worker containers, extend the composition model without moving service locators back into domain/application crates.

