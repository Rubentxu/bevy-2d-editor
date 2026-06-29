# Spec: Scene Instance Placement (Hito 2)

> Change: `scene-instance-placement` · Phase: sddk-spec · Path: A-full
> Source proposal: [`./proposal.md`](./proposal.md) · Source explore: [`./explore-report.md`](./explore-report.md)
> Authoritative references: [ADR-0005 §Overrides/§Versioning](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md), [ADR-0006 §Normative Rules](../../adr/0006-authoring-first-roadmap-after-bsn-migration.md), [ADR-0003 §serde forward-compat](../../adr/0003-forward-compat-via-serde-json-value.md), [Spec: scene-instance-overrides](../scene-instance-overrides/spec.md), [Spec: project-asset-browser-and-scene-asset-authoring](../project-asset-browser-and-scene-asset-authoring/spec.md), [Spec: scene-document](../scene-document/spec.md), [Spec: command-system](../command-system/spec.md), [Roadmap Capability 2](../../specs/post-bsn-authoring-roadmap.md).

---

## §1. Capability Delta Table

| Capability | Status | Summary of change |
|------------|--------|-------------------|
| `scene-instance-placement` | **NEW** | Place, remove, and replace Scene Assets as Scene Instances inside a `SceneDocument`; single-root gate; preview projection; version-tracked resync. |
| `scene-document-model` | **MODIFIED** | `SceneDocument` gains additive `instances: BTreeMap<StableId, SceneInstance>` (`#[serde(default)]`); back-compat with pre-existing JSON. |
| `command-system` | **MODIFIED** | `Command` enum gains 3 instance variants (`PlaceInstance`/`RemoveInstance`/`ReplaceInstanceAsset`); processor `apply` arms + mechanical inverses; new `CommandError` variants. |

---

## §2. NEW Capability: `scene-instance-placement`

### Requirement: place-instance-workflow

The system MUST support placing a Scene Asset into the active `SceneDocument` from the Project Asset Browser. Placement MUST mint a fresh `instance_id`, mint a fresh `id_map` via `mint_id_map`, and MUST record `asset_version_seen` equal to the asset's current version.

#### Scenario: S1 — Place a Scene Asset creates a new instance
- GIVEN a `SceneDocument` with empty `instances`
- AND a Scene Asset X with one entity, `version: 1`, `logical_path: "characters/player"`
- WHEN the user activates "Place Instance" on X from the Project Asset Browser
- THEN `place_scene_instance(X.asset_id)` succeeds
- AND `SceneDocument.instances` contains exactly one entry with `instance_id` minted fresh
- AND the new instance's `asset_ref` equals X's `logical_path`
- AND `asset_version_seen` equals `1`
- AND `id_map.len()` equals X's entity count
- AND the `OperationLog.dirty` flag is set.

#### Scenario: S2 — Placement mints id_map entries with namespaced format
- GIVEN the same setup as S1
- WHEN placement completes
- THEN every key in `instance.id_map` corresponds to a `LocalId` from X
- AND every value matches the namespaced pattern `inst_<instance_id>_<local_id>` (no collision with authored `Entity.id`s in `SceneDocument.entities`).

### Requirement: remove-instance-workflow

The system MUST support removing a Scene Instance from the active `SceneDocument`. Removal MUST despawn preview entities tagged with that `instance_id` and MUST NOT touch authored `SceneDocument.entities`.

#### Scenario: S3 — Remove instance drops only that instance and its preview
- GIVEN a `SceneDocument` containing one authored entity E1 and one instance I1
- WHEN `remove_scene_instance(I1.instance_id)` succeeds
- THEN `SceneDocument.instances` no longer contains I1
- AND `SceneDocument.entities` still contains E1
- AND preview entities tagged with `SceneInstanceChild(I1.instance_id, _)` are despawned
- AND preview entities authored as E1 remain.

### Requirement: replace-instance-asset-workflow

The system MUST support swapping the `asset_ref` of an existing Scene Instance while preserving `instance_id`. Replacement MUST re-mint `id_map`, classify overrides, and MUST surface a `ResyncReport` (never silently lose overrides).

#### Scenario: S4 — Replace asset rebinds the instance to a new asset
- GIVEN an instance I1 with `asset_ref: "old_player"`, `id_map: { … }`, one Active `OverridePatch`
- AND a Scene Asset Y with `logical_path: "new_player"`, `version: 1`
- WHEN `replace_scene_instance_asset(I1.instance_id, Y.asset_id)` succeeds
- THEN `I1.asset_ref` equals `"new_player"`
- AND `I1.asset_version_seen` equals `1`
- AND `I1.id_map` is re-minted for Y's entities
- AND `I1.overrides` is non-empty (the preserved patch is reclassified against Y's schema, not silently dropped)
- AND `OperationLog.dirty` is set.

### Requirement: single-root-gate

The system MUST reject placement of a Scene Asset that resolves to more than one root entity, surfacing a typed error and NOT mutating the document.

#### Scenario: S5 — Multi-root asset placement is rejected
- GIVEN a Scene Asset Z with two root entities
- WHEN `place_scene_instance(Z.asset_id)` is invoked
- THEN the call returns `CommandError::MultipleRoots`
- AND `SceneDocument.instances` is unchanged
- AND the typed error code is exposed to the UI for surfacing in the Project Asset Browser.

### Requirement: id-map-preservation-on-roundtrip

The system MUST preserve `SceneDocument.instances` and every `id_map` entry byte-equal across `serialize → deserialize`. Pre-existing JSON without `instances` MUST parse unchanged.

#### Scenario: S6 — Instances + id_map survive save/load roundtrip
- GIVEN a `SceneDocument` with one instance I1 whose `id_map` has 3 entries
- WHEN serialized to JSON then deserialized into a fresh `SceneDocument`
- THEN `instances.len()` equals `1`
- AND `instances[I1.instance_id].id_map` is deeply equal to the original
- AND `instance_id`, `asset_ref`, `asset_version_seen`, `overrides`, and `orphaned_overrides` are all byte-equal.

#### Scenario: S7 — Pre-existing JSON without `instances` parses unchanged
- GIVEN a `project.json`-style scene JSON written before this change (no `instances` field)
- WHEN `load_scene(json)` parses it
- THEN parsing succeeds
- AND `instances` defaults to an empty `BTreeMap`
- AND no warning is emitted for the missing field.

### Requirement: version-tracking-and-resync

The system MUST compare `instance.asset_version_seen` against the current asset version on every load and rebuild. When the asset is newer, the system MUST call `resync(asset, &mut instance, new_version)` and MUST surface the resulting `ResyncReport` to the UI; auto-deletion of overrides is forbidden.

#### Scenario: S8 — Asset version bump on load triggers resync
- GIVEN an instance I1 with `asset_version_seen: 1`, `overrides` containing 2 Active patches
- AND the referenced asset is now at `version: 2` (one entity renamed)
- WHEN the scene is loaded
- THEN `resync(asset_v2, &mut I1, 2)` runs
- AND `I1.asset_version_seen` equals `2`
- AND the UI receives a `ResyncReport` reflecting at minimum `{ active, orphaned, stale, conflict, rebound }` counts.

#### Scenario: S9 — Resync never auto-deletes overrides
- GIVEN the same setup as S8 where one patch's target entity was removed in the asset
- WHEN resync runs
- THEN that patch is moved to `I1.orphaned_overrides` (status `Orphaned`)
- AND `I1.overrides` and `I1.orphaned_overrides` together preserve the original patch count (no silent loss).

### Requirement: placement-edge-cases

The system MUST handle E1 (dirty scene), E2 (missing asset), and E10 (empty asset) without silently dropping user intent.

#### Scenario: S10 — Place while the scene is dirty is allowed (E1)
- GIVEN a `SceneDocument` with `OperationLog.dirty == true`
- WHEN `place_scene_instance(X.asset_id)` succeeds
- THEN the new placement joins the existing undo stack
- AND the dirty flag remains `true` (single dirty flag for the scene).

#### Scenario: S11 — Place while the asset is missing is stored as broken (E2)
- GIVEN an `asset_ref` that does not resolve to any catalog entry
- WHEN `place_scene_instance(<missing id>)` is invoked
- THEN the instance is stored with `asset_version_seen: 0`
- AND the UI marks the instance as broken (Validation Center concern)
- AND the document is NOT silently dropped.

#### Scenario: S12 — Placement of an empty asset is rejected (E10)
- GIVEN a Scene Asset with `entities: []`
- WHEN `place_scene_instance(<empty asset id>)` is invoked
- THEN the call returns `CommandError::EmptyAsset`
- AND `SceneDocument.instances` is unchanged.

---

## §3. Edge Case Coverage Matrix

| # | Case | Scenarios |
|---|------|-----------|
| E1 | Place while scene dirty | S10 |
| E2 | Place while asset missing | S11 |
| E4 | Delete instance | S3 |
| E5 | Move/reparent (placement transform as root `OverridePatch` on `editor.Transform2D`) | deferred to design (R1) |
| E6 | Asset version bump | S8, S9 |
| E7 | Override targets deleted asset entity | S9 (orphan routing) |
| E8 | Two instances of the same asset | namespaced `inst_<iid>_<lid>` minting in S2 + inverse independence verified via processor tests |
| E9 | Save/load roundtrip | S6, S7 |
| E10 | Empty asset placement | S12 |
| E11 | Multi-root asset placement | S5 |

E3 (nested instances) is **out of scope** for this change — see §6.

---

## §4. MODIFIED Capability: `scene-document-model`

### Requirement: additive-instances-storage-field

`SceneDocument` MUST gain `instances: BTreeMap<StableId, SceneInstance>` annotated `#[serde(default)]` for back-compat. Authored `Entity` records MUST NOT gain an `instance_id` field — instance references live exclusively in `instances`.

#### Scenario: S13 — SceneDocument JSON gains additive `instances` field
- GIVEN a `SceneDocument` with one placed instance I1
- WHEN serialized to JSON
- THEN the output contains a top-level `instances` object keyed by `instance_id`
- AND `id_map`, `overrides`, `orphaned_overrides` roundtrip transparently.

#### Scenario: S14 — Authored Entity records remain unchanged
- GIVEN any `SceneDocument` with or without instances
- WHEN serialized
- THEN the `entities` array shape is unchanged from the `scene-document` cycle
- AND no authored `Entity` carries an `instance_id` field.

---

## §5. MODIFIED Capability: `command-system`

### Requirement: instance-command-variants

The `Command` enum MUST gain three additive variants: `PlaceInstance`, `RemoveInstance`, `ReplaceInstanceAsset`. The processor MUST implement `apply` arms + mechanical inverses for each. New `CommandError` variants MUST include `MultipleRoots`, `EmptyAsset`, and `InstanceNotFound`.

#### Scenario: S15 — PlaceInstance applies and inverts
- GIVEN an empty `SceneDocument` and a Scene Asset X
- WHEN `PlaceInstance { instance_id, asset_ref, asset_version, id_map }` is applied
- THEN `instances.len()` equals `1`
- AND the inverse is a `RemoveInstance { instance_id }` with the captured pre-state
- AND applying the inverse restores `instances` to its prior state.

#### Scenario: S16 — RemoveInstance applies and inverts
- GIVEN a `SceneDocument` with one instance I1 captured pre-state (asset_ref + id_map + overrides)
- WHEN `RemoveInstance { instance_id: I1.id }` is applied
- THEN `instances` no longer contains I1
- AND the inverse is a `PlaceInstance` restoring I1 verbatim.

#### Scenario: S17 — ReplaceInstanceAsset applies and inverts
- GIVEN an instance I1 with `asset_ref: "old"`
- WHEN `ReplaceInstanceAsset { instance_id: I1.id, new_asset_ref, new_asset_version }` is applied
- THEN `I1.asset_ref` equals `"new"` and a `ResyncReport` is recorded
- AND the inverse is a `ReplaceInstanceAsset` with the captured old `asset_ref` and old `asset_version_seen`.

---

## §6. Out-of-Scope Behaviors (explicit non-goals)

1. **E3 — Nested instances** (an asset containing its own instance). Detect + warn; do not recursively resolve (ADR-0005 §Variants).
2. **Inspector edit-routing to `OverridePatch`** for instance children (R1; design-phase concern).
3. **A3 anchor-entity model** (deferred; placement transform is the first `OverridePatch` on root `Transform2D`).
4. **Batch placement, drag-drop, duplicate** (follow-up; same backend surface).
5. **Multi-root asset support** (E11 deferred past this change).
6. **Live override authoring UI** beyond the read-only preview projection.
7. **Async / parallel resync** (single-threaded WASM target).
8. **A `SceneInstanceChild`-aware inspector** — selection from preview into inspector is read-only for this change.

---

## §7. Acceptance Criteria Checklist

- [ ] `SceneDocument.instances` field exists with `#[serde(default)]`; old JSON parses unchanged (S7).
- [ ] `place_scene_instance`, `remove_scene_instance`, `replace_scene_instance_asset`, `get_scene_instances` WASM functions exist.
- [ ] `Command::PlaceInstance`, `Command::RemoveInstance`, `Command::ReplaceInstanceAsset` exist with mechanical inverses (S15, S16, S17).
- [ ] `effective_values`-driven preview projection spawns entities tagged with `SceneInstanceChild(instance_id, local_id)`; authored entities are never confused with projected children.
- [ ] Single-root gate rejects multi-root assets with `CommandError::MultipleRoots` (S5).
- [ ] Save/load preserves `instances` + `id_map` 1:1 (S6).
- [ ] `id_map` uses namespaced `inst_<iid>_<lid>` minting to avoid collisions (S2).
- [ ] Asset version bump triggers `resync` on scene load; `ResyncReport` surfaces to UI; no override is silently deleted (S8, S9).
- [ ] Two instances of the same asset have distinct `instance_id` and `id_map`; overrides do not cross-talk (covered by processor unit tests).
- [ ] Missing asset placement stores `asset_version_seen: 0` and is marked broken in UI (S11).
- [ ] Empty asset placement returns `CommandError::EmptyAsset` (S12).
- [ ] Existing `scene-document`, `command-system`, `scene-instance-overrides` scenarios stay green; no forbidden terminology (`prefab`, `template`, `blueprint`, `archetype`) appears in UI copy.

---

## §8. Verification Plan

| Category | Test file | Scenarios |
|----------|-----------|-----------|
| Rust unit (storage) | `crates/editor-core/tests/scene_document_instances.rs` | S6, S7, S13, S14 |
| Rust unit (command) | extend `crates/editor-core/tests/scene_command_instances.rs` | S3, S15, S16, S17 |
| Rust unit (placement) | `crates/editor-core/tests/scene_instance_placement.rs` | S1, S2, S5, S11, S12 |
| Rust unit (resync trigger) | `crates/editor-core/tests/scene_instance_resync.rs` | S8, S9 |
| Rust unit (two instances) | `crates/editor-core/tests/scene_instance_isolation.rs` | E8 isolation |
| Build gate | `cargo check -p editor-core --target wasm32-unknown-unknown`, `cargo test -p editor-core` | all Rust scenarios |

Gate: `just check` and `just test` must stay green. The PR1/PR2/PR3 chain (Rust storage → Rust bridge + commands → Frontend PAB action) keeps each step independently reviewable.

---

## §9. Open Questions (design-owned, non-blocking)

1. **E5 — Move/reparent placement transform**: design confirms the placement transform is the first `OverridePatch` on the asset root's `Transform2D` (Option A) and decides whether the A3 anchor-entity model is needed for reparent UX (R1).
2. **`AssetResolver` shape**: trait method signature (`fn resolve(&AssetReference) -> Option<&SceneAssetDocument>`) vs. closure-based injection. Design picks.
3. **`ASSET_BODY_CACHE` invalidation**: hook into `save_scene_asset` to bust cache by `asset_id`. Design picks.
4. **WASM `place_scene_instance` parameter shape**: accept `asset_id` (catalog lookup) vs. `logical_path` (filesystem path). Design picks.

---

## §10. Result Contract

| Field | Value |
|-------|-------|
| **status** | `success` |
| **executive_summary** | Locks observable behavior for the new `scene-instance-placement` capability and two modified capabilities (`scene-document-model`, `command-system`). 17 Given-When-Then scenarios cover place/remove/replace workflows, single-root gating, id_map preservation across save/load, version-tracked resync on load, and edge cases E1, E2, E4, E6, E7, E8, E9, E10, E11. E3 nested instances and E5 move/reparent are deferred. |
| **capabilities** | new = 1 (`scene-instance-placement`); modified = 2 (`scene-document-model`, `command-system`) |
| **scenarios_total** | 17 (S1–S17) |
| **coverage** | happy_paths: covered · edge_cases: covered (E1, E2, E4, E6, E7, E8, E9, E10, E11 — see §3 matrix) · error_states: covered (multi-root rejection, empty-asset rejection, missing-asset broken-storage, override orphan routing) |
| **next_recommended** | `sddk-design` (sequential, not parallel) — design resolves the 4 open questions and the E5 placement-transform shape |
| **risks** | R1 (inspector routing for instance children) is the highest-complexity work and belongs in design, not spec. R2/R3/R4/R5 from the proposal remain as documented. |