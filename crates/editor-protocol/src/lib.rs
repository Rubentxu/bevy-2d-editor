//! editor-protocol: Wire-types and capability trait definitions for the Bevy 2D Editor.
//!
//! This crate contains the types and traits that cross the WASM boundary between
//! the editor frontend (WASM) and the editor backend (editor-bevy).
//!
//! ## Design
//! - No bevy dependency (B7 assertion)
//! - No WASM FFI crate imports
//! - Serialization-first: all types are serde-serializable
//!
//! ## Contents
//! - `capabilities.rs`: 8 capability trait definitions (SceneApi, SceneAssetApi, etc.)
//! - `dispatch_error.rs`: DispatchError enum for the WASM boundary
//! - `re_exports.rs`: Re-exports from editor_model for WASM-boundary types

pub mod capabilities;
pub mod dispatch_error;
pub mod re_exports;

// Re-export commonly used types at the crate root for convenience.
pub use capabilities::*;
pub use dispatch_error::DispatchError;
