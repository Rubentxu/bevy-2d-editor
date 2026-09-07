# Specification — Single Session and Composition Root

## Goal

Remove hidden initialization order and multiple ownership paths while preserving a pragmatic WASM singleton where necessary.

## Current debt being addressed

The current architecture contains multiple registration mechanisms for store, session and registries, plus remaining editor state thread-locals. This creates temporal connascence: callers must know which registrations occurred before invoking otherwise ordinary operations.

## Required architecture

Only target-specific composition code may own ambient singleton state.

Conceptually:

```text
editor-wasm
  AppContainer
    EditorSession
    ProjectStore adapter
    PreviewRuntime adapter
    Clock
```

Application/model code receives dependencies through constructors, use-case contexts, resources or explicit session access.

## Migration sequence

### S1 — Inventory and ownership matrix

Produce a table for every global/thread-local:

- name;
- current crate;
- writer(s);
- reader(s);
- durability;
- lifecycle;
- target owner;
- migration PR.

### S2 — No-new-global fitness gate

CI blocks new `thread_local!`, `static Mutex`, `OnceLock` and equivalent patterns outside approved target-composition files.

Allowlist the current debt while migrating.

### S3 — ProjectStore

Move service registration out of `editor-model`. `EditorSession` and application use cases hold/use the port explicitly.

### S4 — Extension/importer registries

Make registries session-owned capabilities. Permission checks receive a registry/policy service rather than searching a global registry.

### S5 — Scene/asset/logic state families

Migrate one family at a time. Each migration has parity tests proving no changes to:

- undo/redo;
- dirty semantics;
- switching;
- save/load;
- validation;
- preview rebuild behavior.

### S6 — Runtime/hot-reload state

Prefer Bevy Resources for runtime-only ECS state and application session state for authoring/runtime coordination. Do not put runtime-only Bevy entity references into the durable session.

### S7 — Collapse registration paths

Remove duplicate session/service registration once all consumers use the canonical route.

## Failure behavior

Calling a capability before initialization must return an explicit typed `NotReady`/`Initialization` error at the boundary. It must not depend on a component having rendered or a test polling a random global function.

## Multi-session test

Even if production v1 uses one session, core application tests must be able to create two independent sessions and prove no state leakage.

## UAT anchor

See `UAT-ARCH-003` through `UAT-ARCH-008`.

