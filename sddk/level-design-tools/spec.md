# Spec: Level Design Tools

> Change: `level-design-tools` · Phase: sddk-spec · Path: A-full · Mode: auto
> Source: [`proposal.md`](./proposal.md) · [`explore-report.md`](./explore-report.md)

## §1. Spec Metadata

- **Capabilities:**
  - **NEW**: `tileset-asset` — Tileset Project Asset (image + grid + optional Aseprite metadata)
  - **NEW**: `tile-layer` — `TileLayer` variant in `LevelLayer` enum with sparse `TileRef` grid
  - **NEW**: `tile-brush` — Paint / erase tool for `TileLayer` cells
  - **NEW**: `tileset-import` — Aseprite JSON → Tileset metadata
  - **MODIFIED**: `level-layer` — `LevelLayer` enum gains `TileLayer` variant

## §2. NEW Capability: `tileset-asset`

### Requirement: Tileset Project Asset

A `Tileset` SHALL store image `AssetReference`, `tile_width`, `tile_height`, `columns`, and `spacing`. It MAY store Aseprite metadata. It MUST be a Project Asset with a valid image `AssetReference` that resolves in the Project.

#### Scenario: T1 — Create tileset from image
- GIVEN image asset `assets/tiles/ground.png`
- WHEN a user creates a Tileset with 16x16 tiles and 32 columns
- THEN a Tileset Asset MUST be persisted bound to that image with `tile_width`, `tile_height`, and `columns` exposed downstream

#### Scenario: T2 — Create tileset with Aseprite metadata
- GIVEN a valid Aseprite JSON export alongside the image
- WHEN a user creates a Tileset with that metadata attached
- THEN frames, tags, and slices MUST be stored in the Tileset
- AND the metadata MUST remain optional

#### Scenario: T3 — Reject tileset with missing image
- GIVEN an `AssetReference` path not present in the Project
- WHEN a user attempts to create a Tileset referencing it
- THEN creation MUST be rejected with a validation error
- AND the Project asset list MUST remain unchanged

#### Scenario: T4 — Reject tileset with invalid grid dimensions
- GIVEN `tile_width`, `tile_height`, or `columns` values of zero or negative
- WHEN a user attempts to create a Tileset
- THEN creation MUST be rejected with a validation error identifying the invalid field

## §3. NEW Capability: `tile-layer`

### Requirement: TileLayer in LevelLayer

A `TileLayer` SHALL be a `LevelLayer` variant holding a `Tileset` reference and a sparse grid `HashMap<(i32, i32), TileRef>`. A `TileRef` SHALL contain `tileset_id` and `local_tile_index`. The `TileLayer` MUST persist with its `LevelSceneAsset` and render tiles in the viewport via canvas.

#### Scenario: TL1 — Add TileLayer to empty LevelSceneAsset
- GIVEN an empty `LevelSceneAsset` with a valid `Tileset`
- WHEN a user adds a `TileLayer` referencing that `Tileset`
- THEN the `LevelSceneAsset` MUST persist the new `TileLayer` with empty grid state shown in the layer list

#### Scenario: TL2 — Reject TileLayer referencing missing Tileset
- GIVEN a `Tileset` reference that no longer resolves
- WHEN a user adds a `TileLayer` referencing it
- THEN the editor MUST reject the addition with a reference-validation error
- AND the `LevelSceneAsset` MUST remain unchanged

#### Scenario: TL3 — Paint tile on TileLayer
- GIVEN a `TileLayer` with active brush and valid `Tileset`
- WHEN the user paints at `(3, 5)` with `local_tile_index` 12
- THEN the grid MUST contain `(3, 5) → TileRef`
- AND the viewport MUST display the painted tile

#### Scenario: TL4 — Erase tile from TileLayer
- GIVEN a `TileLayer` with a tile at `(3, 5)`
- WHEN the user erases that cell
- THEN the entry MUST be removed from the grid
- AND the viewport MUST no longer render a tile there

#### Scenario: TL5 — Sparse grid scales with painted tile count
- GIVEN a `TileLayer` on a 1000x1000 grid where 99% of cells are empty
- WHEN the user saves, loads, and renders the `LevelSceneAsset`
- THEN the editor MUST use a sparse `HashMap` representation
- AND serialized size MUST be proportional to painted tile count

## §4. NEW Capability: `tile-brush`

### Requirement: Tile Brush Tool

The Tile Brush SHALL support paint and erase modes on a `TileLayer`: paint places a tile, erase removes it. It SHALL default to a 1x1 footprint. Brush size MAY be configurable later.

#### Scenario: TB1 — Paint tile with brush
- GIVEN an active brush in paint mode targeting a `TileLayer`
- WHEN the user clicks on grid cell `(4, 2)`
- THEN the editor MUST place the selected tile at `(4, 2)` and the viewport MUST show it immediately

#### Scenario: TB2 — Erase tile with brush
- GIVEN an active brush in erase mode and a painted cell at `(4, 2)`
- WHEN the user clicks on that cell
- THEN the editor MUST remove the tile at `(4, 2)` and the viewport MUST clear that render

#### Scenario: TB3 — Switch between paint and erase
- GIVEN an active brush tool
- WHEN the user switches mode from paint to erase or vice versa
- THEN brush behavior MUST switch accordingly
- AND the previous grid state MUST be preserved

## §5. NEW Capability: `tileset-import`

### Requirement: Tileset Import from Aseprite JSON

The editor SHALL import Aseprite JSON exports to create Tileset Assets by parsing the JSON, extracting frame metadata (frames, tags, slices), and creating the asset. The import MUST be non-destructive: existing Tileset Assets with the same identifier MUST NOT be overwritten.

#### Scenario: TI1 — Import valid Aseprite JSON
- GIVEN a valid Aseprite JSON export with frames and tags
- WHEN the user triggers the import
- THEN a new Tileset Asset MUST be created
- AND the Tileset MUST carry the parsed frame metadata

#### Scenario: TI2 — Reject invalid JSON
- GIVEN a file that is not valid JSON or not an Aseprite export
- WHEN the user attempts to import
- THEN the import MUST be rejected with a parser error
- AND no Tileset Asset MUST be created

#### Scenario: TI3 — Reject JSON with missing required fields
- GIVEN an Aseprite JSON missing required fields such as image path or frame list
- WHEN the user attempts to import
- THEN the import MUST be rejected with a schema-validation error
- AND the error MUST identify the missing field

## §6. MODIFIED Capability: `level-layer`

### Requirement: LevelLayer Enum

The `LevelLayer` enum SHALL describe layer kinds in a `LevelSceneAsset` and SHALL have a `TileLayer` variant alongside `SceneInstance`. The enum MUST serialize and deserialize correctly across all variants. The `SceneInstance` variant MUST remain unchanged.
(Previously: `LevelLayer` supported only the `SceneInstance` variant.)

#### Scenario: LL1 — Serialize LevelSceneAsset with TileLayer
- GIVEN a `LevelSceneAsset` containing a `TileLayer`
- WHEN the editor serializes the document
- THEN the output MUST include a `TileLayer` entry with all required fields

#### Scenario: LL2 — Deserialize LevelSceneAsset with TileLayer
- GIVEN a serialized `LevelSceneAsset` containing a `TileLayer` entry
- WHEN the editor loads the document
- THEN a `TileLayer` MUST be reconstructed in memory
- AND all `TileLayer` fields MUST be preserved across the round trip

#### Scenario: LL3 — SceneInstance layer round-trip unaffected
- GIVEN an existing `LevelSceneAsset` containing only a `SceneInstance` layer
- WHEN the editor serializes and deserializes the document
- THEN the `SceneInstance` layer MUST round-trip with identical contents
- AND no regression MUST be introduced by the `TileLayer` variant