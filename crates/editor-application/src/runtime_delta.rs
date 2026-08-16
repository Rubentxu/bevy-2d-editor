//! Runtime delta types for play-mode apply-back (ADR-0042, ADR-0036).
//!
//! `RuntimeDelta` captures the diff between the authoring baseline (captured at
//! `PlayModeEnter`) and the runtime value observed at `PlayModeExit`. Each delta
//! is stored in `EditorSession.runtime_delta_buffer` (ring capped at 64) and is
//! consumed by the ApplyBack workflow.
//!
//! Per ADR-0036, `RuntimeDelta` MUST NOT contain any Bevy Entity identifiers.

use serde::{Deserialize, Serialize};

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

/// Policy governing whether and how a field may have its runtime value
/// applied back to the authoring state (ADR-0042).
///
/// Serialized as part of `ComponentSchema`. Defaults to `Never` for all
/// existing schemas (per D4 — conservative default).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyBackPolicy {
    /// Never apply runtime values back to authoring state.
    Never,
    /// Apply back only when explicitly requested by the user.
    ExplicitOnly,
    /// Apply back is suggested; user may tune the value.
    Tunable,
}

impl Default for ApplyBackPolicy {
    fn default() -> Self {
        ApplyBackPolicy::Never
    }
}
