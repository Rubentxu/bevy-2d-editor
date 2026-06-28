//! Scene Asset Document — editor-owned durable authoring types per ADR-0005.
//! See docs/adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md.

use serde::{Deserialize, Serialize};

use crate::document::ComponentInstance;

/// Opaque stable identity of an entity *inside* a Scene Asset.
/// Never appears as a SceneDocument StableId. Overrides target this.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocalId(pub String);

impl LocalId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Logical Project path (human-readable), e.g. "assets/characters/player".
/// Transparent so it serializes as a plain string.
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
}

/// One entity inside a Scene Asset. Reuses existing ComponentInstance.
/// NOTE: NO children_local_ids — hierarchy lives only in relationships (spec S9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetEntity {
    pub local_id: LocalId,
    pub local_path: String,
    pub name: String,
    pub components: Vec<ComponentInstance>,
}

/// Typed relationship between entities. Mirrors BSN's typed relationship lists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipKind {
    Child,
    #[serde(rename = "custom")]
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetRelationship {
    pub from_local_id: LocalId,
    pub to_local_id: LocalId,
    pub kind: RelationshipKind,
    /// Property-level link if the relationship targets a nested field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_path: Option<Vec<String>>,
}

/// A property the asset exposes for instance overriding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExposedProperty {
    pub name: String,
    pub target_local_id: LocalId,
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

/// Soft role-validation warnings (NOT errors). Spike rules:
/// - Fragment: warn if it has no Child relationships pointing away from it
///   (i.e., it appears usable as a top-level standalone with no children).
/// - Level: warn if the document contains zero entities.
/// - Ui: warn if any relationship targets a non-Ui role entity (cross-role warning).
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
