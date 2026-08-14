# Specification — Scope of Change

## Goal

Make it explicit whether an edit targets one occurrence, several occurrences, a reusable definition or a default/schema/source definition.

## Supported scopes

Depending on capability and selection:

- `ThisInstance`;
- `SelectedInstances`;
- `AllCompatibleInstancesInLevel`;
- `SceneAssetDefinition`;
- `ComponentDefault`;
- `SourceDefinition` when a code-aware capability supports a safe mapping.

## UX contract

For a field with definition/override semantics, the Inspector can show a scope chooser before or during edit. The chosen scope must be visible in Change Workbench for broad edits.

Example:

```text
Speed: 250 → 300

Apply to:
● This Player instance
○ 8 selected Player instances
○ Scene Asset: characters/player
○ Component default: game.Movement.speed
```

## Override semantics

When definition changes affect instances:

- active explicit overrides remain unless user selects an operation to clear/promote them;
- affected override counts are shown;
- stale/conflict outcomes are part of preflight validation.

## Bulk safety

If a scope expands the affected resource/entity count above a configurable threshold, approval changes from low-risk auto apply to Change Workbench review.

## Agent rule

Agent tools must specify intended scope explicitly; ambiguous “change player speed” plans are invalid until the scope is resolved from context or surfaced for review.
