//! Scene Asset Document — editor-owned durable authoring types per ADR-0005.
//!
//! PR2 refactoring: this module is now a thin re-export wrapper.
//! All pure types live in `editor_model::scene_asset`.

// T-02-14 LocalId collapse: canonical LocalId moved to editor_model::ids::LocalId.
// scene_asset::LocalId was the "asset-local" variant ( SceneAssetLocalId).
// Replaced with deprecated alias to prevent divergence.
#[deprecated(since = "0.87.0", note = "Use editor_model::ids::SceneAssetLocalId instead")]
pub type LocalId = editor_model::ids::SceneAssetLocalId;

pub use editor_model::scene_asset::{
    AssetReference, ExposedProperty, LevelLayer, RelationshipKind, RoleWarning,
    SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata, SceneAssetRelationship,
    SceneAssetRole, validate_role,
};

// SceneInstanceLayerKind is not in editor_model (editor_model uses inline enum in LevelLayer).
// Keep the local definition for backward compatibility.
pub use editor_model::scene_asset::SceneInstanceLayerKind as SceneInstanceLayerKind;

// LayerId is now in editor_model::ids. Add deprecated alias for backward compat.
#[deprecated(since = "0.87.0", note = "Use editor_model::ids::LayerId instead")]
pub type LayerId = editor_model::ids::LayerId;

// SceneInstanceLayer and TileLayerId are in editor_model but with different structures.
// Provide local definitions as aliases for the editor-core-specific variants.
pub use editor_model::scene_asset::SceneInstanceLayer as SceneInstanceLayer;

#[cfg(test)]
mod tests {
    // Local tests for scene_asset wrapper.
}
