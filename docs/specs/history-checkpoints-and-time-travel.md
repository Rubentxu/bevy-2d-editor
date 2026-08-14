# Specification — History, Checkpoints and Time-Travel Authoring

## Goal

Evolve undo/redo from a button-level feature into a safe experimentation and agent/import workflow primitive.

## Model

The authoritative state remains the current semantic documents. History stores reversible operation metadata and checkpoints; this is **not event sourcing**.

## Capabilities

### Named checkpoints

Examples:

- `before-agent-enemy-rebalance`;
- `before-ldtk-reimport`;
- `play-tuning-baseline`.

### Timeline

Show meaningful ChangeSets:

```text
10:42 Create Player Scene Asset
10:46 Apply Platformer Character recipe
10:51 Paint Forest room
11:03 Reimport player.aseprite
11:07 Agent: rebalance movement
```

### Inspect historical diff

Selecting a timeline item shows affected resources and semantic diff without mutating current state.

### Restore

Restoring to a checkpoint itself creates a new ChangeSet containing the required inverses/restore operations; it does not delete audit history.

### Experimental branch (future)

The Transaction Kernel should not prevent future temporary branches/sandboxes, but v1 can rely on named checkpoints and Git branches rather than implementing a full internal VCS.

## Retention

History persistence is bounded/configurable. Project-authored state must not depend on an unbounded local operational log being present.
