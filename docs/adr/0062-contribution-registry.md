# ADR-0062 — UI and Extension Features Register Typed Contributions

**Status:** Proposed  
**Date:** 2026-08-19

## Context

Top-level React composition and inspector logic can grow into large switchboards as workspaces, panels, asset editors, graph nodes and extensions expand.

## Decision

Introduce typed contribution registries for commands, panels, inspector sections, workspace tools, asset editors, graph node renderers, menus and status items. Contributions declare capabilities and supported subject kinds. This is compatible with the Extension SDK without becoming an unrestricted plugin loader.

## Considered Options

1. Continue hardcoding feature imports/booleans in App/Inspector.
2. Allow extensions to mutate arbitrary React trees.
3. Create a remote microfrontend system.

## Consequences

- Modular frontend growth.
- Cleaner extension boundary.
- Improved test isolation.
- Needs deterministic ordering and collision/version rules.

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
