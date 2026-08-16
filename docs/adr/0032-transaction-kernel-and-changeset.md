# ADR-0032: Shared Transaction Kernel and ChangeSet, with Domain-Specific Commands

## Status

Accepted — 2026-08-14


## Context

Scene, Scene Asset and Logic domains each implement typed commands, validation, inverse generation and operation logs. The mechanics are converging, while their identity and semantics are intentionally different.

## Decision

Extract a **Transaction Kernel** that owns common mechanics without creating a universal command enum.

`SceneCommand`, `AssetCommand`, `LogicCommand` and future command types remain distinct. A `ChangeSet` composes operations across capabilities/resources and carries:

- origin (`Human`, `Agent`, `Recipe`, `Importer`, `Migration`, `Plugin`, `RuntimeApplyBack`);
- actor/authorship and rationale;
- affected resources;
- typed operations;
- preflight validation;
- semantic diff summary;
- runtime/build effects;
- approval policy;
- inverse/rollback metadata.

## Atomicity

Single-document command groups use direct rollback/inverses. Multi-resource changes use a prepare/apply/commit protocol where supported; otherwise the capability declares compensating operations and partial-failure behavior before approval.

## Non-goals

- not event sourcing;
- not one generic `Command<T>` abstraction that erases domain language;
- not a database transaction engine.

## Consequences

Undo/redo, AI proposals, importer reimports, runtime apply-back and migration previews can share one review/audit language.

## Amendment (2026-08-16)

**Kernel types relocated to editor-model** — `ChangeSet`, `Applier`, `ApprovalPolicy`, `TransactionKernel`, `ApplyReceipt`, `KernelError`, `EffectsSummary`, `DiffSummary`, `ValidationReport`, `ResourceRef` moved from `editor-application` to `editor-model/src/transaction.rs`.

**Rationale**: This breaks the `editor-core → editor-application` circular dependency that blocked the ADR-0031 EditorSession migration. The kernel mechanics are pure domain logic with no Bevy/WASM dependencies, so they belong in `editor-model` (the bottom of the dependency chain). `editor-application` now imports `editor_model::transaction` types for use in its `EditorSession` and `TransactionBridge`.

**Status**: Ratified by owner — 2026-08-16.
