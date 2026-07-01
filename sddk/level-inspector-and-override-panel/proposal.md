# Proposal: Level Inspector & Override Panel

## Intent

Users cannot inspect or edit overrides on placed Scene Instances. When an instance entity is selected, the InspectorPanel renders **raw asset components** — not the effective (merged) values — and shows no per-field override indicator. Worse, there is **no write path at all**: no WASM function exists to create, edit, or revert an override. The read-side algorithms (`effective_values`, `validate_overrides`, `resync`) shipped in v0.19/v0.27 but stop at the WASM boundary. This change closes the gap between those algorithms and the user-facing editing experience.

## Scope

### In Scope
- Render **effective values** (asset + overrides merged) in InspectorPanel when a Scene Instance entity is selected, via existing `effective_values_wasm`
- **Per-field override indicator** — status-aware visual marker (Active=blue, Stale=warning, Conflict=error, Orphaned=dimmed)
- **Per-field revert** action — remove an active override, field reverts to asset value
- **Write-side WASM bridge** — `upsert_override_wasm`, `revert_override_wasm` with operation-log undo/redo
- **Pure mutation helpers** in `scene_instance_overrides.rs`: `upsert_override`, `remove_override`
- New `Command` variants: `UpsertOverride`, `RevertOverride` (follows existing `PlaceInstance` pattern)

### Out of Scope
- Inline override value editing (typing a new value to *create* an override) — follow-up slice
- Side-by-side asset-vs-override comparison view (Workbench v2)
- Layer-aware entity tree for Level Scene Assets (separate change)
- Batch apply-all / revert-all overrides

## Capabilities

> CONTRACT with sddk-spec. Existing capabilities researched in `docs/sddk/`.

### New Capabilities
- `override-crud`: WASM command surface for override mutation — `UpsertOverride`/`RevertOverride` command variants, processor apply/inverse, pure mutation helpers, undo/redo integration via the shared OperationLog.

### Modified Capabilities
- `inspector-panel`: Effective-values rendering + per-field override indicators when a Scene Instance entity is selected. Currently renders only raw asset components for all entity types uniformly.
- `scene-instance-overrides`: Add pure mutation helpers (`upsert_override`, `remove_override`) alongside existing read/classify/resync/rebind functions. Module was explicitly read-only (spec §3 item 1 excluded mutation); now gains write helpers.

## Approach

1. **Read-side UI**: When the selected entity belongs to a Scene Instance, call `effective_values_wasm(instance_id)` → render `ResolvedScene` components. Cross-reference each field against the instance's `component_overrides` to compute per-field indicator state.
2. **Write-side command model**: Add `Command::UpsertOverride { instance_id, target_local_id, component_type_id, field_path, value }` and `Command::RevertOverride { instance_id, target_local_id, component_type_id, field_path }`. Processor `apply()` mutates the instance's override `Vec`; `inverse()` captures the previous value for undo.
3. **Pure helpers**: `upsert_override(&mut instance, override)` replaces if `(local_id, type_id, field_path)` matches, else appends. `remove_override(&mut instance, local_id, type_id, field_path)` removes by key, returns the removed patch for inverse.
4. **WASM bridge**: Thin `#[wasm_bindgen]` wrappers dispatching through the OperationLog (mirrors `place_scene_instance` pattern).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/command.rs` | Modified | New `UpsertOverride`, `RevertOverride` variants |
| `crates/editor-core/src/processor.rs` | Modified | `apply()` + `inverse()` for new variants |
| `crates/editor-core/src/scene_instance_overrides.rs` | Modified | Add `upsert_override`, `remove_override` pure functions |
| `crates/editor-core/src/lib.rs` | Modified | `upsert_override_wasm`, `revert_override_wasm` bindings |
| `frontend/src/components/InspectorPanel.tsx` | Modified | Effective-values rendering + override indicators for instance entities |
| `frontend/src/components/ComponentCard.tsx` | Modified | Override-aware variant: per-field indicator + revert affordance |
| `frontend/src/services/scene-assets.ts` | Modified | TS wrappers for new WASM functions |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Processor nesting: overrides live inside `SceneInstance` inside `SceneDocument` | Medium | Existing `PlaceInstance` already mutates nested instance data — follow same pattern |
| `effective_values` recompute cost on every selection | Low | Profile; cache per instance if needed |
| Field-path mismatch between TS field editors and Rust `ComponentOverride.field_path` | Medium | Share types; TS mirrors Rust `ComponentOverride` shape exactly |
| InspectorPanel branching: instance-entity vs raw-entity rendering | Medium | Clear conditional: if `selectedEntity.instanceId` exists → effective values path |

## Rollback Plan

1. Revert frontend: InspectorPanel and ComponentCard return to raw-component rendering (remove effective-values path and override indicators).
2. Revert Rust: remove `UpsertOverride`/`RevertOverride` from `Command` enum and processor; remove `upsert_override`/`remove_override` from `scene_instance_overrides.rs`; remove WASM bindings from `lib.rs`.
3. No data migration needed — overrides are additive JSON; existing data remains valid.

## Dependencies
- ADR-0009 (`ComponentOverride` model with explicit `component_type_id`) — implemented ✅
- `effective_values_wasm`, `validate_overrides_wasm` — shipped in v0.27.0 ✅
- Existing `Command::PlaceInstance` pattern — reference for new variants

## Success Criteria
- [ ] Selecting a Scene Instance entity shows effective (merged) component values, not raw asset values
- [ ] Overridden fields display a visual indicator matching their status (Active=blue, Stale=warning, Conflict=error)
- [ ] Per-field revert removes the override; field immediately shows the asset value
- [ ] `upsert_override_wasm` / `revert_override_wasm` round-trip correctly (upsert → revert → field restored)
- [ ] Override mutations are undoable via Ctrl+Z (forward/inverse pair in OperationLog)
- [ ] All existing tests pass (112+ Rust, 27+ Playwright) — no regression

## Open Questions / ADR Candidates

1. **Command granularity**: `UpsertOverride` as a single-field command (one LogEntry per field change) vs `Batch` wrapper for multi-field gesture edits. **Recommendation**: single-field + existing `Batch` for multi-field; mirrors `SetComponentField` granularity.

2. **Revert of a non-existent override**: Should `RevertOverride` be a no-op (return Ok) or an error when no matching override exists? **Recommendation**: idempotent no-op (return Ok) to match revert-button semantics.

3. **ADR candidate — Override mutation as `Command` variants vs dedicated WASM surface**: `PlaceInstance`/`RemoveInstance` already live in the shared `Command` enum. `UpsertOverride`/`RevertOverride` should follow the same pattern rather than introducing a parallel `OverrideCommand` enum. This keeps a single OperationLog and processor. **Draft: ADR-0011**.

4. **Field-path navigation in TS**: `ComponentEditor` currently renders flat field paths. Override field paths can be nested (`["editor.Sprite2D", "transform", "translation", "x"]`). The TS side needs logic to match a field-editor widget against an override entry's `field_path` suffix. Design phase must define the matching algorithm.

---

> **Status**: success
> **Summary**: Proposal created for `level-inspector-and-override-panel`. Scope bounded to read-side effective-values rendering + per-field indicators + write-side WASM bridge. Inline editing deferred to follow-up slice.
> **Capabilities**: New: 1 (`override-crud`); Modified: 2 (`inspector-panel`, `scene-instance-overrides`)
> **Risk Level**: Medium
> **Next**: sddk-spec
> **Context Quality**: C1
> **Taxonomy**: override-mutation-gap, effective-values-rendering, inspector-ux-refactor
