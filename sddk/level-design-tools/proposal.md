# Proposal: Level Design Tools

## Intent

The editor has **no tile-based level authoring capability**. Users cannot paint tiles, define tilesets, or build 2D game levels — the first real production use case for a Bevy 2D editor. This change delivers tileset assets and tile painting, the foundation slice of the accepted hybrid research roadmap (PR #27).

## Scope

### In Scope
- `Tileset` Project Asset: image reference + grid metadata (tile_width, tile_height, columns, spacing) + optional Aseprite metadata
- `TileLayer` variant in `LevelLayer` enum: sparse grid of `TileRef` (tileset_id + local tile index)
- Tile brush tool: paint / erase tiles on a grid
- Tile canvas rendering (HTML5 canvas or CSS grid)
- Aseprite JSON import → tileset metadata
- Own rendering pipeline — **no external Bevy tilemap crate** (all lag Bevy 0.19)

### Out of Scope
- `level-intgrid-layer` — deferred (slice 3)
- `level-auto-layer` — deferred (slice 4)
- Bevy tilemap crate integration (bevy_ecs_tilemap / ldtk / tiled — incompatible with Bevy 0.19)
- BSN tile representation (open question — tile layer doesn't need BSN export yet)
- Tile rotation / flipping flags (follow-up)

## Capabilities

> CONTRACT with sddk-spec. No `openspec/specs/` yet — capabilities are net-new.

### New Capabilities
- `tileset-asset`: Tileset Project Asset type — stores image `AssetReference` + grid dimensions + optional Aseprite frames/tags/slices metadata. Managed via existing Project Asset store.
- `tile-layer`: `TileLayer` variant in `LevelLayer` enum + `TileRef` sparse grid data model. Owns grid dimensions and `Option<TileRef>` cells.
- `tile-brush`: Paint / erase tool operating on a `TileLayer` — `PaintTile` / `EraseTile` command variants with OperationLog undo/redo.
- `tileset-import`: Parse Aseprite JSON export → populate tileset grid metadata (frame dims, tags). Single-asset import flow.

### Modified Capabilities
- `level-layer`: `LevelLayer` enum gains `TileLayer` variant alongside existing `SceneInstance`. Serializer (`#[serde(tag = "kind")]`) already forward-compatible — no migration.

## Approach

1. **Rust model** (`tile.rs`): `Tileset`, `TileRef`, `TileLayer` structs mirroring `SceneInstanceLayer` shape (id, name, order + grid data). Add `LevelLayer::Tile(TileLayer)`.
2. **Commands**: `PaintTile { layer_id, cell_x, cell_y, tile_ref }`, `EraseTile { layer_id, cell_x, cell_y }` — follow `PlaceInstance` processor pattern.
3. **WASM bridge**: tileset CRUD + tile paint/erase bindings in `lib.rs`.
4. **Frontend**: `TileCanvas.tsx` (canvas-based tile rendering), `TileBrushToolbar.tsx`, `TilesetPanel.tsx` (tile picker). Service wrappers in `services/tilesets.ts`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/tile.rs` | New | Tileset, TileRef, TileLayer types |
| `crates/editor-core/src/scene_asset.rs` | Modified | `LevelLayer::Tile` variant (line 235) |
| `crates/editor-core/src/command.rs` | Modified | PaintTile, EraseTile variants |
| `crates/editor-core/src/processor.rs` | Modified | apply/inverse for tile commands |
| `crates/editor-core/src/lib.rs` | Modified | module export + WASM bindings |
| `frontend/src/components/TileCanvas.tsx` | New | Canvas tile rendering |
| `frontend/src/components/TileBrushToolbar.tsx` | New | Brush tool UI |
| `frontend/src/services/tilesets.ts` | New | TS WASM wrappers |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| No Bevy tilemap crate for 0.19 — own renderer | Medium | Canvas rendering decouples from Bevy; preview integration deferred |
| Tile canvas perf for large levels (>1000×1000) | Medium | Sparse grid (`HashMap`) not dense array; viewport culling later |
| BSN tile representation undefined | Low | Tile layer is editor-only data; BSN export deferred |

## Rollback Plan

1. Revert `LevelLayer` enum: remove `Tile(TileLayer)` variant — serializer ignores unknown kinds on load.
2. Delete `tile.rs` module and its `lib.rs` export.
3. Remove `PaintTile`/`EraseTile` from `Command` enum and processor.
4. Delete frontend `TileCanvas`, `TileBrushToolbar`, `services/tilesets.ts`.
5. No data migration — net-new feature, existing scenes unaffected.

## Dependencies

- `scene-instance-layer` (PR #29) — merged ✅ — `LevelLayer` enum + `SceneInstanceLayer` pattern
- `level-scene-asset` (PR #28) — merged ✅ — `SceneAssetRole::Level` + layer container
- Aseprite JSON export format (public spec, no API dependency)

## Success Criteria

- [ ] Can create a Tileset Project Asset from a PNG + grid dimensions
- [ ] Can import tileset metadata from Aseprite JSON export
- [ ] Can add a TileLayer to a Level Scene Asset
- [ ] Can paint / erase tiles on a TileLayer with a brush tool (undoable)
- [ ] Tiles render correctly in the viewport canvas
- [ ] Save / load round-trip preserves TileLayer data (serde round-trip test)
- [ ] All existing tests pass (112+ Rust, 27+ Playwright) — no regression
