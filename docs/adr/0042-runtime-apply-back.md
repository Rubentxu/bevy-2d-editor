# ADR-0042: Runtime Apply-Back Is Explicit, Scoped and Authorable-Field Only

## Status

Accepted — 2026-08-14


## Context

Play mode tuning is productive, but most runtime state is transient. Blindly persisting runtime ECS values would capture physics, timers and temporary gameplay state.

## Decision

Capture runtime differences as `RuntimeDelta` records tagged with component/field provenance. Only fields declared **authorable/apply-back eligible** can be proposed for persistence.

On exit or explicit capture, the user may apply selected changes to one of these supported scopes:

- this instance;
- source Scene Asset;
- component/schema default when semantically allowed.

The result is a normal reviewed `ChangeSet` with undo.

## Consequences

The editor gains Unity-like play tuning while improving safety and scope visibility.
