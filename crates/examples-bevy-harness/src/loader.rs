//! Translation layer: `editor_model::SceneAssetDocument` → Bevy `World`.
//!
//! Reads the editor's authoring JSON and spawns Bevy entities with
//! the harness's component shells. Translation rules live here so
//! the harness's public API stays small (`spawn_scene_asset`,
//! `spawn_main_scene`).

use bevy::prelude::*;
use editor_model::scene_asset::SceneAssetDocument;
use serde_json::Value;

use crate::components::{EditorSpriteAsset, EnemyPatrol, PlayerController, Visible};

/// Result of translating one scene asset into a Bevy world.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SpawnReport {
    /// Number of entities spawned from `doc.entities`.
    pub entity_count: usize,
    /// Number of components skipped because no harness translation
    /// exists for the `type_id`. Useful for the integration test.
    pub skipped_component_count: usize,
}

/// Spawn every entity from `doc` into `world` and return a spawn report.
///
/// Translation rules (per `ComponentInstance.type_id`):
/// - `editor.Name` → Bevy `Name` component.
/// - `editor.Transform2D` → Bevy `Transform` component.
/// - `editor.Sprite2D` → Bevy `Sprite` component + `EditorSpriteAsset`
///   for the image path (the harness does not load image data).
/// - `editor.Visible` → harness `Visible(bool)` component.
/// - `game.PlayerController` → harness `PlayerController`.
/// - `game.EnemyPatrol` → harness `EnemyPatrol`.
/// - Anything else → skipped (counted in `skipped_component_count`).
pub fn spawn_scene_asset(world: &mut World, doc: &SceneAssetDocument) -> SpawnReport {
    let mut report = SpawnReport::default();
    for entity in &doc.entities {
        let mut bundle = SpawnFromDoc::default();
        for comp in &entity.components {
            match comp.type_id.as_str() {
                "editor.Name" => bundle.name = parse_name(&comp.values),
                "editor.Transform2D" => bundle.transform = parse_transform(&comp.values),
                "editor.Sprite2D" => {
                    bundle.sprite = parse_sprite(&comp.values);
                    if let Some(path) = comp.values.get("asset").and_then(Value::as_str) {
                        bundle.asset = Some(EditorSpriteAsset(path.to_string()));
                    }
                }
                "editor.Visible" => {
                    if let Some(v) = comp.values.get("visible").and_then(Value::as_bool) {
                        bundle.visible = Some(Visible(v));
                    }
                }
                "game.PlayerController" => {
                    bundle.player = Some(PlayerController {
                        speed: comp
                            .values
                            .get("speed")
                            .and_then(Value::as_f64)
                            .unwrap_or(200.0) as f32,
                        jump_force: comp
                            .values
                            .get("jump_force")
                            .and_then(Value::as_f64)
                            .unwrap_or(400.0) as f32,
                    });
                }
                "game.EnemyPatrol" => {
                    bundle.enemy = Some(EnemyPatrol {
                        speed: comp
                            .values
                            .get("speed")
                            .and_then(Value::as_f64)
                            .unwrap_or(60.0) as f32,
                        patrol_range: comp
                            .values
                            .get("patrol_range")
                            .and_then(Value::as_f64)
                            .unwrap_or(150.0) as f32,
                    });
                }
                _ => report.skipped_component_count += 1,
            }
        }
        bundle.spawn_into(world, &entity.name);
        report.entity_count += 1;
    }
    report
}

/// Helper bundle used internally to compose an entity before flushing
/// to the world. `Default::default()` initialises every field with a
/// harmless placeholder so partial JSON can still spawn an entity.
#[derive(Default)]
struct SpawnFromDoc {
    name: Option<Name>,
    transform: Option<Transform>,
    sprite: Option<Sprite>,
    asset: Option<EditorSpriteAsset>,
    visible: Option<Visible>,
    player: Option<PlayerController>,
    enemy: Option<EnemyPatrol>,
}

impl SpawnFromDoc {
    fn spawn_into(self, world: &mut World, fallback_name: &str) {
        let name = self
            .name
            .unwrap_or_else(|| Name::new(fallback_name.to_string()));
        let transform = self.transform.unwrap_or_default();
        let mut entity = world.spawn((name, transform));
        if let Some(sprite) = self.sprite {
            entity.insert(sprite);
        }
        if let Some(asset) = self.asset {
            entity.insert(asset);
        }
        if let Some(visible) = self.visible {
            entity.insert(visible);
        }
        if let Some(player) = self.player {
            entity.insert(player);
        }
        if let Some(enemy) = self.enemy {
            entity.insert(enemy);
        }
    }
}

fn parse_name(values: &Value) -> Option<Name> {
    values
        .get("name")
        .and_then(Value::as_str)
        .map(|s| Name::new(s.to_string()))
}

fn parse_transform(values: &Value) -> Option<Transform> {
    let translation = values
        .get("translation")
        .map(|t| {
            let x = t.get("x").and_then(Value::as_f64).unwrap_or(0.0) as f32;
            let y = t.get("y").and_then(Value::as_f64).unwrap_or(0.0) as f32;
            Vec3::new(x, y, 0.0)
        })
        .unwrap_or(Vec3::ZERO);
    let rotation_z = values
        .get("rotation")
        .and_then(Value::as_f64)
        .unwrap_or(0.0) as f32;
    let scale = values
        .get("scale")
        .map(|s| {
            let x = s.get("x").and_then(Value::as_f64).unwrap_or(1.0) as f32;
            let y = s.get("y").and_then(Value::as_f64).unwrap_or(1.0) as f32;
            Vec3::new(x, y, 1.0)
        })
        .unwrap_or(Vec3::ONE);
    Some(Transform {
        translation,
        rotation: Quat::from_rotation_z(rotation_z),
        scale,
    })
}

fn parse_sprite(values: &Value) -> Option<Sprite> {
    let color = values.get("color").map(|c| {
        let r = c.get("r").and_then(Value::as_f64).unwrap_or(1.0) as f32;
        let g = c.get("g").and_then(Value::as_f64).unwrap_or(1.0) as f32;
        let b = c.get("b").and_then(Value::as_f64).unwrap_or(1.0) as f32;
        let a = c.get("a").and_then(Value::as_f64).unwrap_or(1.0) as f32;
        Color::srgba(r, g, b, a)
    })?;
    Some(Sprite {
        color,
        ..Sprite::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values() -> &'static str {
        r#"{
            "type_id": "editor.Name",
            "values": { "name": "Test" }
        }"#
    }

    #[test]
    fn parse_name_uses_values_field() {
        let ci: editor_model::component::ComponentInstance =
            serde_json::from_str(values()).unwrap();
        let name = parse_name(&ci.values);
        assert_eq!(name.unwrap().as_str(), "Test");
    }

    #[test]
    fn parse_transform_defaults_identity() {
        let v = serde_json::json!({});
        let t = parse_transform(&v).unwrap();
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.scale, Vec3::ONE);
    }
}
