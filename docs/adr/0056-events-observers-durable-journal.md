# ADR-0056 — Bevy Events/Observers Are Ephemeral Runtime Signals; Change Journal Is Durable

**Status:** Proposed  
**Date:** 2026-08-19

## Context

The system needs both fast runtime notification and durable/replayable change history. Treating these as one mechanism would either over-persist runtime noise or make audit/history unreliable.

## Decision

Use Bevy Events/Observers/change detection for runtime communication inside editor/preview runtimes. Persist semantic ChangeSet/transaction records in the durable journal. A durable semantic change may emit runtime events; runtime events do not automatically become journal records.

## Considered Options

1. Event-source all Bevy events.
2. Build a custom global event bus for all communication.
3. Use only polling.

## Consequences

- Clear event semantics.
- Avoids custom bus duplication.
- Durable history stays deterministic.
- Correlation IDs are required across mechanisms.

## Architecture Guardrails

- preserve stable semantic identity;
- preserve Transaction Kernel ownership of authoring mutations;
- keep generated/derived runtime state rebuildable;
- add architecture fitness checks before relying on convention;
- migration must be incremental and covered by UAT.

## References

- ADR-0030 — Compile-Time Hexagonal Crate Boundaries
- ADR-0032 — Shared Transaction Kernel and ChangeSet
- ADR-0034 — Typed EditorBackend Contract
- ADR-0036 — Runtime Preview Adapter
- ADR-0046 — Semantic Editor Model Authority
- ADR-0047 — Logic Graph Model Split
