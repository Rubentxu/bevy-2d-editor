//! Command System types for the Bevy 2D Editor.
//!
//! Defines the typed Command enum, metadata, envelope, result, and error types
//! that form the editor's mutation surface. Per Hito 0 §6.4, commands are
//! semantic (not raw diffs), fully reversible, and carry authorship metadata
//! for future agent auditing.

use crate::document::{ComponentInstance, StableId};
use crate::scene_asset::{AssetReference, LocalId};
use crate::scene_instance::SceneInstance;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Typed command enum covering the 8 semantic operations from Hito 0 §6.4
/// plus a `Batch` wrapper for gesture grouping.
///
/// Uses `#[serde(tag = "type")]` so each variant serializes as
/// `{"type": "CreateEntity", ...}` — self-describing and extensible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum Command {
    /// Add a new entity to the document.
    CreateEntity {
        id: StableId,
        name: String,
        #[serde(default)]
        components: Vec<ComponentInstance>,
    },
    /// Remove an entity from the document. Children are reparented to root.
    DeleteEntity {
        id: StableId,
    },
    /// Attach a new component instance to an existing entity.
    AddComponent {
        entity_id: StableId,
        type_id: String,
        #[serde(default)]
        values: serde_json::Value,
    },
    /// Remove a component instance from an entity.
    RemoveComponent {
        entity_id: StableId,
        type_id: String,
    },
    /// Update one field of a component instance.
    /// `field_path` is dotted, e.g. `"translation.x"` or `"color.r"`.
    SetComponentField {
        entity_id: StableId,
        type_id: String,
        field_path: String,
        value: serde_json::Value,
    },
    /// Move an entity under a new parent. `old_parent` is captured pre-state
    /// for inverse generation; the caller may leave it as `None` and the
    /// processor will populate it during apply.
    ReparentEntity {
        entity_id: StableId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_parent: Option<StableId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        new_parent: Option<StableId>,
    },
    /// Change an entity's human-readable name. The `id` MUST NOT change.
    RenameEntity {
        entity_id: StableId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_name: Option<String>,
        new_name: String,
    },
    /// Group multiple commands into a single atomic history entry.
    /// On failure, all previously applied commands in the batch are rolled back.
    Batch {
        label: String,
        commands: Vec<Command>,
    },
    /// Place a Scene Asset as a new Scene Instance in the document.
    /// ADR-0007 §Command surface: instances share the scene OperationLog.
    PlaceInstance {
        instance_id: StableId,
        asset_ref: AssetReference,
        asset_version: u32,
        id_map: BTreeMap<LocalId, StableId>,
        /// Components owned by the placed occurrence (placement-time).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        instance_components: Vec<crate::document::ComponentInstance>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        component_overrides: Vec<crate::scene_instance::ComponentOverride>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        orphaned_component_overrides: Vec<crate::scene_instance::ComponentOverride>,
    },
    /// Remove a Scene Instance from the document.
    /// Inverse is PlaceInstance restoring the full captured pre-state.
    RemoveInstance {
        instance_id: StableId,
    },
    /// Replace the asset_ref of an existing Scene Instance.
    /// Runs resync to reclassify overrides; captures pre-state for inverse.
    ReplaceInstanceAsset {
        instance_id: StableId,
        new_asset_ref: AssetReference,
        new_asset_version: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        captured_old: Option<SceneInstance>,
    },
    /// Insert or replace a component override on a Scene Instance.
    /// Inverse: RevertOverride if no prior override existed; UpsertOverride{old_patch} otherwise.
    UpsertOverride {
        instance_id: StableId,
        target_local_id: LocalId,
        component_type_id: crate::schema::ComponentTypeId,
        field_path: Vec<String>,
        value: serde_json::Value,
    },
    /// Remove a component override from a Scene Instance.
    /// Idempotent no-op when absent; inverse re-inserts the captured patch.
    RevertOverride {
        instance_id: StableId,
        target_local_id: LocalId,
        component_type_id: crate::schema::ComponentTypeId,
        field_path: Vec<String>,
    },
    /// Paint a tile onto a TileLayer in a Level Scene Asset.
    /// Inverse is EraseTile with the same coordinate.
    PaintTile {
        /// The parent Scene Asset document (Level) that owns the TileLayer.
        asset_ref: AssetReference,
        /// ID of the TileLayer inside the asset.
        layer_id: crate::tile_layer::TileLayerId,
        /// Grid coordinate to paint.
        coord: crate::tileset::TileCoord,
        /// The tile reference to paint.
        tile_ref: crate::tileset::TileRef,
    },
    /// Erase a tile from a TileLayer in a Level Scene Asset.
    /// Inverse is PaintTile restoring the captured tile_ref.
    EraseTile {
        asset_ref: AssetReference,
        layer_id: crate::tile_layer::TileLayerId,
        coord: crate::tileset::TileCoord,
    },
}

/// Metadata attached to each command for future agent auditing (Hito 0 §6.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// Who issued the command: `"user"`, `"agent:<id>"`, or `"system"`.
    pub authorship: String,
    /// Unix milliseconds when the command was issued.
    pub timestamp: u64,
    /// Optional human/agent explanation of why the command was issued.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

impl CommandMetadata {
    /// Convenience constructor with current epoch millis.
    pub fn now(authorship: impl Into<String>) -> Self {
        Self {
            authorship: authorship.into(),
            timestamp: current_timestamp_ms(),
            rationale: None,
        }
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }
}

/// Envelope wrapping a command with its metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub command: Command,
    pub metadata: CommandMetadata,
}

/// Result of applying a command: the inverse command (for undo) and the
/// post-apply document snapshot (for state confirmation / future logging).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandResult {
    pub inverse: Command,
    pub snapshot: crate::document::SceneDocument,
}

/// Typed errors returned by command validation and application.
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Entity not found: {0}")]
    EntityNotFound(StableId),

    #[error("Duplicate entity id: {0}")]
    DuplicateId(StableId),

    #[error("Unknown schema: {0}")]
    UnknownSchema(String),

    #[error("Field not found: {0}")]
    FieldNotFound(String),

    #[error("Reparent would create cycle through {0}")]
    WouldCreateCycle(StableId),

    #[error("Batch failed at command {index}: {source}")]
    BatchFailed {
        index: usize,
        #[source]
        source: Box<CommandError>,
    },

    #[error("JSON serialization error: {0}")]
    JsonError(String),

    /// Asset has more than one root entity; single-root gate failed.
    #[error("Multiple roots: asset has {0} root entities, expected 1")]
    MultipleRoots(usize),

    /// Asset has no entities; cannot place an empty instance.
    #[error("Empty asset: cannot place instance with zero entities")]
    EmptyAsset,

    /// Instance not found in SceneDocument.instances.
    #[error("Instance not found: {0}")]
    InstanceNotFound(StableId),

    /// Generic error for tile operations and other cases.
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for CommandError {
    fn from(e: serde_json::Error) -> Self {
        CommandError::JsonError(e.to_string())
    }
}

/// Best-effort current time in Unix milliseconds.
/// Falls back to 0 if the system clock is unavailable (rare in WASM).
fn current_timestamp_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        // wasm-bindgen provides js_sys::Date::now() but we keep the core lib
        // portable. The default 0 is fine for tests; production callers can
        // set the timestamp explicitly.
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ComponentInstance;

    #[test]
    fn test_create_entity_serializes_with_type_tag() {
        let cmd = Command::CreateEntity {
            id: StableId::new("ent_01"),
            name: "Player".to_string(),
            components: vec![],
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"CreateEntity\""));
        assert!(json.contains("\"id\":\"ent_01\""));
        assert!(json.contains("\"name\":\"Player\""));
    }

    #[test]
    fn test_delete_entity_serializes_with_type_tag() {
        let cmd = Command::DeleteEntity {
            id: StableId::new("ent_01"),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"DeleteEntity\""));
        assert!(json.contains("\"id\":\"ent_01\""));
    }

    #[test]
    fn test_add_component_serializes() {
        let cmd = Command::AddComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            values: serde_json::json!({"translation": {"x": 0.0, "y": 0.0}}),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"AddComponent\""));
        assert!(json.contains("\"type_id\":\"editor.Transform2D\""));
        assert!(json.contains("\"translation\""));
    }

    #[test]
    fn test_set_component_field_serializes() {
        let cmd = Command::SetComponentField {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            field_path: "translation.x".to_string(),
            value: serde_json::json!(100.0),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"SetComponentField\""));
        assert!(json.contains("\"field_path\":\"translation.x\""));
    }

    #[test]
    fn test_reparent_entity_serializes_with_optional_parent() {
        let cmd = Command::ReparentEntity {
            entity_id: StableId::new("ent_01"),
            old_parent: Some(StableId::new("ent_root")),
            new_parent: None,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"ReparentEntity\""));
        // None for new_parent should be omitted (skip_serializing_if)
        assert!(!json.contains("\"new_parent\""));
        assert!(json.contains("\"old_parent\":\"ent_root\""));
    }

    #[test]
    fn test_reparent_entity_without_old_parent() {
        let cmd = Command::ReparentEntity {
            entity_id: StableId::new("ent_01"),
            old_parent: None,
            new_parent: Some(StableId::new("ent_root")),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        // Both None should be omitted
        assert!(!json.contains("old_parent"));
        assert!(json.contains("\"new_parent\":\"ent_root\""));
    }

    #[test]
    fn test_batch_serializes_with_label() {
        let cmd = Command::Batch {
            label: "drag-gesture".to_string(),
            commands: vec![
                Command::CreateEntity {
                    id: StableId::new("ent_01"),
                    name: "Foo".to_string(),
                    components: vec![],
                },
                Command::DeleteEntity {
                    id: StableId::new("ent_01"),
                },
            ],
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"Batch\""));
        assert!(json.contains("\"label\":\"drag-gesture\""));
        assert!(json.contains("CreateEntity"));
        assert!(json.contains("DeleteEntity"));
    }

    #[test]
    fn test_envelope_roundtrip() {
        let env = CommandEnvelope {
            command: Command::CreateEntity {
                id: StableId::new("ent_01"),
                name: "Player".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Name".to_string(),
                    values: serde_json::json!({"name": "Player"}),
                }],
            },
            metadata: CommandMetadata::now("user"),
        };
        let json = serde_json::to_string(&env).unwrap();
        let roundtripped: CommandEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(env, roundtripped);
    }

    #[test]
    fn test_metadata_roundtrip() {
        let m = CommandMetadata {
            authorship: "agent:test".to_string(),
            timestamp: 1234567890,
            rationale: Some("Move player to spawn".to_string()),
        };
        let json = serde_json::to_string(&m).unwrap();
        let roundtripped: CommandMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(m, roundtripped);
    }

    #[test]
    fn test_metadata_with_rationale_helper() {
        let m = CommandMetadata::now("user").with_rationale("test reason");
        assert_eq!(m.authorship, "user");
        assert_eq!(m.rationale, Some("test reason".to_string()));
    }

    #[test]
    fn test_command_deserialize_from_json() {
        let json = r#"{
            "type": "CreateEntity",
            "id": "ent_test",
            "name": "Spawn",
            "components": []
        }"#;
        let cmd: Command = serde_json::from_str(json).unwrap();
        match cmd {
            Command::CreateEntity { id, name, components } => {
                assert_eq!(id.as_str(), "ent_test");
                assert_eq!(name, "Spawn");
                assert!(components.is_empty());
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_command_error_display() {
        let err = CommandError::EntityNotFound(StableId::new("ent_missing"));
        let msg = err.to_string();
        assert!(msg.contains("ent_missing"));
    }

    #[test]
    fn test_upsert_override_serializes_with_type_tag() {
        let cmd = Command::UpsertOverride {
            instance_id: StableId::new("inst_1"),
            target_local_id: LocalId::new("root"),
            component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
            value: serde_json::json!("cannon.png"),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"UpsertOverride\""));
        assert!(json.contains("\"instance_id\":\"inst_1\""));
        assert!(json.contains("\"target_local_id\":\"root\""));
        assert!(json.contains("\"component_type_id\":\"editor.Sprite2D\""));
        assert!(json.contains("\"field_path\":[\"asset\"]"));
        assert!(json.contains("\"value\":\"cannon.png\""));
    }

    #[test]
    fn test_revert_override_serializes_with_type_tag() {
        let cmd = Command::RevertOverride {
            instance_id: StableId::new("inst_1"),
            target_local_id: LocalId::new("root"),
            component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"RevertOverride\""));
        assert!(json.contains("\"instance_id\":\"inst_1\""));
        assert!(json.contains("\"target_local_id\":\"root\""));
        assert!(json.contains("\"component_type_id\":\"editor.Sprite2D\""));
        assert!(json.contains("\"field_path\":[\"asset\"]"));
    }

    #[test]
    fn test_upsert_override_roundtrip() {
        let cmd = Command::UpsertOverride {
            instance_id: StableId::new("inst_1"),
            target_local_id: LocalId::new("root"),
            component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
            value: serde_json::json!("cannon.png"),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let roundtripped: Command = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, roundtripped);
    }

    #[test]
    fn test_revert_override_roundtrip() {
        let cmd = Command::RevertOverride {
            instance_id: StableId::new("inst_1"),
            target_local_id: LocalId::new("root"),
            component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string(), "x".to_string()],
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let roundtripped: Command = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, roundtripped);
    }
}
