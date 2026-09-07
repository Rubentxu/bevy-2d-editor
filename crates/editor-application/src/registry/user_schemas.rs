//! User-defined component schema registry — H2.2 (single-session composition root).
//!
//! `UserSchemaRegistry` is the canonical implementation of
//! `editor_model::ports::UserSchemaRegistryPort`. It owns both:
//!
//! 1. **Built-in schemas** (6 editor.* schemas seeded at `EditorSession::with_builtins`).
//!    These are immutable from the registry's API perspective — registration and
//!    removal are rejected for any type_id starting with `editor.`.
//! 2. **User-defined schemas** added at runtime through `register` / `unregister`.
//!
//! The registry is `Send + Sync`-capable via `Arc<Mutex<...>>` wrappers in
//! `editor_model::ports`. Mutations are routed through the port cell so that
//! `editor-bevy` and `editor-wasm` callers cannot bypass the session boundary
//! (per ADR-0057 — composition root, ADR-0059 — mutation path).

use std::collections::HashMap;

use editor_model::ports::UserSchemaRegistryPort;
use editor_model::schema::{
    ApplyBackPolicy, ComponentSchema, FieldDef, FieldType, SchemaError, SchemaKind, is_builtin_type,
};

/// In-memory registry of component schemas (built-ins + user-defined).
///
/// H2.2: replaces `editor_bevy::schema::ComponentSchemaRegistry` and the
/// `USER_SCHEMAS` thread_local. Constructed by `EditorSession::with_builtins`
/// and registered into the `USER_SCHEMA_REGISTRY` port cell at the
/// composition root (`editor-wasm`).
#[derive(Debug)]
pub struct UserSchemaRegistry {
    schemas: HashMap<String, ComponentSchema>,
}

impl UserSchemaRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Create a registry seeded with the 6 built-in editor.* schemas
    /// (Name, Transform2D, Sprite2D, Visible, Locked, LogicBinding).
    ///
    /// Returns `Err` only if a future seed introduces a SceneComponent
    /// schema without a `bound_scene_asset_ref`. Today's seeds are all
    /// `Simple`, so `Ok` is guaranteed.
    pub fn with_builtins() -> Result<Self, SchemaError> {
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
            kind: SchemaKind::Simple,
            bound_scene_asset_ref: None,
            auto_spawn: true,
            apply_back: ApplyBackPolicy::Never,
        })?;

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
            kind: SchemaKind::Simple,
            bound_scene_asset_ref: None,
            auto_spawn: true,
            apply_back: ApplyBackPolicy::Never,
        })?;

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
            kind: SchemaKind::Simple,
            bound_scene_asset_ref: None,
            auto_spawn: true,
            apply_back: ApplyBackPolicy::Never,
        })?;

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
            kind: SchemaKind::Simple,
            bound_scene_asset_ref: None,
            auto_spawn: true,
            apply_back: ApplyBackPolicy::Never,
        })?;

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
            kind: SchemaKind::Simple,
            bound_scene_asset_ref: None,
            auto_spawn: true,
            apply_back: ApplyBackPolicy::Never,
        })?;

        // editor.LogicBinding
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
            kind: SchemaKind::Simple,
            bound_scene_asset_ref: None,
            auto_spawn: true,
            apply_back: ApplyBackPolicy::Never,
        })?;

        Ok(registry)
    }

    /// Direct insert helper used by `with_builtins` and tests. Public so the
    /// port-trait `register` impl can delegate to it after built-in checks.
    pub fn insert(&mut self, schema: ComponentSchema) -> Result<(), SchemaError> {
        if schema.kind == SchemaKind::SceneComponent && schema.bound_scene_asset_ref.is_none() {
            return Err(SchemaError::MissingBoundSceneAsset(schema.type_id));
        }
        self.schemas.insert(schema.type_id.clone(), schema);
        Ok(())
    }

    /// Direct remove helper used by tests and the port-trait impl. Returns
    /// the removed schema if found.
    pub fn remove(&mut self, type_id: &str) -> Option<ComponentSchema> {
        self.schemas.remove(type_id)
    }
}

impl Default for UserSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl UserSchemaRegistryPort for UserSchemaRegistry {
    fn register(&mut self, schema: ComponentSchema) -> Result<(), SchemaError> {
        // Allow overrides of existing entries (built-in or user). For new
        // entries with the `editor.*` prefix, reject — only the canonical
        // seeds should be registered as built-ins. This fixes a pre-existing
        // bug where `bind_scene_to_schema` could not override built-ins.
        let is_existing = self.schemas.contains_key(&schema.type_id);
        if !is_existing && is_builtin_type(&schema.type_id) {
            return Err(SchemaError::CannotRegisterBuiltin(schema.type_id));
        }
        self.insert(schema)
    }

    fn unregister(&mut self, type_id: &str) -> Result<(), SchemaError> {
        if is_builtin_type(type_id) {
            return Err(SchemaError::CannotUnregisterBuiltin(type_id.to_string()));
        }
        self.remove(type_id);
        Ok(())
    }

    fn is_builtin(&self, type_id: &str) -> bool {
        is_builtin_type(type_id)
    }

    fn combined_view(&self) -> Vec<ComponentSchema> {
        self.schemas.values().cloned().collect()
    }

    fn get(&self, type_id: &str) -> Option<ComponentSchema> {
        self.schemas.get(type_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::schema::{ComponentSchema, FieldDef, FieldType, SchemaKind};

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
            ..Default::default()
        }
    }

    // ───── Built-in seed tests (carry-over from editor-bevy schema tests) ─────

    #[test]
    fn with_builtin_seeds_has_six_builtins() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        let view = registry.combined_view();
        assert_eq!(view.len(), 6);
    }

    #[test]
    fn with_builtin_seeds_contains_known_type_ids() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        for id in [
            "editor.Name",
            "editor.Transform2D",
            "editor.Sprite2D",
            "editor.Visible",
            "editor.Locked",
            "editor.LogicBinding",
        ] {
            assert!(registry.get(id).is_some(), "missing {id}");
        }
    }

    #[test]
    fn with_builtin_seeds_transform2d_fields() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        let schema = registry.get("editor.Transform2D").unwrap();
        let names: Vec<&str> = schema.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"translation"));
        assert!(names.contains(&"rotation"));
        assert!(names.contains(&"scale"));
    }

    #[test]
    fn with_builtin_seeds_visible_locked_editorial_only() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        let visible = registry.get("editor.Visible").unwrap();
        assert!(!visible.exports_to_bevy);
        let locked = registry.get("editor.Locked").unwrap();
        assert!(!locked.exports_to_bevy);
    }

    // ───── Port-trait behaviour ─────

    #[test]
    fn register_rejects_builtin() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        let result = registry.register(user_schema("editor.NewName"));
        assert!(matches!(result, Err(SchemaError::CannotRegisterBuiltin(_))));
    }

    #[test]
    fn register_adds_user_schema() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        registry.register(user_schema("game.PlayerHealth")).unwrap();
        assert!(registry.get("game.PlayerHealth").is_some());
    }

    #[test]
    fn register_replaces_existing_user() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        registry.register(user_schema("game.EnemyAI")).unwrap();

        let mut replacement = user_schema("game.EnemyAI");
        replacement.fields.push(FieldDef {
            name: "speed".to_string(),
            field_type: FieldType::F32,
            default: serde_json::json!(1.0),
            constraints: vec![],
        });
        registry.register(replacement).unwrap();

        let schema = registry.get("game.EnemyAI").unwrap();
        assert_eq!(schema.fields.len(), 2);
    }

    #[test]
    fn unregister_removes_user_schema() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        registry.register(user_schema("game.Foo")).unwrap();
        assert!(registry.get("game.Foo").is_some());
        registry.unregister("game.Foo").unwrap();
        assert!(registry.get("game.Foo").is_none());
    }

    #[test]
    fn unregister_rejects_builtin() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        let result = registry.unregister("editor.Transform2D");
        assert!(matches!(
            result,
            Err(SchemaError::CannotUnregisterBuiltin(_))
        ));
    }

    #[test]
    fn unregister_nonexistent_is_noop() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        assert!(registry.unregister("game.NeverRegistered").is_ok());
    }

    #[test]
    fn is_builtin_recognises_editor_prefix() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        assert!(registry.is_builtin("editor.Transform2D"));
        assert!(!registry.is_builtin("game.Player"));
    }

    #[test]
    fn combined_view_includes_builtins_and_user() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        registry.register(user_schema("game.Bar")).unwrap();
        let view = registry.combined_view();
        assert_eq!(view.len(), 7);
        assert!(view.iter().any(|s| s.type_id == "editor.Name"));
        assert!(view.iter().any(|s| s.type_id == "game.Bar"));
    }

    #[test]
    fn insert_scene_component_requires_bound_asset() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        let schema = ComponentSchema {
            type_id: "game.Enemy".to_string(),
            display_name: "Enemy".to_string(),
            fields: vec![],
            exports_to_bevy: true,
            source_location: None,
            kind: SchemaKind::SceneComponent,
            bound_scene_asset_ref: None,
            ..Default::default()
        };
        let result = registry.register(schema);
        assert!(matches!(
            result,
            Err(SchemaError::MissingBoundSceneAsset(_))
        ));
    }

    #[test]
    fn insert_scene_component_with_bound_asset_succeeds() {
        let mut registry = UserSchemaRegistry::with_builtins().unwrap();
        let schema = ComponentSchema {
            type_id: "game.Enemy".to_string(),
            display_name: "Enemy".to_string(),
            fields: vec![],
            exports_to_bevy: true,
            source_location: None,
            kind: SchemaKind::SceneComponent,
            bound_scene_asset_ref: Some("level1".to_string()),
            ..Default::default()
        };
        assert!(registry.register(schema).is_ok());
    }
}
