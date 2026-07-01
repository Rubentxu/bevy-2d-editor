# Spec: Level Inspector & Override Panel

> Change: `level-inspector-and-override-panel` · Phase: sddk-spec · Path: A-full · Mode: auto
> Source: [`proposal.md`](./proposal.md) · [`explore-report.md`](./explore-report.md)

## §1. Spec Metadata

- **Capabilities:**
  - **NEW**: `override-crud` — write-side WASM surface for override mutation
  - **MODIFIED**: `inspector-panel` — effective-values rendering + per-field indicators
  - **MODIFIED**: `scene-instance-overrides` — pure mutation helpers added
- **Authoritative refs:**
  - [ADR-0005 §Overrides / §Versioning](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md)
  - [ADR-0009 — Component Override Model](../../adr/0009-component-override-model.md)
  - [spec: scene-instance-overrides](../scene-instance-overrides/spec.md)
  - [spec: ui-panels §inspector-panel](../ui-panels/spec.md)

---

## §2. NEW Capability: `override-crud`

Write-side WASM surface for override mutation: `UpsertOverride` / `RevertOverride` Command variants, processor apply/inverse, pure helpers, undo/redo through the shared `OperationLog`.

### Requirement: UpsertOverride command writes one field override

`Command::UpsertOverride { instance_id, target_local_id, component_type_id, field_path, value }` MUST insert a new override or replace an existing entry matching `(target_local_id, component_type_id, field_path)`. The inverse MUST restore the previous value (or `RevertOverride` if none existed). MUST integrate with `OperationLog`.

#### Scenario: S1 — Upsert inserts into empty overrides

- GIVEN instance with empty `component_overrides`
- WHEN `Command::UpsertOverride` is applied
- THEN `component_overrides.len()` equals `1`
- AND the inverse is `Command::RevertOverride`

#### Scenario: S2 — Upsert replaces a same-key override

- GIVEN instance with one override on `(root, editor.Sprite2D, ["asset"]) = "cannon.png"`
- WHEN `UpsertOverride` with same key and `value: "enemy.png"` is applied
- THEN the entry's `value` equals `"enemy.png"` AND total length stays `1`

### Requirement: RevertOverride is idempotent

`Command::RevertOverride { instance_id, target_local_id, component_type_id, field_path }` MUST remove a matching override. If no match exists, MUST be a no-op returning `Ok` (not error). Inverse MUST re-insert the removed patch.

#### Scenario: S3 — Revert removes the matching override

- GIVEN instance with one override on field `asset`
- WHEN `Command::RevertOverride` is applied for that key
- THEN `component_overrides` is empty
- AND inverse restores the override on undo

#### Scenario: S4 — Revert of absent override is no-op

- GIVEN instance with empty `component_overrides`
- WHEN `Command::RevertOverride` is dispatched
- THEN command returns `Ok` AND overrides remain empty

### Requirement: WASM bridge dispatches via OperationLog

`upsert_override_wasm(...)` and `revert_override_wasm(...)` MUST dispatch through the shared `OperationLog` and return `CommandResult` JSON. Round-trip `upsert → revert` MUST restore the asset value.

#### Scenario: S5 — Upsert → revert round-trip

- GIVEN asset field value `"player.png"`
- WHEN `upsert_override_wasm` sets `"cannon.png"`, then `revert_override_wasm`
- THEN `effective_values_wasm` returns `"player.png"` AND overrides are empty

---

## §3. MODIFIED Capability: `inspector-panel`

### MODIFIED Requirements

### Requirement: Inspector shows entity name and components

When an entity is selected, the Inspector MUST show the entity name (editable) and all its components. When the selected entity belongs to a placed Scene Instance, the Inspector MUST render **effective (merged) component values** from `effective_values_wasm` with **per-field override indicators** matching each override's `status` (Active=blue, Stale=warning, Conflict=error, Orphaned=dimmed). Non-instance entities continue to render raw component values unchanged.
(Previously: Inspector rendered raw asset components uniformly for all entity types with no per-field override indicator.)

#### Scenario: Inspector shows selected entity

- GIVEN entity "Player" with components [Name, Transform2D]
- WHEN user selects Player (non-instance entity)
- THEN Inspector shows name="Player" (editable) and 2 components (raw values)

#### Scenario: Empty selection shows placeholder

- GIVEN no entity selected
- WHEN Inspector renders
- THEN "Select an entity" placeholder is shown

#### Scenario: S6 — Instance entity shows effective value + blue indicator

- GIVEN instance overrides `editor.Sprite2D.asset` to `"cannon.png"`
- WHEN the instance's child entity is selected in Hierarchy
- THEN field shows `"cannon.png"` AND a blue override indicator is rendered

#### Scenario: S7 — Stale and Conflict override colors

- GIVEN instance with one Stale and one Conflict override
- WHEN instance is selected
- THEN Stale field shows warning indicator AND Conflict field shows error indicator

### ADDED Requirements

### Requirement: Override count badge on Inspector header

The Inspector MUST show an "Overrides" section with per-status counts (active, stale, orphaned, conflict) when a Scene Instance entity is selected.

#### Scenario: S8 — Override counts render

- GIVEN instance with 2 active, 1 stale, 1 orphaned overrides
- WHEN instance is selected
- THEN "Overrides" section shows those counts

### Requirement: Per-field revert affordance

Each overridden field MUST show a revert affordance. Clicking MUST dispatch `Command::RevertOverride` for that field's key.

#### Scenario: S9 — Revert affordance removes override

- GIVEN instance entity selected with active override on field `asset`
- WHEN user clicks the revert affordance on that field
- THEN `Command::RevertOverride` is dispatched AND the field value reverts to the asset value

### Requirement: Resync warning surfaces stale overrides

When `get_resync_reports` reports non-zero `stale` or `conflict` counts for the selected instance, the Inspector MUST show a warning banner with the count and a button to open the Override / Resync Workbench.

#### Scenario: S10 — Stale-override banner with action

- GIVEN resync report shows 2 stale overrides for the selected instance
- WHEN instance is selected
- THEN Inspector shows banner "2 overrides need review" with workbench button

---

## §4. MODIFIED Capability: `scene-instance-overrides`

### ADDED Requirements

### Requirement: `upsert_override` mutates a SceneInstance

`upsert_override(&mut instance, patch)` MUST replace an existing override matching `(target_local_id, component_type_id, field_path)` and otherwise append. MUST NOT mutate `id_map` or `instance_components`. After the call the new patch's `status` MUST be `Active`.

#### Scenario: S11 — Upsert appends to empty overrides

- GIVEN instance with empty `component_overrides`
- WHEN `upsert_override(&mut instance, patch)` runs
- THEN `component_overrides.len()` equals `1` AND the appended patch's `status` is `Active`

### Requirement: `remove_override` returns the removed patch

`remove_override(&mut instance, target_local_id, component_type_id, field_path) -> Option<ComponentOverride>` MUST remove and return the matching override, or `None` if absent. MUST be idempotent.

#### Scenario: S12 — Remove returns the captured patch

- GIVEN instance with one override `(root, editor.Sprite2D, [asset]) = "x"`
- WHEN `remove_override(...)` runs
- THEN `component_overrides` is empty AND the returned `Some(patch)` has `value == "x"`

#### Scenario: S13 — Remove of absent override returns None

- GIVEN instance with empty `component_overrides`
- WHEN `remove_override(...)` runs
- THEN the function returns `None` AND state is unchanged

---

## §5. Out-of-Scope Behaviors

1. Inline value editing to CREATE a new override on an unoverridden field (follow-up slice).
2. Side-by-side asset-vs-override comparison view (Workbench v2).
3. Layer-aware entity tree for Level Scene Assets (separate change).
4. Batch apply-all / revert-all overrides.
5. Push-to-asset workflow ("Apply" on overrides — ADR candidate).

---

## §6. Acceptance Criteria

1. New `Command::UpsertOverride` and `Command::RevertOverride` variants in `command.rs`; `processor.rs` apply/inverse passes S1–S4.
2. New pure helpers `upsert_override`, `remove_override` in `scene_instance_overrides.rs`; S11–S13 pass.
3. WASM bindings `upsert_override_wasm`, `revert_override_wasm` in `lib.rs`; S5 passes.
4. `InspectorPanel` renders effective values + per-field override indicators when a Scene Instance entity is selected; S6–S10 pass.
5. Override mutations are undoable via Ctrl+Z (forward/inverse pair in `OperationLog`).
6. All existing 112+ Rust and 27+ Playwright tests pass (no regression).

---

## §7. Open Questions for Design

1. **Field-path matching in TS**: nested override paths (e.g. `["editor.Sprite2D","transform","translation","x"]`) vs flat widget paths — design must define the suffix-matching algorithm.
2. **Resync poll cadence**: poll `get_resync_reports` on selection only vs on every mutation — design must pick.
3. **TS field editor integration**: `ComponentEditor` is type-aware but currently unaware of override state — design must add the override-status prop and revert affordance wiring.