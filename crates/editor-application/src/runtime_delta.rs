//! Runtime delta types for play-mode apply-back (ADR-0042, ADR-0036).
//!
//! `RuntimeDelta` captures the diff between the authoring baseline (captured at
//! `PlayModeEnter`) and the runtime value observed at `PlayModeExit`. Each delta
//! is stored in `EditorSession.runtime_delta_buffer` (ring capped at 64) and is
//! consumed by the ApplyBack workflow.
//!
//! Per ADR-0036, `RuntimeDelta` MUST NOT contain any Bevy Entity identifiers.
//!
//! `ApplyBackPolicy` and `ApplyBackScope` are defined in the application layer
//! (here). `editor_core::ComponentSchema` carries a parallel
//! `editor_core::ApplyBackPolicy` enum that is serde-compatible with this one
//! (identical tag names, default = Never). The two enums live in different
//! crates because `editor-application` cannot import from `editor-core` in
//! non-wasm32 builds (per ADR-0031/0032 dependency rules). ADR-0050 documents
//! this duplication and the convention that any new variant must be added to
//! BOTH enums in the same change.

use serde::{Deserialize, Serialize};

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

/// A single field-level delta recorded during play mode.
///
/// Captured when a `Tunable` field's runtime value differs from its authoring
/// baseline. The `apply_back` workflow presents this as a selectable delta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeDelta {
    /// Stable ID of the scene instance carrying this field.
    pub instance_id: String,
    /// Local ID of the target entity within the instance.
    pub target_local_id: String,
    /// Component type identifier (e.g., `"editor.Transform2D"`).
    pub component_type_id: String,
    /// Dotted field path within the component (e.g., `"translation.x"`).
    pub field_path: String,
    /// JSON value of the field at authoring time (snapshot on `PlayModeEnter`).
    pub baseline_value: serde_json::Value,
    /// JSON value of the field at `PlayModeExit`.
    pub runtime_value: serde_json::Value,
    /// Unix milliseconds when this delta was captured.
    pub captured_at_ms: u64,
    /// Whether this field is eligible for apply-back.
    ///
    /// `false` when `apply_back` policy is `Never` for this component schema.
    pub apply_back_eligible: bool,
}
