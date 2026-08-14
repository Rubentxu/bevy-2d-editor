# Specification — Semantic Editor Model

## Purpose

Define the representation-independent domain model that all persistence, runtime and automation adapters consume.

## Requirements

### SEM-1 Authority

The semantic model is authoritative after parse/import and before write/export.

### SEM-2 Identity

Persistent resource/entity identities are typed and stable. Runtime Bevy IDs are mappings only.

### SEM-3 Extension preservation

Formats may attach representation/extension bags needed for forward-compatible round-trip. Unknown data must not silently disappear when an adapter claims semantic-lossless fidelity.

### SEM-4 Determinism

Equal semantic state serializes deterministically under a specific writer/version.

### SEM-5 Migration

Every document type declares a format version. Migrations are pure where practical:

```rust
fn migrate(input: Vn) -> Result<VnPlus1, MigrationError>
```

### SEM-6 Fidelity contracts

Each adapter reports one:

- `Lossless`;
- `SemanticLossless`;
- `ExportOnlyLossy`.

## Core model families

- Project metadata;
- SceneDocument;
- SceneAssetDocument;
- SceneInstance / overrides;
- component schemas/instances;
- Level layers;
- WorldDocument;
- LogicGraphAsset;
- ExternalSource provenance;
- ChangeSet/history metadata references.

## Acceptance scenarios

1. JSON → model → JSON produces deterministic equivalent output.
2. BSN → model → BSN preserves all supported semantic elements.
3. unsupported BSN syntax produces explicit fidelity/unsupported issues, not silent loss.
4. Bevy preview rebuild does not change serialized authoring model.
5. migration corpus upgrades old documents deterministically.
