# Kernel Exploration: level-inspector-and-override-panel

## Context Quality

- **Level**: C1 — The override data model is well-defined and tested (ADR-0005, ADR-0009, level-design-layers-research). The read-side WASM surface exists (validate, effective_values, resync). What's missing is the **write-side** (mutation) surface and the **UX** for inline override inspection/editing.
- **Evidence Present**:
  - `crates/editor-core/src/scene_instance.rs` — `ComponentOverride`, `ComponentOverrideStatus`, `SceneInstance` (3 concept groups: asset components, instance components, overrides)
  - `crates/editor-core/src/scene_instance_overrides.rs` — Pure functions: `classify_overrides`, `validate_overrides`, `effective_values`, `resync`, `try_rebind` — all well-tested (1285 lines incl. tests)
  - `crates/editor-core/src/scene_asset.rs` — `SceneAssetDocument`, `LevelLayer`, `SceneInstanceLayer`, `ExposedProperty`
  - `crates/editor-core/src/lib.rs:715-800` — WASM bindings for override read operations
  - `frontend/src/components/InspectorPanel.tsx` — Override summary panel (counts: active/stale/orphaned/conflict), collapsible issues list
  - `frontend/src/components/HierarchyPanel.tsx:227` — Override status dot per entity
  - `frontend/src/components/ComponentCard.tsx` + `ComponentEditor.tsx` — Type-aware field editors (Vec2, Color, Number, Bool, String, Anchor, JSON)
  - `frontend/src/services/scene-assets.ts` — TS types for overrides, WASM wrappers
- **Missing Context**: No write-side WASM function exists for creating/editing/reverting an override. No effective-values rendering in the Inspector. No "Level Inspector" distinct from the generic InspectorPanel.
- **Recommended Effort**: deepen

## Current State

### Override Data Model (Solid)

The `ComponentOverride` type is a non-destructive patch:
```
ComponentOverride {
  target_local_id: LocalId,      // entity inside the Scene Asset
  component_type_id: ComponentTypeId,
  field_path: Vec<String>,       // path within the component's JSON values
  value: serde_json::Value,      // the override value
  status: ComponentOverrideStatus, // Active | Orphaned | Stale | Conflict
}
```

Overrides live on `SceneInstance` in two vectors: `component_overrides` (active/stale/conflict) and `orphaned_component_overrides`. A `SceneInstance` also carries `instance_components` (placement-time data like `editor.Transform2D`), which are **not overrides** — they're owned by the instance.

### Read-Side WASM Surface (Exists, 4 functions)

| WASM function | Purpose |
|---|---|
| `validate_overrides_wasm` | Returns `Vec<OverrideIssue>` with codes: `missing_entity`, `missing_component`, `duplicate_field`, `missing_field`, `type_conflict` |
| `effective_values_wasm` | Returns `ResolvedScene` — merged asset + active overrides (read-only) |
| `try_rebind_wasm` | Attempts exact-match rebinding of an orphaned patch |
| `get_resync_reports` | Drains accumulated `[instance_id, ResyncReport]` from load/replace ops |

The `effective_values` function **already computes** the merged projection that the inspector needs. It returns per-entity components with overrides applied. This is the critical building block.

### Write-Side WASM Surface (MISSING)

There is **no** WASM function to:
- Create a new override on an instance
- Edit an existing override's value
- Revert (remove) an override
- Apply an override back to the asset
- Change an override's status manually

This is the primary gap. The override data model supports mutation (it's a `Vec<ComponentOverride>`), but no command/mutation path has been wired through WASM.

### Current Inspector UX (Read-Only, Summary-Level)

The `InspectorPanel` shows:
1. Entity name (editable)
2. Component cards with field editors (only for raw entity components — `scene.entities[].components`)
3. Override summary: count badges (active/stale/orphaned/conflict)
4. Collapsible override issues list (code + message per issue)
5. Resync report summary (if available)

**Critically**: when a Scene Instance entity is selected, the inspector does **not** show the effective (merged) component values. It shows only the raw asset entity components without override annotations. The `ComponentCard` renders plain component values — there's no concept of "this field is overridden" at the field level.

### No Distinct "Level Inspector"

The ROADMAP name "Level Inspector" is aspirational. There is no separate panel for Level Scene Assets. The current `InspectorPanel` handles all entity types uniformly. A Level Scene Asset with `LevelLayer::SceneInstance` layers would need:
- Layer-aware entity selection (entities inside layers, not just top-level scene entities)
- Per-instance override inspection (selecting a placed Scene Instance shows its overrides)
- Inline override editing

## Affected Areas

- **`crates/editor-core/src/scene_instance_overrides.rs`** — May need new pure functions for override mutation (upsert, remove, update value). Currently only has read/classify/validate/resync.
- **`crates/editor-core/src/lib.rs:715-800`** — WASM surface needs new write functions (add_override_wasm, update_override_wasm, revert_override_wasm). Must integrate with the Operation Log for undo/redo.
- **`frontend/src/components/InspectorPanel.tsx`** — Major refactor needed: when a Scene Instance entity is selected, render effective values (via `effectiveValues()`) instead of raw asset components. Add per-field override indicators and revert actions.
- **`frontend/src/components/ComponentCard.tsx`** — Needs override-aware variant: show override bar/indicator on overridden fields, show revert button per field.
- **`frontend/src/components/ComponentEditor.tsx`** — No structural change needed (type-aware widgets are reusable), but needs to accept "is overridden" state and potentially a "revert" affordance.
- **`frontend/src/services/scene-assets.ts`** — Needs new WASM wrappers for override mutation operations.
- **`frontend/src/components/HierarchyPanel.tsx`** — Already has override dot indicator; may need enhancement for Level Scene Assets with layers.

## Approaches

### 1. Full Override-Field-Level Inspector (A-full)

Extend the existing InspectorPanel to show effective values per field, with override indicators and inline revert/edit, plus add write-side WASM for override CRUD.

- **Pros**:
  - Matches Unity/Godot industry-standard UX (override bar, per-field revert)
  - Full control: inspect, edit, revert at field granularity
  - `effective_values` already provides the merged data; just needs UI rendering
  - Enables future "apply to asset" workflow (Override/Resync Workbench v2)
- **Cons**:
  - Requires new write-side WASM surface (3-4 new functions + Operation Log integration)
  - ComponentCard/ComponentEditor need override-aware variants
  - InspectorPanel needs conditional rendering logic (instance entity vs raw entity)
  - Higher complexity (C2)
- **Effort**: Medium-High

### 2. Read-Only Effective Values + Revert-Only (A-lite)

Render effective values in the inspector with override indicators, but only support "revert" (removing an override), not editing existing overrides or creating new ones inline.

- **Pros**:
  - Lower risk: no mutation path beyond `revert` (which is a targeted removal)
  - Still demonstrates the effective-value projection
  - Simpler WASM surface (1-2 new functions)
- **Cons**:
  - Can't edit override values inline — users must use the Workbench or asset authoring mode
  - Half-measure; doesn't fully satisfy "edit component values inline" from the ROADMAP
- **Effort**: Medium

### 3. Instance Summary Panel + External Editor (A-min)

Keep the override summary as-is (counts + issues list) and add a separate "Override Inspector" modal or side panel that shows override patches as a list with edit/revert per patch.

- **Pros**:
  - Minimal disruption to existing InspectorPanel
  - Clear separation of concerns
- **Cons**:
  - Doesn't show overrides in context (not next to the fields they override)
  - Requires user to navigate to a separate panel
  - Deviates from industry-standard in-context inspection
- **Effort**: Low-Medium

## UX Research Findings (Unity & Godot)

### Unity Prefab Override Inspector Patterns

**Three levels of override indication:**
1. **Value level** — Blue override bar to the left of the property label. Bold text for overridden values. Height matches the property.
2. **Component level** — Plus/minus badges on component icons for added/removed components. Blue bar spans the full component height.
3. **Object level** — Overrides dropdown showing all overrides in a consolidated view, with Apply All / Revert All.

**Key interactions:**
- Right-click on a property → context menu with "Apply" / "Revert"
- Per-component cog menu → "Modified Component" → Revert/Apply
- Overrides dropdown → side-by-side comparison view (asset value vs instance value)
- Multi-select overrides with Ctrl/Shift → batch Apply/Revert

**Design system principles (from Unity Editor Design System):**
- The override bar is a semantic concept — users associate it with "this value differs from the source"
- Aggregate view at object level for efficiency
- Allow pushing overrides back to the parent (Apply) and reverting to inherited (Revert)
- In-context actions (context menu on the value, not in a separate panel)

### Godot Scene Inheritance & Editable Children

**Scene Inheritance (like Unity prefabs):**
- Changed properties show a yellow "reset" icon in the inspector
- "Manage Only Differences" principle — child scenes only store deviations
- Parent changes auto-propagate to children
- Cannot replace nodes from parent scene — only override properties

**Editable Children (ad-hoc):**
- Right-click instance node → "Editable Children"
- Directly edit internal nodes of an instanced scene
- One-time, no reusability, changes not saved to asset

**Key design tension (from Godot proposals #2280):**
- "Pin" vs "inherit by default" — Godot inherits by default; override is opt-in by changing the value
- Godot removes the override when child matches parent, which causes confusion
- Reset icon = "revert to parent value"

### Recommendations for Bevy 2D Editor

Based on research, the ideal UX follows these patterns:
1. **Effective values rendering**: Show merged values (asset + overrides) as the default view
2. **Per-field override indicator**: A visual marker (bar, dot, color) on fields that have an active override
3. **Per-field revert**: Contextual action to remove an individual override
4. **Per-component override summary**: Badge or indicator when a component has any overridden fields
5. **Side-by-side comparison** (deferred): For the Override/Resync Workbench v2 — show asset value vs override value

## Recommendation

**Approach 1 (A-full)** with phased delivery:

**Phase 1 — Read-side effective values rendering + override indicators:**
- Use `effectiveValues()` to render merged component values when a Scene Instance entity is selected
- Add override indicator (bar/dot) on fields that have an active override
- Add per-field revert action (remove override, revert to asset value)
- Status-aware rendering: Stale → warning style, Conflict → error style, Orphaned → dimmed/disabled

**Phase 2 — Write-side override CRUD:**
- New WASM: `upsert_override_wasm(instance_id, target_local_id, component_type_id, field_path, value)`
- New WASM: `revert_override_wasm(instance_id, target_local_id, component_type_id, field_path)`
- Integration with Operation Log for undo/redo
- Inline editing: when user edits a field on an instance entity, it creates an override (not a direct asset mutation)

**Phase 3 — Level-aware inspection (deferred if not blocking):**
- When a Level Scene Asset is open, show layer-aware entity tree
- Selecting a Scene Instance inside a layer shows its overrides
- This may be a separate change or folded into level-scene-asset-slice

**Why A-full**: The ROADMAP explicitly says "edit component values inline." A-lite (read-only + revert) doesn't satisfy that. The existing read-side functions (`effective_values`, `validate_overrides`) provide 60% of the foundation. The write-side gap is the primary work item, and it's a natural extension of the existing pattern.

## Risks

1. **Operation Log integration complexity**: Override mutations must be undoable. The existing `OperationLog` works on `SceneDocument`, but overrides live on `SceneInstance` which is a field inside the scene document. Need to verify the command/snapshot infrastructure supports nested mutations.
2. **InspectorPanel conditional rendering**: The panel currently renders all entities uniformly. Adding instance-aware logic (effective values vs raw values) introduces branching. Risk of rendering the wrong data for non-instance entities.
3. **Effective values performance**: `effective_values` does a full merge of asset + overrides every call. For large assets with many instances, this could be slow. May need caching or incremental computation.
4. **Field path navigation**: `ComponentEditor` currently renders flat field paths. Nested field paths in overrides (`field_path: ["transform", "translation", "x"]`) need the editor to navigate into nested objects. The existing `apply_field_path` in Rust handles this, but the TS side would need equivalent logic.
5. **Level Scene Asset vs SceneDocument confusion**: Overrides live on `SceneInstance` inside a `SceneDocument`. But Level Scene Assets contain `SceneInstanceLayer` with `SceneInstance`s inside `SceneAssetDocument.layers`. The override inspection UI must work in both contexts (scene document instances AND asset-local layer instances).

## Ready for Proposal

**Yes.** The research gate is satisfied: Unity and Godot patterns are well-documented and directly applicable. The codebase has the data model and read-side infrastructure. The primary work is the write-side WASM surface and the inspector UI refactor.

**What the orchestrator should tell the user:**
- The override system's read-side is solid (validate, effective_values, resync all exist)
- The missing piece is the write-side: no WASM function to create/edit/revert overrides
- A-full is recommended: effective values rendering + override indicators + write-side CRUD
- Estimated complexity: C2 (multiple coordinated changes: Rust WASM, TS services, React components)
- The "Level" in "Level Inspector" means the inspector must handle entities inside Level Scene Asset layers, not a separate panel

---

## Standard Envelope

- **status**: complete
- **executive_summary**: The override data model and read-side WASM surface are well-built and tested. The primary gap is the write-side: no function to create, edit, or revert overrides. The existing `effective_values` function provides the merged projection needed for inspector rendering. UX research confirms the industry-standard pattern: effective values + per-field override indicator + inline revert. Recommendation: A-full approach with phased delivery.
- **artifacts**: `sddk/level-inspector-and-override-panel/explore-report.md`
- **next_recommended**: sddk-propose
- **risks**: Operation Log integration for nested mutations; InspectorPanel conditional rendering; effective_values performance for large assets; nested field path navigation in TS
- **context_quality**: C1
- **taxonomy**: dominant_axis = override-mutation-gap; secondary_axes = [inspector-ux-refactor, effective-values-rendering, level-aware-selection]
