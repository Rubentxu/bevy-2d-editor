# Design: Level Inspector & Override Panel

## Technical Approach

Close the override read/write gap in two coordinated layers: (1) extend the existing `Command` enum (`command.rs`) with `UpsertOverride`/`RevertOverride` variants dispatched through the shared `OPERATION_LOG` — mirroring `PlaceInstance`/`RemoveInstance`; (2) add a Rust pure helper `field_override_index` + WASM bridge so the inspector renders effective values with per-field override indicators without duplicating field-path navigation in TS. The read-side merge (`effective_values`) already exists and is reused unchanged.

This maps directly to the proposal: write-side commands follow the `PlaceInstance` pattern; read-side rendering branches on the existing `inst_` entity-id prefix and consumes `effective_values_wasm`.

## Architecture Decisions

### Decision: Command surface for override mutation

**Choice**: Add `UpsertOverride` / `RevertOverride` to `Command` (`command.rs`), dispatched via `dispatch_command` + thread-local `OPERATION_LOG`.
**Alternatives considered**: (a) separate `OverrideCommand` enum + parallel log; (b) extend `AssetCommand` (`asset_command.rs`).
**Rationale**: Overrides live on `SceneInstance` inside `SceneDocument`, not on `SceneAssetDocument`. ADR-0007 deliberately keeps `AssetCommand`/`ASSET_OPERATION_LOG` for asset authoring only. `PlaceInstance`/`RemoveInstance` already mutate nested instance data through `Command`; overrides are the same nesting. A parallel enum would split undo history and break `Batch` grouping of place+override gestures.

### Decision: `field_path` as `Vec<String>` in new variants

**Choice**: New variants carry `field_path: Vec<String>`, NOT the dotted `String` used by `Command::SetComponentField`.
**Alternatives considered**: dotted `String` for consistency with `SetComponentField`.
**Rationale**: Override identity is the 3-tuple `(target_local_id, component_type_id, field_path: Vec<String>)` defined by `ComponentOverride`. `upsert_override`/`remove_override` match on that exact key. A dotted string would require lossy join/split and mis-resolve if a key ever contained `.`. `AssetCommand::SetComponentValue` already established `Vec<String>` as the structured-path pattern; this is the second use.

### Decision: Per-field override indicators computed in Rust

**Choice**: New pure `field_override_index(instance) -> Vec<FieldOverrideEntry>` in `scene_instance_overrides.rs`, exposed via `override_field_status_wasm(instance_json)`. Returns `{local_id, type_id, field_path[], status}` per stored override.
**Alternatives considered**: TS-side cross-reference of `component_overrides` against rendered field paths.
**Rationale**: Field-path walking logic already exists three times in Rust (`classify_overrides`, `validate_overrides`, `effective_values`). Duplicating it in TS is the exact risk the proposal flagged. The index is O(overrides), cheap, and keeps the read-side composition clean: `effective_values` (values) + `field_override_index` (indicators) are independent projections over the same data.

### Decision: Upsert forces `status = Active`; revert is idempotent

**Choice**: `upsert_override` stores the patch with `status = Active` regardless of input. `RevertOverride` on a non-existent key is a no-op (self-inverse).
**Alternatives considered**: validate-on-write (assign Stale/Conflict at upsert time); error on revert-missing.
**Rationale**: Status re-evaluation is the `resync`/`classify_overrides` concern, triggered on asset load/replace — never at edit time (existing pattern: `PlaceInstance` stores overrides verbatim). Idempotent revert matches revert-button UX semantics and mirrors `RemoveComponent`'s no-op-when-absent behavior.

## Data Flow

    InspectorPanel (instance entity selected)
        │ 1. extract instance_id + local_id from entity.id ("inst_{iid}_{lid}")
        ├─→ effective_values_wasm(instance, asset)  → ResolvedEntity.components (merged)
        ├─→ override_field_status_wasm(instance)    → [{local_id,type_id,field_path,status}]
        └─→ render ComponentCard per component, indicator per field via index lookup

    User clicks "revert field"
        └─→ revert_override_wasm(iid, local_id, type_id, field_path[])
             └─→ Command::RevertOverride → dispatch_command
                  ├─→ processor::apply → remove_override (pure) → inverse (UpsertOverride if existed)
                  └─→ OPERATION_LOG.record → undoable via Ctrl+Z

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/editor-core/src/command.rs` | Modify | Add `UpsertOverride`/`RevertOverride` variants to `Command` |
| `crates/editor-core/src/processor.rs` | Modify | `validate` + `apply` + inverse for new variants (mirror `PlaceInstance` nesting) |
| `crates/editor-core/src/scene_instance_overrides.rs` | Modify | Add `upsert_override`, `remove_override`, `field_override_index` pure fns |
| `crates/editor-core/src/lib.rs` | Modify | Add `override_field_status_wasm`, `upsert_override_wasm`, `revert_override_wasm` |
| `frontend/src/services/scene-assets.ts` | Modify | TS wrappers + `FieldOverrideEntry` type mirroring Rust |
| `frontend/src/components/InspectorPanel.tsx` | Modify | Instance-entity branch: effective-values render + indicator lookup |
| `frontend/src/components/ComponentCard.tsx` | Modify | Accept optional `overrideStatus` per field; render indicator + revert button |

## Interfaces / Contracts

```rust
// command.rs — new variants (field_path is Vec<String>, see Decision 2)
Command::UpsertOverride { instance_id: StableId, target_local_id: LocalId,
    component_type_id: ComponentTypeId, field_path: Vec<String>, value: serde_json::Value }
Command::RevertOverride { instance_id: StableId, target_local_id: LocalId,
    component_type_id: ComponentTypeId, field_path: Vec<String> }

// scene_instance_overrides.rs — pure helpers (key = local_id+type_id+field_path)
pub fn upsert_override(inst: &mut SceneInstance, patch: ComponentOverride); // status forced Active
pub fn remove_override(inst: &mut SceneInstance, local_id, type_id, field_path) -> Option<ComponentOverride>;
pub fn field_override_index(inst: &SceneInstance) -> Vec<FieldOverrideEntry>;
```

Inverse table: `UpsertOverride` → if key existed, `UpsertOverride{old patch}`; else `RevertOverride{key}`. `RevertOverride` → if removed, `UpsertOverride{removed patch}`; else self (no-op).

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `upsert_override`/`remove_override` key match, replace-vs-append, remove-returns-patch | Rust `#[test]`, mirror existing `scene_instance_overrides` tests |
| Unit | `field_override_index` maps every stored override; orphaned excluded | Rust `#[test]` |
| Unit | processor forward/inverse round-trip for both variants; no-op revert | Rust `#[test]`, mirror `test_forward_inverse_roundtrip_*` |
| Integration | WASM `upsert_override_wasm` → `revert_override_wasm` → field restored | `cargo test` wasm-bindgen test harness |
| E2E | Select instance entity → effective values shown; revert field → indicator clears; Ctrl+Z restores | Playwright, extend existing 27-test suite |

## Migration / Rollout

No migration required. Overrides are additive JSON on `SceneInstance`; existing scenes without overrides serialize identically (vecs empty, `skip_serializing_if`).

## Open Questions

- [ ] Instance-entity rendering scope: does the inspector show only the selected resolved entity (`inst_{iid}_{lid}` → one local_id), or the whole resolved scene? **Recommendation**: single selected entity now; full-scene tree is the level-scene-asset-slice follow-up.
- [ ] Should `upsert_override` reject a patch whose `field_path` doesn't resolve in the current asset? **Recommendation**: no — store verbatim; `validate_overrides` surfaces `missing_field` independently (matches `PlaceInstance`).

## ADR Candidates

- **ADR-0011 — Override mutation as `Command` variants on the shared OperationLog** (not a parallel `OverrideCommand` enum, not `AssetCommand`). Hard to reverse (touches undo/history contract), surprising (overrides feel like they "belong" to instances/assets), real trade-off (single-log simplicity vs enum bloat).
- **ADR-0012 — `Vec<String>` field_path in override commands** despite `SetComponentField` using dotted `String`. Surprising inconsistency without the identity-matching rationale; hard to reverse once serialized.
- **ADR-0013 — Override indicator projection in Rust (`field_override_index`)** rather than TS-side derivation. Real trade-off (Rust surface growth vs avoiding duplicated field-walk logic); hard to reverse once the WASM contract is consumed by the frontend.
