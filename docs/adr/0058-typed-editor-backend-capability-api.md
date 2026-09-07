# ADR-0058 — Typed EditorBackend Capability API

Status: Proposed

## Context

The frontend currently exposes a very broad WASM API through globals and bridge-name strings. This creates type drift, name coupling and makes component tests depend on runtime initialization.

## Decision

Introduce an injectable TypeScript `EditorBackend` split into capability interfaces.

Production uses `WasmEditorBackend`. Tests may use in-memory, recording and fault-injecting implementations.

Only backend adapter modules may import raw generated WASM bindings or invoke transitional bridge globals.

## Consequences

- React features become independent of transport;
- TypeScript can detect API drift;
- tests can simulate errors without browser/WASM boot;
- migration is incremental by capability.

## Non-decision

No mandatory code-generation technology is selected here. A spike decides whether wrappers are handwritten or generated/shared through protocol schemas.

