//! DynamicScene Export — adapter that materializes a SceneDocument into a
//! Bevy-compatible runtime scene representation.
//!
//! This module implements Hito 0 §9.5 (the editor → Bevy runtime mapping).
//! The output is a stable JSON artifact that a real Bevy 0.19 application
//! can consume. The export is one-way (read-only) and never fails on
//! business-rule violations — only on parse/serialize errors.
//!
//! ## Mapping summary
//!
//! | Editor schema | Bevy component |
//! |---|---|
//! | `editor.Name` | `bevy.Name { name }` |
//! | `editor.Transform2D` | `bevy.Transform { translation, rotation, scale }` |
//! | `editor.Sprite2D` (asset set) | `bevy.Sprite { asset, color, anchor }` |
//! | `editor.Sprite2D` (empty asset) | (omitted) + warning |
//! | `editor.Visible` / `editor.Locked` | (silently omitted) |
//! | Unknown type_id | (omitted) + warning |
//! | Entity `parent` | `parent_stable_id` (Bevy mints IDs at load time) |
//!
//! See ADR-0004 for the anchor mapping decision (Bevy native `Sprite::anchor`).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use thiserror::Error;

use crate::document::{ComponentInstance, Entity, SceneDocument};

/// Format version. Bump on breaking schema changes to the exported JSON.
pub const EXPORT_VERSION: &str = "0.1.0";

/// Top-level export artifact — the JSON-serializable Bevy-compatible scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicSceneExport {
    /// Format version.
    pub version: String,
    /// The source SceneDocument's scene_id, for traceability.
    pub source_scene_id: String,
    /// All entities in the export, in document order.
    pub entities: Vec<EntityExport>,
    /// Non-fatal issues encountered during the mapping (e.g., missing assets,
    /// unknown components, orphans). Empty on a clean export.
    pub warnings: Vec<ExportWarning>,
}

/// One entity in the exported scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityExport {
    /// The editor StableId (opaque string). Not a Bevy Entity ID.
    pub stable_id: String,
    /// Human-readable entity name.
    pub name: String,
    /// StableId of the parent entity, or `None` if this entity is a root.
    /// Bevy mints its own Entity IDs at load time; this reference is resolved
    /// by the loader before inserting `ChildOf`.
    pub parent_stable_id: Option<String>,
    /// Bevy components attached to this entity. Keys are `bevy.*` prefixed.
    /// `BTreeMap` ensures deterministic alphabetical key ordering.
    pub components: BTreeMap<String, serde_json::Value>,
}

/// Non-fatal issue encountered during export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportWarning {
    /// The StableId of the affected entity, if applicable.
    pub entity_stable_id: Option<String>,
    /// The editor schema type_id of the affected component, if applicable.
    pub component_type_id: Option<String>,
    /// Human-readable message describing the issue.
    pub message: String,
}

/// Errors that can occur during export. Only returned for parse/serialize
/// failures — business-rule violations are reported as warnings.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Failed to parse SceneDocument JSON: {0}")]
    ParseError(String),
    #[error("Failed to serialize export: {0}")]
    SerializeError(String),
}

/// Export a SceneDocument to a Bevy-compatible JSON artifact.
///
/// Never fails on business-rule violations (missing assets, unknown schemas,
/// invalid values, orphan parents) — those become warnings. Only returns
/// an error if the input itself cannot be parsed or the output cannot be
/// serialized.
pub fn export_dynamic_scene(doc: &SceneDocument) -> Result<DynamicSceneExport, ExportError> {
    let mut warnings: Vec<ExportWarning> = Vec::new();

    // Validate parent references up front.
    let valid_ids: HashSet<&str> = doc.entities.iter().map(|e| e.id.as_str()).collect();

    let entity_exports: Vec<EntityExport> = doc
        .entities
        .iter()
        .map(|entity| build_entity_export(entity, &valid_ids, &mut warnings))
        .collect();

    Ok(DynamicSceneExport {
        version: EXPORT_VERSION.to_string(),
        source_scene_id: doc.scene_id.clone(),
        entities: entity_exports,
        warnings,
    })
}

fn build_entity_export(
    entity: &Entity,
    valid_ids: &HashSet<&str>,
    warnings: &mut Vec<ExportWarning>,
) -> EntityExport {
    let parent_stable_id = resolve_parent(entity, valid_ids, warnings);
    let components = map_components(entity, warnings);
    EntityExport {
        stable_id: entity.id.to_string(),
        name: entity.name.clone(),
        parent_stable_id,
        components,
    }
}

fn resolve_parent(
    entity: &Entity,
    valid_ids: &HashSet<&str>,
    warnings: &mut Vec<ExportWarning>,
) -> Option<String> {
    match &entity.parent {
        None => None,
        Some(p) => {
            if valid_ids.contains(p.as_str()) {
                Some(p.to_string())
            } else {
                warnings.push(ExportWarning {
                    entity_stable_id: Some(entity.id.to_string()),
                    component_type_id: None,
                    message: format!(
                        "Entity {} parent {} not found in scene; exporting as root",
                        entity.id, p
                    ),
                });
                None
            }
        }
    }
}

fn map_components(
    entity: &Entity,
    warnings: &mut Vec<ExportWarning>,
) -> BTreeMap<String, serde_json::Value> {
    let mut out: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    for component in &entity.components {
        match component.type_id.as_str() {
            "editor.Name" => {
                out.insert("bevy.Name".to_string(), map_name(component, entity, warnings));
            }
            "editor.Transform2D" => {
                out.insert(
                    "bevy.Transform".to_string(),
                    map_transform(component, entity, warnings),
                );
            }
            "editor.Sprite2D" => {
                if let Some(sprite) = map_sprite(component, entity, warnings) {
                    out.insert("bevy.Sprite".to_string(), sprite);
                }
            }
            "editor.Visible" | "editor.Locked" => {
                // Editorial-only components: silently skipped per §9.5.
            }
            unknown => {
                warnings.push(ExportWarning {
                    entity_stable_id: Some(entity.id.to_string()),
                    component_type_id: Some(unknown.to_string()),
                    message: format!(
                        "Skipping unknown component '{}' on entity {}",
                        unknown, entity.id
                    ),
                });
            }
        }
    }

    out
}

fn map_name(
    component: &ComponentInstance,
    entity: &Entity,
    warnings: &mut Vec<ExportWarning>,
) -> serde_json::Value {
    match component.values.get("name").and_then(|v| v.as_str()) {
        Some(name) => serde_json::json!({ "name": name }),
        None => {
            warnings.push(ExportWarning {
                entity_stable_id: Some(entity.id.to_string()),
                component_type_id: Some("editor.Name".to_string()),
                message: format!("Name missing on entity {}; using empty string", entity.id),
            });
            serde_json::json!({ "name": "" })
        }
    }
}

fn map_transform(
    component: &ComponentInstance,
    entity: &Entity,
    warnings: &mut Vec<ExportWarning>,
) -> serde_json::Value {
    // Translation: Vec2 {x, y} → [x, y, 0].
    // Distinguish absent (silent default) from present-but-invalid (warn + default).
    let translation = parse_vec2_or_warn(
        component.values.get("translation"),
        "translation",
        "editor.Transform2D",
        entity,
        warnings,
    )
    .unwrap_or([0.0_f32, 0.0_f32, 0.0_f32]);

    // Rotation: f32 radians around z → Bevy quaternion [x, y, z, w].
    // Absent → identity silently. Invalid type → warn + identity.
    let rotation_rad = match component.values.get("rotation") {
        None => 0.0_f32,
        Some(v) => match v.as_f64() {
            Some(r) => r as f32,
            None => {
                warnings.push(ExportWarning {
                    entity_stable_id: Some(entity.id.to_string()),
                    component_type_id: Some("editor.Transform2D".to_string()),
                    message: format!(
                        "Transform2D rotation invalid on entity {}; using 0",
                        entity.id
                    ),
                });
                0.0_f32
            }
        },
    };
    let half = rotation_rad * 0.5;
    let (s, c) = half.sin_cos();
    let rotation = [0.0_f32, 0.0_f32, s, c];

    // Scale: Vec2 {x, y} → [x, y, 1]. Same absent vs invalid pattern.
    let scale = parse_vec2_or_warn(
        component.values.get("scale"),
        "scale",
        "editor.Transform2D",
        entity,
        warnings,
    )
    .unwrap_or([1.0_f32, 1.0_f32, 1.0_f32]);

    serde_json::json!({
        "translation": translation,
        "rotation": rotation,
        "scale": scale,
    })
}

/// Parse a `Vec2` from `Option<&serde_json::Value>`.
///
/// - `None` → silently returns `None` (caller uses default, no warning).
/// - `Some(value)` with valid `{x, y}` → returns `Some([x, y, z])`.
/// - `Some(value)` with invalid shape → emits a warning, returns `None`
///   (caller uses default).
fn parse_vec2_or_warn(
    value: Option<&serde_json::Value>,
    field_name: &str,
    component_type_id: &str,
    entity: &Entity,
    warnings: &mut Vec<ExportWarning>,
) -> Option<[f32; 3]> {
    let v = match value {
        None => return None,
        Some(v) => v,
    };
    let z = if field_name == "translation" { 0.0 } else { 1.0 };
    match (v.get("x").and_then(|x| x.as_f64()), v.get("y").and_then(|y| y.as_f64())) {
        (Some(x), Some(y)) => Some([x as f32, y as f32, z]),
        _ => {
            warnings.push(ExportWarning {
                entity_stable_id: Some(entity.id.to_string()),
                component_type_id: Some(component_type_id.to_string()),
                message: format!(
                    "Transform2D {} invalid on entity {}; using default",
                    field_name, entity.id
                ),
            });
            None
        }
    }
}

fn map_sprite(
    component: &ComponentInstance,
    entity: &Entity,
    warnings: &mut Vec<ExportWarning>,
) -> Option<serde_json::Value> {
    let asset = component
        .values
        .get("asset")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if asset.is_empty() {
        warnings.push(ExportWarning {
            entity_stable_id: Some(entity.id.to_string()),
            component_type_id: Some("editor.Sprite2D".to_string()),
            message: format!(
                "Sprite2D on entity {} has empty asset path; Sprite omitted",
                entity.id
            ),
        });
        return None;
    }

    // Color: defaults to white if missing/invalid.
    let color = match component.values.get("color").and_then(|v| {
        let r = v.get("r").and_then(|x| x.as_f64())? as f32;
        let g = v.get("g").and_then(|x| x.as_f64())? as f32;
        let b = v.get("b").and_then(|x| x.as_f64())? as f32;
        let a = v.get("a").and_then(|x| x.as_f64())? as f32;
        Some([r, g, b, a])
    }) {
        Some(c) => c,
        None => {
            warnings.push(ExportWarning {
                entity_stable_id: Some(entity.id.to_string()),
                component_type_id: Some("editor.Sprite2D".to_string()),
                message: format!(
                    "Sprite2D color invalid or missing on entity {}; using white",
                    entity.id
                ),
            });
            [1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32]
        }
    };

    // Anchor: defaults to Center if invalid.
    let anchor = component
        .values
        .get("anchor")
        .and_then(|v| v.as_str())
        .and_then(anchor_str_to_bevy)
        .unwrap_or_else(|| {
            warnings.push(ExportWarning {
                entity_stable_id: Some(entity.id.to_string()),
                component_type_id: Some("editor.Sprite2D".to_string()),
                message: format!("Sprite2D anchor invalid on entity {}; using Center", entity.id),
            });
            "Center"
        });

    Some(serde_json::json!({
        "asset": asset,
        "color": color,
        "anchor": anchor,
    }))
}

/// Map our `Anchor` enum string to Bevy's `bevy_sprite::Anchor` PascalCase string.
/// Returns `None` if the string doesn't match any known anchor.
fn anchor_str_to_bevy(s: &str) -> Option<&'static str> {
    Some(match s {
        "Center" => "Center",
        "TopLeft" => "TopLeft",
        "TopRight" => "TopRight",
        "BottomLeft" => "BottomLeft",
        "BottomRight" => "BottomRight",
        "TopCenter" => "TopCenter",
        "BottomCenter" => "BottomCenter",
        "CenterLeft" => "CenterLeft",
        "CenterRight" => "CenterRight",
        _ => return None,
    })
}

/// Returns the normalized offset (relative to sprite size) for a given anchor string.
/// Bevy 0.19's `Anchor(Vec2)` follows the convention that the offset is the pivot point's
/// position relative to the sprite center, where (0, 0) = sprite center, (-0.5, -0.5) =
/// bottom-left corner, (0.5, 0.5) = top-right corner.
///
/// Returns `(0.0, 0.0)` (= `Anchor::CENTER`) for unknown or empty strings.
///
/// Bevy-free on purpose: this is the canonical table. The Bevy-dependent helper
/// `bevy_anchor::anchor_str_to_bevy_anchor` wraps this to produce a `bevy::sprite::Anchor`.
pub fn anchor_str_to_normalized_offset(s: &str) -> (f32, f32) {
    match s {
        "Center" => (0.0, 0.0),
        "TopLeft" => (-0.5, 0.5),
        "TopCenter" => (0.0, 0.5),
        "TopRight" => (0.5, 0.5),
        "CenterLeft" => (-0.5, 0.0),
        "CenterRight" => (0.5, 0.0),
        "BottomLeft" => (-0.5, -0.5),
        "BottomCenter" => (0.0, -0.5),
        "BottomRight" => (0.5, -0.5),
        _ => (0.0, 0.0),
    }
}

/// Returns true if the string is one of the 9 known anchor names.
pub fn is_known_anchor_str(s: &str) -> bool {
    matches!(
        s,
        "Center"
            | "TopLeft"
            | "TopCenter"
            | "TopRight"
            | "CenterLeft"
            | "CenterRight"
            | "BottomLeft"
            | "BottomCenter"
            | "BottomRight"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Entity, StableId};
    use serde_json::json;

    fn make_doc(entities: Vec<Entity>) -> SceneDocument {
        SceneDocument {
            version: "0.1".to_string(),
            scene_id: "scene_001".to_string(),
            name: "Test".to_string(),
            entities,
        }
    }

    fn name_component(name: &str) -> ComponentInstance {
        ComponentInstance {
            type_id: "editor.Name".to_string(),
            values: json!({ "name": name }),
        }
    }

    fn transform_component(translation: (f32, f32), rotation: f32, scale: (f32, f32)) -> ComponentInstance {
        ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: json!({
                "translation": { "x": translation.0, "y": translation.1 },
                "rotation": rotation,
                "scale": { "x": scale.0, "y": scale.1 },
            }),
        }
    }

    fn sprite_component(asset: &str, color: (f32, f32, f32, f32), anchor: &str) -> ComponentInstance {
        ComponentInstance {
            type_id: "editor.Sprite2D".to_string(),
            values: json!({
                "asset": asset,
                "color": { "r": color.0, "g": color.1, "b": color.2, "a": color.3 },
                "anchor": anchor,
            }),
        }
    }

    fn entity(id: &str, name: &str, components: Vec<ComponentInstance>) -> Entity {
        Entity {
            id: StableId::new(id),
            name: name.to_string(),
            parent: None,
            components,
        }
    }

    fn child(id: &str, name: &str, parent_id: &str, components: Vec<ComponentInstance>) -> Entity {
        Entity {
            id: StableId::new(id),
            name: name.to_string(),
            parent: Some(StableId::new(parent_id)),
            components,
        }
    }

    // Scenario 1: Empty document.
    #[test]
    fn test_export_empty_document() {
        let doc = make_doc(vec![]);
        let export = export_dynamic_scene(&doc).unwrap();

        assert_eq!(export.version, "0.1.0");
        assert_eq!(export.source_scene_id, "scene_001");
        assert_eq!(export.entities.len(), 0);
        assert_eq!(export.warnings.len(), 0);

        let json = serde_json::to_string(&export).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["version"], "0.1.0");
        assert_eq!(parsed["source_scene_id"], "scene_001");
        assert!(parsed["entities"].as_array().unwrap().is_empty());
        assert!(parsed["warnings"].as_array().unwrap().is_empty());
    }

    // Scenario 2: Name component.
    #[test]
    fn test_export_name_component() {
        let doc = make_doc(vec![entity("ent_01", "Player", vec![name_component("Player")])]);
        let export = export_dynamic_scene(&doc).unwrap();

        assert_eq!(export.entities.len(), 1);
        assert_eq!(export.warnings.len(), 0);
        assert_eq!(export.entities[0].stable_id, "ent_01");
        assert_eq!(export.entities[0].name, "Player");
        assert_eq!(export.entities[0].parent_stable_id, None);
        assert_eq!(export.entities[0].components["bevy.Name"]["name"], "Player");
    }

    // Scenario 3: Translation z=0.
    #[test]
    fn test_export_transform_translation_z_zero() {
        let doc = make_doc(vec![entity(
            "e1",
            "T",
            vec![transform_component((100.0, 200.0), 0.0, (1.0, 1.0))],
        )]);
        let export = export_dynamic_scene(&doc).unwrap();
        let t = &export.entities[0].components["bevy.Transform"];
        assert_eq!(t["translation"], json!([100.0, 200.0, 0.0]));
        assert_eq!(t["scale"], json!([1.0, 1.0, 1.0]));
    }

    // Scenario 4: Rotation → quaternion.
    #[test]
    fn test_export_transform_rotation_quaternion() {
        let half_pi = std::f32::consts::FRAC_PI_2;
        let doc = make_doc(vec![entity(
            "e1",
            "T",
            vec![transform_component((0.0, 0.0), half_pi, (1.0, 1.0))],
        )]);
        let export = export_dynamic_scene(&doc).unwrap();
        let t = &export.entities[0].components["bevy.Transform"];
        let rot = t["rotation"].as_array().unwrap();
        // Bevy quaternion [x, y, z, w] for z-axis rotation of π/2:
        // half = π/4, so [0, 0, sin(π/4), cos(π/4)]
        assert_eq!(rot.len(), 4);
        assert!((rot[0].as_f64().unwrap() - 0.0).abs() < 1e-6);
        assert!((rot[1].as_f64().unwrap() - 0.0).abs() < 1e-6);
        let s = rot[2].as_f64().unwrap();
        let c = rot[3].as_f64().unwrap();
        let expected_s = (half_pi * 0.5_f32).sin() as f64;
        let expected_c = (half_pi * 0.5_f32).cos() as f64;
        assert!((s - expected_s).abs() < 1e-5);
        assert!((c - expected_c).abs() < 1e-5);
    }

    // Scenario 5: Scale z=1.
    #[test]
    fn test_export_transform_scale_z_one() {
        let doc = make_doc(vec![entity(
            "e1",
            "T",
            vec![transform_component((0.0, 0.0), 0.0, (2.0, 3.0))],
        )]);
        let export = export_dynamic_scene(&doc).unwrap();
        let t = &export.entities[0].components["bevy.Transform"];
        assert_eq!(t["scale"], json!([2.0, 3.0, 1.0]));
    }

    // Scenario 6: Sprite with all fields.
    #[test]
    fn test_export_sprite_all_fields() {
        let doc = make_doc(vec![entity(
            "e1",
            "Sprite",
            vec![sprite_component("assets/player.png", (1.0, 0.0, 0.0, 1.0), "Center")],
        )]);
        let export = export_dynamic_scene(&doc).unwrap();
        let s = &export.entities[0].components["bevy.Sprite"];
        assert_eq!(s["asset"], "assets/player.png");
        assert_eq!(s["color"], json!([1.0, 0.0, 0.0, 1.0]));
        assert_eq!(s["anchor"], "Center");
        assert_eq!(export.warnings.len(), 0);
    }

    // Scenario 7: All 9 anchors.
    #[test]
    fn test_export_sprite_all_9_anchors() {
        let anchors = [
            "Center",
            "TopLeft",
            "TopRight",
            "BottomLeft",
            "BottomRight",
            "TopCenter",
            "BottomCenter",
            "CenterLeft",
            "CenterRight",
        ];
        for anchor in anchors {
            let doc = make_doc(vec![entity(
                "e1",
                "Sprite",
                vec![sprite_component("a.png", (1.0, 1.0, 1.0, 1.0), anchor)],
            )]);
            let export = export_dynamic_scene(&doc).unwrap();
            assert_eq!(export.warnings.len(), 0, "anchor {} should not warn", anchor);
            assert_eq!(
                export.entities[0].components["bevy.Sprite"]["anchor"],
                anchor,
                "anchor {} should round-trip",
                anchor
            );
        }
    }

    // Scenario 8: Empty asset → warning + omit.
    #[test]
    fn test_export_sprite_empty_asset_warning() {
        let doc = make_doc(vec![entity(
            "e1",
            "Sprite",
            vec![sprite_component("", (1.0, 1.0, 1.0, 1.0), "Center")],
        )]);
        let export = export_dynamic_scene(&doc).unwrap();
        assert!(!export.entities[0].components.contains_key("bevy.Sprite"));
        assert_eq!(export.warnings.len(), 1);
        assert_eq!(export.warnings[0].entity_stable_id, Some("e1".to_string()));
        assert_eq!(export.warnings[0].component_type_id, Some("editor.Sprite2D".to_string()));
        assert!(export.warnings[0].message.contains("empty asset"));
    }

    // Scenario 9: Missing color → white + warning.
    #[test]
    fn test_export_sprite_missing_color_default() {
        let sprite = ComponentInstance {
            type_id: "editor.Sprite2D".to_string(),
            values: json!({ "asset": "a.png", "anchor": "Center" }),
        };
        let doc = make_doc(vec![entity("e1", "Sprite", vec![sprite])]);
        let export = export_dynamic_scene(&doc).unwrap();
        assert_eq!(export.entities[0].components["bevy.Sprite"]["color"], json!([1.0, 1.0, 1.0, 1.0]));
        assert_eq!(export.warnings.len(), 1);
        assert!(export.warnings[0].message.contains("color"));
    }

    // Scenario 10: Invalid anchor → Center + warning.
    #[test]
    fn test_export_sprite_invalid_anchor_default() {
        let doc = make_doc(vec![entity(
            "e1",
            "Sprite",
            vec![sprite_component("a.png", (1.0, 1.0, 1.0, 1.0), "NotAValidAnchor")],
        )]);
        let export = export_dynamic_scene(&doc).unwrap();
        assert_eq!(export.entities[0].components["bevy.Sprite"]["anchor"], "Center");
        assert_eq!(export.warnings.len(), 1);
        assert!(export.warnings[0].message.contains("anchor"));
    }

    // Scenario 11: Editorial components silent.
    #[test]
    fn test_export_editorial_components_silent() {
        let doc = make_doc(vec![entity(
            "e1",
            "E",
            vec![
                name_component("X"),
                ComponentInstance {
                    type_id: "editor.Visible".to_string(),
                    values: json!({ "visible": true }),
                },
                ComponentInstance {
                    type_id: "editor.Locked".to_string(),
                    values: json!({ "locked": false }),
                },
            ],
        )]);
        let export = export_dynamic_scene(&doc).unwrap();
        assert_eq!(export.entities[0].components.len(), 1);
        assert!(export.entities[0].components.contains_key("bevy.Name"));
        assert!(!export.entities[0].components.contains_key("bevy.Visible"));
        assert!(!export.entities[0].components.contains_key("bevy.Locked"));
        assert_eq!(export.warnings.len(), 0);
    }

    // Scenario 12: Unknown component → skip + warning.
    #[test]
    fn test_export_unknown_component_warning() {
        let doc = make_doc(vec![entity(
            "e1",
            "E",
            vec![
                name_component("X"),
                ComponentInstance {
                    type_id: "game.PlayerHealth".to_string(),
                    values: json!({ "hp": 100 }),
                },
            ],
        )]);
        let export = export_dynamic_scene(&doc).unwrap();
        assert_eq!(export.entities[0].components.len(), 1);
        assert_eq!(export.warnings.len(), 1);
        assert_eq!(
            export.warnings[0].component_type_id,
            Some("game.PlayerHealth".to_string())
        );
        assert!(export.warnings[0].message.contains("unknown") || export.warnings[0].message.contains("Skipping"));
    }

    // Scenario 13: Parent-child hierarchy.
    #[test]
    fn test_export_parent_child_hierarchy() {
        let doc = make_doc(vec![
            entity("a", "Parent", vec![name_component("Parent")]),
            child("b", "Child", "a", vec![name_component("Child")]),
        ]);
        let export = export_dynamic_scene(&doc).unwrap();
        assert_eq!(export.entities.len(), 2);
        assert_eq!(export.entities[0].parent_stable_id, None);
        assert_eq!(export.entities[1].parent_stable_id, Some("a".to_string()));
    }

    // Scenario 14: Orphan → root + warning.
    #[test]
    fn test_export_orphan_promoted_to_root() {
        let doc = make_doc(vec![
            entity("a", "Root", vec![name_component("Root")]),
            child("b", "Orphan", "nonexistent", vec![name_component("Orphan")]),
        ]);
        let export = export_dynamic_scene(&doc).unwrap();
        assert_eq!(export.entities[1].parent_stable_id, None);
        assert_eq!(export.warnings.len(), 1);
        assert_eq!(export.warnings[0].entity_stable_id, Some("b".to_string()));
        assert!(export.warnings[0].message.contains("parent"));
    }

    // Scenario 15: Invalid Vec2 → default + warning.
    #[test]
    fn test_export_invalid_vec2_default() {
        let bad_transform = ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: json!({ "translation": { "x": "not_a_number" } }),
        };
        let doc = make_doc(vec![entity("e1", "T", vec![bad_transform])]);
        let export = export_dynamic_scene(&doc).unwrap();
        assert_eq!(
            export.entities[0].components["bevy.Transform"]["translation"],
            json!([0.0, 0.0, 0.0])
        );
        assert!(export.warnings[0].message.contains("translation"));
    }

    // Scenario 16: Default Transform2D when values empty.
    #[test]
    fn test_export_default_transform() {
        let empty_transform = ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: json!({}),
        };
        let doc = make_doc(vec![entity("e1", "T", vec![empty_transform])]);
        let export = export_dynamic_scene(&doc).unwrap();
        let t = &export.entities[0].components["bevy.Transform"];
        assert_eq!(t["translation"], json!([0.0, 0.0, 0.0]));
        assert_eq!(t["rotation"], json!([0.0, 0.0, 0.0, 1.0]));
        assert_eq!(t["scale"], json!([1.0, 1.0, 1.0]));
        assert_eq!(export.warnings.len(), 0);
    }

    // Scenario 17: Determinism.
    #[test]
    fn test_export_deterministic() {
        let mut entities = vec![];
        for i in 0..10 {
            entities.push(entity(
                &format!("e{}", i),
                &format!("E{}", i),
                vec![
                    name_component(&format!("E{}", i)),
                    transform_component((i as f32, i as f32 * 2.0), 0.1, (1.0, 1.0)),
                    sprite_component("a.png", (1.0, 0.5, 0.25, 1.0), "Center"),
                ],
            ));
        }
        let doc = make_doc(entities);

        let export1 = export_dynamic_scene(&doc).unwrap();
        let export2 = export_dynamic_scene(&doc).unwrap();
        let json1 = serde_json::to_string(&export1).unwrap();
        let json2 = serde_json::to_string(&export2).unwrap();
        assert_eq!(json1, json2);
    }

    // Scenario 20: 50 entities.
    #[test]
    fn test_export_50_entities() {
        let entities: Vec<Entity> = (0..50)
            .map(|i| {
                entity(
                    &format!("e{:03}", i),
                    &format!("E{}", i),
                    vec![name_component(&format!("E{}", i)), transform_component((0.0, 0.0), 0.0, (1.0, 1.0))],
                )
            })
            .collect();
        let doc = make_doc(entities);
        let export = export_dynamic_scene(&doc).unwrap();
        assert_eq!(export.entities.len(), 50);
        assert_eq!(export.warnings.len(), 0);
    }

    // Scenario 21: Component order independent of input order.
    #[test]
    fn test_export_component_order_independent() {
        let doc1 = make_doc(vec![entity(
            "e1",
            "E",
            vec![name_component("X"), transform_component((0.0, 0.0), 0.0, (1.0, 1.0))],
        )]);
        let doc2 = make_doc(vec![entity(
            "e1",
            "E",
            vec![transform_component((0.0, 0.0), 0.0, (1.0, 1.0)), name_component("X")],
        )]);
        let export1 = export_dynamic_scene(&doc1).unwrap();
        let export2 = export_dynamic_scene(&doc2).unwrap();
        let json1 = serde_json::to_string(&export1).unwrap();
        let json2 = serde_json::to_string(&export2).unwrap();
        assert_eq!(json1, json2);
    }

    // Scenario: missing name in editor.Name → warning.
    #[test]
    fn test_export_missing_name_warning() {
        let bad_name = ComponentInstance {
            type_id: "editor.Name".to_string(),
            values: json!({}),
        };
        let doc = make_doc(vec![entity("e1", "E", vec![bad_name])]);
        let export = export_dynamic_scene(&doc).unwrap();
        assert_eq!(export.entities[0].components["bevy.Name"]["name"], "");
        assert!(export.warnings[0].message.contains("Name"));
    }

    // ===== anchor_str_to_bevy_anchor helper (verified via normalized offsets) =====
    //
    // The Bevy-dependent function `anchor_str_to_bevy_anchor(s)` returns a
    // `bevy::sprite::Anchor` whose internal `Vec2` matches the normalized offset
    // returned by `anchor_str_to_normalized_offset(s)`. We test the normalized
    // offset function (which is bevy-free) and rely on `cargo check` + Playwright
    // E2E tests to verify the Bevy dependency. This avoids the libudev-sys native
    // test issue on Fedora.

    #[test]
    fn test_anchor_str_to_bevy_anchor_center() {
        assert_eq!(anchor_str_to_normalized_offset("Center"), (0.0, 0.0));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_top_left() {
        assert_eq!(anchor_str_to_normalized_offset("TopLeft"), (-0.5, 0.5));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_top_center() {
        assert_eq!(anchor_str_to_normalized_offset("TopCenter"), (0.0, 0.5));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_top_right() {
        assert_eq!(anchor_str_to_normalized_offset("TopRight"), (0.5, 0.5));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_center_left() {
        assert_eq!(anchor_str_to_normalized_offset("CenterLeft"), (-0.5, 0.0));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_center_right() {
        assert_eq!(anchor_str_to_normalized_offset("CenterRight"), (0.5, 0.0));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_bottom_left() {
        assert_eq!(anchor_str_to_normalized_offset("BottomLeft"), (-0.5, -0.5));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_bottom_center() {
        assert_eq!(anchor_str_to_normalized_offset("BottomCenter"), (0.0, -0.5));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_bottom_right() {
        assert_eq!(anchor_str_to_normalized_offset("BottomRight"), (0.5, -0.5));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_invalid_defaults_to_center() {
        assert_eq!(anchor_str_to_normalized_offset("NotAnAnchor"), (0.0, 0.0));
    }

    #[test]
    fn test_anchor_str_to_bevy_anchor_empty_string_defaults_to_center() {
        assert_eq!(anchor_str_to_normalized_offset(""), (0.0, 0.0));
    }

    #[test]
    fn test_is_known_anchor_str() {
        // Known anchors
        assert!(is_known_anchor_str("Center"));
        assert!(is_known_anchor_str("TopLeft"));
        assert!(is_known_anchor_str("TopCenter"));
        assert!(is_known_anchor_str("TopRight"));
        assert!(is_known_anchor_str("CenterLeft"));
        assert!(is_known_anchor_str("CenterRight"));
        assert!(is_known_anchor_str("BottomLeft"));
        assert!(is_known_anchor_str("BottomCenter"));
        assert!(is_known_anchor_str("BottomRight"));
        // Unknown / empty / casing
        assert!(!is_known_anchor_str("center")); // lowercase
        assert!(!is_known_anchor_str(""));
        assert!(!is_known_anchor_str("NotAnAnchor"));
    }
}
