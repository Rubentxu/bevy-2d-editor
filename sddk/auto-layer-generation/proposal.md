# Proposal: Auto-Layer Generation

## Intent

Tile painting shipped in `level-design-tools` (PR #34), but every tile is placed by hand — including transition tiles for terrain, walls, and coastlines. Manually placing the right border/corner tile for every adjacency is the single most tedious task in tile-based level design. Auto-tile generation eliminates it: define transition rules once, paint a source layer, and the editor computes the correct tiles automatically.

## Scope

### In Scope
- `AutoLayer` variant in `LevelLayer` enum — a **derived** layer, never hand-painted
- Rule engine: 3×3 neighbor pattern match on a source layer → emit `TileRef` placements
- Rule definitions as editor-owned JSON (serde-stable, AI-editable)
- **First rule source: `TileLayer`** (terrain / edge-matching transitions)
- `RegenerateAutoLayer` command — discrete, OperationLog-undoable
- Rule editor UI (3×3 pattern → output tile mapping, with live preview)

### Out of Scope
- **IntGrid as rule source** — deferred to a follow-up change AFTER `level-intgrid-layer` ships. The research roadmap (`level-design-layers-research/design.md` §Roadmap item 6) orders IntGrid before AutoLayer; skipping IntGrid would break the accepted dependency chain.
- Wang / 47-bit corner terrain — start with 3×3 neighbor matching (covers walls, simple transitions)
- Real-time regeneration on every paint stroke — manual trigger first
- Bevy preview of generated tiles — canvas authoring rendering only
- BSN projection of AutoLayer (no tile BSN representation exists yet)

## Capabilities

> CONTRACT with sddk-spec. No `openspec/specs/` exists — capabilities are net-new.

### New Capabilities
- `auto-layer`: `AutoLayer` variant in `LevelLayer` + `AutoRule` definitions (3×3 pattern → tile mapping) + generation engine that reads a source `TileLayer` and produces computed `TileRef` placements. The layer stores rules + a cached generated result; regeneration is an explicit, undoable command.

### Modified Capabilities
- None. The generation engine reads the existing `TileLayer.grid` via its public API — no data-model change to the tile-layer capability.

## Approach

1. **Rust model** (`auto_layer.rs`): `AutoLayer { id, name, order, source_layer_id: LayerId, tileset_id, rules: Vec<AutoRule>, cached: TileGrid }`. `AutoRule { pattern: Pattern3x3, output: Vec<TileRef>, chance: Option<f32> }`. Add `LevelLayer::Auto(AutoLayer)`.
2. **Generation engine**: iterate source `TileLayer` cells; for each, match its 3×3 neighborhood against rule patterns; write emitted tiles into `AutoLayer.cached`.
3. **Commands**: `RegenerateAutoLayer { layer_id }` follows the existing `PaintTile` processor pattern — apply writes computed tiles, inverse restores prior cache.
4. **WASM bridge**: `regenerate_auto_layer_wasm`, rule CRUD bindings — JSON-string convention per codebase.
5. **Frontend**: `AutoLayerPanel.tsx` (rule editor: 3×3 pattern grid → tile picker), integrated into the existing layer panel.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-core/src/auto_layer.rs` | New | `AutoLayer`, `AutoRule`, `Pattern3x3`, generation engine |
| `crates/editor-core/src/scene_asset.rs` | Modified | `LevelLayer::Auto` variant (line 236) — tagged enum, non-breaking |
| `crates/editor-core/src/command.rs` | Modified | `RegenerateAutoLayer` variant |
| `crates/editor-core/src/processor.rs` | Modified | apply / inverse for regenerate |
| `crates/editor-core/src/lib.rs` | Modified | module export + WASM bindings |
| `frontend/src/components/AutoLayerPanel.tsx` | New | Rule editor UI |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Users expect LDtk-style IntGrid→AutoLayer; we ship TileLayer-sourced only | Medium | TileLayer-sourced terrain auto-tile solves the dominant use case (walls/transitions). IntGrid source is an additive follow-up, not a redesign. Document in CHANGELOG. |
| 3×3 matching insufficient for complex terrain (inner corners, T-junctions) | Medium | Start simple; Wang/full-terrain is a documented follow-up. 3×3 covers straight walls + corners. |
| Rule format surface is large (randomization, Perlin, flip modes) | Medium | Ship minimal subset: pattern + output + optional `chance`. Defer Perlin/flip/symmetric. |
| Cached result stale after source edits | Low | Regeneration is explicit; UI shows stale indicator when source changed since last regen. |

## Rollback Plan

1. Revert `LevelLayer::Auto` variant — `#[serde(tag = "kind")]` means old files load unchanged (unknown kinds ignored).
2. Delete `auto_layer.rs` module and its `lib.rs` export.
3. Remove `RegenerateAutoLayer` from `Command` enum and processor.
4. Delete frontend `AutoLayerPanel.tsx`.
5. No data migration — net-new derived layer; source TileLayers are read-only and unaffected.

## Dependencies

- `level-design-tools` (PR #34) — merged ✅ — `TileLayer`, `Tileset`, `LevelLayer::Tile`, paint/erase WASM surface
- `scene-instance-layer` (PR #29) — merged ✅ — `LevelLayer` enum + tagged-variant pattern
- Research gate: `level-design-layers-research` (PR #27) — cleared ✅ — Approach C (own tile model) accepted

## Success Criteria

- [ ] Can add an AutoLayer to a Level Scene Asset referencing a source TileLayer
- [ ] Can define a 3×3 terrain rule (pattern → output tiles) in the rule editor
- [ ] Regeneration produces correct transition tiles from the source TileLayer
- [ ] Regeneration is undoable via OperationLog (apply / inverse)
- [ ] Editing the source TileLayer marks the AutoLayer as stale until next regen
- [ ] Save / load round-trip preserves AutoLayer rules + cached result (serde test)
- [ ] All existing tests pass — no regression
