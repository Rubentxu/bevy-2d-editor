//! SEM-5 migration framework (SDD-0046 S3).
//!
//! Every editor document type declares a format version and exposes a typed,
//! pure migration function that upgrades old documents to the current version.
//!
//! # SEM-5 (from `docs/specs/semantic-editor-model.md`)
//!
//! > Every document type declares a format version. Migrations are pure where
//! > practical: `fn migrate(input: Vn) -> Result<VnPlus1, MigrationError>`
//!
//! # ADR-0046 rule 4
//!
//! > Migrations operate on semantic versions and typed structures, not
//! > arbitrary string replacements.
//!
//! This module therefore exposes typed `migrate::<type>(version, &mut T)`
//! functions. Each function is a no-op when the document is already at
//! `CURRENT_VERSION` and fails loudly (`UnsupportedVersion`) for documents
//! NEWER than the editor understands — a future document must never be
//! silently corrupted by an older editor.
//!
//! # Current versions (v0.98.0)
//!
//! All five core document types are at version 1. The V0→V1 steps only
//! materialize fields that serde already defaults — they make the version
//! discipline explicit and give future V2+ steps a defined upgrade path.

use crate::adapter::AdapterError;
use crate::{LogicGraphAsset, ProjectMetadata, SceneAssetDocument, SceneDocument, WorldDocument};
use std::error::Error;

// ─────────────────────────────────────────────────────────────────────────────
// Version constants
// ─────────────────────────────────────────────────────────────────────────────

/// Current format version of [`SceneDocument`].
pub const SCENE_DOCUMENT_VERSION: u32 = 1;
/// Current format version of [`SceneAssetDocument`].
pub const SCENE_ASSET_DOCUMENT_VERSION: u32 = 1;
/// Current format version of [`WorldDocument`].
pub const WORLD_DOCUMENT_VERSION: u32 = 1;
/// Current format version of [`LogicGraphAsset`].
pub const LOGIC_GRAPH_ASSET_VERSION: u32 = 1;
/// Current format version of [`ProjectMetadata`].
pub const PROJECT_METADATA_VERSION: u32 = 1;

// ─────────────────────────────────────────────────────────────────────────────
// MigrationError
// ─────────────────────────────────────────────────────────────────────────────

/// Errors raised by the SEM-5 migration framework.
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    /// The document's version is newer than the editor supports.
    ///
    /// Opening a future-format document in an older editor must fail loudly,
    /// never silently corrupt the data.
    #[error(
        "document '{type_name}' has unsupported version {version} (this editor supports up to v{max})"
    )]
    UnsupportedVersion {
        /// Document type name for diagnostics.
        type_name: &'static str,
        /// Version found in the document.
        version: u32,
        /// Highest version this editor understands.
        max: u32,
    },

    /// A migration step failed to apply.
    #[error("migration of '{type_name}' from v{from} to v{to} failed: {source}")]
    MigrationFailed {
        /// Document type name for diagnostics.
        type_name: &'static str,
        /// Version being migrated from.
        from: u32,
        /// Version being migrated to.
        to: u32,
        /// Underlying failure.
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}

impl From<MigrationError> for AdapterError {
    fn from(err: MigrationError) -> Self {
        AdapterError::Decode {
            adapter: "json.project.v1".into(),
            source: Box::new(err),
        }
    }
}

/// Parse a string-versioned document's version field into the internal u32
/// representation.
///
/// The string format is `"<major>.<minor>"`; the MAJOR component is the
/// migration-relevant version (`"0.1" → 0`, `"99.0" → 99`). Non-numeric
/// strings are a migration failure.
pub fn parse_version_string(type_name: &'static str, version: &str) -> Result<u32, MigrationError> {
    let major = version.split('.').next().unwrap_or(version);
    match major.parse::<u32>() {
        Ok(n) => Ok(n),
        Err(_) => Err(MigrationError::MigrationFailed {
            type_name,
            from: 0,
            to: 1,
            source: format!("unrecognised version string '{version}'").into(),
        }),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-type migration functions
// ─────────────────────────────────────────────────────────────────────────────

/// Typed migration functions, one per core document type.
///
/// Each function takes the document's declared version and a mutable document
/// reference, upgrades the document in place to [`CURRENT_VERSION`] and
/// returns `Ok(())`. A version equal to the current constant is a no-op.
pub mod migrate {
    use super::*;
    use std::collections::BTreeMap;

    /// Migrate a [`SceneDocument`] to [`SCENE_DOCUMENT_VERSION`].
    ///
    /// V0 → V1: ensure the `instances` map exists (older documents parsed via
    /// serde defaults already materialize it; this makes the step explicit).
    pub fn scene_document(version: u32, doc: &mut SceneDocument) -> Result<(), MigrationError> {
        match version {
            SCENE_DOCUMENT_VERSION => Ok(()),
            0 => {
                if doc.instances.is_empty() {
                    doc.instances = BTreeMap::new();
                }
                Ok(())
            }
            other => Err(MigrationError::UnsupportedVersion {
                type_name: "SceneDocument",
                version: other,
                max: SCENE_DOCUMENT_VERSION,
            }),
        }
    }

    /// Migrate a [`SceneAssetDocument`] to [`SCENE_ASSET_DOCUMENT_VERSION`].
    ///
    /// V0 → V1: no-op — every field added since v0 is `#[serde(default)]` and
    /// deserialization already materializes defaults.
    pub fn scene_asset_document(
        version: u32,
        _doc: &mut SceneAssetDocument,
    ) -> Result<(), MigrationError> {
        match version {
            SCENE_ASSET_DOCUMENT_VERSION => Ok(()),
            0 => Ok(()),
            other => Err(MigrationError::UnsupportedVersion {
                type_name: "SceneAssetDocument",
                version: other,
                max: SCENE_ASSET_DOCUMENT_VERSION,
            }),
        }
    }

    /// Migrate a [`WorldDocument`] to [`WORLD_DOCUMENT_VERSION`].
    ///
    /// V0 → V1: no-op — the type shipped at v1.
    pub fn world_document(version: u32, _doc: &mut WorldDocument) -> Result<(), MigrationError> {
        match version {
            WORLD_DOCUMENT_VERSION => Ok(()),
            0 => Ok(()),
            other => Err(MigrationError::UnsupportedVersion {
                type_name: "WorldDocument",
                version: other,
                max: WORLD_DOCUMENT_VERSION,
            }),
        }
    }

    /// Migrate a [`LogicGraphAsset`] to [`LOGIC_GRAPH_ASSET_VERSION`].
    ///
    /// V0 → V1: no-op — the type shipped at v1.
    pub fn logic_graph_asset(
        version: u32,
        _doc: &mut LogicGraphAsset,
    ) -> Result<(), MigrationError> {
        match version {
            LOGIC_GRAPH_ASSET_VERSION => Ok(()),
            0 => Ok(()),
            other => Err(MigrationError::UnsupportedVersion {
                type_name: "LogicGraphAsset",
                version: other,
                max: LOGIC_GRAPH_ASSET_VERSION,
            }),
        }
    }

    /// Migrate a [`ProjectMetadata`] to [`PROJECT_METADATA_VERSION`].
    ///
    /// V0 → V1: ensure the world-catalog fields exist (`worlds`,
    /// `active_world`). These were added in v0.95.0 (World Workspace) behind
    /// `#[serde(default)]`; this step makes the upgrade explicit.
    pub fn project_metadata(version: u32, doc: &mut ProjectMetadata) -> Result<(), MigrationError> {
        match version {
            PROJECT_METADATA_VERSION => Ok(()),
            0 => {
                // Materialize defaults for the ADR-0037 fields (v0.95.0).
                // serde already does this on parse; the step documents intent.
                if doc.worlds.is_empty() {
                    doc.worlds = Vec::new();
                }
                if doc.active_world.is_none() {
                    doc.active_world = None;
                }
                Ok(())
            }
            other => Err(MigrationError::UnsupportedVersion {
                type_name: "ProjectMetadata",
                version: other,
                max: PROJECT_METADATA_VERSION,
            }),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_asset::{SceneAssetMetadata, SceneAssetRole};

    /// Spec §sem3-version-constants scenario 5: all constants are 1.
    #[test]
    fn current_version_constants_are_one() {
        assert_eq!(SCENE_DOCUMENT_VERSION, 1);
        assert_eq!(SCENE_ASSET_DOCUMENT_VERSION, 1);
        assert_eq!(WORLD_DOCUMENT_VERSION, 1);
        assert_eq!(LOGIC_GRAPH_ASSET_VERSION, 1);
        assert_eq!(PROJECT_METADATA_VERSION, 1);
    }

    /// Spec §sem3-version-constants scenario 3: "0.1" parses to 0.
    #[test]
    fn parse_version_string_ok() {
        assert_eq!(parse_version_string("SceneDocument", "0.1").unwrap(), 0);
    }

    /// Spec §sem3-version-constants scenario 4: bogus string is a migration error.
    #[test]
    fn parse_version_string_bogus() {
        let err = parse_version_string("SceneDocument", "bogus").unwrap_err();
        assert!(matches!(err, MigrationError::MigrationFailed { .. }));
    }

    /// Spec §sem3-migrate-functions scenario 6: current version is a no-op.
    #[test]
    fn migrate_current_version_is_noop() {
        let mut doc = SceneDocument {
            version: "0.1".into(),
            scene_id: "s1".into(),
            name: "N".into(),
            entities: vec![],
            instances: Default::default(),
        };
        let before = doc.clone();
        migrate::scene_document(SCENE_DOCUMENT_VERSION, &mut doc).unwrap();
        assert_eq!(doc, before);
    }

    /// Spec §sem3-migrate-functions scenario 9: future version rejected.
    #[test]
    fn future_version_rejected() {
        let mut doc = SceneDocument {
            version: "99.0".into(),
            scene_id: "s1".into(),
            name: "N".into(),
            entities: vec![],
            instances: Default::default(),
        };
        let err = migrate::scene_document(999, &mut doc).unwrap_err();
        assert!(matches!(
            err,
            MigrationError::UnsupportedVersion { version: 999, .. }
        ));
    }

    /// Spec §sem3-migrate-functions scenario 7: V0 ProjectMetadata gains worlds.
    #[test]
    fn migrate_v0_project_metadata_materializes_worlds() {
        let mut pm = ProjectMetadata {
            version: "0.1".into(),
            name: "p".into(),
            scenes: vec!["s1".into()],
            schemas: vec![],
            active_scene: None,
            scene_assets: vec![],
            worlds: vec![],
            active_world: None,
        };
        migrate::project_metadata(0, &mut pm).unwrap();
        assert!(pm.worlds.is_empty());
        assert!(pm.active_world.is_none());
        assert_eq!(pm.scenes, vec!["s1"]);
    }

    /// Spec §sem3-migrate-functions scenario 8: V0 SceneDocument gains instances.
    #[test]
    fn migrate_v0_scene_document_materializes_instances() {
        let mut doc = SceneDocument {
            version: "0.1".into(),
            scene_id: "s1".into(),
            name: "N".into(),
            entities: vec![],
            instances: Default::default(),
        };
        migrate::scene_document(0, &mut doc).unwrap();
        assert!(doc.instances.is_empty());
        assert_eq!(doc.scene_id, "s1");
    }

    /// Spec §sem3-migration-error scenario 2: converts into AdapterError::Decode.
    #[test]
    fn migration_error_converts_to_adapter_decode() {
        let err = MigrationError::UnsupportedVersion {
            type_name: "SceneDocument",
            version: 999,
            max: 1,
        };
        let adapter_err: AdapterError = err.into();
        match adapter_err {
            AdapterError::Decode { adapter, source } => {
                assert_eq!(adapter, "json.project.v1");
                assert!(source.to_string().contains("SceneDocument"));
            }
            other => panic!("expected Decode, got {other:?}"),
        }
    }

    /// SceneAssetDocument migrate at current and V0 both no-op.
    #[test]
    fn scene_asset_migrate_noop_both_versions() {
        let mut doc = crate::SceneAssetDocument {
            asset_id: "a".into(),
            logical_path: "actors/a".into(),
            role: SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            layers: vec![],
        };
        migrate::scene_asset_document(1, &mut doc).unwrap();
        migrate::scene_asset_document(0, &mut doc).unwrap();
        assert_eq!(doc.asset_id, "a");
    }
}
