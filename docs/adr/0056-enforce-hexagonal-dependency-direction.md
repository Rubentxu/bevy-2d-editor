# ADR-0056 — Enforce Hexagonal Dependency Direction

Status: Proposed

## Context

The target architecture already describes `editor-application` as the owner of application ports and infrastructure crates as adapters. The current workspace still contains dependency edges from application toward concrete web storage and Bevy integration.

This weakens DIP and allows infrastructure details to spread into use cases.

## Decision

Enforce this dependency direction:

```text
editor-model <- editor-application <- editor-storage-web
                                  <- editor-bevy
                                  <- editor-wasm composition
```

`editor-wasm` may depend on application, Bevy and web storage to assemble the target runtime.

Critical crate-edge rules are checked using `cargo metadata` in CI.

## Consequences

Positive:

- browser/Bevy-free application tests;
- native or remote adapters can be added later;
- invalid imports become build/CI failures;
- responsibilities become easier to locate.

Negative:

- transitional reexports and adapters are needed;
- some WASM code moves between crates;
- existing crate names may temporarily not match their contents.

## Migration

1. Add graph gate with current exceptions.
2. Move/duplicate ports to application-facing modules while preserving public aliases.
3. Invert ProjectStore implementation dependency.
4. Move Bevy-specific application composition out of `editor-application`.
5. Remove exceptions.

## Rejected

- keep current edges and rely on developer discipline;
- split every bounded context into a crate immediately;
- rewrite the workspace in one migration.

