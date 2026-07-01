# Design: level-design-tools

> Slices `level-tilesets` + `level-tile-layer` (research doc §Roadmap items 3–4).
> Builds on accepted `level-design-layers-research` model. NEW capability: no
> external Bevy tilemap crate exists for Bevy 0.19, so the editor owns tile data
> and the authoring canvas renders in the frontend.

## Technical Approach

Extend the existing `LevelLayer` enum with a `Tile(TileLayer)` variant and add
**Tileset** as a new, first-class Project Asset kind (NOT a `SceneAssetRole` —
the research doc explicitly chose "separate Project Asset kind"). Tilesets get
their own OPFS directory (`tilesets/`), catalog type, and WASM surface, mirroring
the proven `SceneAsset` catalog+body split (ADR-0008). Tile authoring renders on
an HTML5 Canvas in the frontend; Bevy preview integration is deferred behind a
future adapter (research doc constraint: "no hard coupling to a specific Bevy
tilemap crate").

## Architecture Decisions

| # | Decision | Choice | Alternatives | Rationale |
|---|----------|--------|--------------|-----------|
| D1 | Tileset identity | Separate Project Asset kind: `TilesetCatalogEntry` + `tilesets/<path>.tileset.json` | (a) Scene Asset with a `tileset` role | Research doc §"Rejected: Tiled-style separate tilemap asset" mandates a distinct kind; reusing SceneAsset conflates image-atlas metadata with entity composition |
| D2 | TileRef shape | `{ tileset_ref: AssetReference, local_index: u32 }` | pixel-space `{pixel_x,pixel_y}` | Index matches atlas/sprite-sheet convention; survives tile resize; self-describing per-tile (AI-editable, JSON-stable per research doc) |
| D3 | Grid key | `TileCoord { x: i32, y: i32 }` in **tile-space**, origin top-left | raw `(i32,i32)` tuple / `(col,row)` | Tuple → JSON array (poor AI legibility); dedicated struct → `{"x":3,"y":5}` (self-documenting). Top-left origin matches image/canvas convention; pixel conversion = `x*tile_w, y*tile_h` |
| D4 | Tiles-per-layer | Data model allows multi-tileset (TileRef carries its own ref); **slice-1 UI constrains to one tileset** | monotileset-only model | Future-proofs without UI cost; avoids a second migration when multi-tileset arrives (LDtk/Tiled support it) |
| D5 | Authoring render | HTML5 Canvas (frontend) | CSS Grid divs / WebGL | Sparse grids + pan/zoom + brush preview need imperative draw; CSS divs don't scale past ~2k cells; WebGL is overkill for 2D tiles |
| D6 | Bevy preview | **Deferred** — no runtime adapter this slice | integrate `bevy_ecs_tilemap` | All three Bevy tilemap crates target 0.18; research doc forbids coupling until Tile Layer slice decides. Canvas authoring is the deliverable |
| D7 | Aseprite import | Regular-grid only (uniform `tile_w×tile_h`, row-major `local_index`) | irregular sprite frames | Slice-1 tilesets are uniform grids; irregular frames need per-tile rects → defer to a follow-up |

## Data Flow

```
TilesetPanel ──createTileset──▶ tilesets.ts ──▶ create_tileset_wasm ──▶ TilesetCatalog + OPFS body
                                                                                │
TileCanvas ◀──tileset grid── tilesets.ts                                         │
     │                                                                           │
     └─paint/erase─▶ paint_tile_wasm / erase_tile_wasm ─▶ mutate TileLayer.tiles │
                                                            (inside SceneAssetDocument)
                                                                       │
                                                           set_asset_document_wasm
                                                           + save_scene_asset (OPFS)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/editor-core/src/tileset.rs` | Create | `TilesetAsset`, `TilesetCatalogEntry`, `TileRef`, `TileCoord` |
| `crates/editor-core/src/tile_layer.rs` | Create | `TileLayer` + `LevelLayer::Tile` variant (extends `scene_asset.rs`) |
| `crates/editor-core/src/scene_asset.rs` | Modify | Add `Tile(TileLayer)` to `LevelLayer` enum; re-export from `tile_layer` |
| `crates/editor-core/src/persistence.rs` | Modify | `TILESETS_DIR`, `tileset_path()`, `TilesetCatalogEntry` in `ProjectMetadata` (`#[serde(default)]`) |
| `crates/editor-core/src/lib.rs` | Modify | `pub mod tileset; pub mod tile_layer;` + re-exports + tileset WASM surface + `tileset_catalog` thread_local |
| `frontend/src/services/tilesets.ts` | Create | WASM bridge wrappers (waitForEngine pattern) |
| `frontend/src/components/TileCanvas.tsx` | Create | HTML5 Canvas paint/erase + grid + brush cursor |
| `frontend/src/components/TilesetPanel.tsx` | Create | Tileset browser, import, grid preview, tile picker |

## Interfaces / Contracts

Non-obvious pattern — extending the tagged enum (non-breaking: old level files
with no tile layer deserialize unchanged):

```rust
// tile_layer.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TileCoord { pub x: i32, pub y: i32 }   // tile-space, top-left origin

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileRef { pub tileset_ref: AssetReference, pub local_index: u32 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileLayer {
    pub id: LayerId,
    pub name: String,
    pub order: i32,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tiles: std::collections::HashMap<TileCoord, TileRef>,  // sparse
}

// scene_asset.rs  (add variant)
pub enum LevelLayer {
    SceneInstance(SceneInstanceLayer),
    Tile(TileLayer),           // serializes as { "kind": "tile", ... }
}
```

WASM surface follows the **existing JSON-string convention** (NOT the typed
returns in the proposal — the codebase uniformly returns `Result<String, JsValue>`):

```rust
create_tileset(name, image_ref, tile_w, tile_h, columns) -> catalog_entry_json
list_tilesets() -> catalog_json
paint_tile_wasm(asset_json, layer_id, coord_json, tile_ref_json) -> asset_json
erase_tile_wasm(asset_json, layer_id, coord_json) -> asset_json
import_tileset_from_aseprite(name, json) -> catalog_entry_json   // regular grid only
```

## Testing Strategy

| Layer | What | How |
|-------|------|-----|
| Unit | `TileCoord` serde round-trip; `LevelLayer::Tile` deserializes from/with old SceneInstance layers (back-compat); `tileset_path` format | `#[test]` in persistence.rs / tile_layer.rs |
| Unit | Sparse grid paint/erase idempotency; erase-then-read = empty | Rust unit tests on a `TileLayer` |
| Integration | OPFS tileset create→list→open round-trip | Existing WASM bridge test harness |
| E2E | Paint tiles on canvas → save → reload → tiles persist | Playwright (project convention) |

## Migration / Rollout

No migration. `ProjectMetadata.tilesets` uses `#[serde(default)]` so pre-existing
`project.json` loads unchanged. `LevelLayer::Tile` is additive (tagged enum).

## Open Questions

- [ ] Bevy runtime tile rendering adapter crate — **deferred to a follow-up**; research doc gates it.
- [ ] BSN projection of tile data (no canonical `.bsn` tile form yet) — out of scope, output-only deferred like the research doc states.

## ADR Candidates

- **Tileset as separate Project Asset kind** (D1) — hard to reverse (persisted shape + catalog), surprising vs. reusing SceneAssetRole, real trade-off → ADR-NNN.
