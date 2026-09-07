# ADR-0059 — Single Transaction/Mutation Dispatch Path

Status: Proposed

## Context

A legacy direct command route and TransactionKernel route coexist. This was useful as a migration rollback mechanism but creates duplicated mutation semantics if retained indefinitely.

## Decision

TransactionKernel/application command dispatch becomes the sole normal mutation path before v1.0.

Scene, Asset and Logic commands remain distinct domain languages. The decision does **not** introduce one generic command enum.

Human, agent, plugin, importer and runtime apply-back operations must enter through the same policy/attribution architecture appropriate to their risk.

## Migration

1. Characterize both paths with parity fixtures.
2. Route all production callers through kernel.
3. Run dual-path comparison in tests where useful.
4. Remove runtime dispatch toggle.
5. Delete legacy implementation after evidence.

## Consequences

- one place for validation/history/effects/policy;
- fewer semantic divergences;
- rollback becomes git/release rollback rather than permanent runtime duality.

