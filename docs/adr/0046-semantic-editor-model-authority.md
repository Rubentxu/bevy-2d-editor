# ADR-0046: Semantic Editor Model Is the Authoritative Source of Truth

> **Renumbering note:** Originally numbered ADR-0029 in the evolution pack. Renumbered to ADR-0046 to avoid collision with the repository's existing ADR-0029 (Frontend Performance Budget Contract, Accepted 2026-08-02). No other changes were made.

## Status

Accepted — 2026-08-14


## Relationship to existing ADRs

**Supersedes ADR-0001 in source-of-truth semantics.** ADR-0001 remains historically valid in rejecting `DynamicScene`/runtime entities as authoring authority and in choosing an editor-owned representation. The new decision is that **JSON itself is no longer the authority; the semantic editor model is**.

## Context

The project now has several representations of the same authoring concepts: editor JSON documents, BSN IR/text, generated Rust, preview Bevy ECS state and frontend DTOs. Treating one serialization syntax as “the truth” risks coupling domain evolution to file format evolution, especially as Bevy's BSN/editor support matures.

## Decision

The authoritative state is the normalized semantic editor model (`SceneDocument`, `SceneAssetDocument`, instances, schemas, logic graphs, world metadata and related typed values). JSON and BSN are adapters/encodings of that model.

A loaded artifact may also retain representation metadata required for lossless or minimally destructive write-back, but representation metadata never changes semantic invariants.

```text
JSON ───────┐
BSN ────────┼──> Semantic Editor Model ───> Commands/Capabilities
Imports ────┘             │
                          ├──> JSON writer
                          ├──> BSN writer
                          └──> Bevy runtime projection
```

## Rules

1. Domain types cannot expose JSON-specific semantics merely because JSON is convenient.
2. Unknown extension data may be preserved in an extension bag when necessary for forward compatibility.
3. Every representation adapter declares fidelity: lossless, semantic-lossless, or lossy/export-only.
4. Migrations operate on semantic versions and typed structures, not arbitrary string replacements.
5. Runtime state is never promoted to authoring authority implicitly.

## Considered options

### Keep JSON as the authority
Rejected: overly constrains BSN-native/file-native evolution.

### Make BSN the sole authority
Rejected today: editor-only metadata, logic/world concepts and Bevy API evolution still require an editor-owned semantic layer.

### Semantic model authority with adapters
Accepted: preserves existing strengths while allowing JSON and BSN to evolve independently.

## Consequences

- existing JSON remains fully supported during migration;
- ADR-0001 status should point to this ADR;
- adapter contract/golden tests become mandatory;
- the future project format can use BSN where it is the best native representation without forcing every editor concept into BSN.
