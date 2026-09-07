# UAT — Canonical v1 Game Creation

## Goal

Prove the actual product, not only fixture parsers.

Use `examples/platformer-minimal` as a starting reference or create a successor fixture designed for UI authoring evidence.

## UAT-GAME-001 — Fresh project

1. Start with empty project/storage.
2. Create the main scene/level through UI.
3. Save and reload.

Expected: no manual editor JSON edits required.

## UAT-GAME-002 — Reusable player actor

1. Create Scene Asset actor.
2. Add required transform/sprite/gameplay components.
3. Save.
4. Place instance in main scene.

Expected: definition and instance are clearly distinguishable.

## UAT-GAME-003 — Multiple instances + override

1. Create enemy actor.
2. Place at least two instances.
3. Override one field on one instance.
4. Change definition.
5. Resync.

Expected: override remains understandable and validation reports conflict if one is created.

## UAT-GAME-004 — Logic gameplay

1. Create/open logic graph from scene/instance context.
2. Connect sensor/controller/actuator nodes for a simple behavior.
3. Save and return to scene.

Expected: binding is visible and navigable.

## UAT-GAME-005 — Level tools

Author a representative level area with tile/auto-layer tools included in v1 promise.

Expected: undo grouping, save/reopen and regeneration state are correct.

## UAT-GAME-006 — External asset import

Import one supported source workflow, inspect provenance, then reimport a changed source.

Expected: update is previewed/reviewed according to policy and no unrelated edits are lost.

## UAT-GAME-007 — Play/debug

1. Enter play.
2. Exercise behavior.
3. Inspect runtime diagnostics/logic activation.
4. Stop.

Expected: authoring state remains intact and diagnostics navigate back to durable sources.

## UAT-GAME-008 — Apply-back

Where feature is supported:

1. Modify tunable runtime value.
2. Exit play.
3. Review runtime delta.
4. Apply selected change.
5. Undo.

Expected: reviewed reversible authoring mutation.

## UAT-GAME-009 — BSN/export

Export supported actor/scene path.

Expected: deterministic validated output matching declared compatibility.

## UAT-GAME-010 — Recovery

Inject one save/import failure during the project workflow.

Expected: user can recover without recreating project; diagnostics explain state.

## UAT-GAME-011 — Keyboard critical path

Repeat a representative subset using keyboard for navigation, entity operations, inspector edit, save, search and play.

## UAT-GAME-012 — Close/reopen project

Cold restart after all work.

Expected: complete game project reconstructs consistently and can play again.

