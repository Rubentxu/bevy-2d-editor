//! Component Schema Registry — Bevy adapter facade.
//!
//! H2.2 (single-session composition root): this module is a thin
//! Bevy-side facade over the canonical registry in
//! `editor_application::registry::user_schemas`, accessed via the
//! `UserSchemaRegistryPort` port cell in `editor_model::ports`.
//!
//! Value types (`ComponentSchema`, `FieldType`, `Constraint`, `FieldDef`,
//! `SourceLocation`, `SchemaKind`, `SchemaError`, `ApplyBackPolicy`,
//! `ComponentTypeId`) are re-exported from `editor_model::schema` so
//! existing call sites continue to compile. The legacy `REGISTRY`
//! `OnceLock<ComponentSchemaRegistry>` and `USER_SCHEMAS` `thread_local!`
//! have been removed — built-in seeds live on `EditorSession.user_schemas`
//! at the WASM composition root.
//!
//! The facade functions in this module are kept for backward source
//! compatibility with WASM exports and Bevy internals:
//!
//! - `register_schema` / `unregister_schema` delegate to the port cell.
//! - `combined_registry()` returns a `Vec<ComponentSchema>` snapshot from
//!   the port cell (callers that previously iterated `ComponentSchemaRegistry`
//!   must be updated to iterate the `Vec` — see `code_export.rs`).
//! - `is_builtin_type` delegates to `editor_model::schema::is_builtin_type`.

pub use editor_model::schema::{
    ApplyBackPolicy, ComponentSchema, ComponentTypeId, Constraint, FieldDef, FieldType,
    SchemaError, SchemaKind, SourceLocation, is_builtin_type,
};

use std::sync::Arc;

// ─────────────────────────────────────────────────────────────────────────────
// Mutable user-schema facade (H2.2 — port-cell-backed)
// ─────────────────────────────────────────────────────────────────────────────

/// Register a user-defined schema in memory (does NOT save to OPFS).
/// Built-in `editor.*` schemas are rejected.
pub fn register_schema(schema: ComponentSchema) -> Result<(), SchemaError> {
    crate::schema::ports_bridge::with_registry_write(|reg| reg.register(schema))
}

/// Unregister a user-defined schema from memory (does NOT delete OPFS file).
/// Built-in `editor.*` schemas are rejected. Missing schemas are a no-op success.
pub fn unregister_schema(type_id: &str) -> Result<(), SchemaError> {
    crate::schema::ports_bridge::with_registry_write(|reg| reg.unregister(type_id))
}

/// Returns a snapshot of all schemas (built-ins + user-defined).
///
/// H2.2: returns `Vec<ComponentSchema>` instead of the legacy
/// `ComponentSchemaRegistry` struct (which moved to
/// `editor_application::registry::user_schemas::UserSchemaRegistry`).
/// Callers that need iteration should iterate the `Vec` directly; callers
/// that need lookup should call `get_schema(type_id)`.
pub fn combined_registry() -> Vec<ComponentSchema> {
    crate::schema::ports_bridge::with_registry_read(|reg| reg.combined_view())
}

/// Look up a schema by type_id from the combined (built-ins + user) view.
///
/// H2.2 replacement for the previous `ComponentSchemaRegistry::get` method
/// on the returned-by-`combined_registry()` value.
pub fn get_schema(type_id: &str) -> Option<ComponentSchema> {
    crate::schema::ports_bridge::with_registry_read(|reg| reg.get(type_id))
}

/// Iterate the combined (built-ins + user) view.
///
/// H2.2 replacement for the previous `ComponentSchemaRegistry::iter`.
/// Provided as a free function because `Vec` already has `iter()`; this
/// exists for source-compat with sites that used `combined_registry().iter()`.
pub fn iter_schemas() -> Vec<ComponentSchema> {
    combined_registry()
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal: ports bridge helper
// ─────────────────────────────────────────────────────────────────────────────

/// Test-only helper used by `editor-bevy` unit tests to initialise the
/// `USER_SCHEMA_REGISTRY` port cell with the 6 built-in schemas.
///
/// H2.2: `editor-bevy` cannot import `editor_application::UserSchemaRegistry`
/// directly (H1.4 dependency direction), so the seed data is duplicated
/// here as a test fixture. The canonical seeds live in
/// `editor_application::registry::user_schemas::UserSchemaRegistry::with_builtins`.
///
/// Production code MUST NOT use this module — production code path is
/// `editor_wasm::init_project_store` → `register_user_schema_registry` →
/// `editor-bevy` facades.
#[doc(hidden)]
pub mod __test_only {
    use super::*;
    use editor_model::ports::{UserSchemaRegistryPort, register_user_schema_registry};
    use std::sync::{Arc, Mutex};

    fn seed() -> Vec<ComponentSchema> {
        vec![
            ComponentSchema {
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
                ..Default::default()
            },
            ComponentSchema {
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
                ..Default::default()
            },
            ComponentSchema {
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
                ..Default::default()
            },
            ComponentSchema {
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
                ..Default::default()
            },
            ComponentSchema {
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
                ..Default::default()
            },
            ComponentSchema {
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
                ..Default::default()
            },
        ]
    }

    /// Register the test registry in the port cell. Idempotent within a
    /// single test process — re-registration is a no-op.
    pub fn register_builtins() {
        // Use a one-shot inner registry; the port cell stores an Arc clone.
        struct TestRegistry {
            schemas: HashMap<String, ComponentSchema>,
        }
        impl UserSchemaRegistryPort for TestRegistry {
            fn register(&mut self, schema: ComponentSchema) -> Result<(), SchemaError> {
                self.schemas.insert(schema.type_id.clone(), schema);
                Ok(())
            }
            fn unregister(&mut self, type_id: &str) -> Result<(), SchemaError> {
                self.schemas.remove(type_id);
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

        let mut registry = TestRegistry {
            schemas: HashMap::new(),
        };
        for schema in seed() {
            // Built-in seed: bypass `register` validation (it would reject
            // editor.* as new entries). Use insert directly.
            registry.schemas.insert(schema.type_id.clone(), schema);
        }
        let arc: Arc<Mutex<dyn UserSchemaRegistryPort>> = Arc::new(Mutex::new(registry));
        register_user_schema_registry(Arc::clone(&arc));
    }

    /// Returns the test registry handle (used by tests that want to add
    /// user schemas to the same registry the facades see).
    pub fn handle() -> Arc<Mutex<dyn UserSchemaRegistryPort>> {
        editor_model::ports::with_user_schema_registry().expect(
            "register_builtins() must be called before handle() — typically via \
             init_user_schema_registry() in test setup",
        )
    }
}

// Re-export HashMap for the test-only module above.
use std::collections::HashMap;

mod ports_bridge {
    use super::*;
    use editor_model::ports::{UserSchemaRegistryPort, with_user_schema_registry as cell_get};

    /// Convenience: run `f` with the registered registry (or panic if not
    /// initialised, since Bevy facade calls assume the composition root
    /// has been wired).
    ///
    /// Bevy systems that run before `editor_wasm::init_project_store`
    /// would panic here — but per ADR-0057, the composition root is the
    /// only owner of session lifecycle, so the cell is always populated
    /// before any Bevy system observes the editor.
    ///
    /// `read_only` selects between `&*guard` (for `is_builtin`,
    /// `combined_view`, `get`) and `&mut *guard` (for `register`,
    /// `unregister`). The caller picks the variant at the call site.
    pub(crate) fn with_registry_read<R>(f: impl FnOnce(&dyn UserSchemaRegistryPort) -> R) -> R {
        let arc: Arc<std::sync::Mutex<dyn UserSchemaRegistryPort>> = cell_get().expect(
            "USER_SCHEMA_REGISTRY not initialised — composition root must call \
             `editor_model::ports::register_user_schema_registry` before Bevy \
             systems observe the editor",
        );
        let guard = arc.lock().expect("USER_SCHEMA_REGISTRY lock poisoned");
        f(&*guard)
    }

    pub(crate) fn with_registry_write<R>(
        f: impl FnOnce(&mut dyn UserSchemaRegistryPort) -> R,
    ) -> R {
        let arc: Arc<std::sync::Mutex<dyn UserSchemaRegistryPort>> = cell_get().expect(
            "USER_SCHEMA_REGISTRY not initialised — composition root must call \
             `editor_model::ports::register_user_schema_registry` before Bevy \
             systems observe the editor",
        );
        let mut guard = arc.lock().expect("USER_SCHEMA_REGISTRY lock poisoned");
        f(&mut *guard)
    }
}
