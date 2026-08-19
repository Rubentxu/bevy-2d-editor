# SPEC-PROTOCOL-002 — Typed WASM Protocol v2

**Status:** Proposed  
**ADR:** 0058

## Goal

Reduce a wide per-function bridge to a small versioned protocol while retaining typed capability semantics.

## Top-level contract

```rust
pub enum CommandEnvelope {
    Scene(SceneCommand),
    Assets(AssetCommand),
    Logic(LogicCommand),
    World(WorldCommand),
    Runtime(RuntimeCommand),
    Changes(ChangeCommand),
    Import(ImportCommand),
}

pub enum QueryEnvelope {
    Scene(SceneQuery),
    Assets(AssetQuery),
    Logic(LogicQuery),
    World(WorldQuery),
    Runtime(RuntimeQuery),
    Graph(GraphQuery),
    Validation(ValidationQuery),
    Changes(ChangeQuery),
}

pub enum EditorNotification {
    SemanticChanged(...),
    SelectionChanged(...),
    ValidationChanged(...),
    RuntimeChanged(...),
    GraphChanged(...),
    ChangeWorkbenchChanged(...),
    TraceAvailable(...),
}
```

## Rules

- typed variants/DTOs;
- serializable and versioned;
- no raw JSON for critical payloads except explicit extension bags;
- no Bevy types crossing boundary;
- no `bevy::Entity`;
- stable IDs use protocol wrappers;
- errors have stable codes and readable messages.

## Frontend capabilities

Expose narrow service interfaces such as SceneBackend, AssetBackend, LogicBackend, WorldBackend, RuntimeBackend, GraphBackend, ValidationBackend and ChangeBackend. They may share one transport.

## Notifications

Use subscribe -> notification -> targeted store invalidation/query. Remove polling for migrated areas.

## Migration

1. new transport behind old gateway;
2. capability-by-capability switch;
3. delete old window bindings.

Track dual-path debt explicitly.

## CI

Rust/TS bindings must match. Critical `unknown` payloads fail unless allowlisted with removal task.
