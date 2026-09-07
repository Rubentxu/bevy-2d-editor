# Target Architecture — v1 Direction

## System view

```text
React Workbench
    |
    v
EditorBackend (typed capability API)
    |
    v
WASM Composition Root
    |
    +--> EditorApplication / EditorSession
    |       +--> domain model
    |       +--> TransactionKernel
    |       +--> use cases
    |       +--> ports
    |
    +--> Web ProjectStore adapter
    |
    +--> Bevy PreviewRuntime adapter
```

## Session shape

La forma exacta es emergente; la responsabilidad no.

```rust
EditorSession
├── documents/workspace state
├── scene sessions
├── asset sessions
├── logic sessions
├── world sessions
├── change/history state
├── runtime/apply-back state
├── validation state
└── capability registries
```

No se requiere convertirlo en una mega-struct plana. Puede contener subservicios cohesivos.

## Frontend shape

```text
frontend/src/
  app/
    providers/
    shell/
    routing/
  backend/
    EditorBackend.ts
    wasm/
    testing/
  features/
    scene/
    assets/
    logic/
    world/
    code/
    runtime/
    validation/
    changes/
    project/
  shared/
    ui/
    types/
```

## Workspace model direction

Separar:

- tipo de documento activo;
- estado runtime (edit/play/pause);
- layout/workspace;
- selección contextual.

Evitar que un enum global `EditorMode` sea responsable de todas esas dimensiones.

## Mutation flow

```text
UI intent
  -> typed capability command
  -> application use case
  -> ChangeSet/TransactionKernel when applicable
  -> domain mutation
  -> persistence + projection effects
  -> typed result/events
  -> UI refresh/optimistic reconciliation
```

No existen rutas especiales de mutación para AI, plugins o importadores que salten el kernel/policies.

## Extension direction

Extensiones aportan contribuciones tipadas:

- commands/proposals;
- validators;
- recipes;
- importers;
- logic node descriptors.

No reciben referencias directas a stores internos.

## Native/remote future

La arquitectura permite sustituir el runtime sin cambiar features React:

```text
EditorBackend
  ├── WasmEditorBackend
  ├── NativeIpcEditorBackend   (future)
  └── RemoteEditorBackend      (future)
```

No se implementan antes de existir un caso de uso que lo justifique.

