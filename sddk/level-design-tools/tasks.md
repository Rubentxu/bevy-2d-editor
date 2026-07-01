# Tasks: level-design-tools (tilesets + tile painting)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~1000 (300 backend + 100 WASM + 400 frontend + 200 tests) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR1 (backend types + OPFS) → PR2 (WASM bridge) → PR3 (frontend UI + tests) |
| Delivery strategy | auto-chain |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Backend domain types + OPFS persistence | PR 1 | Tileset/TileLayer types, LevelLayer::Tile variant, OPFS save/load. Base: main. |
| 2 | Command + Processor + WASM + TS bindings | PR 2 | PaintTile/EraseTile commands, processor handling, JS bridge. Stacked on PR1. |
| 3 | Frontend UI + integration + tests | PR 3 | Canvas, panels, brush toolbar, unit + Playwright E2E. Stacked on PR2. |

## Phase 1: Backend Domain Types (PR 1)

- [ ] 1.1 Create `crates/editor-core/src/tileset.rs` — `TilesetAsset`, `TilesetId` (newtype), `TilesetMetadata` structs with `Serialize/Deserialize`; image dim + tile_w/tile_h validation in constructor.
- [ ] 1.2 RED test in `crates/editor-core/src/tileset.rs` — assert invalid tile_w (0) and dim_not_multiple of tile_w return typed errors.
- [ ] 1.3 GREEN — finalize `TilesetAsset::new(name, image_w, image_h, tile_w, tile_h)` returning `Result`.
- [ ] 1.4 Create `crates/editor-core/src/tile_layer.rs` — `TileCoord { x: i32, y: i32 }`, `TileRef { tileset_ref: AssetReference, local_index: u32 }`, `TileLayer` with `tiles: HashMap<TileCoord, TileRef>`.
- [ ] 1.5 RED test in `tile_layer.rs` — paint(coord, ref) and erase(coord) update sparse grid; serde round-trip preserves key order.
- [ ] 1.6 Modify `crates/editor-core/src/scene_asset.rs` — add `LevelLayer::Tile(TileLayer)` variant to enum (line 235); keep `#[serde(tag = "kind")]` compatibility.
- [ ] 1.7 Modify `crates/editor-core/src/persistence.rs` — add `TILESETS_DIR = "tilesets"` const, `tileset_path(logical_path)` fn (mirrors `asset_path`), update `ProjectMetadata` with `tilesets: BTreeMap<String, TilesetCatalogEntry>`.
- [ ] 1.8 Add OPFS bridge fns in `persistence.rs` — `js_save_tileset`, `js_load_tileset`, `js_delete_tileset` mirroring existing `js_save_file` async pattern.
- [ ] 1.9 Modify `crates/editor-core/src/lib.rs` — declare `pub mod tileset; pub mod tile_layer;` and re-export key types via `pub use`.

## Phase 2: Command + Processor + WASM Bridge (PR 2)

- [ ] 2.1 Modify `crates/editor-core/src/command.rs` — add `Command::PaintTile { layer_id, coord, tile_ref }` and `Command::EraseTile { layer_id, coord }` variants with `Serialize/Deserialize`.
- [ ] 2.2 RED test in `processor.rs` — PaintTile returns inverse `EraseTile`; EraseTile returns inverse `PaintTile` restoring prior TileRef.
- [ ] 2.3 GREEN — extend `apply()` in `processor.rs` to handle PaintTile/EraseTile; locate LevelLayer by id, mutate sparse HashMap, capture prior ref for inverse.
- [ ] 2.4 Add `create_tileset_wasm(json)` and `import_tileset_from_aseprite_wasm(json)` to `lib.rs` — return `Result<String, JsValue>` with serialized TilesetAsset JSON.
- [ ] 2.5 Add `add_tile_layer_to_level_wasm(asset_json, name)` and `paint_tile_wasm(asset_json, layer_id, x, y, tileset_ref, local_index)` to `lib.rs`; erase variant too. Mirror existing `create_scene_instance_layer_wasm` style.
- [ ] 2.6 Modify `frontend/src/engine-bridge.ts` — add TS bindings for all 5 new WASM fns returning JSON strings; type the parsed return shapes.

## Phase 3: Frontend UI + Integration (PR 3)

- [ ] 3.1 Create `frontend/src/services/tilesets.ts` — typed wrapper around engine-bridge for tileset CRUD + import.
- [ ] 3.2 Create `frontend/src/components/TileCanvas.tsx` — HTML5 Canvas rendering tile grid with `requestAnimationFrame`; props: layer, tileset image, zoom.
- [ ] 3.3 Create `frontend/src/components/TilesetPanel.tsx` — tileset list + create button + aseprite import trigger + per-tileset tile picker grid.
- [ ] 3.4 Create `frontend/src/components/TileBrushToolbar.tsx` — paint/erase mode toggle + selected tileset_id + local_index display.
- [ ] 3.5 Integrate `TilesetPanel` into editor layout — mount in `TopBar.tsx` or new right-side panel; dispatch paint/erase on canvas click.

## Phase 4: Verification

- [ ] 4.1 Unit tests in `tile_layer.rs` — serde key order deterministic (BTreeMap iteration), 1000-tile paint/erase perf < 50ms.
- [ ] 4.2 Integration test in `crates/editor-core/tests/tileset_roundtrip.rs` — create tileset → save OPFS → reload → assert equal JSON.
- [ ] 4.3 Playwright E2E in `e2e/level-tiles.spec.ts` — create tileset, add tile layer, paint cell, save project, reload page, verify painted cell persists.