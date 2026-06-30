# Level Design Layers — Research & Model Decision

> Research-first slice (Hito 2 Order 6). This document **chooses the Level Layer
> model** and explains rejected alternatives, per ADR-0006 §Capability 5 Acceptance
> Criteria. **It does NOT implement any layer kind** beyond what already exists.
> The implementation slice that builds on this doc is a separate change.

## Status

Accepted (2026-06-30). Authored during grill-with-docs + research, finalized after
external review of Tiled, LDtk, and the Bevy tilemap ecosystem.

## Related Decisions

- ADR-0005 — Scene Asset as BSN-aligned reusable scene model
- ADR-0006 — Authoring-first roadmap after BSN migration (Capability 5: 2D Level Design Tools)
- ADR-0009 — ComponentOverride migration prerequisite (closed before this slice)
- Grill-with-docs session for Order 6 (engram topics `glossary/level-scene-asset`,
  `glossary/level-layer`, `glossary/scene-instance-layer`, `architecture/level-layer-ordering`,
  `architecture/scene-instance-layer-kind-name`)

## Canonical Layer Model (chosen)

A **Level Scene Asset** is a `SceneAssetDocument` with `role: "level"`. It contains
an ordered list of **Level Layers**. Each Level Layer is one of these kinds:

| Layer Kind | Status | First implementation slice? |
|------------|--------|------------------------------|
| **Scene Instance Layer** | Chosen | ✅ Yes (next slice after this one) |
| **Tile Layer** | Deferred | ❌ Separate slice |
| **IntGrid Layer** | Deferred | ❌ Coupled to Auto Layer slice |
| **Auto Layer** | Deferred | ❌ Requires Tile + IntGrid as data sources |

### Scene Instance Layer (chosen for next slice)

**Purpose**: Organize placed Scene Instances inside a Level Scene Asset.

**Semantics**:

- A Scene Instance belongs to exactly one Scene Instance Layer (layer ownership rule).
- The layer owns the placement; it is not a view over globally stored instances.
- Each layer has a soft typed `kind` (`actors`, `props`, `spawns`, `triggers`,
  `collision`, `custom`) AND a free human-readable `name`.
- Each layer has an `order` (the layer order, coarse).
- Each placed Scene Instance has its own `instance_components`
  (e.g., `editor.Transform2D`) and optional per-instance `z` (fine order).

**Why chosen first**:

- Already supported by `SceneInstance` + `ComponentOverride` (post-ADR-0009).
- Useful authoring workflow immediately: place actors, props, spawns, triggers,
  checkpoints, pickups, doors.
- No new infrastructure required for placement itself.
- AI-editability: the instance/subset structure is naturally JSON-stable.
- BSN-friendly: places Scene Asset + applies `instance_components` to the root.

**Rejected alternative**: a generic Object Layer using arbitrary placed entities
(similar to Tiled Object Layers). Rejected because we already have a richer model
in `SceneInstance` that carries identity, overrides, and asset references. A
generic Object Layer would duplicate that substrate.

### Tile Layer (deferred)

**Purpose**: Tile-based visual rendering using a tileset atlas. Equivalent to
Tiled Tile Layers and LDtk Tile layers.

**Why deferred**:

- Requires tileset authoring (image + grid-size metadata). That is its own
  slice (`tilesets`).
- Requires an atlas + render strategy on Bevy side. Bevy ecosystem options
  (`bevy_ecs_tilemap`, `bevy_ecs_tiled`, `bevy_ecs_ldtk`) each have different
  tradeoffs. Hard-coupling too early risks locking the editor into a specific
  runtime crate.
- `.bsn` write-back for tile data is still a TBD question (BSN does not yet have
  a canonical tile representation).

**Future trigger**: when the editor needs visible tile-based backgrounds beyond
what Scene Asset instances can provide.

### IntGrid Layer (deferred)

**Purpose**: Semantic integer-value grid. Equivalent to LDtk IntGrid layers.

**Why deferred**:

- Requires a painting tool for integer values with custom colors per value.
- Tightly coupled to Auto Layer rules (IntGrid layers are the typical data source
  for Auto Layer rules in LDtk).
- The `editor` workflow does not yet need semantic grids; collisions can be
  modeled via a `collision` Scene Instance Layer with invisible `editor.Collider2D`
  `instance_components`.

**Future trigger**: when the editor needs typed semantic grids (collision,
terrain type, danger zones) that are independent of placed instances.

### Auto Layer (deferred)

**Purpose**: Generated layer whose contents are computed from a rule engine over
IntGrid and/or Tile Layer inputs. Equivalent to LDtk Auto Layers and Tiled
Automapping.

**Why deferred**:

- Requires the rule engine itself (input/output layer naming, pattern matching,
  random modes, probability weights).
- Requires Tile Layer + IntGrid Layer as data sources — neither exists yet.
- BSN projection for rule output is an open question.

**Future trigger**: when Tile + IntGrid exist and authoring users need
rule-driven tile decoration (grass-on-dirt, fence-on-edge, etc.).

## Cross-Editor Mapping

| Concept | Bevy 2D Editor | LDtk | Tiled |
|---------|----------------|------|-------|
| Level container | Level Scene Asset | World | Map |
| Placed actors/props | Scene Instance Layer | Entity Layer | Object Layer |
| Visual tiles | Tile Layer (deferred) | Tile Layer | Tile Layer |
| Semantic grid | IntGrid Layer (deferred) | IntGrid Layer | (none) |
| Rule-generated tiles | Auto Layer (deferred) | Auto Layer | Automapping + Wang Brush |
| Terrain transitions | Tile Layer + Auto Layer (deferred) | Tile Layer + Auto Layer | Terrain/Wang Set |

## Bevy tilemap ecosystem (research notes)

The Bevy 2D editor must remain runtime-agnostic for the level design slice. The
following crates are candidates for **future** Tile Layer runtime, but none are
chosen yet:

- `bevy_ecs_tilemap`: tile-per-entity, mature, supports layers/animations/isometric/hex.
- `bevy_ecs_tiled`: Tiled (.tmx/.json) loader; bridges to `bevy_ecs_tilemap`.
- `bevy_ecs_ldtk`: LDtk loader; bridges to `bevy_ecs_tilemap`.

**Decision**: defer integration until Tile Layer slice starts. The editor-owned
JSON model is the source of truth; these crates become runtime render adapters
behind a thin Bevy-side bridge. No coupling in this slice.

## BSN Projection Considerations

`crates/editor-core/src/bsn_ir.rs` and `bsn_codegen.rs` already produce
`bsn!`/`bsn_list!` source from a `SceneAssetDocument` via `BsnIr`.

### Scene Instance Layer projection (concrete, supported)

- Each placed Scene Instance becomes a Scene in the projected `bsn_list!`.
- The Scene Asset referenced by the Instance is composed (included) at that point.
- The Instance's `instance_components` (e.g., `editor.Transform2D`) become
  Component entries on the root entity of that composed Scene.
- Per-instance `z` maps to component order in the projected `bsn!` block.
- Layer `order` maps to Scene ordering in `bsn_list!`.

### Tile Layer projection (unknown, deferred)

- BSN does not yet have a canonical tile representation.
- We do not assume `bevy_ecs_tilemap` data format as the persisted shape.
- Decision deferred to Tile Layer slice.

### IntGrid Layer projection (unknown, deferred)

- A grid of integer values is JSON-trivial, but the editor-owned schema for it
  is not designed. The Component Schema Registry approach may be reused, but
  the data shape of an IntGrid cell vs a Component Instance is fundamentally
  different.

### Auto Layer projection (unknown, deferred)

- Rules need a JSON-stable, AI-editable format. The terrain/automapping rule
  shapes from Tiled and LDtk are documented but not selected.

## Rejected Alternatives (per ADR-0006 §Acceptance Criteria)

### Rejected: Tiled-style separate tilemap asset

Tiled stores tiles, terrains, and automapping rules in separate JSON assets
referenced by the map. We rejected this because:

- It splits the Level Scene Asset's authority into multiple files, contradicting
  the editor-owned source-of-truth rule.
- The schema for tiles + terrains would be a new asset type alongside Scene
  Assets. Until we have evidence the user needs tile authoring, that cost is
  premature.
- A future Tile Layer slice can introduce tile-set assets as a separate Project
  Asset kind without committing to the same model as Tiled.

### Rejected: LDtk-style IntGrid as the canonical grid primitive

LDtk defaults to IntGrid as the primary authoring layer. We rejected this
because:

- Our first useful workflow (placed Scene Assets) does not need IntGrid.
- Making IntGrid canonical before proving Scene Instance Layer is the wrong
  order. The grill-with-docs session explicitly chose Scene Instance Layer first.
- IntGrid is still available in the deferred layer kinds if needed later.

### Rejected: Generic `LevelDocument` separate from `SceneAssetDocument`

We considered creating a new `LevelDocument` type instead of reusing
`SceneAssetDocument(role=level)`. Rejected because:

- Duplicate source-of-truth problem (Scene Assets vs Level Documents).
- Breaks alignment with the BSN model (BSN expects a Scene; levels are Scenes).
- ADR-0005 already places Level as a `SceneAssetRole`.

### Rejected: Including `instance_components` design in this slice

`instance_components` is conceptually tied to Scene Instance Layer but is a
data-model change in `SceneInstance`. This slice explicitly defers it to the
implementation slice that builds on this design. Reason: the design has to be
frozen before changing the persisted shape, and ADR-0009 just landed (migration
prerequisite). Mixing another persisted-shape change in the research slice
would re-open questions we just closed.

## Implementation Roadmap (next slices, NOT in this document's slice)

When this design is approved, the next slices are:

1. **`level-scene-asset`**: persistence + WASM surface for `SceneAssetDocument(role=level)` and `instance_components` on `SceneInstance`.
2. **`scene-instance-layer`**: authoring UI for Scene Instance Layers (create/reorder/place inside).
3. **`level-tilesets`**: tileset Project Asset + Tile Layer data model.
4. **`level-tile-layer`**: Tile Layer authoring UI + Bevy tilemap adapter.
5. **`level-intgrid-layer`**: IntGrid Layer authoring UI.
6. **`level-auto-layer`**: Auto Layer + rule engine + decoration.

These can be reordered/merged later; the order above reflects the dependency
chain (tilesets → tile layer → intgrid → auto).

## Acceptance Criteria for Future Implementation Slices

Future slices that build on this doc MUST satisfy:

- [ ] Use the canonical layer kinds (`Scene Instance Layer`, `Tile Layer`,
      `IntGrid Layer`, `Auto Layer`) and not invent synonyms.
- [ ] Each layer is owned by exactly one Layer (no cross-layer instance sharing).
- [ ] Layer `order` + per-instance `z` is the only ordering semantic.
- [ ] Layer `kind` is soft typed; new kinds can be added with `custom`.
- [ ] All persisted layer data is editor-owned JSON, BSN-friendly.
- [ ] Bevy runtime integration is behind an adapter; no hard coupling to a
      specific Bevy tilemap crate until the Tile Layer slice decides.

## References

- Grill-with-docs session for Order 6 (engram topics listed above).
- LDtk docs: Layers, Auto Layers, Rules, IntGrid Layers, Layer Instances JSON.
- Tiled docs: Tile Layers, Automapping, Using Terrains (Wang sets unified with Terrains in Tiled 1.5).
- Bevy crates: `bevy_ecs_tilemap`, `bevy_ecs_tiled`, `bevy_ecs_ldtk` (research only; not chosen).
- Internal: `crates/editor-core/src/bsn_ir.rs`, `bsn_codegen.rs`.
- Internal: `CONTEXT.md` (Level Scene Asset, Level Layer, Scene Instance Layer glossary).
- ADRs: 0005, 0006, 0009.