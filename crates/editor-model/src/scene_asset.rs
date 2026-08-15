//! Scene Asset Document — editor-owned durable authoring types per ADR-0005.
//! Level Layer types per docs/sddk/level-design-layers-research/design.md.

use serde::{Deserialize, Serialize};

use crate::component::ComponentInstance;
use crate::ids::SceneAssetLocalId;
use crate::scene_instance::SceneInstance;

/// Logical Project path (human-readable), e.g. "assets/characters/player".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetReference(pub String);

impl AssetReference {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Soft validation policy, not a separate asset type (ADR-0005 §Roles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneAssetRole {
    Actor,
    Fragment,
    Screen,
    Level,
    Ui,
    Effect,
    Logic,
}

/// Editor-owned durable authoring document for a Scene Asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetDocument {
    pub asset_id: String,
    pub logical_path: String,
    pub role: SceneAssetRole,
    pub version: u32,
    pub entities: Vec<SceneAssetEntity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<SceneAssetRelationship>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exposed_properties: Vec<ExposedProperty>,
    #[serde(default)]
    pub metadata: SceneAssetMetadata,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<LevelLayer>,
}

/// One entity inside a Scene Asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneAssetEntity {
    pub local_id: SceneAssetLocalId,
    pub local_path: String,
    pub name: String,
    pub components: Vec<ComponentInstance>,
}

/// Typed relationship between entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipKind {
    #[serde(rename = "child")]
    Child,
    #[serde(rename = "custom")]
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetRelationship {
    pub from_local_id: SceneAssetLocalId,
    pub to_local_id: SceneAssetLocalId,
    pub kind: RelationshipKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_path: Option<Vec<String>>,
}

/// A property the asset exposes for instance overriding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExposedProperty {
    pub name: String,
    pub target_local_id: SceneAssetLocalId,
    pub field_path: Vec<String>,
    pub default_value: serde_json::Value,
}

/// Spike-simple metadata: all Option<String> to avoid ISO-8601/tag parsing.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SceneAssetMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleWarning {
    pub code: String,
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
    Actors,
    Props,
    Spawns,
    Triggers,
    Collision,
    Custom,
}

/// A Scene Instance Layer inside a Level Scene Asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneInstanceLayer {
    pub id: LayerId,
    pub name: String,
    #[serde(rename = "layer_kind")]
    pub kind: SceneInstanceLayerKind,
    pub order: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<SceneInstance>,
}

/// A Level Layer of a Level Scene Asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LevelLayer {
    SceneInstance(SceneInstanceLayer),
    Tile(crate::tile_layer::TileLayer),
    Auto(crate::auto_layer::AutoLayer),
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
        };
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: SceneAssetDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, SceneAssetRole::Logic);
        assert_eq!(parsed.asset_id, "lga_jump");
    }
}
