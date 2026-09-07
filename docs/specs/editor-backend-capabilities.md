# Specification — Typed Editor Backend Capabilities

## Goal

Replace the large raw TS↔WASM global API with a typed, injectable frontend backend contract.

## Contract

```ts
export interface EditorBackend {
  project: ProjectApi;
  scene: SceneApi;
  assets: SceneAssetApi;
  logic: LogicApi;
  world: WorldApi;
  code: CodeApi;
  runtime: RuntimeApi;
  validation: ValidationApi;
  changes: ChangesApi;
}
```

Each API is narrow and capability-oriented.

## Rules

- React feature components never reference `window.dispatch_*`, raw WASM exports or `bridge-call` strings.
- WASM mapping is isolated under `frontend/src/backend/wasm/`.
- DTO conversion is centralized at the backend boundary.
- Domain/service errors are mapped into discriminated TS result/error types.
- Test code can inject an in-memory/fake/recording backend.
- Existing raw test hooks may remain temporarily under a test-only adapter with an expiry condition.

## Example

```ts
interface SceneApi {
  snapshot(): Promise<SceneDocument>;
  createEntity(input: CreateEntity): Promise<MutationReceipt>;
  renameEntity(id: StableId, name: string): Promise<MutationReceipt>;
  reparentEntity(id: StableId, parent: StableId | null): Promise<MutationReceipt>;
  setField(input: SetField): Promise<MutationReceipt>;
}
```

The component calls `scene.reparentEntity`; it does not construct Rust `CommandEnvelope` metadata.

## Generation vs handwritten bindings

This is an experimental detail. A spike must compare:

1. handwritten TypeScript interfaces + wrapper implementation;
2. generated types from `wasm-bindgen` declarations;
3. shared schema/codegen from `editor-protocol`.

Decision criteria:

- compile-time coverage;
- ergonomics;
- drift risk;
- bundle cost;
- testability;
- compatibility/versioning.

## Events/subscriptions

Do not introduce a generic event bus by default. Capability-specific subscriptions are allowed when polling is demonstrably inferior:

```ts
runtime.onMetrics(...)
project.onExternalChange(...)
validation.onIssuesChanged(...)
```

## Migration

Move capability by capability. The original bridge remains as a compatibility adapter until no production caller exists.

## Fitness rule

Production files outside `frontend/src/backend/wasm/` must not contain:

- `window as any` for backend operations;
- direct imports from generated WASM module;
- stringly `callBridge("...")` mutations.

