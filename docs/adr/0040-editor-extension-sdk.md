# ADR-0040: Editor Extension SDK Is Capability-First and Transactional

## Status

Accepted — 2026-08-14


## Context

Godot/Defold demonstrate that editor extensibility compounds product value. A public ABI designed too early would freeze unstable internals.

## Decision

Design an internal `Editor Extension SDK` around stable capabilities before shipping a marketplace or binary plugin ABI.

Extension categories:

- commands/actions;
- menus/palette entries;
- validators;
- importers/reimporters;
- recipes;
- inspectors/property editors;
- asset processors;
- panels/tools;
- runtime diagnostic providers.

All durable extension mutations use capabilities/`ChangeSet`; direct mutable access to `EditorSession` is forbidden.

## Rollout

1. internal Rust extension interfaces;
2. at least three built-in extensions prove the surface;
3. stabilize protocol;
4. evaluate WASM/native plugin ABI and distribution model.

## Consequences

Extensibility becomes architecturally possible without prematurely committing to a marketplace or unsafe plugin model.
