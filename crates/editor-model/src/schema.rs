//! Component Schema Registry — canonical value types.
//!
//! H2.2 (single-session composition root): this module is the canonical
//! location for the editor's component schema value types. Both
//! `editor-bevy` (legacy) and `editor-application` (current) consume the
//! types defined here. The registry implementation lives in
//! `editor_application::registry::user_schemas` and is accessed through
//! the `UserSchemaRegistryPort` trait in `editor_model::ports`.
//!
//! The shape of every type in this module is the production-proven shape
//! that was previously defined in the editor-bevy schema module. The legacy
//! model-local types (Option-based `SourceLocation`, tag-based `FieldType`,
//! `Range`/`Step`/`Pattern`/`Required` constraints, `location`/`description`
//! fields on `FieldDef` and `ComponentSchema`) have been removed in H2.2;
//! the canonical set is narrower and matches the JSON shapes already
//! shipped to v0.72.0+ clients.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─────────────────────────────────────────────────────────────────────────────
// Apply-back policy (ADR-0042, ADR-0050)
// ─────────────────────────────────────────────────────────────────────────────

/// Policy governing whether and how a component's runtime values may be
/// applied back to the authoring state (ADR-0050, ADR-0042).
///
/// Serialized as part of `ComponentSchema`. Defaults to `Never` for all
/// existing schemas (per D4 — conservative default).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyBackPolicy {
    /// Never apply runtime values back to authoring state.
    #[default]
    Never,
    /// Apply back only when explicitly requested by the user.
    ExplicitOnly,
    /// Apply back is suggested; user may tune the value.
    Tunable,
}

// ─────────────────────────────────────────────────────────────────────────────
// Identity (ADR-0031)
// ─────────────────────────────────────────────────────────────────────────────

/// Opaque component type identifier used by the Component Schema Registry.
/// Transparent so it serializes as a plain string, e.g. `editor.Transform2D`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentTypeId(
    /// Inner string id (e.g. `editor.Transform2D`).
    pub String,
);

impl ComponentTypeId {
    /// Construct a new `ComponentTypeId` from any string-convertible value.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrow the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Source location
// ─────────────────────────────────────────────────────────────────────────────

/// Source location in a Rust source file.
/// Used for "jump to definition" navigation from component schema to source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Project-relative path to the source file.
    pub file_id: String,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number; defaults to 1 when missing in JSON.
    #[serde(default = "default_source_location_column")]
    pub column: u32,
}

fn default_source_location_column() -> u32 {
    1
}

// ─────────────────────────────────────────────────────────────────────────────
// Field definitions
// ─────────────────────────────────────────────────────────────────────────────

/// Field type enumeration for schema field definitions.
///
/// Hito 4 Order 7 (`scene-component-authoring`): added `ComponentRef` and
/// `Enum` variants. `ComponentRef` is `#[serde(untagged)]` to serialize as
/// a plain string (the type_id) for backward compat with v0.72.0 and
/// earlier clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldType {
    /// UTF-8 string value.
    String,
    /// 32-bit floating-point.
    F32,
    /// Boolean value.
    Bool,
    /// 2D vector of f32.
    Vec2,
    /// RGBA color.
    Color,
    /// Bevy sprite anchor (serialized as `String`).
    Anchor,
    /// Reference to a project asset (logical path).
    AssetReference,
    /// Reference to another component schema by type_id.
    /// Serializes as a plain string (the type_id).
    ComponentRef(String),
    /// Bounded string-enum; only values in `variants` are accepted.
    Enum {
        /// Allowed variant names.
        variants: Vec<String>,
    },
}

/// Constraint on a field value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    /// Inclusive lower bound for numeric fields.
    Min(f32),
    /// Inclusive upper bound for numeric fields.
    Max(f32),
    /// String must be non-empty.
    NonEmpty,
}

/// A single field definition within a component schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    /// Name of the field (Rust identifier in the generated code).
    pub name: String,
    /// Type discriminator for the field.
    pub field_type: FieldType,
    /// Default value when not specified; null by default.
    #[serde(default = "default_json_value")]
    pub default: serde_json::Value,
    /// Active constraints on this field's value.
    #[serde(default)]
    pub constraints: Vec<Constraint>,
}

fn default_json_value() -> serde_json::Value {
    serde_json::Value::Null
}

// ─────────────────────────────────────────────────────────────────────────────
// Schema discriminator
// ─────────────────────────────────────────────────────────────────────────────

/// Discriminator for component schema purpose.
///
/// Hito 4 Order 7 (`scene-component-authoring`): `SceneComponent` indicates
/// the schema wraps a `SceneAssetDocument` and is authored via Bevy 0.19's
/// `#[derive(SceneComponent)]` semantics. `Simple` is the legacy default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaKind {
    /// Regular component with typed fields.
    #[default]
    Simple,
    /// Bound to a `SceneAssetDocument`; instancing spawns the bound scene
    /// (when `auto_spawn: true`, which is the default).
    SceneComponent,
}

// ─────────────────────────────────────────────────────────────────────────────
// ComponentSchema
// ─────────────────────────────────────────────────────────────────────────────

/// A component schema defining the structure of a component type.
///
/// Hito 4 Order 7: added 3 fields, all `#[serde(default)]` for backward
/// compatibility with v0.72.0 and earlier clients (they default to
/// `kind = Simple`, `bound_scene_asset_ref = None`, `auto_spawn = false`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentSchema {
    /// Type identifier (e.g. `editor.Transform2D` or `game.PlayerHealth`).
    pub type_id: String,
    /// Human-readable label shown in the inspector.
    pub display_name: String,
    /// Field definitions in declaration order.
    pub fields: Vec<FieldDef>,
    /// Whether this component exports to Bevy runtime.
    /// Editorial-only components (Visible, Locked) set this to false.
    pub exports_to_bevy: bool,
    /// Optional source location for "jump to definition" navigation.
    /// Points to the Rust struct definition in the editor's source files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,
    /// Hito 4 Order 7: schema discriminator.
    #[serde(default)]
    pub kind: SchemaKind,
    /// Hito 4 Order 7: when `kind == SceneComponent`, references the bound
    /// `SceneAssetDocument` id. Otherwise `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound_scene_asset_ref: Option<String>,
    /// Hito 4 Order 7: when `kind == SceneComponent`, controls whether
    /// instancing auto-spawns the bound scene (Bevy parity) or requires
    /// explicit opt-in. Defaults to `true` for SceneComponent schemas.
    #[serde(default = "default_auto_spawn")]
    pub auto_spawn: bool,
    /// Apply-back policy for fields of this component (ADR-0042, ADR-0050).
    ///
    /// Defaults to `Never` — runtime values are never applied back to the
    /// authoring state unless the schema explicitly opts in.
    #[serde(default)]
    pub apply_back: ApplyBackPolicy,
}

fn default_auto_spawn() -> bool {
    // Default `true` so new SceneComponent schemas match Bevy 0.19
    // `#[derive(SceneComponent)]` behavior out of the box.
    true
}

impl Default for ComponentSchema {
    fn default() -> Self {
        Self {
            type_id: String::new(),
            display_name: String::new(),
            fields: Vec::new(),
            exports_to_bevy: true,
            source_location: None,
            kind: SchemaKind::default(), // Simple
            bound_scene_asset_ref: None,
            auto_spawn: default_auto_spawn(), // true
            apply_back: ApplyBackPolicy::Never,
        }
    }
}

impl ComponentSchema {
    /// True if this schema is bound to a scene asset.
    pub fn is_scene_component(&self) -> bool {
        matches!(self.kind, SchemaKind::SceneComponent)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

/// Errors returned by schema registry mutations.
#[derive(Debug, Error)]
pub enum SchemaError {
    /// Attempted to register a new schema with a built-in `editor.*` type id.
    #[error("Cannot register built-in schema: {0}")]
    CannotRegisterBuiltin(String),

    /// Attempted to unregister a built-in `editor.*` type id.
    #[error("Cannot unregister built-in schema: {0}")]
    CannotUnregisterBuiltin(String),

    /// Attempted to delete a built-in `editor.*` type id.
    #[error("Cannot delete built-in schema: {0}")]
    CannotDeleteBuiltin(String),

    /// Schema lookup failed.
    #[error("Schema not found: {0}")]
    NotFound(String),

    /// Hito 4 Order 7: SceneComponent schema must reference a bound scene asset.
    #[error("SceneComponent schema {0} is missing bound_scene_asset_ref")]
    MissingBoundSceneAsset(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Built-in predicate
// ─────────────────────────────────────────────────────────────────────────────

/// Returns true if the type_id is a built-in (starts with `editor.`).
/// Built-ins are immutable: cannot be registered, unregistered, or deleted.
///
/// H2.2: this is a pure function over the type_id. The port-trait impl
/// delegates to it for the `is_builtin` method.
pub fn is_builtin_type(type_id: &str) -> bool {
    type_id.starts_with("editor.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_type_id_roundtrip() {
        let id = ComponentTypeId::new("editor.Transform2D");
        assert_eq!(id.as_str(), "editor.Transform2D");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"editor.Transform2D\"");
        let parsed: ComponentTypeId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn field_type_component_ref_serializes_as_string() {
        let ft = FieldType::ComponentRef("game.Enemy".to_string());
        let json = serde_json::to_string(&ft).unwrap();
        assert_eq!(json, "\"game.Enemy\"");
        let parsed: FieldType = serde_json::from_str("\"game.Player\"").unwrap();
        assert_eq!(parsed, FieldType::ComponentRef("game.Player".to_string()));
    }

    #[test]
    fn field_type_enum_serializes_with_variants() {
        let ft = FieldType::Enum {
            variants: vec!["red".to_string(), "green".to_string(), "blue".to_string()],
        };
        let json = serde_json::to_string(&ft).unwrap();
        let parsed: FieldType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ft);
    }

    #[test]
    fn source_location_default_column() {
        let json = r#"{"file_id": "lib.rs", "line": 10}"#;
        let loc: SourceLocation = serde_json::from_str(json).unwrap();
        assert_eq!(loc.column, 1);
    }

    #[test]
    fn source_location_serde_roundtrip() {
        let loc = SourceLocation {
            file_id: "src/components/player.rs".to_string(),
            line: 42,
            column: 7,
        };
        let json = serde_json::to_string(&loc).unwrap();
        let roundtrip: SourceLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip, loc);
    }

    #[test]
    fn schema_backward_compat_no_kind_field() {
        // v0.72.0 and earlier schemas serialized without `kind`. The
        // #[serde(default)] ensures deserialization still works.
        let json = r#"{
            "type_id": "game.Old",
            "display_name": "Old",
            "fields": [],
            "exports_to_bevy": true
        }"#;
        let schema: ComponentSchema = serde_json::from_str(json).unwrap();
        assert_eq!(schema.kind, SchemaKind::Simple);
        assert_eq!(schema.bound_scene_asset_ref, None);
        assert!(schema.auto_spawn);
    }

    #[test]
    fn schema_kind_default_is_simple() {
        let schema = ComponentSchema {
            type_id: "game.Test".to_string(),
            display_name: "Test".to_string(),
            fields: vec![],
            exports_to_bevy: true,
            source_location: None,
            ..Default::default()
        };
        assert_eq!(schema.kind, SchemaKind::Simple);
        assert!(schema.auto_spawn, "auto_spawn defaults to true");
    }

    #[test]
    fn is_builtin_type_recognizes_editor_prefix() {
        assert!(is_builtin_type("editor.Transform2D"));
        assert!(is_builtin_type("editor.Name"));
        assert!(is_builtin_type("editor."));
    }

    #[test]
    fn is_builtin_type_rejects_other_prefixes() {
        assert!(!is_builtin_type("game.PlayerHealth"));
        assert!(!is_builtin_type("my.Foo"));
        assert!(!is_builtin_type(""));
    }
}
