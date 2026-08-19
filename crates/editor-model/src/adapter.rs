//! Editor Adapter trait and runtime types for semantic model encoding/decoding.
//!
//! Adapters wrap existing writer/parser sites and expose a uniform `EditorAdapter`
//! interface. Every adapter declares its [`AdapterFidelity`] so callers can reason
//! about round-trip guarantees.
//!
//! # Architecture
//!
//! The trait lives in `editor-model` (ADR-0030: bevy-free, wasm-free). Implementations
//! live in the consumer crate. The registry (`OnceLock`, `init_registry`, `all_adapters`)
//! has been moved to `editor-bevy` (ARCH-030) — `editor-model` owns only the trait.
//!
//! # Fidelity levels
//!
//! - [`AdapterFidelity::Lossless`] — `encode` + `decode` is byte-exact.
//! - [`AdapterFidelity::SemanticLossless`] — semantics survive; formatting may differ.
//! - [`AdapterFidelity::ExportOnlyLossy`] — `encode` supported; `decode` returns
//!   [`AdapterError::ExportOnly`].

use crate::scene_asset::{SceneAssetDocument, SceneAssetRole};
use crate::{LogicGraphAsset, ProjectMetadata, SceneDocument, WorldDocument};
use std::error::Error;

/// The semantic model variants that adapters can encode/decode.
///
/// The enum owns its data directly (no borrow, no `Box` indirection) so
/// `decode` never needs to leak memory to produce a `'static` value
/// (SDD-0046 S2: D1).
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticModel {
    /// A scene document (the root editing document).
    Scene(SceneDocument),
    /// A reusable scene asset.
    SceneAsset(SceneAssetDocument),
    /// A visual logic graph asset.
    LogicGraph(LogicGraphAsset),
    /// A world document.
    World(WorldDocument),
    /// Project-level metadata.
    ProjectMetadata(ProjectMetadata),
}

/// Runtime fidelity declaration for an [`EditorAdapter`].
///
/// Each variant carries a `description` that is spec-defined and immutable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterFidelity {
    /// Byte-exact round-trip: `encode` then `decode` recovers the original exactly.
    Lossless,
    /// Semantic round-trip: all editor-model fields survive but formatting and
    /// field ordering may change.
    SemanticLossless,
    /// Encode-only: `decode` is not supported and returns [`AdapterError::ExportOnly`].
    ExportOnlyLossy,
}

impl AdapterFidelity {
    /// Returns the spec-defined static description for this fidelity level.
    ///
    /// # Examples
    ///
    /// ```
    /// use editor_model::adapter::AdapterFidelity;
    /// assert_eq!(AdapterFidelity::Lossless.description(), "encode+decode round-trip exact; no data loss");
    /// assert_eq!(AdapterFidelity::SemanticLossless.description(), "encode+decode preserves semantics; formatting may differ");
    /// assert_eq!(AdapterFidelity::ExportOnlyLossy.description(), "encode only; decode is not supported");
    /// ```
    pub const fn description(self) -> &'static str {
        match self {
            Self::Lossless => "encode+decode round-trip exact; no data loss",
            Self::SemanticLossless => "encode+decode preserves semantics; formatting may differ",
            Self::ExportOnlyLossy => "encode only; decode is not supported",
        }
    }
}

/// Errors returned by [`EditorAdapter`] encode/decode operations.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    /// Adapter failed to encode the given model.
    #[error("adapter '{adapter}' failed to encode: {source}")]
    Encode {
        /// Name of the adapter that failed.
        adapter: String,
        /// Underlying error source.
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    /// Adapter failed to decode the given bytes.
    #[error("adapter '{adapter}' failed to decode: {source}")]
    Decode {
        /// Name of the adapter that failed.
        adapter: String,
        /// Underlying error source.
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    /// Adapter is export-only; decode is not supported.
    #[error("adapter '{adapter}' is export-only; decode is not supported")]
    ExportOnly {
        /// Name of the export-only adapter.
        adapter: String,
    },
    /// Adapter does not support the given model variant.
    #[error("adapter '{adapter}' does not support model variant '{model}'")]
    UnsupportedModel {
        /// Name of the adapter that rejected the model.
        adapter: String,
        /// The model variant name that was unsupported.
        model: String,
    },
    /// Adapter does not support the given scene asset role.
    #[error("adapter '{adapter}' does not support role '{role:?}'")]
    UnsupportedRole {
        /// Name of the adapter that rejected the role.
        adapter: String,
        /// The scene asset role that was unsupported.
        role: SceneAssetRole,
    },
}

/// Object-safe trait for encoding/decoding semantic editor model variants.
///
/// Implementors wrap existing writer sites (JSON, BSN, Bevy projections) and
/// declare their [`fidelity`](EditorAdapter::fidelity) honestly.
///
/// # Design notes
///
/// - `encode` takes `&SemanticModel` (borrowed); the owned model is cheap to
///   clone (all variants `Clone`) and encoders serialize by reference.
/// - `decode` returns an owned `SemanticModel` — no lifetime trick, no
///   `Box::leak` (SDD-0046 S2: D1).
/// - `Send + Sync` bounds allow adapters to be stored in the global registry.
pub trait EditorAdapter: Send + Sync {
    /// Returns a non-empty, stable identifier for this adapter.
    fn name(&self) -> &str;

    /// Returns this adapter's declared fidelity level.
    fn fidelity(&self) -> AdapterFidelity;

    /// Encode a semantic model variant to a byte vector.
    fn encode(&self, model: &SemanticModel) -> Result<Vec<u8>, AdapterError>;

    /// Decode a byte vector back to an owned semantic model variant.
    fn decode(&self, bytes: &[u8]) -> Result<SemanticModel, AdapterError>;

    /// Returns `true` if this adapter supports the given scene asset role.
    ///
    /// Default implementation returns `true` (adapters that handle all roles).
    fn supports(&self, _role: SceneAssetRole) -> bool {
        true
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn fidelity_lossless_description() {
        assert_eq!(
            AdapterFidelity::Lossless.description(),
            "encode+decode round-trip exact; no data loss"
        );
    }

    #[test]
    fn fidelity_semantic_lossless_description() {
        assert_eq!(
            AdapterFidelity::SemanticLossless.description(),
            "encode+decode preserves semantics; formatting may differ"
        );
    }

    #[test]
    fn fidelity_export_only_lossy_description() {
        assert_eq!(
            AdapterFidelity::ExportOnlyLossy.description(),
            "encode only; decode is not supported"
        );
    }

    #[test]
    fn adapter_error_encode_includes_name() {
        let err = AdapterError::Encode {
            adapter: "test-adapter".into(),
            source: "oops".into(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("test-adapter"),
            "expected adapter name in error: {msg}"
        );
    }

    #[test]
    fn adapter_error_decode_includes_name() {
        let err = AdapterError::Decode {
            adapter: "test-adapter".into(),
            source: "oops".into(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("test-adapter"),
            "expected adapter name in error: {msg}"
        );
    }

    #[test]
    fn adapter_error_export_only_includes_name() {
        let err = AdapterError::ExportOnly {
            adapter: "test-adapter".into(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("test-adapter"),
            "expected adapter name in error: {msg}"
        );
        assert!(
            msg.contains("export-only"),
            "expected export-only text: {msg}"
        );
    }

    #[test]
    fn adapter_error_unsupported_model_includes_name_and_variant() {
        let err = AdapterError::UnsupportedModel {
            adapter: "json.scene.v1".into(),
            model: "LogicGraph".into(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("json.scene.v1"),
            "expected adapter name: {msg}"
        );
        assert!(msg.contains("LogicGraph"), "expected model variant: {msg}");
        // Exact wording from spec §sem-adapter-errors scenario 2
        assert_eq!(
            msg.as_str(),
            "adapter 'json.scene.v1' does not support model variant 'LogicGraph'"
        );
    }

    #[test]
    fn adapter_error_unsupported_role_includes_name_and_role() {
        let err = AdapterError::UnsupportedRole {
            adapter: "bsn-export".into(),
            role: SceneAssetRole::Logic,
        };
        let msg = err.to_string();
        assert!(msg.contains("bsn-export"), "expected adapter name: {msg}");
        assert!(msg.contains("Logic"), "expected role: {msg}");
    }

    // ── SDD-0046 S2 tests (PR1) ────────────────────────────────────────────

    /// Spec §sem2-owned-model scenario 1: no `Box` indirection.
    ///
    /// An owned enum stores data inline; a `Box`-based enum would be
    /// pointer-sized (~16-24 bytes). `SceneDocument` with entities + instances
    /// cannot fit in 32 bytes, so the enum must hold data inline.
    #[test]
    fn semantic_model_has_no_box_indirection() {
        let size = size_of::<SemanticModel>();
        assert!(
            size > 32,
            "SemanticModel should hold data inline, got {size} bytes"
        );
    }

    /// Spec §sem2-owned-model scenario 3: clone is a deep copy.
    #[test]
    fn clone_is_deep_copy() {
        let doc = SceneDocument {
            version: "0.1".into(),
            scene_id: "s1".into(),
            name: "Original".into(),
            entities: vec![],
            instances: BTreeMap::new(),
            extension_data: BTreeMap::new(),
        };
        let model = SemanticModel::Scene(doc);

        let mut cloned = model.clone();
        if let SemanticModel::Scene(cloned_doc) = &mut cloned {
            cloned_doc.name = "Mutated".into();
        }
        if let SemanticModel::Scene(original_doc) = &model {
            assert_eq!(original_doc.name, "Original");
        }
    }

}
