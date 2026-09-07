# Spike R-02 — Typed Backend Bindings

## Question

How should Rust/WASM capabilities become stable TypeScript APIs with minimal drift?

## Alternatives

1. Handwritten `EditorBackend` interfaces + explicit Wasm adapter.
2. Generate DTOs/types from Rust/protocol schema.
3. Use wasm-bindgen generated declarations directly inside adapter and handwrite semantic capability wrappers.
4. Hybrid: protocol-generated DTOs + handwritten behavior interfaces.

## Required prototype

Implement only Scene API subset:

- get snapshot;
- create;
- rename;
- reparent;
- set field;
- undo/redo.

## Tests

- signature drift compile failure;
- structured error mapping;
- fake backend feature test;
- runtime Wasm backend E2E;
- bundle delta;
- developer ergonomics comparison.

## Decision rule

Prefer the simplest approach that catches drift at compile time and keeps raw WASM details out of features.

