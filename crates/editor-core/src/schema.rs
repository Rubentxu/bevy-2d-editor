//! Component Schema Registry for the Bevy 2D Editor.
//!
//! Provides a global registry of component schemas that define the structure
//! of component instances used in scene documents.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Field type enumeration for schema field definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    String,
    F32,
    Bool,
    Vec2,
    Color,
    Anchor,
    AssetReference,
}

/// Constraint on a field value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    Min(f32),
    Max(f32),
    NonEmpty,
}

/// A single field definition within a component schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    #[serde(default = "default_json_value")]
    pub default: serde_json::Value,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
}

fn default_json_value() -> serde_json::Value {
    serde_json::Value::Null
}

/// A component schema defining the structure of a component type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentSchema {
    pub type_id: String,
    pub display_name: String,
    pub fields: Vec<FieldDef>,
    /// Whether this component exports to Bevy runtime.
    /// Editorial-only components (Visible, Locked) set this to false.
    pub exports_to_bevy: bool,
}

/// Global registry of all component schemas.
#[derive(Debug, Clone)]
pub struct ComponentSchemaRegistry {
    schemas: HashMap<String, ComponentSchema>,
}

impl ComponentSchemaRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Get a schema by its type_id.
    pub fn get(&self, type_id: &str) -> Option<&ComponentSchema> {
        self.schemas.get(type_id)
    }

    /// Insert a schema into the registry.
    pub fn insert(&mut self, schema: ComponentSchema) {
        self.schemas.insert(schema.type_id.clone(), schema);
    }

    /// Iterate over all schemas.
    pub fn iter(&self) -> impl Iterator<Item = &ComponentSchema> {
        self.schemas.values()
    }

    /// Create a registry seeded with the 5 built-in editor schemas.
    pub fn with_builtin_seeds() -> Self {
        let mut registry = Self::new();

        // editor.Name
        registry.insert(ComponentSchema {
            type_id: "editor.Name".to_string(),
            display_name: "Name".to_string(),
            fields: vec![FieldDef {
                name: "name".to_string(),
                field_type: FieldType::String,
                default: serde_json::json!(""),
                constraints: vec![],
            }],
            exports_to_bevy: true,
        });

        // editor.Transform2D
        registry.insert(ComponentSchema {
            type_id: "editor.Transform2D".to_string(),
            display_name: "Transform 2D".to_string(),
            fields: vec![
                FieldDef {
                    name: "translation".to_string(),
                    field_type: FieldType::Vec2,
                    default: serde_json::json!({"x": 0.0, "y": 0.0}),
                    constraints: vec![],
                },
                FieldDef {
                    name: "rotation".to_string(),
                    field_type: FieldType::F32,
                    default: serde_json::json!(0.0),
                    constraints: vec![],
                },
                FieldDef {
                    name: "scale".to_string(),
                    field_type: FieldType::Vec2,
                    default: serde_json::json!({"x": 1.0, "y": 1.0}),
                    constraints: vec![],
                },
            ],
            exports_to_bevy: true,
        });

        // editor.Sprite2D
        registry.insert(ComponentSchema {
            type_id: "editor.Sprite2D".to_string(),
            display_name: "Sprite 2D".to_string(),
            fields: vec![
                FieldDef {
                    name: "asset".to_string(),
                    field_type: FieldType::AssetReference,
                    default: serde_json::json!(""),
                    constraints: vec![],
                },
                FieldDef {
                    name: "color".to_string(),
                    field_type: FieldType::Color,
                    default: serde_json::json!({"r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0}),
                    constraints: vec![],
                },
                FieldDef {
                    name: "anchor".to_string(),
                    field_type: FieldType::Anchor,
                    default: serde_json::json!("Center"),
                    constraints: vec![],
                },
            ],
            exports_to_bevy: true,
        });

        // editor.Visible
        registry.insert(ComponentSchema {
            type_id: "editor.Visible".to_string(),
            display_name: "Visible".to_string(),
            fields: vec![FieldDef {
                name: "visible".to_string(),
                field_type: FieldType::Bool,
                default: serde_json::json!(true),
                constraints: vec![],
            }],
            exports_to_bevy: false,
        });

        // editor.Locked
        registry.insert(ComponentSchema {
            type_id: "editor.Locked".to_string(),
            display_name: "Locked".to_string(),
            fields: vec![FieldDef {
                name: "locked".to_string(),
                field_type: FieldType::Bool,
                default: serde_json::json!(false),
                constraints: vec![],
            }],
            exports_to_bevy: false,
        });

        registry
    }
}

impl Default for ComponentSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global singleton registry instance.
static REGISTRY: OnceLock<ComponentSchemaRegistry> = OnceLock::new();

/// Get the global component schema registry.
/// Initializes with built-in seeds on first call.
pub fn global_registry() -> &'static ComponentSchemaRegistry {
    REGISTRY.get_or_init(|| ComponentSchemaRegistry::with_builtin_seeds())
}

#[cfg(test)]
mod tests {
    use super::*;

    // §3.1: Built-in schemas are present
    #[test]
    fn test_registry_has_5_builtin_schemas() {
        let registry = ComponentSchemaRegistry::with_builtin_seeds();
        let count = registry.iter().count();
        assert_eq!(count, 5);
    }

    // §3.2: Known type_id returns its schema
    #[test]
    fn test_get_schema_known_type_id() {
        let registry = ComponentSchemaRegistry::with_builtin_seeds();
        let schema = registry.get("editor.Transform2D");
        assert!(schema.is_some());
        assert_eq!(schema.unwrap().type_id, "editor.Transform2D");
    }

    // §3.3: Unknown type_id returns None (no panic)
    #[test]
    fn test_get_schema_unknown_returns_none() {
        let registry = ComponentSchemaRegistry::with_builtin_seeds();
        let schema = registry.get("editor.NonExistent");
        assert!(schema.is_none());
    }

    // §3.4: Transform2D fields are defined
    #[test]
    fn test_transform2d_fields_defined() {
        let registry = ComponentSchemaRegistry::with_builtin_seeds();
        let schema = registry.get("editor.Transform2D").unwrap();

        let field_names: Vec<&str> = schema.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"translation"));
        assert!(field_names.contains(&"rotation"));
        assert!(field_names.contains(&"scale"));

        // Check field types
        let translation_type = schema
            .fields
            .iter()
            .find(|f| f.name == "translation")
            .map(|f| &f.field_type);
        assert_eq!(translation_type, Some(&FieldType::Vec2));
    }

    // §3.5: Name schema defaults
    #[test]
    fn test_name_schema_default() {
        let registry = ComponentSchemaRegistry::with_builtin_seeds();
        let schema = registry.get("editor.Name").unwrap();

        assert_eq!(schema.fields.len(), 1);
        let name_field = &schema.fields[0];
        assert_eq!(name_field.name, "name");
        assert_eq!(name_field.field_type, FieldType::String);
        assert_eq!(name_field.default, serde_json::json!(""));
    }

    // §3.6: Sprite2D asset is logical path
    #[test]
    fn test_sprite2d_asset_is_logical_path() {
        let registry = ComponentSchemaRegistry::with_builtin_seeds();
        let schema = registry.get("editor.Sprite2D").unwrap();

        let asset_field = schema.fields.iter().find(|f| f.name == "asset").unwrap();
        assert_eq!(asset_field.field_type, FieldType::AssetReference);
        // Asset reference is a logical path string
        assert_eq!(asset_field.default, serde_json::json!(""));
    }

    // §3.7: Visible and Locked editorial-only
    #[test]
    fn test_visible_locked_editorial_only() {
        let registry = ComponentSchemaRegistry::with_builtin_seeds();

        let visible = registry.get("editor.Visible").unwrap();
        assert!(!visible.exports_to_bevy);
        assert_eq!(visible.fields.len(), 1);
        assert_eq!(visible.fields[0].name, "visible");
        assert_eq!(visible.fields[0].field_type, FieldType::Bool);

        let locked = registry.get("editor.Locked").unwrap();
        assert!(!locked.exports_to_bevy);
        assert_eq!(locked.fields.len(), 1);
        assert_eq!(locked.fields[0].name, "locked");
        assert_eq!(locked.fields[0].field_type, FieldType::Bool);
    }

    // §3.8: Global singleton
    #[test]
    fn test_global_registry_singleton() {
        let reg1 = global_registry();
        let reg2 = global_registry();
        assert_eq!(reg1 as *const _, reg2 as *const _);
        assert_eq!(reg1.iter().count(), 5);
    }
}
