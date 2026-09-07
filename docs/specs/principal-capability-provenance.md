# Specification — Typed Principal, Capability and Provenance

## Goal

Remove authorization semantics that depend on actor string prefixes while keeping wire/backward compatibility.

## Domain/application types

Illustrative shape:

```rust
enum Principal {
    Human(UserId),
    Agent(AgentId),
    Extension(ExtensionId),
    Importer(ImporterId),
    Runtime,
    System,
}

enum MutationCapability {
    SceneRead,
    SceneWrite,
    AssetRead,
    AssetWrite,
    LogicWrite,
    SourceWrite,
    ProjectRead,
    RuntimeApplyBack,
    ProposeChanges,
}
```

Exact capability granularity is a checkpoint decision.

## Compatibility boundary

Legacy serialized inputs such as:

```text
actor = "extension:foo"
actor = "importer:bar"
```

are parsed at the protocol/WASM boundary into typed values. Internal policy code does not repeatedly parse prefixes.

## Authorization

Policy receives:

```text
Principal + requested capability + resource refs + ChangeSet metadata
```

and returns a typed decision with reason.

## Auditability

Receipts/history preserve enough provenance to answer:

- who/what proposed it;
- who approved it;
- which resources changed;
- which policy allowed it;
- which validation ran;
- resulting revision/change id.

## UAT

Attempt same denied extension mutation through all exposed entry points. All must produce semantically equivalent permission denial.

