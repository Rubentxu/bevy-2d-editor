//! Scene Asset Document — editor-owned durable authoring types per ADR-0005.
//! Level Layer types per docs/sddk/level-design-layers-research/design.md.

use serde::{Deserialize, Serialize};

use crate::component::ComponentInstance;
use crate::ids::SceneAssetLocalId;
use crate::scene_instance::SceneInstance;
use std::collections::BTreeMap;

/// Logical Project path (human-readable), e.g. "assets/characters/player".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetReference(pub String);

impl AssetReference {
    /// Construct a new AssetReference from a path string.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Borrow the inner path string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Soft validation policy, not a separate asset type (ADR-0005 §Roles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneAssetRole {
    /// A character, enemy, or other gameplay-active entity.
    Actor,
    /// A reusable structural piece (wall, floor tile, decoration).
    Fragment,
    /// A full screen or UI layout composition.
    Screen,
    /// A level layout container.
    Level,
    /// A UI composition.
    Ui,
    /// A visual effect (particles, post-processing).
    Effect,
    /// A logic graph asset (role per ADR-0005 §Logic).
    Logic,
}

/// Editor-owned durable authoring document for a Scene Asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetDocument {
    /// Stable identifier for this asset.
    pub asset_id: String,
    /// Logical project path.
    pub logical_path: String,
    /// Role discriminator for soft validation.
    pub role: SceneAssetRole,
    /// Monotonically increasing version number.
    pub version: u32,
    /// All entities defined in this asset.
    pub entities: Vec<SceneAssetEntity>,
    /// Typed relationships between entities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<SceneAssetRelationship>,
    /// Properties exposed for instance-level overriding.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exposed_properties: Vec<ExposedProperty>,
    /// Asset metadata (tags, timestamps, notes).
    #[serde(default)]
    pub metadata: SceneAssetMetadata,
    /// Level layers for this asset.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<LevelLayer>,
    /// Unknown JSON fields preserved for forward compatibility (ADR-0046 rule 2).
    #[serde(default, flatten)]
    pub extension_data: BTreeMap<String, serde_json::Value>,
}

/// One entity inside a Scene Asset.
///
/// Fidelity note (SDD-0046 S2 D4 + S4): unknown JSON fields are preserved in
/// `extension_data` (ADR-0046 rule 2 / SEM-3). The previous
/// `#[serde(deny_unknown_fields)]` violated the `JsonProjectAdapter::Lossless`
/// claim by rejecting documents that round-tripped through formats that added
/// fields; S2 removed the rejection, S4 upgrades drop → preserve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetEntity {
    /// Stable local identifier within this asset.
    pub local_id: SceneAssetLocalId,
    /// Hierarchical path within the asset (e.g. "root/door").
    pub local_path: String,
    /// Human-readable name.
    pub name: String,
    /// Components attached to this entity.
    pub components: Vec<ComponentInstance>,
    /// Unknown JSON fields preserved for forward compatibility (ADR-0046 rule 2).
    #[serde(default, flatten)]
    pub extension_data: BTreeMap<String, serde_json::Value>,
}

/// Typed relationship between entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipKind {
    /// Child relationship (parent contains child).
    #[serde(rename = "child")]
    Child,
    /// Custom named relationship with a string payload.
    #[serde(rename = "custom")]
    Custom(String),
}

/// Typed relationship between entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetRelationship {
    /// Source entity of the relationship.
    pub from_local_id: SceneAssetLocalId,
    /// Target entity of the relationship.
    pub to_local_id: SceneAssetLocalId,
    /// Kind of relationship.
    pub kind: RelationshipKind,
    /// Optional dot-separated field path within the target component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_path: Option<Vec<String>>,
}

/// A property the asset exposes for instance overriding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExposedProperty {
    /// Name of the exposed property.
    pub name: String,
    /// Local ID of the entity that owns this property.
    pub target_local_id: SceneAssetLocalId,
    /// Dot-separated path to the field within the target component.
    pub field_path: Vec<String>,
    /// Default value used when no override is set.
    pub default_value: serde_json::Value,
}

/// Spike-simple metadata: all `Option<String>` to avoid ISO-8601/tag parsing.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SceneAssetMetadata {
    /// Comma-separated tag string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    /// ISO-8601 creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO-8601 last-update timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Soft role-validation warning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleWarning {
    /// Machine-readable warning code.
    pub code: String,
    /// Human-readable warning message.
    pub message: String,
}

/// Soft role-validation warnings (NOT errors).
pub fn validate_role(role: SceneAssetRole, doc: &SceneAssetDocument) -> Vec<RoleWarning> {
    let mut warnings = Vec::new();

    match role {
        SceneAssetRole::Fragment => {
            let has_child_rel = doc
                .relationships
                .iter()
                .any(|r| matches!(r.kind, RelationshipKind::Child));
            if !has_child_rel {
                warnings.push(RoleWarning {
                    code: "fragment_standalone".to_string(),
                    message: "Fragment has no Child relationships; it may be intended as a standalone reusable piece without a natural hierarchy attachment point.".to_string(),
                });
            }
        }
        SceneAssetRole::Level => {
            if doc.entities.is_empty() {
                warnings.push(RoleWarning {
                    code: "level_empty".to_string(),
                    message:
                        "Level has zero entities; it may be an unfinished or placeholder document."
                            .to_string(),
                });
            }
        }
        SceneAssetRole::Ui => {
            let cross_role = matches!(
                doc.role,
                SceneAssetRole::Actor
                    | SceneAssetRole::Fragment
                    | SceneAssetRole::Screen
                    | SceneAssetRole::Level
                    | SceneAssetRole::Effect
            );
            if cross_role && !doc.relationships.is_empty() {
                warnings.push(RoleWarning {
                    code: "ui_cross_role".to_string(),
                    message: "Ui asset contains relationships to entities that may not be Ui role; cross-role hierarchies are untested.".to_string(),
                });
            }
        }
        _ => {}
    }

    warnings
}

// =============================================================================
// Level Layer types
// =============================================================================

use crate::ids::LayerId;

/// Soft-typed Scene Instance Layer category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneInstanceLayerKind {
    /// Entities that have gameplay behaviour (players, enemies, NPCs).
    Actors,
    /// Decorative or non-interactive elements.
    Props,
    /// Spawn points and checkpoint markers.
    Spawns,
    /// Trigger volumes and zone definitions.
    Triggers,
    /// Collision geometry.
    Collision,
    /// User-defined layer kind.
    Custom,
}

/// A Scene Instance Layer inside a Level Scene Asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneInstanceLayer {
    /// Unique identifier for this layer.
    pub id: LayerId,
    /// Human-readable layer name.
    pub name: String,
    /// Kind discriminator.
    #[serde(rename = "layer_kind")]
    pub kind: SceneInstanceLayerKind,
    /// Z-ordering index.
    pub order: i32,
    /// Scene Instances placed on this layer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<SceneInstance>,
}

/// A Level Layer of a Level Scene Asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LevelLayer {
    /// A layer holding placed Scene Instances.
    SceneInstance(SceneInstanceLayer),
    /// A layer holding a TileLayer.
    Tile(crate::tile_layer::TileLayer),
    /// A layer holding an AutoLayer.
    Auto(crate::auto_layer::AutoLayer),
    /// A layer holding an IntGridLayer.
    IntGrid(crate::int_grid::IntGridLayer),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logic_role_serializes_as_snake_case_and_round_trips() {
        let role = SceneAssetRole::Logic;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"logic\"");
        let parsed: SceneAssetRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SceneAssetRole::Logic);
    }

    #[test]
    fn scene_asset_document_with_logic_role_round_trips() {
        let doc = SceneAssetDocument {
            asset_id: "lga_jump".to_string(),
            logical_path: "logic/jump".to_string(),
            role: SceneAssetRole::Logic,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            layers: vec![],
            extension_data: BTreeMap::new(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: SceneAssetDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, SceneAssetRole::Logic);
        assert_eq!(parsed.asset_id, "lga_jump");
    }

    #[test]
    fn int_grid_layer_round_trips() {
        use crate::int_grid::{IntGridCell, IntGridLayer, IntGridLayerId, IntGridSchemaKind};

        let mut layer = IntGridLayer::new(
            IntGridLayerId::new("ig_collision".to_string()),
            "Collision".to_string(),
        )
        .with_identifier("Collision")
        .with_order(1)
        .with_dimensions(100, 80)
        .with_schema_kind(IntGridSchemaKind::Values);

        layer.paint_cell(0, 0, 1, Some("solid".to_string()));
        layer.paint_cell(1, 0, 0, None);
        layer.paint_cell(5, 10, 3, Some("water".to_string()));

        let level_layer = LevelLayer::IntGrid(layer.clone());
        let json = serde_json::to_string(&level_layer).unwrap();

        // Verify the JSON contains the expected kind tag
        assert!(json.contains("\"kind\":\"int_grid\""));

        let parsed: LevelLayer = serde_json::from_str(&json).unwrap();
        match parsed {
            LevelLayer::IntGrid(parsed_layer) => {
                assert_eq!(parsed_layer.id.as_str(), "ig_collision");
                assert_eq!(parsed_layer.name, "Collision");
                assert_eq!(parsed_layer.identifier.as_deref(), Some("Collision"));
                assert_eq!(parsed_layer.order, 1);
                assert_eq!(parsed_layer.grid_width, 100);
                assert_eq!(parsed_layer.grid_height, 80);
                assert_eq!(parsed_layer.schema_kind, IntGridSchemaKind::Values);
                assert_eq!(parsed_layer.cell_count(), 3);

                let cell = parsed_layer.get_cell(5, 10).unwrap();
                assert_eq!(cell.value, 3);
                assert_eq!(cell.identifier.as_deref(), Some("water"));
            }
            other => panic!("Expected LevelLayer::IntGrid, got {:?}", other),
        }
    }

    #[test]
    fn int_grid_layer_tile_ref_round_trips() {
        use crate::int_grid::{IntGridLayer, IntGridLayerId, IntGridSchemaKind};

        let mut layer = IntGridLayer::with_tile_ref(
            IntGridLayerId::new("ig_tiles".to_string()),
            "Tile References".to_string(),
        )
        .with_order(0);

        layer.paint_cell(0, 0, 5, None); // tile index 5
        layer.paint_cell(10, 20, 12, None); // tile index 12

        let level_layer = LevelLayer::IntGrid(layer);
        let json = serde_json::to_string(&level_layer).unwrap();
        let parsed: LevelLayer = serde_json::from_str(&json).unwrap();

        match parsed {
            LevelLayer::IntGrid(parsed_layer) => {
                assert_eq!(parsed_layer.schema_kind, IntGridSchemaKind::TileRef);
                assert_eq!(parsed_layer.cell_count(), 2);
                assert_eq!(parsed_layer.get_cell(0, 0).unwrap().value, 5);
                assert_eq!(parsed_layer.get_cell(10, 20).unwrap().value, 12);
            }
            other => panic!("Expected LevelLayer::IntGrid, got {:?}", other),
        }
    }

    // ── SDD-0046 S2 D4 tests ────────────────────────────────────────────────

    /// Spec §sem2-scene-asset-entity scenario 12: extra fields parse and drop.
    #[test]
    fn scene_asset_entity_extra_field_parses() {
        let json = r#"{
            "local_id": "e1",
            "local_path": "root",
            "name": "Hero",
            "components": [],
            "unknown_field": 42
        }"#;
        let parsed: SceneAssetEntity =
            serde_json::from_str(json).expect("SceneAssetEntity must tolerate unknown fields (D4)");
        assert_eq!(parsed.local_path, "root");
        assert_eq!(parsed.name, "Hero");
    }

    /// Spec §sem2-scene-asset-entity scenario 13: a full SceneAssetDocument
    /// containing an entity with an extra field still parses.
    #[test]
    fn old_project_json_with_extra_fields_still_parses() {
        let json = r#"{
            "asset_id": "hero",
            "logical_path": "actors/hero",
            "role": "actor",
            "version": 1,
            "entities": [
                {
                    "local_id": "e1",
                    "local_path": "root",
                    "name": "Hero",
                    "components": [],
                    "future_field": {"a": 1}
                }
            ],
            "relationships": [],
            "metadata": {},
            "layers": [],
            "future_top_level": true
        }"#;
        let parsed: SceneAssetDocument = serde_json::from_str(json)
            .expect("SceneAssetDocument must tolerate unknown fields (D4)");
        assert_eq!(parsed.asset_id, "hero");
        assert_eq!(parsed.entities.len(), 1);
        assert_eq!(parsed.entities[0].name, "Hero");
    }
}
