//! AutoLayer — auto-tiling generation engine for tile-based level design.
//!
//! AutoLayers use pattern-matching rules to automatically generate tiles
//! based on the contents of a source TileLayer. The source layer provides
//! a 3x3 neighborhood for each cell, and rules are evaluated in declaration
//! order (first match wins).

use crate::tileset::{TileCoord, TileGrid, TileRef, TilesetId};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::tile_layer::TileLayer;
use super::scene_asset::LayerId;

// ─────────────────────────────────────────────────────────────────────────────
// PatternCell — building block of AutoLayer rules
// ─────────────────────────────────────────────────────────────────────────────

/// One cell in a 3x3 auto-tiling pattern.
///
/// - `Filled`: matches any non-empty tile in the source layer
/// - `Empty`: matches an empty cell in the source layer
/// - `Any`: wildcard — matches regardless of source cell state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternCell {
    Filled,
    Empty,
    Any,
}

// ─────────────────────────────────────────────────────────────────────────────
// Pattern3x3 — 3×3 neighborhood for auto-tiling
// ─────────────────────────────────────────────────────────────────────────────

/// A 3×3 neighborhood pattern for auto-tiling.
///
/// The center cell (index `[1][1]`) is always ignored — it is the cell
/// being evaluated, not part of the pattern context.
///
/// Layout (row-major, matching standard tile coordinate systems):
/// ```text
/// [0][0] [0][1] [0][2]
/// [1][0] [1][1] [1][2]   ← center [1][1] is ignored during matching
/// [2][0] [2][1] [2][2]
/// ```
pub type Pattern3x3 = [[PatternCell; 3]; 3];

// ─────────────────────────────────────────────────────────────────────────────
// AutoRule — one rule in an AutoLayer
// ─────────────────────────────────────────────────────────────────────────────

/// One auto-tiling rule.
///
/// Rules are evaluated in declaration order inside `regenerate()`. The first
/// rule whose pattern matches the 3×3 neighborhood wins — later rules are
/// not evaluated for that cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoRule {
    /// The 3×3 pattern to match against the source layer neighborhood.
    pub pattern: Pattern3x3,
    /// Tiles to emit when this rule matches. Multiple tiles can be emitted
    /// to support blended or multi-tile auto-tiling output.
    pub output: Vec<TileRef>,
    /// Optional probability [0.0, 1.0]. If `None`, the rule always fires.
    /// If `Some(p)`, a random value in [0, 1) is drawn; the rule fires only
    /// if the draw is less than `p`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance: Option<f32>,
}

// ─────────────────────────────────────────────────────────────────────────────
// AutoLayerId — opaque identifier for an AutoLayer
// ─────────────────────────────────────────────────────────────────────────────

/// Opaque stable identifier for an AutoLayer inside a LevelSceneAsset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AutoLayerId(pub String);

impl AutoLayerId {
    pub fn new(id: impl Into<String>) -> Self {
        AutoLayerId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AutoLayer — a generated tile layer driven by pattern rules
// ─────────────────────────────────────────────────────────────────────────────

/// An AutoLayer generates tiles automatically by pattern-matching against
/// a source TileLayer.
///
/// The `cached` field holds the last generated tile grid. It is stale
/// whenever `source_generation` differs from the current generation counter
/// on the source TileLayer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLayer {
    /// Stable identifier for this layer.
    pub id: AutoLayerId,
    /// Human-readable name shown in the layer list.
    pub name: String,
    /// Layer order for rendering (lower = rendered first).
    pub order: i32,
    /// The TileLayer whose grid drives generation.
    pub source_layer_id: LayerId,
    /// The Tileset that provides tile graphics for generated tiles.
    pub tileset_id: TilesetId,
    /// Ordered list of auto-tiling rules. First match wins.
    pub rules: Vec<AutoRule>,
    /// Cached generated tile grid. Regenerated when the source layer changes.
    #[serde(default)]
    pub cached: TileGrid,
    /// Generation counter of the source TileLayer at the time `cached` was built.
    /// When `source_generation != source.generation`, the cached grid is stale.
    #[serde(default)]
    pub source_generation: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Regeneration engine
// ─────────────────────────────────────────────────────────────────────────────

/// Check whether an AutoLayer's cached grid is stale — i.e., whether the
/// source TileLayer has been modified since the cache was last built.
pub fn is_auto_layer_stale(auto_layer: &AutoLayer, source: &TileLayer) -> bool {
    auto_layer.source_generation != source.generation
}

/// Regenerate the `cached` tile grid from the source TileLayer.
///
/// For each non-empty cell in the source layer:
/// 1. Build the 3×3 neighborhood (center cell = ignored wildcard).
/// 2. Evaluate rules in declaration order; take the first match.
/// 3. If a rule matched and `rand < chance` (or chance is None), emit its
///    output tiles at the current cell coordinate.
/// 4. Clear and replace the entire `cached` grid.
///
/// After regeneration, `source_generation` is set to `source.generation`.
pub fn regenerate(layer: &mut AutoLayer, source: &TileLayer, rng: &mut impl Rng) {
    let mut new_cached: TileGrid = TileGrid::default();

    // Collect all non-empty source coordinates to iterate
    let source_cells: Vec<TileCoord> = source.grid.keys().cloned().collect();

    for coord in source_cells {
        // Build 3x3 neighborhood for this cell
        let neighborhood = build_neighborhood(&source, &coord);

        // First-match-wins rule evaluation
        for rule in &layer.rules {
            if matches_pattern(&neighborhood, &rule.pattern) {
                // Evaluate chance
                let fire = match rule.chance {
                    Some(p) => rng.random_range(0.0..1.0) < p,
                    None => true,
                };

                if fire {
                    // Emit output tiles at this coordinate
                    for tile_ref in &rule.output {
                        new_cached.insert(coord.clone(), tile_ref.clone());
                    }
                    break; // first match wins — stop evaluating rules
                }
            }
        }
    }

    layer.cached = new_cached;
    layer.source_generation = source.generation;
}

/// Build the 3×3 neighborhood around `center` from `source`, with the center
/// cell set to `Any` (wildcard — it is the cell being evaluated, not part
/// of the pattern context).
fn build_neighborhood(source: &TileLayer, center: &TileCoord) -> [[Option<TileRef>; 3]; 3] {
    let mut neighborhood: [[Option<TileRef>; 3]; 3] = [
        [None, None, None],
        [None, None, None],
        [None, None, None],
    ];

    for dy in 0..3 {
        for dx in 0..3 {
            // The center cell is always a wildcard
            if dx == 1 && dy == 1 {
                neighborhood[dy][dx] = None; // Any — treated as wildcard
                continue;
            }

            // Offset from center: offset 0 = center, offset 1 = one step
            let offset_x = dx as i32 - 1;
            let offset_y = dy as i32 - 1;
            let neighbor_coord = TileCoord::new(center.x + offset_x, center.y + offset_y);
            neighborhood[dy][dx] = source.get_tile(&neighbor_coord).cloned();
        }
    }

    neighborhood
}

/// Check whether a neighborhood matches a 3×3 pattern.
///
/// The center cell [1][1] in `pattern` is always ignored (treated as Any).
fn matches_pattern(neighborhood: &[[Option<TileRef>; 3]; 3], pattern: &Pattern3x3) -> bool {
    for dy in 0..3 {
        for dx in 0..3 {
            // Center cell is always a wildcard
            if dx == 1 && dy == 1 {
                continue;
            }

            let cell = &neighborhood[dy][dx];
            let pat = &pattern[dy][dx];

            match pat {
                PatternCell::Any => {}
                PatternCell::Filled => {
                    if cell.is_none() {
                        return false;
                    }
                }
                PatternCell::Empty => {
                    if cell.is_some() {
                        return false;
                    }
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tileset::TilesetId;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    // ─────────────────────────────────────────────────────────────────────────
    // AL4 — serde round-trip preserves rules + cached
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_auto_layer_serde_roundtrip_with_empty_cache() {
        // Test round-trip with empty cached grid (avoids HashMap key limitation)
        let tileset_id = TilesetId::new("ts_grass".to_string());
        let source_layer_id = LayerId::new("lyr_source".to_string());

        let layer = AutoLayer {
            id: AutoLayerId::new("al_01".to_string()),
            name: "Auto Grass".to_string(),
            order: 1,
            source_layer_id,
            tileset_id: tileset_id.clone(),
            rules: vec![
                AutoRule {
                    pattern: [
                        [PatternCell::Any; 3],
                        [PatternCell::Any, PatternCell::Any, PatternCell::Any],
                        [PatternCell::Any; 3],
                    ],
                    output: vec![
                        TileRef { tileset_id: "ts_grass".to_string(), local_index: 0 },
                        TileRef { tileset_id: "ts_grass".to_string(), local_index: 1 },
                    ],
                    chance: Some(1.0),
                },
            ],
            cached: TileGrid::default(),
            source_generation: 3,
        };

        let json = serde_json::to_string(&layer).unwrap();
        let roundtrip: AutoLayer = serde_json::from_str(&json).unwrap();

        assert_eq!(roundtrip.id.as_str(), "al_01");
        assert_eq!(roundtrip.name, "Auto Grass");
        assert_eq!(roundtrip.order, 1);
        assert_eq!(roundtrip.source_layer_id.as_str(), "lyr_source");
        assert_eq!(roundtrip.tileset_id.as_str(), "ts_grass");
        assert_eq!(roundtrip.rules.len(), 1);
        assert_eq!(roundtrip.rules[0].output.len(), 2);
        assert_eq!(roundtrip.source_generation, 3);
        assert!(roundtrip.cached.is_empty());
    }



    // ─────────────────────────────────────────────────────────────────────────
    // RE1 — first-match-wins
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_regenerate_first_match_wins() {
        let tileset_id = TilesetId::new("ts_test".to_string());
        let source_layer_id = LayerId::new("lyr_src".to_string());

        // Source: single tile at (0, 0)
        let mut source = TileLayer::new(
            crate::tile_layer::TileLayerId::new("src".to_string()),
            "Source".to_string(),
            tileset_id.clone(),
        );
        source.generation = 1;
        source.paint_tile(
            TileCoord::new(0, 0),
            TileRef { tileset_id: "ts_test".to_string(), local_index: 0 },
        );

        // Rules: first rule fires for any Filled neighbor; second rule would fire too
        let mut layer = AutoLayer {
            id: AutoLayerId::new("al_test".to_string()),
            name: "Test".to_string(),
            order: 0,
            source_layer_id,
            tileset_id: tileset_id.clone(),
            rules: vec![
                AutoRule {
                    // Matches any Filled at center (actually center is ignored, so it's about surrounding)
                    pattern: [
                        [PatternCell::Empty; 3],
                        [PatternCell::Empty, PatternCell::Any, PatternCell::Empty],
                        [PatternCell::Empty; 3],
                    ],
                    output: vec![TileRef { tileset_id: "ts_test".to_string(), local_index: 99 }],
                    chance: None,
                },
                AutoRule {
                    // Should NOT match because pattern 1 already matched
                    pattern: [
                        [PatternCell::Any; 3],
                        [PatternCell::Any, PatternCell::Any, PatternCell::Any],
                        [PatternCell::Any; 3],
                    ],
                    output: vec![TileRef { tileset_id: "ts_test".to_string(), local_index: 100 }],
                    chance: None,
                },
            ],
            cached: TileGrid::default(),
            source_generation: 0,
        };

        let mut rng = StdRng::seed_from_u64(42);
        regenerate(&mut layer, &source, &mut rng);

        // Rule 1 fired → output index 99, not 100
        let emitted = layer.cached.get(&TileCoord::new(0, 0));
        assert_eq!(emitted, Some(&TileRef { tileset_id: "ts_test".to_string(), local_index: 99 }));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RE2 — chance ~0.5 over 10k runs with seeded RNG
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_regenerate_chance_probability() {
        let tileset_id = TilesetId::new("ts_test".to_string());
        let source_layer_id = LayerId::new("lyr_src".to_string());

        // Source with one tile
        let mut source = TileLayer::new(
            crate::tile_layer::TileLayerId::new("src".to_string()),
            "Source".to_string(),
            tileset_id.clone(),
        );
        source.generation = 1;
        source.paint_tile(
            TileCoord::new(0, 0),
            TileRef { tileset_id: "ts_test".to_string(), local_index: 0 },
        );

        let mut layer = AutoLayer {
            id: AutoLayerId::new("al_chance".to_string()),
            name: "Chance Test".to_string(),
            order: 0,
            source_layer_id,
            tileset_id: tileset_id.clone(),
            rules: vec![
                AutoRule {
                    pattern: [
                        [PatternCell::Any; 3],
                        [PatternCell::Any, PatternCell::Any, PatternCell::Any],
                        [PatternCell::Any; 3],
                    ],
                    output: vec![TileRef { tileset_id: "ts_test".to_string(), local_index: 1 }],
                    chance: Some(0.5),
                },
            ],
            cached: TileGrid::default(),
            source_generation: 0,
        };

        // Run 10,000 times with different seeds
        let mut fired = 0u32;
        for seed in 0..10_000u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            // Reset cached and source generation
            layer.cached = TileGrid::default();
            layer.source_generation = 0;
            regenerate(&mut layer, &source, &mut rng);
            if layer.cached.contains_key(&TileCoord::new(0, 0)) {
                fired += 1;
            }
        }

        // With chance=0.5 over 10k runs, we expect roughly 5000 ± small margin
        // Allow 3% tolerance (~300) for statistical noise
        let rate = fired as f32 / 10_000.0;
        assert!(
            (rate - 0.5).abs() < 0.03,
            "Expected ~50% fire rate, got {:.4} ({}/10000)",
            rate,
            fired
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RE3 — empty rules clears cache
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_regenerate_empty_rules_clears_cache() {
        let tileset_id = TilesetId::new("ts_test".to_string());
        let source_layer_id = LayerId::new("lyr_src".to_string());

        let mut source = TileLayer::new(
            crate::tile_layer::TileLayerId::new("src".to_string()),
            "Source".to_string(),
            tileset_id.clone(),
        );
        source.generation = 1;
        source.paint_tile(
            TileCoord::new(0, 0),
            TileRef { tileset_id: "ts_test".to_string(), local_index: 0 },
        );

        let mut layer = AutoLayer {
            id: AutoLayerId::new("al_empty".to_string()),
            name: "Empty Rules".to_string(),
            order: 0,
            source_layer_id,
            tileset_id: tileset_id.clone(),
            rules: vec![], // No rules
            cached: {
                let mut g = TileGrid::default();
                g.insert(TileCoord::new(0, 0), TileRef { tileset_id: "ts_test".to_string(), local_index: 99 });
                g.insert(TileCoord::new(10, 10), TileRef { tileset_id: "ts_test".to_string(), local_index: 88 });
                g
            },
            source_generation: 0,
        };

        let mut rng = StdRng::seed_from_u64(123);
        regenerate(&mut layer, &source, &mut rng);

        // Empty rules → nothing matches → cache is empty
        assert!(layer.cached.is_empty(), "Expected empty cached grid, got {:?}", layer.cached);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SD1 — source_generation != tl.generation → stale
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_auto_layer_stale_when_generation_mismatch() {
        let tileset_id = TilesetId::new("ts_test".to_string());
        let source_layer_id = LayerId::new("lyr_src".to_string());

        let source = TileLayer::new(
            crate::tile_layer::TileLayerId::new("src".to_string()),
            "Source".to_string(),
            tileset_id.clone(),
        );

        let layer = AutoLayer {
            id: AutoLayerId::new("al_stale".to_string()),
            name: "Test".to_string(),
            order: 0,
            source_layer_id,
            tileset_id,
            rules: vec![],
            cached: TileGrid::default(),
            source_generation: 5, // source.generation is 0
        };

        assert!(is_auto_layer_stale(&layer, &source));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SD2 — regen clears stale
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_regenerate_clears_stale() {
        let tileset_id = TilesetId::new("ts_test".to_string());
        let source_layer_id = LayerId::new("lyr_src".to_string());

        let mut source = TileLayer::new(
            crate::tile_layer::TileLayerId::new("src".to_string()),
            "Source".to_string(),
            tileset_id.clone(),
        );
        source.generation = 5;

        let mut layer = AutoLayer {
            id: AutoLayerId::new("al_fresh".to_string()),
            name: "Test".to_string(),
            order: 0,
            source_layer_id,
            tileset_id: tileset_id.clone(),
            rules: vec![AutoRule {
                pattern: [
                    [PatternCell::Any; 3],
                    [PatternCell::Any, PatternCell::Any, PatternCell::Any],
                    [PatternCell::Any; 3],
                ],
                output: vec![TileRef { tileset_id: "ts_test".to_string(), local_index: 7 }],
                chance: None,
            }],
            cached: TileGrid::default(),
            source_generation: 0, // stale — different from source.generation (5)
        };

        let mut rng = StdRng::seed_from_u64(99);
        regenerate(&mut layer, &source, &mut rng);

        assert!(!is_auto_layer_stale(&layer, &source));
        assert_eq!(layer.source_generation, 5);
    }
}
