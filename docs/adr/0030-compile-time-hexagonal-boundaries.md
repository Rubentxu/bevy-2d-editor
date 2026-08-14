# ADR-0030: Compile-Time Hexagonal Crate Boundaries

## Status

Accepted — 2026-08-14


## Context

The conceptual architecture already distinguishes domain, preview, storage and UI, but `editor-core` allows them to import each other because they share a crate. Convention is no longer sufficient at the current project size.

## Decision

Create compile-time boundaries around at least:

- `editor-model`;
- `editor-application`;
- `editor-bevy`;
- `editor-storage-web`;
- `editor-wasm`;
- `editor-protocol`.

Additional adapters such as filesystem storage are introduced when their execution environment exists.

Dependency direction is inward: adapters depend on application/model; application does not depend on adapters.

## Considered options

### Keep one crate with modules
Rejected as the long-term target because imports do not enforce architecture.

### Split every bounded context into a crate immediately
Rejected: excessive fragmentation and migration cost.

### Minimal strategic crate split
Accepted: enforce important boundaries while keeping local module freedom.

## Consequences

- native unit tests become cheaper;
- Bevy/WASM compile cost is removed from pure model tests;
- cyclic dependency pressure reveals missing ports;
- migration requires temporary re-exports and careful PR sequencing.
