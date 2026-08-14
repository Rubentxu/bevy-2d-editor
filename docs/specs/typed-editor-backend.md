# Specification — Typed Frontend Backend Contract

## Goal

Make React a presentation/client layer over stable capabilities and remove implicit global bridge APIs.

## API groups

```text
SceneApi
SceneAssetApi
WorldApi
LogicApi
RuntimeApi
CodeApi
ValidationApi
SearchApi
ChangeApi
ProjectApi
```

## Contract generation

Preferred order:

1. define Rust DTO/protocol types in `editor-protocol`;
2. generate or verify TypeScript representations;
3. expose narrow WASM functions/modules;
4. construct `EditorBackend` in one frontend composition module.

## Error envelope

```ts
type BackendError = {
  code: string;
  message: string;
  resource?: ResourceRef;
  details?: unknown;
};
```

Do not require UI code to parse Rust error strings to understand failure type.

## Testability

Frontend E2E may use production WASM. Component/integration tests can use `FakeEditorBackend` with deterministic responses.

## Migration target

- zero new `window as any` functions immediately;
- move existing bridge calls by feature;
- remove direct Playwright hooks from `App.tsx` where the injectable backend/test harness can replace them.
