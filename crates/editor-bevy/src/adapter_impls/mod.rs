//! Adapter implementations for the semantic editor model contract (SDD-0046).
//!
//! Each adapter wraps an existing writer/reader site and declares its
//! [`editor_model::adapter::AdapterFidelity`]. The [`all_adapters_init`] factory
//! is called once at WASM startup to populate the global registry.
//!
//! # Adapters
//!
//! | Adapter | Fidelity | Wrapped sites |
//! |---|---|---|
//! | [`JsonProjectAdapter`](json::JsonProjectAdapter) | Lossless | 6 JSON writer/reader sites |
//! | [`BsnExportAdapter`](bsn::BsnExportAdapter) | SemanticLossless | 1 BSN writer site |
//! | [`BevyRuntimeAdapter`](bevy::BevyRuntimeAdapter) | ExportOnlyLossy | 4 Bevy projection sites |

pub mod bevy;
pub mod bsn;
pub mod json;

use editor_model::adapter::EditorAdapter;

// Re-export the three adapter types for convenience.
pub use bevy::BevyRuntimeAdapter;
pub use bsn::BsnExportAdapter;
pub use json::JsonProjectAdapter;

/// Returns the owned vector of all adapter instances.
///
/// This factory is passed to [`editor_model::adapter::set_registry_fn`] at WASM
/// startup. Each call produces a fresh `Vec` — the registry takes ownership by
/// leaking the boxes to `'static`.
pub fn all_adapters_init() -> Vec<Box<dyn EditorAdapter + Send + Sync>> {
    vec![
        Box::new(json::JsonProjectAdapter::new()),
        Box::new(bsn::BsnExportAdapter::new()),
        Box::new(bevy::BevyRuntimeAdapter::new()),
    ]
}
