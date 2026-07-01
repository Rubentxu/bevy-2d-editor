# Design: Auto-Layer Generation

> Change: `auto-layer-generation` · Phase: sddk-design · Path: A-full · Mode: auto
> Source: [`proposal.md`](./proposal.md) · [`spec.md`](./spec.md)

## Technical Approach

Add a **derived** `LevelLayer::Auto(AutoLayer)` variant that holds 3×3 transition
rules plus a cached `TileGrid`. A generation engine reads the source `TileLayer`
grid (via its public `get_tile` API — no tile-layer model change), matches each
cell's neighborhood against rules in declared order, and writes emitted tiles
into `AutoLayer.cached`. Regeneration is an explicit, undoable `AssetCommand`.
Staleness is tracked by a generation counter on `TileLayer`, bumped on every
paint/erase. This matches the research doc's "editor owns derived data" stance
and the level-design-tools canvas-authoring decision (no Bevy runtime for tiles).

## Architecture Decisions

| # | Decision | Choice | Alternatives | Rationale |
|---|----------|--------|--------------|-----------|
| D1 | AutoLayer storage | **Cached `TileGrid`** persisted on the layer; explicit regen | recompute-on-load | `chance` makes generation non-deterministic (RE2) — recomputing on load would silently change results. AL4 round-trip requires cache to survive save/load. Avoids load latency. |
| D2 | Pattern3x3 wildcard semantics | **Presence-based**: `PatternCell { Filled, Empty, Any }`. `Filled` = source tile exists at offset; `Empty` = none; `Any` = don't-care | tile-index matching | Proposal ships "minimal subset"; presence-match covers binary terrain (wall/not-wall, coast/water). Index-based multi-terrain is a documented follow-up. `Any` is essential for partial-edge rules. |
| D3 | Stale detection | **Generation counter** `TileLayer.generation: u64`, bumped in `paint_tile`/`erase_tile`; `AutoLayer.source_generation: u64` captured at regen. Stale = `source_generation != tl.generation` | (a) content hash, (b) dirty flag set by paint | O(1) check, survives save/load, no hashing cost, matches existing `rebuild_count` counter pattern. Decouples paint from AutoLayer knowledge (paint just bumps its own counter). |
| D4 | Rule serde format | **Self-documenting structs**, `Pattern3x3 = [[PatternCell; 3]; 3]`, snake_case enums. `AutoRule { pattern, output: Vec<TileRef>, chance: Option<f32> }` | compact bitfield / LDtk rule-grid binary | AI-editable + JSON-stable per research doc. 3×3 nested array maps 1:1 to the rule-editor UI grid. |
| D5 | Regenerate undo path | **`AssetCommand::RegenerateAutoLayer`** through `dispatch_asset_command` → `ASSET_OPERATION_LOG` | mimic `paint_tile` direct mutation | Spec RG2 mandates undo. **Code-verified gap**: the proposal claims a "PaintTile processor pattern" exists, but `paint_tile` (`lib.rs:3083`) bypasses every operation log — it is NOT undoable. AssetCommand is the documented asset-level undo surface, so regenerate must route there, not copy paint_tile. |
| D6 | Rule CRUD path | **Direct WASM mutation** mirroring `paint_tile` (mutate cache + `set_asset_document_wasm`) | route every rule edit through AssetCommand | Rule tweaks are light edits like tile paints; the spec requires undo only for regenerate (RG2), not per-rule edits (UI1). Avoids inventing 3+ AssetCommand variants. **Asymmetry noted**: regenerate is undoable, rule edits are not — same gap tile paint already has. |
| D7 | `source_layer_id` type | **`LayerId`** (matches proposal/spec wording); resolved via `.as_str()` against `TileLayerId` since TileLayer uses its own opaque `TileLayerId(pub String)` | `TileLayerId` | Both are `#[serde(transparent)] String`, so interop is trivial string compare. `LayerId` is the cross-cutting layer-reference concept used by `SceneInstanceLayer`. |

## Data Flow

```
AutoLayerPanel ──add/update/remove rule──▶ auto rule CRUD wasm ──▶ mutate SceneAssetDocument.layers[Auto]
                                                     │  (direct, like paint_tile)
                                                     └─ set_asset_document_wasm + cache sync

AutoLayerPanel ──Regenerate──▶ regenerate_auto_layer_wasm(asset_ref, layer_id)
                                    │
                                    ▼
                          AssetCommand::RegenerateAutoLayer
                                    │  (captured old cache for inverse)
                          ┌─────────▼──────────┐
                 source TileLayer.generation   │  (read-only)
                 iterate cells ∪ neighbors      │
                 match Pattern3x3 in rule order │
                 write first-match output      │
                 store new cache + source_gen  │
                          └─────────┬──────────┘
                                    ▼
                          ASSET_OPERATION_LOG (undo restores old cache)

paint_tile / erase_tile ──▶ tl.generation += 1  ──▶ AutoLayer stale on next read (SD1)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/editor-core/src/auto_layer.rs` | Create | `AutoLayer`, `AutoRule`, `Pattern3x3`, `PatternCell`, generation engine `regenerate()` |
| `crates/editor-core/src/scene_asset.rs` | Modify | `LevelLayer::Auto(AutoLayer)` variant (tagged enum, non-breaking, line 236) |
| `crates/editor-core/src/tile_layer.rs` | Modify | Add `generation: u64` (`#[serde(default)]`), bump in `paint_tile`/`erase_tile` |
| `crates/editor-core/src/asset_command.rs` | Modify | `AssetCommand::RegenerateAutoLayer` variant + apply/inverse (verify enum shape) |
| `crates/editor-core/src/lib.rs` | Modify | `pub mod auto_layer;` re-exports + `regenerate_auto_layer_wasm` + rule CRUD WASM surface |
| `frontend/src/services/autoLayer.ts` | Create | WASM bridge wrappers (waitForEngine pattern) |
| `frontend/src/components/AutoLayerPanel.tsx` | Create | Rule editor: 3×3 pattern grid → output picker → chance slider → Regenerate btn |

## Interfaces / Contracts

Non-obvious pattern — extending the tagged enum (non-breaking; old files load
unchanged). The `generation` field uses `#[serde(default)]` so existing
TileLayers deserialize with `generation: 0`:

```rust
// auto_layer.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternCell { Filled, Empty, Any }   // presence-based wildcard

pub type Pattern3x3 = [[PatternCell; 3]; 3];   // [row][col], center = [1][1]

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoRule {
    pub pattern: Pattern3x3,
    pub output: Vec<TileRef>,                 // emitted at center cell
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chance: Option<f32>,                   // [0.0, 1.0]; None = always
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLayer {
    pub id: LayerId,
    pub name: String,
    pub order: i32,
    pub source_layer_id: LayerId,             // references a LevelLayer::Tile by id
    pub tileset_id: TilesetId,                // default picker tileset
    #[serde(default)]
    pub rules: Vec<AutoRule>,
    #[serde(default)]
    pub cached: TileGrid,                      // read-only via brush (AL3)
    #[serde(default)]
    pub source_generation: u64,                // 0 = never regenerated (stale)
}

// scene_asset.rs (add variant)
pub enum LevelLayer {
    SceneInstance(SceneInstanceLayer),
    Tile(TileLayer),
    Auto(AutoLayer),                           // serializes as { "kind": "auto", ... }
}

// tile_layer.rs (add field)
pub struct TileLayer { /* ...existing... */
    #[serde(default)]
    pub generation: u64,                       // bumped on paint/erase; stale-detection key
}
```

WASM surface follows the **JSON-string convention** (regenerate returns updated
asset JSON + result), with direct typed params for rule CRUD (mirroring
`paint_tile`'s signature style):

```rust
// Undoable — routes through AssetCommand
regenerate_auto_layer_wasm(asset_ref, layer_id) -> asset_json   // captures old cache for undo

// Direct mutation — mirrors paint_tile (NOT individually undoable)
add_auto_rule_wasm(asset_ref, layer_id, rule_json)     -> asset_json
update_auto_rule_wasm(asset_ref, layer_id, rule_index, rule_json) -> asset_json
remove_auto_rule_wasm(asset_ref, layer_id, rule_index) -> asset_json
is_auto_layer_stale_wasm(asset_ref, layer_id)          -> bool   // SD1 indicator
```

## Testing Strategy

| Layer | What | How |
|-------|------|-----|
| Unit | `Pattern3x3` serde round-trip; `AutoLayer`/`AutoRule` round-trip (AL4) | `#[test]` in auto_layer.rs |
| Unit | Engine: first-match-wins (RE1); chance gating (~0.5 over 10k runs) (RE2); empty rules clears cache (RE3) | Rust unit tests with deterministic seed |
| Unit | Stale: paint bumps `generation`; stale flips true; regen clears (SD1/SD2) | Rust unit tests on TileLayer + AutoLayer |
| Unit | Regenerate apply → inverse → apply restores prior cache (RG2) | Rust unit tests via `asset_command::apply` |
| Unit | Back-compat: old `SceneAssetDocument` with no Auto variant deserializes; `generation` defaults to 0 | Serde tests on mixed-layer docs |
| Integration | Add AutoLayer → define rule → regenerate → undo → redo round-trip via WASM bridge | Existing WASM bridge harness |
| E2E | Rule editor: toggle pattern → pick tile → regenerate → tiles appear in canvas preview | Playwright (project convention) |

## Migration / Rollout

No migration. `LevelLayer::Auto` is additive (tagged enum — old files ignore
unknown kinds). `TileLayer.generation` uses `#[serde(default)]` so pre-existing
tile layers load with `generation: 0`. `AutoLayer.source_generation: 0` means
"never regenerated" = stale-by-default, which is correct (a fresh AutoLayer
with empty cache is trivially stale until first regen).

## Open Questions

- [ ] **Verify `AssetCommand` enum shape** (`crates/editor-core/src/asset_command.rs`) before adding the `RegenerateAutoLayer` variant — design assumes it mirrors `Command`'s tagged-enum + capture-pre-state-for-inverse pattern used by `dispatch_asset_command`. Not blocking: the WASM dispatch path is clear.
- [ ] RNG source for `chance`: use `rand` crate (already a dep?) or a WASM-safe `js_sys`-seeded RNG. Deterministic-by-seed is preferable for testability (RE2).
- [ ] IntGrid as rule source — explicitly deferred to post-`level-intgrid-layer`; out of scope here.

## ADR Candidates

- **Stale detection via generation counter on TileLayer** (D3) — hard to reverse (persisted field on a shared type), surprising vs. hash/flag approaches, real trade-off (counter drift if paint paths forget to bump) → ADR-NNN.
- **Regenerate undoable via AssetCommand while rule edits are not** (D5/D6 asymmetry) — hard to reverse (two mutation paths), surprising that regen undoes but rule tweaks don't, real trade-off (avoiding AssetCommand-variant proliferation) → ADR-NNN.
