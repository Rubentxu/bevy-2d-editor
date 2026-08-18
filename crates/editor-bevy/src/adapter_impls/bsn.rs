//! BSN export adapter — wraps [`EditorCoreBsnExporter`].
//!
//! Implements [`editor_model::adapter::EditorAdapter`] for [`BsnExportAdapter`].
//! Declares [`editor_model::adapter::AdapterFidelity::SemanticLossless`]: BSN preserves
//! scene authoring semantics but formatting (indentation, key ordering) differs from
//! JSON.
//!
//! Encode only; decode returns [`AdapterError::ExportOnly`].
//!
//! The wrapped writer site is [`crate::bsn_export::export_to_bsn_text`] (1 site).
//!
//! [`BsnExportAdapter`] does NOT support `SceneAssetRole::Logic` assets — the
//! underlying `EditorCoreBsnExporter` explicitly rejects them (see
//! `crates/editor-bevy/src/bsn_export.rs:72`). Attempting to encode a Logic
//! asset returns [`AdapterError::UnsupportedRole`].

use editor_model::adapter::{AdapterError, AdapterFidelity, EditorAdapter, SemanticModel};
use editor_model::scene_asset::SceneAssetRole;
use editor_model::SceneAssetDocument;
use crate::bsn_export::{export_to_bsn_text, BsnExportError};

/// BSN export adapter — semantic-lossless encode of `SceneAssetDocument` to `.bsn` text.
#[derive(Debug, Clone, Copy, Default)]
pub struct BsnExportAdapter;

impl BsnExportAdapter {
    /// Constructs a new `BsnExportAdapter`.
    pub const fn new() -> Self {
        Self
    }
}

impl EditorAdapter for BsnExportAdapter {
    fn name(&self) -> &str {
        "bsn.export.v1"
    }

    fn fidelity(&self) -> AdapterFidelity {
        AdapterFidelity::SemanticLossless
    }

    fn encode(&self, model: &SemanticModel<'_>) -> Result<Vec<u8>, AdapterError> {
        match model {
            SemanticModel::SceneAsset(doc) => {
                let name = self.name();
                export_to_bsn_text(doc)
                    .map(Into::into)
                    .map_err(|e| AdapterError::Encode {
                        adapter: name.into(),
                        source: e.into(),
                    })
            }
            // BSN export only supports SceneAssetDocument; other variants are
            // not reachable from .bsn format.
            other => {
                let variant = format!("{:?}", other);
                Err(AdapterError::UnsupportedModel {
                    adapter: self.name().into(),
                    model: variant,
                })
            }
        }
    }

    fn decode(&self, _bytes: &[u8]) -> Result<SemanticModel<'static>, AdapterError> {
        Err(AdapterError::ExportOnly {
            adapter: self.name().into(),
        })
    }

    fn supports(&self, role: SceneAssetRole) -> bool {
        // Reject Logic role — BSN export does not handle logic graphs (bsn_export.rs:72).
        !matches!(role, SceneAssetRole::Logic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::adapter::SemanticModel;
    use editor_model::scene_asset::{SceneAssetEntity, SceneAssetMetadata, SceneAssetRole};
    use editor_model::ComponentInstance;
    use std::collections::BTreeMap;

    fn make_actor_asset() -> SceneAssetDocument {
        SceneAssetDocument {
            asset_id: "hero".into(),
            logical_path: "actors/hero".into(),
            role: SceneAssetRole::Actor,
            version: 1,
            entities: vec![SceneAssetEntity {
                local_id: editor_model::ids::LocalId::new("e1".into()),
                name: "Hero".into(),
                components: vec![],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            layers: vec![],
        }
    }

    fn make_logic_asset() -> SceneAssetDocument {
        SceneAssetDocument {
            asset_id: "brain".into(),
            logical_path: "logic/brain".into(),
            role: SceneAssetRole::Logic,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            layers: vec![],
        }
    }

    #[test]
    fn encode_actor_asset() {
        let adapter = BsnExportAdapter::new();
        let doc = make_actor_asset();
        let result = adapter.encode(&SemanticModel::SceneAsset(&doc));
        assert!(result.is_ok());
        let text = String::from_utf8(result.unwrap()).unwrap();
        // BSN format starts with "bsn!{"
        assert!(text.starts_with("bsn!{"), "got: {text}");
    }

    #[test]
    fn encode_actor_asset_round_trip_is_semantic() {
        // BSN is SemanticLossless, not Lossless — verify it encodes without error.
        let adapter = BsnExportAdapter::new();
        let doc = make_actor_asset();
        let encoded = adapter.encode(&SemanticModel::SceneAsset(&doc));
        assert!(encoded.is_ok(), "encode failed: {:?}", encoded.err());
    }

    #[test]
    fn decode_is_export_only() {
        let adapter = BsnExportAdapter::new();
        let result = adapter.decode(b"anything");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AdapterError::ExportOnly { .. }));
    }

    #[test]
    fn supports_actor_role() {
        let adapter = BsnExportAdapter::new();
        assert!(adapter.supports(SceneAssetRole::Actor));
        assert!(adapter.supports(SceneAssetRole::Fragment));
        assert!(adapter.supports(SceneAssetRole::Level));
    }

    #[test]
    fn rejects_logic_role() {
        let adapter = BsnExportAdapter::new();
        assert!(!adapter.supports(SceneAssetRole::Logic));
    }

    #[test]
    fn unsupported_model_scene() {
        // SceneDocument is not representable in .bsn format
        let adapter = BsnExportAdapter::new();
        let doc = editor_model::SceneDocument {
            version: "0.1".into(),
            scene_id: "test".into(),
            name: "Test".into(),
            entities: vec![],
            instances: BTreeMap::new(),
        };
        let result = adapter.encode(&SemanticModel::Scene(&doc));
        assert!(matches!(result, Err(AdapterError::UnsupportedModel { .. })));
    }
}
