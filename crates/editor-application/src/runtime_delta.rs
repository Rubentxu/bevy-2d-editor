//! Apply-back policy + scope types (ADR-0042, ADR-0050).
//!
//! `ApplyBackPolicy` and `ApplyBackScope` live canonically in `editor-application`
//! (this file). `editor-core::ComponentSchema` carries a parallel pair of enums
//! (defined in `crates/editor-core/src/schema.rs`) that is serde-compatible with
//! these. The mirror is forced by the dep direction (`editor-application → editor-core`,
//! not the reverse) — see ADR-0050 §"Why a Mirror Pair Instead of Single Source".
//!
//! `RuntimeDelta` was moved to `editor-model` in v0.90 PR1 so that Bevy systems
//! in `editor-core` can write to `EditorSession.runtime_delta_buffer` through the
//! `EditorSessionPort` trait without importing `editor-application`.

use serde::{Deserialize, Serialize};

// Re-export RuntimeDelta so existing editor-application code can keep using
// `crate::RuntimeDelta` (and downstream code in editor-core can use
// `editor_model::RuntimeDelta`).
pub use editor_model::RuntimeDelta;

/// Maximum number of deltas in `EditorSession.runtime_delta_buffer` (v0.90 PR6).
///
/// Used to cap the ring buffer at runtime-delta creation time and to
/// enforce the cap on every mutable access (see the `EditorSessionPort`
/// `runtime_delta_buffer_mut` impl in `editor-application::session`).
pub const RUNTIME_DELTA_BUFFER_CAP: usize = 64;

/// Policy governing whether and how a component's runtime values may be
/// applied back to the authoring state (ADR-0042, ADR-0050).
///
/// Serialized as part of `RuntimeDelta.apply_back_eligible` derivations and
/// consumed by the ApplyBack workflow. Defaults to `Never` (D4).
///
/// **Mirror note (ADR-0050):** `editor_core::ApplyBackPolicy` is a parallel
/// enum with identical serde representation. New variants must be added to
/// both enums in the same commit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyBackPolicy {
    /// Never apply runtime values back to authoring state.
    #[default]
    Never,
    /// Apply back only when explicitly requested by the user.
    ExplicitOnly,
    /// Apply back is suggested; user may tune the value.
    Tunable,
}

/// Scope of an apply-back operation (ADR-0050).
///
/// v1 only supports `ThisInstance` — apply-back targets the same scene
/// instance that produced the delta.
///
/// **Mirror note (ADR-0050):** `editor_core::ApplyBackScope` is a parallel
/// enum with identical serde representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyBackScope {
    /// Apply back only to the same scene instance that produced the delta.
    ThisInstance,
}
