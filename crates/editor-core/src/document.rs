//! SceneDocument data model for the Bevy 2D Editor.
//!
//! This module contains the core types for representing editor scenes
//! as structured JSON documents.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use crate::scene_instance::SceneInstance;

/// Opaque stable identifier for entities.
/// Uses #[serde(transparent)] so it serializes as a plain string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StableId(String);

impl StableId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StableId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Local identifier for an entity within a scene.
/// Uses #[serde(transparent)] so it serializes as a plain string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
#[derive(Default)]
pub struct LocalId(String);

impl LocalId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for LocalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 2D vector with x and y components.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }
}

/// RGBA color with floating-point components in [0, 1] range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn srgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

/// Anchor point for sprites and other positioned elements.
/// Serializes as PascalCase strings (e.g., "Center", "TopLeft").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Anchor {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
    CenterLeft,
    CenterRight,
}

/// The root document type representing a complete scene.
/// This is the source-of-truth structure stored as JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneDocument {
    pub version: String,
    pub scene_id: String,
    pub name: String,
    pub entities: Vec<Entity>,
    /// Placed Scene Instances indexed by StableId.
    /// Serialized as a BTreeMap for deterministic key ordering.
    /// Defaults to empty BTreeMap when absent from older documents (S7).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub instances: BTreeMap<StableId, SceneInstance>,
}

impl Default for SceneDocument {
    fn default() -> Self {
        Self {
            version: "0.1".to_string(),
            scene_id: String::new(),
            name: String::new(),
            entities: Vec::new(),
            instances: BTreeMap::new(),
        }
    }
}

/// A single entity within a scene with its associated components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: StableId,
    /// Local identifier within the scene. Falls back to id if not set.
    #[serde(default, skip_serializing_if = "LocalId::is_empty")]
    pub local_id: LocalId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<StableId>,
    pub components: Vec<ComponentInstance>,
}

/// A component instance attaching typed values to an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentInstance {
    pub type_id: String,
    #[serde(default)]
    pub values: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    // §2.1: Serialize a populated SceneDocument
    #[test]
    fn test_serialize_populated_scene() {
        let doc = SceneDocument {
            version: "0.1".to_string(),
            scene_id: "scene_001".to_string(),
            name: "Test Scene".to_string(),
            entities: vec![Entity {
                id: StableId::new("ent_01"),
                local_id: LocalId::new("ent_01"),
                name: "Player".to_string(),
                parent: None,
                components: vec![],
            }],
            instances: BTreeMap::new(),
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("\"version\":\"0.1\""));
        assert!(json.contains("\"scene_id\":\"scene_001\""));
        assert!(json.contains("\"name\":\"Test Scene\""));
        assert!(json.contains("\"entities\""));
    }

    // §2.1: Serialize an empty scene
    #[test]
    fn test_serialize_empty_scene() {
        let doc = SceneDocument {
            version: "0.1".to_string(),
            scene_id: "empty".to_string(),
            name: "Empty Scene".to_string(),
            entities: vec![],
            instances: BTreeMap::new(),
        };

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: SceneDocument = serde_json::from_str(&json).unwrap();
        assert!(deserialized.entities.is_empty());
    }

    // §2.2: Deserialize a well-formed scene
    #[test]
    fn test_deserialize_well_formed_scene() {
        let json = r#"{
            "version": "0.1",
            "scene_id": "scene_001",
            "name": "Test Scene",
            "entities": [
                {
                    "id": "ent_01",
                    "name": "Player",
                    "components": [
                        {
                            "type_id": "editor.Transform2D",
                            "values": {"translation": {"x": 100.0, "y": 200.0}}
                        }
                    ]
                }
            ]
        }"#;

        let doc: SceneDocument = serde_json::from_str(json).unwrap();
        assert_eq!(doc.version, "0.1");
        assert_eq!(doc.scene_id, "scene_001");
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(doc.entities[0].name, "Player");
        assert_eq!(doc.entities[0].components[0].type_id, "editor.Transform2D");
    }

    // §2.3: Roundtrip preserves hierarchy
    #[test]
    fn test_roundtrip_preserves_hierarchy() {
        let doc = SceneDocument {
            version: "0.1".to_string(),
            scene_id: "scene_001".to_string(),
            name: "Hierarchy Test".to_string(),
            entities: vec![
                Entity {
                    id: StableId::new("parent_01"),
                    local_id: LocalId::new("parent_01"),
                    name: "Parent".to_string(),
                    parent: None,
                    components: vec![],
                },
                Entity {
                    id: StableId::new("child_01"),
                    local_id: LocalId::new("child_01"),
                    name: "Child".to_string(),
                    parent: Some(StableId::new("parent_01")),
                    components: vec![],
                },
            ],
            instances: BTreeMap::new(),
        };

        let json = serde_json::to_string(&doc).unwrap();
        let roundtripped: SceneDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(doc.version, roundtripped.version);
        assert_eq!(doc.entities.len(), roundtripped.entities.len());
        assert_eq!(doc.entities[1].parent, roundtripped.entities[1].parent);
    }

    // §2.4: Rename preserves ID
    #[test]
    fn test_rename_preserves_id() {
        let mut entity = Entity {
            id: StableId::new("ent_01J..."),
            local_id: LocalId::new("ent_01J..."),
            name: "Player".to_string(),
            parent: None,
            components: vec![],
        };

        entity.name = "PlayerSpawn".to_string();
        assert_eq!(entity.id.as_str(), "ent_01J...");
    }

    // §2.5: IDs are opaque and value-comparable
    #[test]
    fn test_ids_are_opaque() {
        let id1 = StableId::new("ent_01");
        let id2 = StableId::new("ent_02");
        let id1_clone = StableId::new("ent_01");

        assert_eq!(id1, id1_clone);
        assert_ne!(id1, id2);
        // IDs are opaque strings, not indices
        assert_eq!(id1.as_str(), "ent_01");
    }

    // §2.6: Vec2, Color, Anchor JSON shapes
    #[test]
    fn test_vec2_color_anchor_json_shapes() {
        let vec2 = Vec2::new(10.0, 20.0);
        let vec2_json = serde_json::to_string(&vec2).unwrap();
        assert!(vec2_json.contains("\"x\":10"));
        assert!(vec2_json.contains("\"y\":20"));

        let color = Color::new(1.0, 0.5, 0.25, 1.0);
        let color_json = serde_json::to_string(&color).unwrap();
        assert!(color_json.contains("\"r\":1"));
        assert!(color_json.contains("\"g\":0.5"));
        assert!(color_json.contains("\"b\":0.25"));
        assert!(color_json.contains("\"a\":1"));

        let anchor = Anchor::Center;
        let anchor_json = serde_json::to_string(&anchor).unwrap();
        assert_eq!(anchor_json, "\"Center\"");
    }

    // §2.7: Unknown field preserved
    #[test]
    fn test_unknown_field_preserved() {
        let json = r#"{
            "version": "0.1",
            "scene_id": "scene_001",
            "name": "Test",
            "entities": [
                {
                    "id": "ent_01",
                    "name": "Test",
                    "components": [
                        {
                            "type_id": "editor.Custom",
                            "values": {"unknown_field": "preserved", "x": 1}
                        }
                    ]
                }
            ]
        }"#;

        let doc: SceneDocument = serde_json::from_str(json).unwrap();
        let values = &doc.entities[0].components[0].values;
        assert_eq!(values.get("unknown_field").unwrap(), "preserved");
        assert_eq!(values.get("x").unwrap(), 1);
    }

    // §2.8: Version field preserved
    #[test]
    fn test_version_field_preserved() {
        let doc = SceneDocument {
            version: "0.1".to_string(),
            scene_id: "scene_001".to_string(),
            name: "Test".to_string(),
            entities: vec![],
            instances: BTreeMap::new(),
        };

        let json = serde_json::to_string(&doc).unwrap();
        let roundtripped: SceneDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtripped.version, "0.1");
    }

    // §2.9: Instance has namespaced type_id
    #[test]
    fn test_instance_has_namespaced_type_id() {
        let instance = ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: serde_json::json!({}),
        };

        assert!(instance.type_id.contains("editor."));
        let json = serde_json::to_string(&instance).unwrap();
        assert!(json.contains("\"type_id\":\"editor.Transform2D\""));
    }

    // §2.10: Component instance structure
    #[test]
    fn test_component_instance_structure() {
        let instance = ComponentInstance {
            type_id: "editor.Sprite2D".to_string(),
            values: serde_json::json!({
                "color": {"r": 1.0, "g": 0.0, "b": 0.0, "a": 1.0},
                "anchor": "Center"
            }),
        };

        let json = serde_json::to_string(&instance).unwrap();
        let roundtripped: ComponentInstance = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtripped.type_id, "editor.Sprite2D");
        assert_eq!(roundtripped.values["color"]["r"], 1.0);
    }
}
