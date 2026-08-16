//! Component Schema Registry — value types only.
//!
//! The registry itself (ComponentSchemaRegistry, add_schema, get_schema) stays in
//! editor-core. Only the value types that carry no Bevy dependencies live here.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Apply-back policy types (ADR-0042, ADR-0050)
// ─────────────────────────────────────────────────────────────────────────────

/// Policy governing whether and how a field may have its runtime value
/// applied back to the authoring state.
///
/// Opaque identity for a component type (e.g. "editor.Transform2D").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentTypeId(pub String);

impl ComponentTypeId {
    /// Construct a new ComponentTypeId from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    /// Borrow the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Source location for a schema field (file:line:col).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Path to the source file that defined this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    /// 1-based line number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// 1-based column number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// Kind of a schema field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldType {
    /// UTF-8 string value.
    String,
    /// 32-bit signed integer.
    I32,
    /// 32-bit floating-point.
    F32,
    /// Boolean value.
    Bool,
    /// 2D vector of f32.
    Vec2,
    /// 3D vector of f32.
    Vec3,
    /// RGBA color.
    Color,
    /// Enumerated value from a fixed list.
    Enum {
        /// Allowed variant names.
        variants: Vec<String>,
    },
    /// Reference to another asset.
    AssetRef,
    /// Reference to a scene asset.
    SceneRef,
    /// Extension point for custom types.
    Custom {
        /// Name of the custom type.
        type_name: String,
    },
}

/// Constraint on a schema field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Constraint {
    /// Numeric range (inclusive bounds).
    Range {
        /// Minimum allowed value.
        min: f32,
        /// Maximum allowed value.
        max: f32,
    },
    /// Inclusive lower bound.
    Min {
        /// Minimum allowed value.
        value: f32,
    },
    /// Inclusive upper bound.
    Max {
        /// Maximum allowed value.
        value: f32,
    },
    /// Quantized step value.
    Step {
        /// Step size.
        value: f32,
    },
    /// Regular expression constraint on string values.
    Pattern {
        /// Regex pattern (UTF-8).
        regex: String,
    },
    /// Field must be present and non-null.
    Required,
}

/// One named field of a component schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    /// Name of the field.
    pub name: String,
    /// Type discriminator and optional metadata.
    pub field_type: FieldType,
    /// Default value when not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    /// Active constraints on this field's value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,
    /// Source location where this field was defined.
    #[serde(default)]
    pub location: SourceLocation,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Schema purpose discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaKind {
    /// Regular component with typed fields.
    #[default]
    Simple,
    /// Bound to a SceneAssetDocument (Bevy 0.19 #[derive(SceneComponent)] semantics).
    SceneComponent,
}

/// A registered component type and its field definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentSchema {
    /// Type identifier for this component.
    pub type_id: ComponentTypeId,
    /// For `SceneComponent` kind, the bound scene asset path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound_scene_asset_ref: Option<String>,
    /// Whether placing an instance auto-spawns the bound scene.
    #[serde(default = "default_auto_spawn")]
    pub auto_spawn: bool,
    /// Field definitions in declaration order.
    pub fields: Vec<FieldDef>,
    /// Kind discriminator.
    pub kind: SchemaKind,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Source location of the schema definition.
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
                    constraints: vec![Constraint::Range {
                        min: 0.0,
                        max: 360.0,
                    }],
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
