# ADR-0062 — BSN Anti-Corruption Layer

Status: Proposed

## Context

The editor must interoperate closely with Bevy scene/BSN evolution while preserving editor-specific identities, metadata, overrides, workflows and forward compatibility.

## Decision

The editor semantic model remains independent. BSN/Bevy compatibility is mediated through an explicit IR/codec adapter layer.

Bevy-specific changes should primarily affect adapter code and contract fixtures, not force the domain model to mirror every Bevy syntax/API detail.

## Consequences

- Bevy upgrades are less invasive;
- round-trip compatibility is measurable;
- unsupported constructs can produce precise diagnostics;
- some duplication between semantic model and IR is accepted as an anti-corruption cost.

## Revisit

If Bevy later exposes a stable semantic scene AST that fully matches editor requirements, reassess whether the intermediate IR can be simplified.

