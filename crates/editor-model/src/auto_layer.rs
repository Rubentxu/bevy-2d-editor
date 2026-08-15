//! AutoLayer — auto-tiling generation engine for tile-based level design.

use crate::ids::LayerId;
use crate::tile_layer::TileLayer;
use crate::tileset::{TileCoord, TileGrid, TileRef, TilesetId};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// One cell in a 3x3 auto-tiling pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternCell {
    Filled,
    Empty,
    Any,
}

/// A 3×3 neighborhood pattern for auto-tiling.
pub type Pattern3x3 = [[PatternCell; 3]; 3];

/// One auto-tiling rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoRule {
    pub pattern: Pattern3x3,
    pub output: Vec<TileRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance: Option<f32>,
}

/// Opaque stable identifier for an AutoLayer inside a LevelSceneAsset.
pub type AutoLayerId = LayerId;

/// An AutoLayer generates tiles automatically by pattern-matching against a source TileLayer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLayer {
    pub id: AutoLayerId,
    pub name: String,
    pub order: i32,
    pub source_layer_id: LayerId,
    pub tileset_id: TilesetId,
    pub rules: Vec<AutoRule>,
    #[serde(default)]
    pub cached: TileGrid,
    #[serde(default)]
    pub source_generation: u64,
}

/// Check whether an AutoLayer's cached grid is stale.
pub fn is_auto_layer_stale(auto_layer: &AutoLayer, source: &TileLayer) -> bool {
    auto_layer.source_generation != source.generation
}

/// Regenerate the `cached` tile grid from the source TileLayer.
pub fn regenerate(layer: &mut AutoLayer, source: &TileLayer, rng: &mut impl Rng) {
    let mut new_cached: TileGrid = TileGrid::default();

    let source_cells: Vec<TileCoord> = source.grid.keys().cloned().collect();

    for coord in source_cells {
        let neighborhood = build_neighborhood(source, &coord);

        for rule in &layer.rules {
            if matches_pattern(&neighborhood, &rule.pattern) {
                let fire = match rule.chance {
                    Some(p) => rng.random_range(0.0..1.0) < p,
                    None => true,
                };

                if fire {
                    for tile_ref in &rule.output {
                        new_cached.insert(coord.clone(), tile_ref.clone());
                    }
                    break;
                }
            }
        }
    }

    layer.cached = new_cached;
    layer.source_generation = source.generation;
}

fn build_neighborhood(source: &TileLayer, center: &TileCoord) -> [[Option<TileRef>; 3]; 3] {
    let mut neighborhood: [[Option<TileRef>; 3]; 3] =
        [[None, None, None], [None, None, None], [None, None, None]];

    for dy in 0..3 {
        for dx in 0..3 {
            if dx == 1 && dy == 1 {
                neighborhood[dy][dx] = None;
                continue;
            }

            let offset_x = dx as i32 - 1;
            let offset_y = dy as i32 - 1;
            let neighbor_coord = TileCoord::new(center.x + offset_x, center.y + offset_y);
            neighborhood[dy][dx] = source.get_tile(&neighbor_coord).cloned();
        }
    }

    neighborhood
}

fn matches_pattern(neighborhood: &[[Option<TileRef>; 3]; 3], pattern: &Pattern3x3) -> bool {
    for dy in 0..3 {
        for dx in 0..3 {
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

    #[test]
    fn test_auto_layer_serde_roundtrip_with_empty_cache() {
        let tileset_id = TilesetId::new("ts_grass".to_string());
        let source_layer_id = LayerId::new("lyr_source".to_string());

        let layer = AutoLayer {
            id: AutoLayerId::new("al_01".to_string()),
            name: "Auto Grass".to_string(),
            order: 1,
            source_layer_id,
            tileset_id: tileset_id.clone(),
            rules: vec![AutoRule {
                pattern: [
                    [PatternCell::Any; 3],
                    [PatternCell::Any, PatternCell::Any, PatternCell::Any],
                    [PatternCell::Any; 3],
                ],
                output: vec![
                    TileRef {
                        tileset_id: "ts_grass".to_string(),
                        local_index: 0,
                    },
                    TileRef {
                        tileset_id: "ts_grass".to_string(),
                        local_index: 1,
                    },
                ],
                chance: Some(1.0),
            }],
            cached: TileGrid::default(),
            source_generation: 3,
        };

        let json = serde_json::to_string(&layer).unwrap();
        let roundtrip: AutoLayer = serde_json::from_str(&json).unwrap();

        assert_eq!(roundtrip.id.as_str(), "al_01");
        assert_eq!(roundtrip.rules.len(), 1);
        assert!(roundtrip.cached.is_empty());
    }

    #[test]
    fn test_regenerate_first_match_wins() {
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
            TileRef {
                tileset_id: "ts_test".to_string(),
                local_index: 0,
            },
        );

        let mut layer = AutoLayer {
            id: AutoLayerId::new("al_test".to_string()),
            name: "Test".to_string(),
            order: 0,
            source_layer_id,
            tileset_id: tileset_id.clone(),
            rules: vec![
                AutoRule {
                    pattern: [
                        [PatternCell::Empty; 3],
                        [PatternCell::Empty, PatternCell::Any, PatternCell::Empty],
                        [PatternCell::Empty; 3],
                    ],
                    output: vec![TileRef {
                        tileset_id: "ts_test".to_string(),
                        local_index: 99,
                    }],
                    chance: None,
                },
                AutoRule {
                    pattern: [
                        [PatternCell::Any; 3],
                        [PatternCell::Any, PatternCell::Any, PatternCell::Any],
                        [PatternCell::Any; 3],
                    ],
                    output: vec![TileRef {
                        tileset_id: "ts_test".to_string(),
                        local_index: 100,
                    }],
                    chance: None,
                },
            ],
            cached: TileGrid::default(),
            source_generation: 0,
        };

        let mut rng = StdRng::seed_from_u64(42);
        regenerate(&mut layer, &source, &mut rng);

        let emitted = layer.cached.get(&TileCoord::new(0, 0));
        assert_eq!(
            emitted,
            Some(&TileRef {
                tileset_id: "ts_test".to_string(),
                local_index: 99
            })
        );
    }

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
            source_generation: 5,
        };

        assert!(is_auto_layer_stale(&layer, &source));
    }
}
