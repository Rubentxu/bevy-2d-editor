# Specification — Runtime Apply-Back

## Goal

Allow deliberate persistence of useful play-mode tuning while rejecting transient game state.

## Authorability metadata

Fields may declare:

- `apply_back = never`;
- `apply_back = tunable`;
- `apply_back = explicit_only`.

Default for unknown fields: `never` until schema metadata or adapter semantics prove safety.

## Capture

On play start, capture authoring baseline for eligible fields. During play, runtime adapter records changed eligible values with provenance.

## Stop workflow

```text
Runtime changes detected

Player.speed       250 → 300   [selected]
Camera.zoom        1.0 → 1.15  [selected]
Enemy.health       100 → 80    [not authorable: runtime gameplay state]
```

For each selected change, choose/derive target scope:

- this instance;
- source Scene Asset;
- schema/default.

Then create a normal `ChangeSet` and pass through validation/workbench.

## Acceptance

- physics-generated Transform drift is not persisted by default;
- explicit tuning fields can be applied and undone;
- scope changes correctly affect instances/overrides;
- exiting play without approval restores baseline authoring state.
