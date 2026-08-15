//! Component Schema Registry — value types only.
//!
//! The registry itself (ComponentSchemaRegistry, add_schema, get_schema) stays in
//! editor-core. Only the value types that carry no Bevy dependencies live here.

use serde::{Deserialize, Serialize};

/// Opaque identity for a component type (e.g. "editor.Transform2D").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentTypeId(pub String);

impl ComponentTypeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Source location for a schema field (file:line:col).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SourceLocation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// Kind of a schema field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldType {
    String,
    I32,
    F32,
    Bool,
    Vec2,
    Vec3,
    Color,
    Enum { variants: Vec<String> },
    AssetRef,
    SceneRef,
    Custom { type_name: String },
}

/// Constraint on a schema field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Constraint {
    Range { min: f32, max: f32 },
    Min { value: f32 },
    Max { value: f32 },
    Step { value: f32 },
    Pattern { regex: String },
    Required,
}

/// One named field of a component schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,
    #[serde(default)]
    pub location: SourceLocation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Schema purpose discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaKind {
    #[default]
    Simple,
    /// Bound to a SceneAssetDocument (Bevy 0.19 #[derive(SceneComponent)] semantics).
    SceneComponent,
}

/// A registered component type and its field definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentSchema {
    pub type_id: ComponentTypeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound_scene_asset_ref: Option<String>,
    #[serde(default = "default_auto_spawn")]
    pub auto_spawn: bool,
    pub fields: Vec<FieldDef>,
    pub kind: SchemaKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub location: SourceLocation,
}

fn default_auto_spawn() -> bool {
    true
}

impl ComponentSchema {
    /// Return all field names.
    pub fn field_names(&self) -> Vec<&str> {
        self.fields.iter().map(|f| f.name.as_str()).collect()
    }

    /// Return the SchemaKind label.
    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            SchemaKind::Simple => "simple",
            SchemaKind::SceneComponent => "scene_component",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_schema_round_trips() {
        let schema = ComponentSchema {
            type_id: ComponentTypeId::new("editor.Transform2D"),
            bound_scene_asset_ref: None,
            auto_spawn: true,
            fields: vec![
                FieldDef {
                    name: "position".to_string(),
                    field_type: FieldType::Vec2,
                    default_value: Some(serde_json::json!({"x": 0.0, "y": 0.0})),
                    constraints: vec![],
                    location: SourceLocation::default(),
                    description: None,
                },
                FieldDef {
                    name: "rotation".to_string(),
                    field_type: FieldType::F32,
                    default_value: Some(serde_json::json!(0.0)),
                    constraints: vec![Constraint::Range { min: 0.0, max: 360.0 }],
                    location: SourceLocation::default(),
                    description: Some("Rotation in degrees".to_string()),
                },
            ],
            kind: SchemaKind::Simple,
            description: Some("2D transform component".to_string()),
            location: SourceLocation::default(),
        };

        let json = serde_json::to_string(&schema).unwrap();
        let parsed: ComponentSchema = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.type_id.as_str(), "editor.Transform2D");
        assert_eq!(parsed.fields.len(), 2);
        assert_eq!(parsed.kind, SchemaKind::Simple);
        assert!(parsed.auto_spawn);
    }

    #[test]
    fn scene_component_schema_round_trips() {
        let schema = ComponentSchema {
            type_id: ComponentTypeId::new("editor.SceneSpawner"),
            bound_scene_asset_ref: Some("assets/characters/player".to_string()),
            auto_spawn: true,
            fields: vec![],
            kind: SchemaKind::SceneComponent,
            description: None,
            location: SourceLocation::default(),
        };

        let json = serde_json::to_string(&schema).unwrap();
        let parsed: ComponentSchema = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.kind, SchemaKind::SceneComponent);
        assert_eq!(
            parsed.bound_scene_asset_ref.as_deref(),
            Some("assets/characters/player")
        );
    }

    #[test]
    fn field_names_helper() {
        let schema = ComponentSchema {
            type_id: ComponentTypeId::new("test"),
            bound_scene_asset_ref: None,
            auto_spawn: true,
            fields: vec![
                FieldDef {
                    name: "foo".to_string(),
                    field_type: FieldType::String,
                    default_value: None,
                    constraints: vec![],
                    location: SourceLocation::default(),
                    description: None,
                },
                FieldDef {
                    name: "bar".to_string(),
                    field_type: FieldType::Bool,
                    default_value: None,
                    constraints: vec![],
                    location: SourceLocation::default(),
                    description: None,
                },
            ],
            kind: SchemaKind::Simple,
            description: None,
            location: SourceLocation::default(),
        };

        assert_eq!(schema.field_names(), vec!["foo", "bar"]);
    }

    #[test]
    fn kind_label_helper() {
        let simple = ComponentSchema {
            type_id: ComponentTypeId::new("simple"),
            bound_scene_asset_ref: None,
            auto_spawn: true,
            fields: vec![],
            kind: SchemaKind::Simple,
            description: None,
            location: SourceLocation::default(),
        };
        let scene = ComponentSchema {
            type_id: ComponentTypeId::new("scene"),
            bound_scene_asset_ref: Some("path".to_string()),
            auto_spawn: false,
            fields: vec![],
            kind: SchemaKind::SceneComponent,
            description: None,
            location: SourceLocation::default(),
        };
        assert_eq!(simple.kind_label(), "simple");
        assert_eq!(scene.kind_label(), "scene_component");
    }
}
