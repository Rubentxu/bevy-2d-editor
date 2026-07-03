//! Component Schema Registry for the Bevy 2D Editor.
//!
//! Provides a global registry of component schemas that define the structure
//! of component instances used in scene documents.

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;
use thiserror::Error;

/// Opaque component type identifier used by the Component Schema Registry.
/// Transparent so it serializes as a plain string, e.g. `editor.Transform2D`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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

/// Source location in a Rust source file.
/// Used for "jump to definition" navigation from component schema to source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file_id: String,
    pub line: u32,
    #[serde(default = "default_source_location_column")]
    pub column: u32,
}

fn default_source_location_column() -> u32 {
    1
}

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
    /// Optional source location for "jump to definition" navigation.
    /// Points to the Rust struct definition in the editor's source files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,
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

    /// Insert a schema into the registry. If a schema with the same type_id
    /// already exists, it is replaced.
    pub fn insert(&mut self, schema: ComponentSchema) {
        self.schemas.insert(schema.type_id.clone(), schema);
    }

    /// Remove a schema by type_id. Returns the removed schema if found.
    pub fn remove(&mut self, type_id: &str) -> Option<ComponentSchema> {
        self.schemas.remove(type_id)
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
            source_location: None,
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
            source_location: None,
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
            source_location: None,
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
            source_location: None,
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
            source_location: None,
        });

        // editor.LogicBinding — binds a Scene Instance to a LogicGraphAsset
        registry.insert(ComponentSchema {
            type_id: "editor.LogicBinding".to_string(),
            display_name: "Logic Binding".to_string(),
            fields: vec![
                FieldDef {
                    name: "asset_id".to_string(),
                    field_type: FieldType::AssetReference,
                    default: serde_json::json!(""),
                    constraints: vec![],
                },
                FieldDef {
                    name: "version".to_string(),
                    field_type: FieldType::F32,
                    default: serde_json::json!(1.0),
                    constraints: vec![],
                },
            ],
            exports_to_bevy: true,
            source_location: None,
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

/// Get the global component schema registry (built-ins only).
/// Initializes with built-in seeds on first call.
pub fn global_registry() -> &'static ComponentSchemaRegistry {
    REGISTRY.get_or_init(|| ComponentSchemaRegistry::with_builtin_seeds())
}

// ─────────────────────────────────────────────────────────────────────────────
// Mutable user schema registry — runtime additions/deletions
// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    /// Mutable user-defined schemas. Built-ins live in `REGISTRY` (OnceLock)
    /// and are immutable. User schemas can be added/removed at runtime via
    /// `register_schema` / `unregister_schema`.
    static USER_SCHEMAS: RefCell<ComponentSchemaRegistry> = RefCell::new(ComponentSchemaRegistry::new());
}

/// Returns true if the type_id is a built-in (starts with `editor.`).
/// Built-ins are immutable: cannot be registered, unregistered, or deleted.
pub fn is_builtin_type(type_id: &str) -> bool {
    type_id.starts_with("editor.")
}

/// Register a user-defined schema in memory (does NOT save to OPFS).
/// Built-in schemas (editor.*) are rejected.
pub fn register_schema(schema: ComponentSchema) -> Result<(), SchemaError> {
    if is_builtin_type(&schema.type_id) {
        return Err(SchemaError::CannotRegisterBuiltin(schema.type_id));
    }
    USER_SCHEMAS.with(|r| r.borrow_mut().insert(schema));
    Ok(())
}

/// Unregister a user-defined schema from memory (does NOT delete OPFS file).
/// Built-in schemas are rejected. Missing schemas are a no-op success.
pub fn unregister_schema(type_id: &str) -> Result<(), SchemaError> {
    if is_builtin_type(type_id) {
        return Err(SchemaError::CannotUnregisterBuiltin(type_id.to_string()));
    }
    USER_SCHEMAS.with(|r| {
        r.borrow_mut().remove(type_id); // ignore if not present
    });
    Ok(())
}

/// Returns a combined registry containing built-ins + user-defined schemas.
/// User schemas override built-ins if the same type_id is used.
pub fn combined_registry() -> ComponentSchemaRegistry {
    let mut combined = ComponentSchemaRegistry::new();
    for schema in global_registry().iter() {
        combined.insert(schema.clone());
    }
    USER_SCHEMAS.with(|r| {
        for schema in r.borrow().iter() {
            combined.insert(schema.clone());
        }
    });
    combined
}

/// Errors returned by schema registry mutations.
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("Cannot register built-in schema: {0}")]
    CannotRegisterBuiltin(String),

    #[error("Cannot unregister built-in schema: {0}")]
    CannotUnregisterBuiltin(String),

    #[error("Cannot delete built-in schema: {0}")]
    CannotDeleteBuiltin(String),

    #[error("Schema not found: {0}")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // §3.1: Built-in schemas are present
    #[test]
    fn test_registry_has_6_builtin_schemas() {
        let registry = ComponentSchemaRegistry::with_builtin_seeds();
        let count = registry.iter().count();
        assert_eq!(count, 6);
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

    // §Phase 3.1: editor.LogicBinding resolves through global_registry() and combined_registry()
    #[test]
    fn test_logic_binding_schema_in_global_registry() {
        let registry = ComponentSchemaRegistry::with_builtin_seeds();
        let schema = registry.get("editor.LogicBinding");
        assert!(schema.is_some(), "editor.LogicBinding should be in global registry");
        let schema = schema.unwrap();
        assert_eq!(schema.type_id, "editor.LogicBinding");
        assert_eq!(schema.display_name, "Logic Binding");

        // Check fields: asset_id (AssetReference) and version (F32)
        let field_names: Vec<&str> = schema.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"asset_id"), "should have asset_id field");
        assert!(field_names.contains(&"version"), "should have version field");

        let asset_id_field = schema.fields.iter().find(|f| f.name == "asset_id").unwrap();
        assert_eq!(asset_id_field.field_type, FieldType::AssetReference);

        let version_field = schema.fields.iter().find(|f| f.name == "version").unwrap();
        assert_eq!(version_field.field_type, FieldType::F32);
    }

    #[test]
    fn test_logic_binding_schema_in_combined_registry() {
        let combined = combined_registry();
        let schema = combined.get("editor.LogicBinding");
        assert!(schema.is_some(), "editor.LogicBinding should be in combined registry");
    }

    // §3.8: Global singleton
    #[test]
    fn test_global_registry_singleton() {
        let reg1 = global_registry();
        let reg2 = global_registry();
        assert_eq!(reg1 as *const _, reg2 as *const _);
        assert_eq!(reg1.iter().count(), 6);
    }

    // ===== Mutable user schema registry =====

    fn user_schema(type_id: &str) -> ComponentSchema {
        ComponentSchema {
            type_id: type_id.to_string(),
            display_name: type_id.to_string(),
            fields: vec![FieldDef {
                name: "value".to_string(),
                field_type: FieldType::F32,
                default: serde_json::json!(0.0),
                constraints: vec![],
            }],
            exports_to_bevy: true,
            source_location: None,
        }
    }

    #[test]
    fn test_is_builtin_type_editor_prefix_true() {
        assert!(is_builtin_type("editor.Transform2D"));
        assert!(is_builtin_type("editor.Name"));
        assert!(is_builtin_type("editor."));
    }

    #[test]
    fn test_is_builtin_type_game_prefix_false() {
        assert!(!is_builtin_type("game.PlayerHealth"));
        assert!(!is_builtin_type("my.Foo"));
        assert!(!is_builtin_type(""));
    }

    #[test]
    fn test_register_schema_rejects_builtin() {
        let result = register_schema(user_schema("editor.NewName"));
        assert!(matches!(result, Err(SchemaError::CannotRegisterBuiltin(_))));
    }

    #[test]
    fn test_register_schema_adds_user() {
        // Cleanup from any prior test
        let _ = unregister_schema("game.PlayerHealth");

        register_schema(user_schema("game.PlayerHealth")).unwrap();
        let combined = combined_registry();
        assert!(combined.get("game.PlayerHealth").is_some());

        // Cleanup
        let _ = unregister_schema("game.PlayerHealth");
    }

    #[test]
    fn test_register_schema_replaces_existing_user() {
        let _ = unregister_schema("game.EnemyAI");
        register_schema(user_schema("game.EnemyAI")).unwrap();

        // Replace with schema with different field
        let mut replacement = user_schema("game.EnemyAI");
        replacement.fields.push(FieldDef {
            name: "speed".to_string(),
            field_type: FieldType::F32,
            default: serde_json::json!(1.0),
            constraints: vec![],
        });
        register_schema(replacement).unwrap();

        let combined = combined_registry();
        let schema = combined.get("game.EnemyAI").unwrap();
        assert_eq!(schema.fields.len(), 2);

        let _ = unregister_schema("game.EnemyAI");
    }

    #[test]
    fn test_unregister_schema_removes_user() {
        let _ = unregister_schema("game.Foo");
        register_schema(user_schema("game.Foo")).unwrap();
        assert!(combined_registry().get("game.Foo").is_some());
        unregister_schema("game.Foo").unwrap();
        assert!(combined_registry().get("game.Foo").is_none());
    }

    #[test]
    fn test_unregister_schema_rejects_builtin() {
        let result = unregister_schema("editor.Transform2D");
        assert!(matches!(result, Err(SchemaError::CannotUnregisterBuiltin(_))));
    }

    #[test]
    fn test_unregister_schema_nonexistent_is_noop() {
        let result = unregister_schema("game.NeverRegistered");
        assert!(result.is_ok());
    }

    #[test]
    fn test_combined_registry_includes_builtins() {
        let combined = combined_registry();
        assert_eq!(combined.iter().count(), 6);
        assert!(combined.get("editor.Name").is_some());
        assert!(combined.get("editor.Transform2D").is_some());
    }

    #[test]
    fn test_combined_registry_includes_user_added() {
        let _ = unregister_schema("game.Bar");
        register_schema(user_schema("game.Bar")).unwrap();
        let combined = combined_registry();
        assert_eq!(combined.iter().count(), 7);
        assert!(combined.get("game.Bar").is_some());
        let _ = unregister_schema("game.Bar");
    }

    #[test]
    fn test_remove_method_on_registry() {
        let mut reg = ComponentSchemaRegistry::new();
        reg.insert(user_schema("game.X"));
        assert!(reg.get("game.X").is_some());
        let removed = reg.remove("game.X");
        assert!(removed.is_some());
        assert!(reg.get("game.X").is_none());
        // Removing non-existent returns None
        assert!(reg.remove("game.NonExistent").is_none());
    }

    // ===== SourceLocation tests =====

    #[test]
    fn test_source_location_serde_roundtrip() {
        let loc = SourceLocation {
            file_id: "src/components/player.rs".to_string(),
            line: 42,
            column: 7,
        };
        let json = serde_json::to_string(&loc).unwrap();
        let roundtrip: SourceLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.file_id, "src/components/player.rs");
        assert_eq!(roundtrip.line, 42);
        assert_eq!(roundtrip.column, 7);
    }

    #[test]
    fn test_source_location_default_column() {
        // column should default to 1 when deserializing from JSON without column field
        let json = r#"{"file_id": "lib.rs", "line": 10}"#;
        let loc: SourceLocation = serde_json::from_str(json).unwrap();
        assert_eq!(loc.column, 1);
    }

    #[test]
    fn test_source_location_none_in_schema() {
        // Built-in schemas have no source location
        let registry = ComponentSchemaRegistry::with_builtin_seeds();
        let schema = registry.get("editor.Transform2D").unwrap();
        assert!(schema.source_location.is_none());
    }

    #[test]
    fn test_schema_with_source_location() {
        let schema = ComponentSchema {
            type_id: "game.Player".to_string(),
            display_name: "Player".to_string(),
            fields: vec![],
            exports_to_bevy: true,
            source_location: Some(SourceLocation {
                file_id: "src/ecs/components.rs".to_string(),
                line: 10,
                column: 1,
            }),
        };
        let json = serde_json::to_string(&schema).unwrap();
        let roundtrip: ComponentSchema = serde_json::from_str(&json).unwrap();
        assert!(roundtrip.source_location.is_some());
        assert_eq!(roundtrip.source_location.unwrap().file_id, "src/ecs/components.rs");
    }

    #[test]
    fn test_schema_source_location_missing_from_json_is_none() {
        // Existing JSON without source_location should deserialize with None
        let json = r#"{
            "type_id": "game.OldSchema",
            "display_name": "Old Schema",
            "fields": [],
            "exports_to_bevy": true
        }"#;
        let schema: ComponentSchema = serde_json::from_str(json).unwrap();
        assert!(schema.source_location.is_none());
    }
}
