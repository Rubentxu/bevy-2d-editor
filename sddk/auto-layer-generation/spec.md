# Spec: Auto-Layer Generation

> Change: `auto-layer-generation` · Phase: sddk-spec · Path: A-full · Mode: auto
> Source: [`proposal.md`](./proposal.md)

## §1. Spec Metadata

- **Capabilities:**
  - **NEW**: `auto-layer` — `AutoLayer` LevelLayer variant derived from a `TileLayer`, 3×3 rule engine, `RegenerateAutoLayer` command, stale detection, and frontend rule editor

## §2. NEW Capability: `auto-layer`

### Requirement: AutoLayer LevelLayer Variant

An `AutoLayer` SHALL be a `LevelLayer` variant holding a `source_layer_id: LayerId` referencing a `TileLayer`, a `tileset_id`, an ordered `Vec<AutoRule>`, and a cached generated `TileGrid`. The cached grid SHALL be derived and MUST NOT be writable through the tile brush. Serialization MUST round-trip rules and cached grid losslessly.

#### Scenario: AL1 — Add AutoLayer referencing a TileLayer
- GIVEN a `LevelSceneAsset` containing a `TileLayer`
- WHEN a user adds an `AutoLayer` referencing that `TileLayer` by `LayerId`
- THEN the `LevelSceneAsset` MUST persist the new `AutoLayer` with empty rules and an empty cached grid

#### Scenario: AL2 — Reject AutoLayer referencing a non-TileLayer
- GIVEN a `LevelSceneAsset` containing a `SceneInstance` layer
- WHEN a user attempts to add an `AutoLayer` referencing that layer
- THEN the editor MUST reject the addition with a reference-validation error
- AND the `LevelSceneAsset` MUST remain unchanged

#### Scenario: AL3 — Cached grid is read-only via brush
- GIVEN an `AutoLayer` with a non-empty cached grid
- WHEN the user activates the tile brush on that layer
- THEN the brush MUST refuse to write to the cached grid
- AND the layer panel MUST render the layer as auto-generated

#### Scenario: AL4 — Round-trip preserves rules and cache
- GIVEN an `AutoLayer` with non-empty rules and a populated cached grid
- WHEN the editor serializes and deserializes the `LevelSceneAsset`
- THEN the `AutoLayer` MUST reconstruct with identical rules and cached grid contents

### Requirement: 3×3 Rule Engine

An `AutoRule` SHALL define a `Pattern3x3` matched against the source `TileLayer` neighborhood, a `Vec<TileRef>` of output tiles, and an optional `chance` weight in `[0.0, 1.0]`. The engine MUST evaluate each source cell in declared rule order and emit the first matching rule's output. Cells with no match MUST be cleared in the cached grid.

#### Scenario: RE1 — First matching rule wins
- GIVEN two rules R1 and R2 whose `Pattern3x3` both match a given neighborhood
- WHEN the engine processes that source cell
- THEN the cached grid MUST contain R1's output
- AND R2 MUST NOT be evaluated for that cell

#### Scenario: RE2 — Chance gates emission
- GIVEN a rule with `chance: 0.5` whose pattern matches
- WHEN the engine processes the matching source cell
- THEN the rule's output MUST be emitted with approximate probability 0.5 over many runs
- AND non-firing cells MUST be cleared in the cached grid

#### Scenario: RE3 — Empty rule set clears cache
- GIVEN an `AutoLayer` with no rules and a populated cached grid
- WHEN `RegenerateAutoLayer` runs
- THEN the cached grid MUST be empty

### Requirement: Regenerate Command

The editor SHALL provide a `RegenerateAutoLayer { layer_id }` command that recomputes the cached grid from the source `TileLayer` and current rules. The command MUST be recorded in the `OperationLog` so undo restores the previous cached grid and redo reapplies the new one. The source `TileLayer` MUST NOT be modified.

#### Scenario: RG1 — Regenerate writes derived tiles
- GIVEN an `AutoLayer` with a rule mapping a solid neighborhood to a wall tile
- WHEN the user invokes `RegenerateAutoLayer`
- THEN the cached grid MUST contain the wall tile at every source cell whose 3×3 matches
- AND the source `TileLayer` grid MUST remain unchanged

#### Scenario: RG2 — Undo restores previous cache
- GIVEN an `AutoLayer` whose cached grid changed from C1 to C2 after regeneration
- WHEN the user undoes the regenerate command
- THEN the cached grid MUST equal C1
- AND the source `TileLayer` MUST remain unchanged

#### Scenario: RG3 — Reject when source is missing
- GIVEN an `AutoLayer` whose `source_layer_id` does not resolve to a `TileLayer`
- WHEN the user invokes `RegenerateAutoLayer`
- THEN the command MUST be rejected with a reference-validation error
- AND the cached grid MUST remain unchanged

### Requirement: Stale Detection

The editor MUST mark an `AutoLayer` as stale when its source `TileLayer` changes after the last regeneration. The stale indicator SHALL be visible in the layer panel until the next successful regeneration clears it.

#### Scenario: SD1 — Source edit marks AutoLayer stale
- GIVEN a regenerated `AutoLayer` whose cache is in sync with its source
- WHEN the user paints or erases a tile on the source `TileLayer`
- THEN the `AutoLayer` MUST be marked stale in the layer panel

#### Scenario: SD2 — Successful regenerate clears stale
- GIVEN a stale `AutoLayer`
- WHEN `RegenerateAutoLayer` completes successfully
- THEN the stale indicator MUST be cleared
- AND the layer panel MUST render the layer as in-sync

### Requirement: Rule Editor UI

The frontend SHALL provide an `AutoLayerPanel` exposing the rule list with a 3×3 pattern grid, an output tile picker bound to the `AutoLayer`'s `tileset_id`, and an optional `chance` slider. Edits MUST persist into the `LevelSceneAsset` without a manual reload.

#### Scenario: UI1 — Edit pattern and output
- GIVEN an `AutoLayer` selected in the layer panel
- WHEN the user toggles cells in the 3×3 grid and picks an output tile
- THEN the corresponding `AutoRule` MUST be persisted into the `LevelSceneAsset`
- AND the rule list MUST reflect the edit immediately

#### Scenario: UI2 — Trigger regenerate from the panel
- GIVEN a stale or in-sync `AutoLayer`
- WHEN the user clicks the Regenerate button in `AutoLayerPanel`
- THEN `RegenerateAutoLayer` MUST be dispatched
- AND the cached grid preview MUST update on completion