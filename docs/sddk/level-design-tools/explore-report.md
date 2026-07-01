# Kernel Exploration: level-design-tools

> Hito 2 Order 8 — Tile painting, IntGrid authoring, auto-layer generation.
> Path: A-full | Mode: auto | Artifact store: engram

## Context Quality

- **Level**: C1 — domain well-understood (2D level design tools in Unity/Godot/Tiled/LDtk); ecosystem uncertainty resolved below.
- **Evidence Present**:
  - `docs/sddk/level-design-layers-research/design.md` (PR #27) — canonical layer model already chosen
  - `docs/adr/0006-authoring-first-roadmap-after-bsn-migration.md` — roadmap + research gates
  - `docs/adr/0009-component-override-ecs-bsn-replacement-for-override-patch.md` — override model
  - `crates/editor-core/src/scene_asset.rs` — `LevelLayer`, `SceneInstanceLayer`, `LayerId`, `LayerKind`
  - `crates/editor-core/src/scene_instance.rs` — `SceneInstance` with `instance_components`
  - `crates/editor-core/src/lib.rs:1069-1160` — WASM surface for layer CRUD
  - `CONTEXT.md` — Level Layer, Scene Instance Layer, Tile/IntGrid/Auto Layer glossary entries
- **Missing Context**: BSN tile representation does not exist yet; no Bevy 0.19-compatible tilemap crate published; Aseprite tileset metadata pipeline is unproven.
- **Recommended Effort**: deepen — ready for proposal after approach selection.

## Current State

### Layer model (implemented)

`LevelLayer` is a tagged enum living on `SceneAssetDocument.layers`:

```rust
pub enum LevelLayer {
    SceneInstance(SceneInstanceLayer),  // ← only this variant exists today
    // Tile(TileLayer), IntGrid(IntGridLayer), AutoLayer(AutoLayer) — all deferred
}
```

`SceneInstanceLayer` owns `{ id: LayerId, name, kind: SceneInstanceLayerKind, order, instances: Vec<SceneInstance> }`.

WASM functions exist for CRUD: `list_scene_instance_layers_wasm`, `create_scene_instance_layer_wasm`, `delete_scene_instance_layer_wasm`.

`SceneInstance` carries `instance_components` (placement data like `editor.Transform2D`) and `component_overrides` (non-destructive patches against asset-local entities).

### What does NOT exist

- No `TileLayer`, `IntGridLayer`, or `AutoLayer` variant in `LevelLayer`
- No tileset or atlas concept (`grep` for `tileset`, `atlas`, `sprite_sheet`, `TextureAtlas` → zero matches)
- No tile-painting tool, brush, or grid coordinate system in the model
- No Bevy-side tile rendering — the preview renders `editor.Sprite2D` entities (one image per entity)
- No BSN tile representation — `BsnIr` is entity/component/relationship-only

### Research doc decisions (already accepted)

`level-design-layers-research/design.md` explicitly:
- Chose SceneInstanceLayer as the first layer kind (shipped in PR #28/#29)
- Deferred Tile Layer → requires tileset authoring + render strategy
- Deferred IntGrid Layer → requires painting tool + coupled to Auto Layer
- Deferred Auto Layer → requires rule engine + Tile/IntGrid as data sources
- Rejected: Tiled-style separate tilemap asset, LDtk-style IntGrid-first, generic LevelDocument
- Recommended implementation roadmap: `level-tilesets` → `level-tile-layer` → `level-intgrid-layer` → `level-auto-layer`

## Bevy Tilemap Ecosystem (2026-07-01 research)

### bevy_ecs_tilemap (StarArawn)

| Metric | Value |
|--------|-------|
| Latest version | **0.18.1** (Jan 16, 2026) |
| Targets | **Bevy 0.18** |
| Bevy 0.19 support | **NOT YET** — no 0.19-compatible release |
| Total downloads | 244,735 |
| Reverse deps | 12 |
| Release pattern | Releases 1-3 months after each Bevy version |
| Features | Tile-per-entity, chunked rendering, isometric/hex, animations, sparse maps |

**Verdict**: Mature and well-maintained, but always one Bevy version behind. The project uses **Bevy 0.19** (released June 19, 2026 — 2 weeks ago). A 0.19-compatible `bevy_ecs_tilemap` is likely in progress but not published. Hard-coupling now would block the editor on upstream release timing.

### bevy_ecs_tiled (adrien-bon)

| Metric | Value |
|--------|-------|
| Latest version | **0.12.0** (May 17, 2026) |
| Targets | **Bevy 0.18** + `bevy_ecs_tilemap 0.18` |
| Total downloads | 36,241 |
| Purpose | Loads `.tmx`/`.tsx` Tiled files, bridges to `bevy_ecs_tilemap` |

**Verdict**: Excellent for Tiled file compatibility. Same Bevy-version lag as tilemap. Depends on `bevy_ecs_tilemap`.

### bevy_ecs_ldtk (Trouv)

| Metric | Value |
|--------|-------|
| Latest version | **0.14.0** (Jan 22, 2026) |
| Targets | **Bevy 0.18** + `bevy_ecs_tilemap 0.18` + LDtk 1.5.3 |
| Total downloads | 109,821 |
| Purpose | Loads `.ldtk` projects, spawns LDtk entities/tiles |

**Verdict**: Most popular integration (highest download count). Again Bevy 0.18, not 0.19.

### Bevy first-party tilemap

Bevy Issue #13782 ("First-party tile maps") exists. No working group formed. Not planned for 0.19 or 0.20. The Bevy Editor (when it arrives) will need tilemaps, but timeline is unknown.

### Key conclusion

**All three ecosystem crates are on Bevy 0.18; none support 0.19 yet.** The editor must remain runtime-agnostic for tile rendering. The editor-owned JSON model is the source of truth; any tilemap crate becomes a render adapter behind a Bevy-side bridge. This confirms the design doc's deferral decision.

## Editor Pattern Survey

### Tiled: Terrain brushes + Automapping

- **Tile Layers**: raw tile-ID grids (sparse or dense), one tileset per layer
- **Terrain/Wang sets**: define tile transitions by edge/corner terrain types; brush paints correct transition tiles automatically
- **Automapping**: rule files (`.rules`): input pattern (3x3 or NxN tile match) → output tile placement. Can target multiple layers.
- **Object Layers**: free-form placed objects (points, rectangles, polygons) — equivalent to our Scene Instance Layer
- **Data format**: `.tmx` (XML) or `.json`; tile grids stored as CSV or base64+zlib

### LDtk: IntGrid + Auto Layers + Rules

- **IntGrid Layer**: sparse grid of integer values, each with a color. Used for collision, terrain types, danger zones.
- **Auto Layer**: IntGrid layer + linked tileset + rule groups. Rules match IntGrid patterns and paint tiles.
- **Rule format** (`AutoRuleDef`): pattern (NxN IntGrid match), tileRects (random pick), chance, Perlin filter, flip modes, tile mode (stamp/overlay). Very expressive.
- **Entity Layers**: placed entities with typed fields — equivalent to our Scene Instance Layer
- **Tile randomization**: `tileRectsIds` is an array of tile-ID rectangles; one is picked randomly per match
- **Data format**: pure JSON with published JSON schema (1.5.3)

### Unity: Tilemap + Tile Palette + Grid

- `Tilemap` component + `Grid` component; tiles are `TileBase` assets
- `TilePalette` window for painting; custom brushes (`GridBrush`, `GameObjectBrush`)
- `RuleTile` / `AdvancedRuleTile` for auto-tiling via neighbor matching
- No IntGrid equivalent — collision done via separate `TilemapCollider2D`

### Godot: TileMap + TileSet Atlas

- `TileMap` node with layers; `TileSet` resource defines atlas + terrain + custom data
- `TileSetAtlasSource`: tiles from a texture atlas with defined regions
- Terrain sets: Wang-like peering bits for transitions
- Patterns: reusable tile stamp presets

## Aseprite Metadata

### What Aseprite exports (JSON via `--data`)

```json
{
  "frames": { "filename": { "frame": {"x":0,"y":0,"w":16,"h":16}, "duration": 100 } },
  "meta": {
    "image": "sheet.png",
    "size": {"w": 256, "h": 256},
    "frameTags": [{"name":"walk","from":0,"to":3,"direction":"forward"}],
    "slices": [{"name":"cursor","keys":[{"frame":0,"bounds":{"x":80,"y":0,"w":16,"h":16}}]}],
    "layers": [{"name":"Layer 1"}, {"name":"Group/Child","group":"Group"}]
  }
}
```

### Aseprite tileset mode

Aseprite 1.3+ has a native **Tilemap mode** (`--export-tileset` CLI flag). It creates tilemap layers with a tileset, where each cell references a tile from a tileset base. This is Aseprite's answer to tile-level editing, but:
- The tileset itself is Aseprite-internal, not an external `.tsx` or LDtk tileset
- JSON export includes the tileset grid size and tile data
- No terrain/auto-tile rules — Aseprite is a manual pixel editor

### Pipeline implication for our editor

Aseprite JSON gives us: frame rects (atlas regions), animation tags, slices (named regions), layer hierarchy. We can use this to build a **tileset definition** (image + grid size + tile IDs). But Aseprite does NOT give us terrain data or auto-tiling rules — those must be authored in our editor.

## Affected Areas

| File/Module | Impact |
|-------------|--------|
| `crates/editor-core/src/scene_asset.rs` | Add `TileLayer`, `IntGridLayer`, `AutoLayer` variants to `LevelLayer` enum |
| `crates/editor-core/src/lib.rs` (WASM) | New WASM functions for tile painting, IntGrid editing, tileset management |
| `crates/editor-core/src/schema.rs` | New built-in component types: `editor.Tile2D`, `editor.IntGridCell`, `editor.TilesetRef` |
| `crates/editor-core/src/bsn_ir.rs` + `bsn_codegen.rs` | Tile data projection to BSN (unknown — BSN has no tile representation yet) |
| `crates/editor-core/src/dynamic_scene.rs` | Bevy preview rendering of tiles (adapter strategy needed) |
| `crates/editor-core/src/command.rs` / `processor.rs` | New tile paint commands, IntGrid set commands |
| `frontend/src/` (React) | Tile palette UI, grid painting canvas, IntGrid color editor, auto-layer rule editor |
| `CONTEXT.md` | New glossary entries for Tile Layer, IntGrid Layer, Auto Layer, Tileset, Tile Palette |
| New: tileset persistence | Tileset as a new Project Asset kind or embedded in Level Scene Asset |

## Approaches

### Approach A: Tiled-style Tile Painter (tile-first)

Tile storage is a dedicated `TileLayer` with its own grid data model, separate from Scene Instances.

- **Data model**: `TileLayer { id, name, order, grid_size: (u32,u32), tileset_ref: AssetReference, tiles: SparseGrid<TileId> }`
- **Tools**: brush (paint), eraser, fill bucket, terrain brush (Wang/edge matching)
- **Rendering**: Editor-side canvas draws tiles from tileset atlas; Bevy preview adapter renders via sprite entities or deferred to `bevy_ecs_tilemap`
- **Pros**:
  - Proven UX — every 2D level designer knows Tiled's workflow
  - Dense tile data is efficient (sparse grid, not thousands of entities)
  - Terrain brushes are well-understood
- **Cons**:
  - New data model beyond Scene Instance system
  - Tileset authoring is a prerequisite slice
  - BSN projection unknown (no tile representation in BSN)
- **Effort**: High — tileset model + tile layer + brush tool + terrain rules

### Approach B: LDtk-style IntGrid + Auto Layer (IntGrid-first)

IntGrid is the authoring surface; visual tiles are auto-generated from rules.

- **Data model**: `IntGridLayer { id, name, order, grid_size, cells: SparseGrid<IntValue>, value_defs: [{value:1, color:"#ff0000", name:"wall"}] }` + `AutoLayer { id, name, order, source_layer: LayerId, tileset_ref, rule_groups: [...] }`
- **Tools**: IntGrid paint (integer values + colors), auto-layer rule editor
- **Rendering**: Auto-layer generates tile placements from rule evaluation over IntGrid
- **Pros**:
  - Simpler authoring primitive (just paint integers)
  - Powerful auto-tiling from day one
  - Semantic grid is valuable for collision/gameplay independent of visuals
- **Cons**:
  - Less familiar to Unity users
  - Rule engine is complex (pattern matching, Perlin filters, randomization, flip modes)
  - Requires both IntGrid and Auto Layer to be useful — can't ship one without the other
- **Effort**: High — IntGrid model + auto-layer rule engine + rule editor UI

### Approach C: Hybrid — Tile Layer + Optional IntGrid + Optional Auto Layer (recommended)

Ship in dependency-ordered slices matching the research doc's roadmap.

- **Data model** (built incrementally):
  1. **Tileset** (new Project Asset): `{ id, image_path, tile_width, tile_height, tile_count, tiles: [{id, rect: {x,y,w,h}}] }`
  2. **TileLayer** (new `LevelLayer` variant): `{ id, name, order, grid_width, grid_height, cell_size, tileset_ref, cells: BTreeMap<(i32,i32), TileCell> }`
  3. **IntGridLayer** (new variant, later): `{ id, name, order, grid_width, grid_height, cell_size, cells: BTreeMap<(i32,i32), i32>, value_defs: [...] }`
  4. **AutoLayer** (new variant, last): `{ id, name, order, source_intgrid: LayerId, tileset_ref, rules: [...] }`
- **Tools** (per slice): tile brush → terrain brush → IntGrid painter → auto-layer rule editor
- **Rendering**: Editor canvas draws from tileset atlas. Bevy preview via sprite-per-tile (simple) or `bevy_ecs_tilemap` adapter (when 0.19-compatible).
- **Pros**:
  - Matches the already-accepted implementation roadmap from `level-design-layers-research/design.md`
  - Each slice ships independently — tileset+tile layer is useful without IntGrid
  - Leverages existing `AssetReference` + OPFS persistence patterns
  - Editor-owned JSON stays source of truth; Bevy tilemap crate is optional adapter
  - Sparse `BTreeMap<(i32,i32), TileCell>` is JSON-stable, AI-editable, and efficient for non-dense maps
- **Cons**:
  - Multiple slices needed before all features are available
  - Terrain brush rules still need a format (subset of Tiled Wang or LDtk rules)
- **Effort**: Medium per slice; High across the full change

## Recommendation

**Approach C — Hybrid, sliced per the research doc's roadmap.**

Reasons:
1. **Alignment**: It follows the already-accepted `level-design-layers-research/design.md` implementation roadmap. Reopening that decision would waste completed work.
2. **Pragmatism**: Tileset + Tile Layer alone delivers visible value (paint backgrounds, terrain). IntGrid and Auto Layer are additive, not blocking.
3. **Ecosystem safety**: No Bevy 0.19 tilemap crate exists. The editor-owned JSON model + canvas rendering avoids blocking on upstream. When `bevy_ecs_tilemap` 0.19 lands, it becomes a render adapter behind a trait.
4. **AI-editability**: `BTreeMap<(i32,i32), TileCell>` is sparse, JSON-serializable, and diffable — critical for the project's AI-assisted editing direction.
5. **BSN alignment**: Tile data projection to BSN is deferred (BSN has no tiles yet). The editor-owned model doesn't force a premature BSN decision.

**Slice order for the proposal**:
1. `level-tilesets` — Tileset Project Asset (image + grid metadata + tile definitions). Can import from Aseprite JSON.
2. `level-tile-layer` — `TileLayer` variant + tile brush tool + canvas rendering + WASM surface.
3. `level-intgrid-layer` — `IntGridLayer` variant + IntGrid painter + value definitions (defer to separate change if scope is too large).
4. `level-auto-layer` — `AutoLayer` variant + rule engine + rule editor (defer to separate change).

The proposal should scope slices 1-2 as the primary change, with 3-4 as follow-up changes.

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| BSN has no tile representation — projected output for tiles is undefined | High | Defer BSN tile projection; use DynamicScene export as adapter; revisit when Bevy ships tile BSN |
| No Bevy 0.19 tilemap crate — preview rendering limited to sprite-per-tile | Medium | Start with sprite-per-tile rendering (works today). Add `bevy_ecs_tilemap` adapter when 0.19 version ships. Performance is fine for editor preview sizes. |
| Tileset as new Project Asset kind vs embedded in Level — adds OPFS complexity | Medium | Follow ADR-0008 path-based layout: `assets/tilesets/<name>.tileset.json` + image reference. Reuse existing `AssetCommand` surface. |
| Sparse `BTreeMap<(i32,i32), TileCell>` may be slow for 1000×1000 dense maps | Low | Editor preview doesn't need millions of tiles. Dense format (compressed array) can be a future optimization. Sparse is correct for authoring. |
| Terrain brush rule format is complex (Tiled Wang has 47 edge/corner combinations) | Medium | Start with simple 3x3 neighbor matching (LDtk-style). Full Wang/terrain sets are a follow-up. |
| Canvas rendering in React may not match Bevy preview pixel-perfectly | Low | Both read from the same tileset atlas. Discrepancies are render bugs, not model bugs. |

## Ready for Proposal

**Yes.**

The orchestrator should tell the user:
- The layer model foundation is already implemented (SceneInstanceLayer shipped in PR #28/#29)
- The research doc already chose the canonical layer kinds and deferred Tile/IntGrid/Auto
- This change implements the deferred layers, starting with Tileset + Tile Layer
- The Bevy tilemap ecosystem is one version behind (0.18 vs our 0.19) — we build runtime-agnostic
- Proposed scope: tilesets + tile layer painting. IntGrid and Auto Layer as follow-up changes.
- Next phase: `sddk-propose` to write the change proposal with scope, approach, and slice breakdown.

---

## Envelope

```yaml
status: complete
executive_summary: >
  The level layer model foundation is shipped (SceneInstanceLayer). Tile/IntGrid/Auto
  layers are explicitly deferred in the accepted research design doc. The Bevy tilemap
  ecosystem (bevy_ecs_tilemap/tiled/ldtk) is on Bevy 0.18 — none support our 0.19 yet.
  Recommended approach: hybrid sliced delivery matching the research roadmap — tilesets
  and tile layer first, IntGrid and Auto Layer as follow-up changes. Editor-owned JSON
  remains source of truth; any Bevy tilemap crate is a future render adapter.
context_quality: C1
taxonomy:
  dominant_axes:
    - data_model_extension (new LevelLayer variants + tileset asset)
    - runtime_decoupling (no Bevy 0.19 tilemap crate exists)
    - bsn_projection_unknown (BSN has no tile representation)
    - authoring_ux (tile brush, palette, terrain rules)
next_recommended: sddk-propose
artifacts:
  - sddk/level-design-tools/explore-report
risks:
  - "BSN has no tile representation — projection deferred"
  - "No Bevy 0.19 tilemap crate — sprite-per-tile preview only"
  - "Tileset authoring is a prerequisite slice"
  - "Terrain brush rule format complexity"
```
