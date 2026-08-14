# ADR-0036: Bevy Runtime Preview Is an Ephemeral Projection Adapter

## Status

Accepted — 2026-08-14


## Context

The current design already treats editor data as primary, but runtime metrics, play mode and future apply-back make the boundary increasingly important.

## Decision

The Bevy world is a **projection** of authoring state. Runtime entity IDs never become durable project references.

The runtime adapter provides:

- editor ID ↔ runtime entity mapping;
- projection provenance;
- preview metrics;
- logic activation/causality events;
- rebuild/hot-reload reason;
- capture of authorable runtime deltas.

Runtime changes are discarded by default when play mode stops. Persistence requires ADR-0042's explicit apply-back flow.

## Consequences

Runtime debugging can become rich without weakening document identity or allowing accidental transient-state persistence.
