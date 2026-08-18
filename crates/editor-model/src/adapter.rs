//! Editor Adapter trait and runtime types for semantic model encoding/decoding.
//!
//! Adapters wrap existing writer/parser sites and expose a uniform `EditorAdapter`
//! interface. Every adapter declares its [`AdapterFidelity`] so callers can reason
//! about round-trip guarantees.
//!
//! # Architecture
//!
//! The trait lives in `editor-model` (ADR-0030: bevy-free, wasm-free). Implementations
//! live in the `adapter_impls` module of the consumer crate. The [`all_adapters`]
//! registry is populated via [`set_registry_fn`] — a setter that takes a `fn()`
//! pointer so `editor-model` need not depend on any consumer crate.
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
/// All variants borrow from the source data to keep `encode` zero-alloc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SemanticModel<'a> {
    /// A scene document (the root editing document).
    Scene(&'a SceneDocument),
    /// A reusable scene asset.
    SceneAsset(&'a SceneAssetDocument),
    /// A visual logic graph asset.
    LogicGraph(&'a LogicGraphAsset),
    /// A world document.
    World(&'a WorldDocument),
    /// Project-level metadata.
    ProjectMetadata(&'a ProjectMetadata),
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
/// - `encode` takes `&SemanticModel<'_>` (borrowed) for zero-allocation encoding.
/// - `decode` returns `SemanticModel<'static>` (owned); the lifetime is degenerate
///   in this direction (no borrowed data flows back).
/// - `Send + Sync` bounds allow adapters to be stored in a thread-safe registry.
pub trait EditorAdapter: Send + Sync {
    /// Returns a non-empty, stable identifier for this adapter.
    fn name(&self) -> &str;

    /// Returns this adapter's declared fidelity level.
    fn fidelity(&self) -> AdapterFidelity;

    /// Encode a semantic model variant to a byte vector.
    fn encode(&self, model: &SemanticModel<'_>) -> Result<Vec<u8>, AdapterError>;

    /// Decode a byte vector back to a semantic model variant.
    ///
    /// The returned model is always owned (`'static` lifetime).
    fn decode(&self, bytes: &[u8]) -> Result<SemanticModel<'static>, AdapterError>;

    /// Returns `true` if this adapter supports the given scene asset role.
    ///
    /// Default implementation returns `true` (adapters that handle all roles).
    fn supports(&self, _role: SceneAssetRole) -> bool {
        true
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry seam
// ─────────────────────────────────────────────────────────────────────────────

use std::sync::Arc;

/// Slice of owned adapter boxes — stored behind Arc so callers share ownership.
type AdapterSlice = [Box<dyn EditorAdapter + Send + Sync>];

thread_local! {
    static REGISTRY: std::cell::RefCell<Option<Arc<AdapterSlice>>> =
        const { std::cell::RefCell::new(None) };
}

/// Returns a shared reference to the globally registered adapters.
///
/// Returns an empty slice if [`set_registry_fn`] has not been called yet.
pub fn all_adapters() -> Arc<AdapterSlice> {
    REGISTRY
        .try_with(|cell| (*cell.borrow()).clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::new([]))
}

/// Sets the global adapter registry using a factory function.
///
/// This is the cross-crate seam that avoids an `editor-model → editor-bevy` dep
/// (ADR-0030). Callers pass a `fn()` that returns the owned `Vec`, which is
/// wrapped in `Arc` and stored in the thread-local RefCell.
pub fn set_registry_fn(factory: fn() -> Vec<Box<dyn EditorAdapter + Send + Sync>>) {
    let owned: Vec<Box<dyn EditorAdapter + Send + Sync>> = factory();
    let boxed: Box<AdapterSlice> = owned.into_boxed_slice();
    let arc: Arc<AdapterSlice> = boxed.into();
    REGISTRY.with(|cell| {
        *cell.borrow_mut() = Some(arc);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
}
