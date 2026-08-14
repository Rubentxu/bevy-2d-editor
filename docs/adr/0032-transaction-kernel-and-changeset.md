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
