# Tasks: auto-layer-generation

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~900-1100 (450 backend + 100 WASM + 350 frontend + 200 tests) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR1 (types+engine+counter) → PR2 (AssetCommand+WASM+TS service) → PR3 (UI+tests) |
| Delivery strategy | auto-chain |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Backend types + generation engine + TileLayer counter | PR 1 | AutoLayer/AutoRule/Pattern3x3, regenerate(), generation field. Base: main. |
| 2 | AssetCommand::RegenerateAutoLayer + apply/inverse + WASM + TS service | PR 2 | Undo path, 5 WASM fns, autoLayer.ts bridge. Stacked on PR1. |
| 3 | AutoLayerPanel UI + stale banner + E2E | PR 3 | 3×3 pattern grid, tile picker, chance slider, regen btn, Playwright. Stacked on PR2. |

## Phase 1: Backend Types + Engine (PR 1)

- [ ] 1.1 Create `crates/editor-core/src/auto_layer.rs` — `PatternCell { Filled, Empty, Any }`, `type Pattern3x3 = [[PatternCell; 3]; 3]`, `AutoRule { pattern, output: Vec<TileRef>, chance: Option<f32> }`, `AutoLayer { id, name, order, source_layer_id, tileset_id, rules, cached: TileGrid, source_generation }` with snake_case serde.
- [ ] 1.2 RED test in `auto_layer.rs` — serde round-trip preserves rules + cached (AL4); pre-Auto `SceneAssetDocument` deserializes unchanged.
- [ ] 1.3 Modify `crates/editor-core/src/tile_layer.rs` — add `#[serde(default)] pub generation: u64`; bump `+= 1` in `paint_tile`/`erase_tile`.
- [ ] 1.4 Modify `crates/editor-core/src/scene_asset.rs` — add `LevelLayer::Auto(AutoLayer)` variant to tagged enum; verify old files load.
- [ ] 1.5 Modify `crates/editor-core/src/lib.rs` — `pub mod auto_layer;` + re-export `Pattern3x3`, `AutoRule`, `AutoLayer`.
- [ ] 1.6 Implement `regenerate(layer: &mut AutoLayer, source: &TileLayer, rng: &mut impl Rng)` — iterate cells, build neighborhood via `source.get_tile()`, match rules in declared order, emit first match, set `source_generation = source.generation`.
- [ ] 1.7 RED tests for engine — first-match-wins (RE1), chance ~0.5 over 10k runs with seeded RNG (RE2), empty rules clears cache (RE3).

## Phase 2: AssetCommand + WASM Bridge (PR 2)

- [ ] 2.1 Modify `crates/editor-core/src/asset_command.rs` — verify enum shape (Open Q1); add `AssetCommand::RegenerateAutoLayer { asset_ref, layer_id }` capturing old `cached`+`source_generation` for inverse.
- [ ] 2.2 RED test for RG2 — apply regen → inverse → cached restored to C1.
- [ ] 2.3 Add `regenerate_auto_layer_wasm(asset_ref, layer_id) -> asset_json` and `is_auto_layer_stale_wasm(...) -> bool` to `lib.rs` — route regen through `dispatch_asset_command`.
- [ ] 2.4 Add rule CRUD WASM bindings `add_auto_rule_wasm`, `update_auto_rule_wasm`, `remove_auto_rule_wasm` (JSON-string payload, direct mutation like `paint_tile`).
- [ ] 2.5 Create `frontend/src/services/autoLayer.ts` — typed wrappers using `waitForEngine`; types `AutoRule`, `Pattern3x3`, `PatternCell`.

## Phase 3: Frontend UI + Tests (PR 3)

- [ ] 3.1 Create `frontend/src/components/AutoLayerPanel.tsx` — 3×3 pattern grid (center disabled per D2), tile picker bound to `tileset_id`, `chance` slider, Regenerate button.
- [ ] 3.2 Add `useAutoLayerStale(layerId)` hook + stale banner with "Regenerate to update" CTA.
- [ ] 3.3 Integrate into `LayerPanel` — render when selected layer `kind === "auto"`.
- [ ] 3.4 Unit tests in `auto_layer.rs` — `source_generation != tl.generation` → stale (SD1); regen clears stale (SD2); ref-validation rejects non-Tile source (AL2/RG3).
- [ ] 3.5 Integration test `crates/editor-core/tests/auto_layer_roundtrip.rs` — add layer → rule → regen → undo → redo via WASM harness.
- [ ] 3.6 Playwright E2E `e2e/auto-layer.spec.ts` — toggle pattern, pick tile, Regenerate, assert `TileCanvas` updates, Undo restores.