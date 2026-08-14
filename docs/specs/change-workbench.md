# Specification — Change Workbench

## Goal

Create one trusted surface for reviewing non-trivial changes regardless of origin.

## Supported origins

- human bulk/refactor actions;
- recipe;
- AI agent;
- import/reimport;
- migration;
- plugin/extension;
- runtime apply-back.

## Layout

### Intent header
Origin, actor, rationale, risk and approval requirement.

### Resource tree
Affected scenes/assets/worlds/logic/source files.

### Semantic diff
Show domain concepts rather than raw JSON when possible:

```text
Player Scene Asset
  Transform2D.scale: 1.0 → 1.2
  Health.max: 100 → 120

Level forest
  17 instances affected
  3 instances keep explicit Health.max override
```

### Scope control

```text
This instance
Selected instances
All compatible instances in level
Scene Asset definition
Component/schema default
Rust source definition (when capability supports it)
```

### Validation
Errors, warnings, conflicts and expected post-apply resolution.

### Effects
Preview rebuild, hot reload, reindex, compile/build required, possible restart.

## Actions

- approve all;
- approve selected operation groups;
- reject;
- regenerate/retry source (agent/import-specific);
- create checkpoint then apply;
- rollback last applied ChangeSet.

## Invariants

No UI “approve” action bypasses Transaction Kernel validation. Partial apply must be revalidated as a new normalized ChangeSet.
