# ADR-0064 — Scene Asset Variants and Overrides Use Explicit Provenance

**Status:** Proposed  
**Date:** 2026-08-19

## Context

Reusable Scene Assets and instance overrides become more powerful when designers can create reusable variants and understand where each effective value comes from.

## Decision

Add explicit lineage `Base SceneAsset -> Variant SceneAsset -> Instance -> local overrides`. Effective property resolution returns value plus provenance. Conflicts/stale overrides after base changes are explicit. Variants/overrides remain semantic data and participate in ChangeSet, impact, migration and UAT.

## Considered Options

1. Only base SceneAsset + per-instance overrides.
2. Copy assets to create variants with no lineage.
3. Infer inheritance from file paths.

## Consequences

- Better reusable authoring.
- Provenance improves UX/causality.
- Inheritance/conflicts add complexity.
- Requires cycle prevention and migration rules.

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
