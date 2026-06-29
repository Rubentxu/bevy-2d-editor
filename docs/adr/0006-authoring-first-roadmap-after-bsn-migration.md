# ADR-0006: Authoring-First Roadmap after the BSN Migration

## Status

Accepted (2026-06-29)

## Context

The Bevy 2D Editor completed the BSN-aligned reusable scene model in v0.20.0:

- `SceneAssetDocument`, `SceneInstance`, `BsnIr`, `SceneAssetCatalog`, and `scene_instance_overrides` exist in `editor-core`.
- `EntityTemplate` was deleted as a first-class model.
- `DynamicScene Export` remains an adapter, not the source of truth.
- Physical `.bsn` import/export is still blocked on Bevy's first-party loader/write-back/editor APIs becoming stable.

This leaves the project with a strong architectural substrate but a weaker authoring experience. Most of the new Scene Asset capability is Rust-only; users cannot yet manage Project assets, author Scene Assets directly, place Scene Instances, inspect overrides, or understand validation problems through the UI.

Research into Unity, Godot, Defold, Tiled, LDtk, Aseprite, and Rive suggests that the highest leverage comes from authoring workflows before collaboration or extensibility:

- Unity Prefab Mode and Overrides show that reusable content needs first-class isolated editing, instance override visibility, and apply/revert workflows.
- Godot PackedScenes and inherited scenes show the value and danger of local modifications over reusable scene definitions.
- Defold collections/factories and profiler show the value of resource provenance and runtime visibility.
- Tiled and LDtk show that 2D production speed comes from tile/object layers, terrain/auto-layer rules, typed entity fields, and simple JSON export.
- Aseprite and Rive show that timeline/state-machine authoring is valuable later, but should not precede stable Project and Scene Asset workflows.

## Decision

The next roadmap phase is **Hito 2: Authoring Workflows & 2D Level Production**.

Hito 2 prioritizes turning the existing BSN-aligned core into usable editor workflows before starting collaborative editing, a full plugin system, or physical `.bsn` import/export.

Normative sequence:

1. **Project Asset Browser + Scene Asset Authoring**
2. **Scene Instance Placement**
3. **Override / Resync Workbench**
4. **Validation Center**
5. **2D Level Design Tools**
6. **Live Preview Inspector / Runtime Debugger**

Collaborative editing and the plugin system remain future Hito 3+ candidates, not immediate Hito 2 work.

## Normative Rules

### Source of truth

- `SceneDocument` remains the editor-owned source of truth for scenes.
- `SceneAssetDocument` is the editor-owned source of truth for reusable Scene Assets.
- `DynamicScene Export`, Rust codegen, and future `.bsn` output are adapters.
- Physical `.bsn` files MUST NOT become the primary storage format until Bevy ships stable loader/write-back APIs.

### Terminology

- Use **Scene Asset**, **Scene Instance**, **OverridePatch**, **Scene Asset Catalog**, and **Project Asset Browser**.
- Do not reintroduce `EntityTemplate`, prefab, blueprint, or Defold-style GameObject/Collection/Factory/Proxy vocabulary as project concepts.
- External editor terminology may appear only in research notes and inspiration references.

### Delivery order

- UI must expose existing Rust capabilities before adding new architectural layers.
- Each feature must ship with at least one visible authoring workflow, not only model types.
- Features that mutate scene data must use the typed command pipeline or define why a new command surface is required.
- Each increment must preserve OPFS roundtrip, undo/redo semantics where applicable, and AI-auditability via authored metadata.

### Deferrals

- Do not start CRDT collaborative editing until Project asset identity, Scene Asset authoring, and validation workflows are stable.
- Do not start a broad plugin ABI until schema packs and validation extension points are understood.
- Do not implement `.bsn` file write-back until Bevy's write-back and asset loader APIs stabilize.
- Do not implement broad visual scripting before the editor has stable Scene Asset workflows and runtime validation.

## Hito 2 Capability Program

| Capability | User value | Existing substrate | Primary risk |
|------------|------------|--------------------|--------------|
| Project Asset Browser | Users can see and manage Project-level assets, not just the current scene | `SceneAssetCatalog`, OPFS, multi-scene | Confusing Project asset identity with runtime asset handles |
| Scene Asset Authoring | Users can create reusable actors/fragments/screens/levels in isolation | `SceneAssetDocument`, `BsnIr`, `bsn_codegen` | Recreating old EntityTemplate semantics under a new name |
| Scene Instance Placement | Users can place reusable assets into scenes without deep cloning | `SceneInstance`, `id_map`, `OverridePatch` | Poor ID-map UX and silent data loss |
| Override / Resync Workbench | Users can understand and resolve local changes safely | `scene_instance_overrides` | Hidden stale/orphaned/conflict override states |
| Validation Center | Users can see broken refs, schema issues, export warnings, and override conflicts | Catalog warnings, export warnings, schema registry | Treating warnings as scattered UI messages instead of a system |
| 2D Level Design Tools | Users can build real 2D levels quickly | SceneDocument, Component Schema Registry, Scene Asset roles | Designing a tilemap system before defining layer semantics |
| Runtime Preview Inspector | Users can connect authoring data to runtime preview behavior | Bevy preview, `SceneEntity`, LinearBus, DynamicScene Export | Exposing runtime Bevy Entity identity as editor identity |

## Research Gates

Before each Hito 2 capability enters `sddk-propose`, the exploration phase must answer these questions:

### Project Asset Browser + Scene Asset Authoring

- How do Unity Prefab Mode, Godot PackedScenes, and Defold Collections separate asset editing from scene editing?
- What Project asset hierarchy should map to OPFS paths?
- What is the minimum UI needed to create, rename, delete, duplicate, and open a Scene Asset?
- How should Scene Asset roles (`actor`, `fragment`, `screen`, `level`, `ui`, `effect`) constrain authoring UX?
- Which fields belong in `SceneAssetCatalogEntry` now versus later?

### Scene Instance Placement

- How should the editor display an asset reference plus local overrides without implying a deep copy?
- What command variants are needed for instance placement, replacement, and removal?
- How should `id_map` be minted, shown, and debugged?
- What happens when the referenced Scene Asset is missing?

### Override / Resync Workbench

- Which Unity Prefab Overrides workflows map cleanly to `OverridePatch` apply/revert/reset?
- Which Godot inherited-scene constraints should be copied or explicitly rejected?
- How should orphaned/stale/conflict overrides be sorted, explained, and resolved?
- What should be automatic on open, and what must require explicit user action?

### Validation Center

- What is the project-wide issue taxonomy: error, warning, suggestion, info?
- Which validators run on every edit, on save, on export, and on demand?
- How should validation results link back to scene, asset, component, field path, or override?
- Which warnings should block export or AI proposal application?

### 2D Level Design Tools

- Which Tiled and LDtk concepts map to the Bevy 2D Editor without polluting the domain model?
- Are tile layers represented as components, Scene Assets, or a new Project asset kind?
- Should IntGrid-like semantic layers exist before visual tiles?
- What is the minimal terrain/auto-layer rule format that remains JSON-stable and AI-editable?
- How does this map to Bevy Tilemap ecosystem crates, if any, without hard-coupling too early?

### Runtime Preview Inspector

- What runtime data can be exposed without leaking Bevy Entity IDs into the editor model?
- How can the preview show resource provenance, rebuild count, FPS, and spawn mapping?
- Does Bevy Remote Protocol or diagnostics infrastructure provide useful patterns?
- What should Chronos/runtime tracing validate in future debugging workflows?

## Consequences

### Positive

- Converts the BSN migration from backend architecture into visible authoring value.
- Avoids premature CRDT/plugin complexity.
- Keeps the editor aligned with Bevy's public BSN direction while Bevy's asset workflow matures.
- Gives future AI features richer, more structured Project context.
- Creates smaller, reviewable feature slices with clear acceptance criteria.

### Negative

- Collaborative editing and plugin extensibility are delayed.
- Hito 2 adds more UI surface area before deeper runtime/editor integration.
- Some Scene Asset APIs may need refinement once real authoring workflows expose friction.
- Level design tools may require new document concepts after the first research spike.

## Implementation Direction

1. Add a normative Hito 2 specification that describes scope, research gates, acceptance criteria, and sequencing.
2. Update `docs/ROADMAP.md` so Hito 2 is the active program.
3. Start with `project-asset-browser-and-scene-asset-authoring` as an A-full SDDK cycle.
4. Treat `override-resync-workbench` and `validation-center` as dependent follow-up cycles.
5. Treat 2D level design tools as a research-heavy cycle before implementation.
6. Revisit collaborative editing and plugin system after Hito 2 has stable Project asset semantics.

## References

- Bevy 0.19 release notes — Next Generation Scenes / BSN.
- Bevy 0.18 → 0.19 migration guide — old world serialization rename and BSN caveats.
- ADR-0005 — Scene Asset as the BSN-Aligned Reusable Scene Model.
- Unity Prefab Mode and Prefab Overrides documentation.
- Godot inherited scenes / PackedScene documentation.
- Defold collection factory and profiler documentation.
- Tiled terrain brush and automapping documentation.
- LDtk auto-layers and entity field documentation.
- Aseprite timeline, onion skinning, and tags documentation.
- Rive state machine documentation.
