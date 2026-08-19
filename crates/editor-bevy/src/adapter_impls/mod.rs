//! Adapter implementations for the semantic editor model contract (SDD-0046).
//!
//! Each adapter wraps an existing writer/reader site and declares its
//! [`editor_model::adapter::AdapterFidelity`]. The [`all_adapters_init`] factory
//! is called once to populate the registry via
//! [`adapter_registry::init_registry`](crate::adapter_registry::init_registry).
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
/// This factory is passed to [`editor_bevy::adapter_registry::init_registry`] (SDD-0046
/// S2 D2) at WASM startup. Each call produces a fresh `Vec` — the registry takes
/// ownership exactly once via `OnceLock::set`.
pub fn all_adapters_init() -> Vec<Box<dyn EditorAdapter + Send + Sync>> {
    vec![
        Box::new(json::JsonProjectAdapter::new()),
        Box::new(bsn::BsnExportAdapter::new()),
        Box::new(bevy::BevyRuntimeAdapter::new()),
    ]
}
