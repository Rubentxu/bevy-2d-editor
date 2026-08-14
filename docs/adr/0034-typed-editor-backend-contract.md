# ADR-0034: Typed EditorBackend Contract Replaces Global Window Bridge

## Status

Accepted — 2026-08-14


## Context

The current frontend bridge manually exposes a large WASM surface through `window as any`. This couples tests and features to implementation details and makes protocol drift easy.

## Decision

Define a typed `EditorBackend` made of narrow capability APIs:

```ts
interface EditorBackend {
  scene: SceneApi;
  assets: SceneAssetApi;
  world: WorldApi;
  logic: LogicApi;
  runtime: RuntimeApi;
  code: CodeApi;
  validation: ValidationApi;
  changes: ChangeApi;
}
```

The production implementation wraps WASM. Tests inject an in-memory/fake backend. Rust protocol DTOs and TypeScript types must be generated or contract-checked from one stable protocol definition.

## Rules

- no new `window as any` exports;
- no feature component imports the raw WASM module;
- errors use typed error codes + user-facing messages;
- APIs are versioned by protocol compatibility, not by browser globals.

## Consequences

Frontend decomposition becomes safer, tests become deterministic and future native/remote backends can reuse the same UI capability model.
