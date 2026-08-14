# Migration Strategy — Strangler, Not Rewrite

## Principle

Keep every release runnable. Introduce target abstractions around the existing implementation, then move one responsibility at a time. Serialization compatibility and existing UI behavior are explicit gates.

## Stage 0 — Safety net

Before structural extraction:

- implement GitHub Actions gates;
- freeze a corpus of representative projects/scenes/assets/logic graphs;
- record golden JSON/BSN outputs;
- add architecture dependency checks;
- close/label stale issues that contradict current code;
- record current frontend bundle and WASM sizes.

## Stage 1 — Extract pure model

Move dependency-free types first:

```text
editor-core::document          -> editor-model
editor-core::scene_asset       -> editor-model
editor-core::scene_instance    -> editor-model
editor-core::logic_graph       -> editor-model
schema value types             -> editor-model
```

Compatibility strategy: `editor-core` temporarily re-exports moved types so downstream source changes stay small.

## Stage 2 — Introduce application ports

Create traits around behavior already implemented:

- `Clock`;
- `IdGenerator`;
- `ProjectStore`;
- `PreviewRuntime`;
- `SearchIndex`;
- `BuildRunner`.

First adapters delegate to current implementations. Do not redesign storage formats in the same PR.

## Stage 3 — `EditorSession`

Create an explicit session aggregate/container and migrate global state progressively.

Transitional shape:

```text
editor-wasm thread_local!
    └── RefCell<EditorSession>
```

Move one global family per PR: scene → assets → logic → validation/cache → runtime requests.

## Stage 4 — Transaction Kernel

Extract shared operation-history mechanics from the three existing command domains. Keep `SceneCommand`, `AssetCommand` and `LogicCommand` distinct.

Introduce `ChangeSet` initially as metadata around existing dispatches, then migrate multi-resource workflows to true atomic planning/apply semantics.

## Stage 5 — Typed backend

Wrap current WASM exports behind capability modules and generated/declared TypeScript types. Replace `window as any` usage feature by feature. Tests receive an injectable `EditorBackend` implementation.

## Stage 6 — Storage dual-mode

Keep current OPFS adapter operational. Add format-neutral `ProjectStore` operations and a filesystem adapter. Establish migration/version tests before making filesystem mode canonical for professional workflows.

## Stage 7 — Workflow capabilities

Build World Workspace, 2D manipulation and recipes on the new application/transaction substrate.

## Stage 8 — Agent runtime

Only now connect Rig tools to capability ports and `ChangeSet` review. No agent is allowed to mutate internal stores directly.

## Rollback rule

Every migration PR must leave a compatibility route until the next layer is verified. Deleting the old route is a separate cleanup PR with evidence that no production caller remains.
