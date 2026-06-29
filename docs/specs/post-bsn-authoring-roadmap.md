# Post-BSN Authoring Roadmap Specification

This specification defines the next authoring-focused roadmap after the v0.20.0 BSN migration. It turns the existing Scene Asset architecture into concrete user workflows before collaboration, plugins, or physical `.bsn` write-back.

## Status

Normative planning spec. Accepted by [ADR-0006](../adr/0006-authoring-first-roadmap-after-bsn-migration.md).

## Quick Path

1. Build **Project Asset Browser + Scene Asset Authoring** first.
2. Add **Scene Instance Placement** once Scene Assets can be created and opened.
3. Add **Override / Resync Workbench** once instances exist in real scenes.
4. Add **Validation Center** when project-wide issues become visible across scenes/assets.
5. Research **2D Level Design Tools** before implementing tile/layer data models.
6. Add **Runtime Preview Inspector** once authoring-to-runtime provenance is stable.

## Non-Goals

- CRDT collaborative editing.
- Broad plugin ABI.
- Physical `.bsn` file import/export/write-back.
- Reintroducing `EntityTemplate` or prefab terminology.
- Visual scripting or behavior graph authoring.

## Capability 1 — Project Asset Browser + Scene Asset Authoring

### Outcome

Users can manage Project-level Scene Assets and edit a Scene Asset in an isolated authoring mode.

### Scope

- Project Asset Browser panel.
- Create, rename, duplicate, delete, and open Scene Assets.
- Store Scene Asset documents under Project persistence.
- List assets via `SceneAssetCatalog`.
- Filter by role: `actor`, `fragment`, `screen`, `level`, `ui`, `effect`.
- Open a Scene Asset in isolated mode without mutating the active SceneDocument.

### Out of Scope

- Nested Scene Asset variants.
- Physical `.bsn` files.
- Collaboration.
- Plugin-provided asset types.

### Research Required

| Topic | Why |
|-------|-----|
| Unity Prefab Mode | Understand isolated asset editing and context editing tradeoffs |
| Godot PackedScene / inherited scenes | Understand local modifications and inherited structure constraints |
| Defold Collections / collection factories | Understand reusable hierarchy identity and runtime spawning |
| Bevy BSN scene assets | Track when asset-based BSN becomes stable |
| OPFS project layout | Decide path conventions for assets, scenes, schemas, and future level data |

### Acceptance Criteria

- A user can create a Scene Asset from scratch.
- A user can open it, edit its entities/components, and save it.
- The asset appears in the Project Asset Browser after reload.
- The asset has a stable `asset_id`, normalized `logical_path`, role, and version.
- No UI copy uses EntityTemplate/prefab terminology.
- Tests cover create/list/open/save/delete and OPFS roundtrip.

## Capability 2 — Scene Instance Placement

### Outcome

Users can place Scene Assets into a SceneDocument as Scene Instances without deep-cloning asset content.

### Scope

- Place Scene Asset into the current scene.
- Mint stable `SceneInstance.instance_id` and asset-local-to-scene `id_map`.
- Show Scene Instances in hierarchy/inspector with clear asset provenance.
- Preserve instance identity across save/load.
- Provide missing-asset state if the asset reference cannot resolve.

### Out of Scope

- Variants.
- Nested Scene Instances.
- Apply-to-asset / revert-from-asset UX; handled by Capability 3.

### Research Required

| Topic | Why |
|-------|-----|
| Unity prefab instance display | Avoid confusing asset reference with deep copy |
| Defold collectionfactory returned ID map | Inform `id_map` debugging and provenance |
| Godot scene instance lifecycle | Understand missing/deleted base scene behavior |

### Acceptance Criteria

- Placing a Scene Asset creates a Scene Instance, not duplicated source entities.
- The UI shows the referenced Scene Asset and instance status.
- Save/load preserves `asset_ref`, `asset_version_seen`, `id_map`, and overrides.
- Broken asset references produce visible non-destructive warnings.

## Capability 3 — Override / Resync Workbench

### Outcome

Users can inspect and resolve local changes on Scene Instances safely.

### Scope

- Display active, orphaned, stale, and conflict overrides.
- Explain each override by target entity, component, and field path.
- Revert local override.
- Apply selected override back to the Scene Asset when safe.
- Reset all overrides on an instance.
- Show resync report after asset version changes.

### Out of Scope

- Automatic destructive cleanup.
- Multi-level variants.
- Cross-user merge conflicts.

### Research Required

| Topic | Why |
|-------|-----|
| Unity Prefab Overrides | Apply/Revert UX and nested override semantics |
| Blender Library Overrides | Explicit resync and non-destructive linked-data workflows |
| Godot inherited scene limitations | Avoid silent data loss and uneditable inherited structure traps |

### Acceptance Criteria

- Stale/orphaned/conflict overrides are visible and actionable.
- Automatic resync never deletes user data silently.
- Apply/revert/reset operations are explicit commands or documented exceptions.
- Tests cover active, orphaned, stale, conflict, rebound, apply, revert, reset.

## Capability 4 — Validation Center

### Outcome

Users can see all Project health issues in one place and jump to the affected scene, asset, entity, component, field, or override.

### Scope

- Project-wide validation panel.
- Issue severity: error, warning, suggestion, info.
- Issue sources:
  - broken Scene Asset refs,
  - missing schemas,
  - invalid logical paths,
  - duplicate catalog entries,
  - override conflicts,
  - export warnings,
  - dirty scenes,
  - invalid AI proposals.
- Run validators on save/export and on demand.

### Out of Scope

- Static Rust code analysis.
- External asset import validation.
- Security scanning.

### Research Required

| Topic | Why |
|-------|-----|
| Unity console/validation patterns | Familiar severity and navigation model |
| Defold resource profiler | Provenance and resource reference checks |
| Bevy diagnostics | Future runtime validation integration |

### Acceptance Criteria

- Validation Center lists issues with severity, source, target, and suggested action.
- Selecting an issue navigates to the owning scene/asset when possible.
- Export blocks only on errors; warnings remain non-blocking unless explicitly configured.
- AI proposal application rejects commands that would create validation errors.

## Capability 5 — 2D Level Design Tools

### Outcome

Users can build real 2D levels faster than by manually placing individual sprite entities.

### Scope

- Research-first: define layer model before implementation.
- Candidate layers:
  - tile layer,
  - object/entity layer,
  - IntGrid-like semantic layer,
  - auto-layer generated from rules.
- Scene Assets can be placed as objects/entities.
- Terrain/auto-layer rules remain JSON-stable and AI-editable.

### Out of Scope

- Isometric/3D editing.
- Full Tiled/LDtk compatibility on first cut.
- Hard dependency on a Bevy tilemap crate before adapter boundaries are clear.

### Research Required

| Topic | Why |
|-------|-----|
| Tiled terrain brush / Wang tiles / automapping | Efficient tile painting and rule-based generation |
| LDtk IntGrid / Auto Layers / Entities | Semantic layer design and typed entity fields |
| Bevy tilemap ecosystem | Runtime adapter options without early coupling |
| Aseprite tiled mode/tags | Sprite-sheet and animation metadata integration |

### Acceptance Criteria

- A design doc chooses the layer model and explains rejected alternatives.
- The first implementation supports at least one useful level-authoring workflow.
- Export path to Bevy is specified before UI work starts.
- AI-editability is preserved: layer data must be stable, explicit JSON.

## Capability 6 — Live Preview Inspector / Runtime Debugger

### Outcome

Users can understand how editor-authored data appears in the running preview.

### Scope

- Runtime preview tree.
- Mapping from editor Stable ID / Scene Instance to preview-spawned Bevy entity metadata without exposing Bevy Entity as editor identity.
- FPS, rebuild count, last command, and loaded resource summary.
- Select runtime preview item and navigate to editor source where possible.

### Out of Scope

- Full game debugger.
- Frame-by-frame time travel.
- Production performance profiler.

### Research Required

| Topic | Why |
|-------|-----|
| Defold runtime visual/web profiler | Live resource and hierarchy inspection patterns |
| Godot remote SceneTree | Runtime/editor tree correspondence |
| Bevy diagnostics and remote tooling | Safe preview instrumentation |
| Chronos MCP | Future deep runtime debugging, not first-cut UI |

### Acceptance Criteria

- Inspector shows preview state without mutating editor source data.
- Runtime identity is clearly marked as ephemeral.
- Preview metrics update without breaking existing E2E tests.
- Runtime selection maps back to editor Stable ID when available.

## Sequencing

| Order | Change name | Why now | Depends on |
|-------|-------------|---------|------------|
| 1 | `project-asset-browser-and-scene-asset-authoring` | Makes Scene Assets usable | v0.20.0 |
| 2 | `scene-instance-placement` | Converts assets into reusable scene content | Capability 1 |
| 3 | `override-resync-workbench` | Makes instance-local changes safe and understandable | Capability 2 |
| 4 | `validation-center` | Gives users project-wide confidence before scale | Capabilities 1-3 |
| 5 | `level-design-layers-research` | Prevents premature tilemap model mistakes | Capability 1 |
| 6 | `runtime-preview-inspector` | Connects authoring to runtime behavior | Capability 2+4 |

## Verification Baseline

Each implementation cycle must keep these commands green unless a pre-existing environment limitation is documented:

```bash
just check
just test
```

For Rust-only pure-function phases, add focused inline `#[cfg(test)]` coverage. For UI phases, add Playwright coverage that exercises the visible workflow.

## Future Reconsideration

Revisit these after Hito 2:

| Candidate | Revisit when |
|-----------|--------------|
| Collaborative editing | Project asset identity, validation, and save/load semantics are stable |
| Plugin system | Schema packs and validation extension points have at least one built-in example |
| Physical `.bsn` import/export | Bevy ships stable loader/write-back APIs |
| Visual scripting/state machines | Scene Asset workflows and runtime preview inspection are mature |
