# ADR-0063 — Frontend Feature Slices and No Direct Bridge from UI Components

Status: Proposed

## Context

Frontend coordination has been decomposed from `App.tsx`, but broad hooks and prop bags still couple unrelated features. Some components can invoke bridge commands directly.

## Decision

Organize product behavior by feature slices. UI components call feature/application capability APIs or callbacks, never raw backend bridge names.

`App`/shell are composition/layout concerns. Feature slices own their controllers and command contributions.

No new state-management framework is mandated. Introduce one only when measured React state propagation requires it.

## Consequences

- smaller contracts;
- easier feature tests;
- less central branching;
- migration may temporarily duplicate adapters while features are moved one at a time.

