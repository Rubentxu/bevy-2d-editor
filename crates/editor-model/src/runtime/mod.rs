//! Runtime coordination types — pure (no Bevy, no WASM) types owned by
//! `EditorSession`.
//!
//! These types live in `editor-model` so `editor-application` (which holds
//! `EditorSession`) can own them without creating an
//! `editor-application → editor-bevy` dependency edge (ADR-0030).

pub mod linear_bus;

pub use linear_bus::LinearBus;