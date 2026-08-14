# Specification — Transaction Kernel & ChangeSet

## Goal

Provide one safe mutation pipeline for direct editing, bulk operations, recipes, agents, import/reimport, migrations and runtime apply-back.

## Data model

```rust
pub struct ChangeSet {
    pub id: ChangeId,
    pub origin: ChangeOrigin,
    pub actor: Actor,
    pub rationale: Option<String>,
    pub operations: Vec<TypedOperation>,
    pub affected: Vec<ResourceRef>,
    pub preflight: ValidationReport,
    pub semantic_diff: SemanticDiff,
    pub effects: ChangeEffects,
    pub approval: ApprovalPolicy,
}
```

`TypedOperation` is an envelope around domain-specific commands, not a field-level generic patch language.

## Pipeline

```text
Build → Normalize → Validate → Preview Diff → Approve → Apply → Verify → Commit History
                                                    ↘ failure → rollback/compensate
```

## Requirements

### TX-1 Semantic operations
Operations express domain intent.

### TX-2 Pre-state
Every reversible operation captures enough pre-state to generate inverse behavior.

### TX-3 Atomic local batches
A batch against one aggregate/document is atomic.

### TX-4 Multi-resource strategy
Each capability declares atomic or compensating behavior before apply.

### TX-5 Origin/provenance
All changes record origin and actor.

### TX-6 Approval
Policies:

- `AutoApproveLowRisk`;
- `ReviewRequired`;
- `HumanMandatory`;
- `Forbidden`.

### TX-7 Effects
`ChangeEffects` identifies what must refresh/rebuild/reindex/reload.

### TX-8 Checkpoints
Users/tools can create named checkpoints around risky workflows.

## Example

“Extract selected entities as Scene Asset” may contain:

1. create Scene Asset;
2. copy semantic entities to LocalId space;
3. insert Scene Instance in source scene;
4. preserve transform/placement;
5. validate references;
6. update project catalog;
7. request preview refresh and search reindex.

This is one user action and one reviewable `ChangeSet`.
