//! `RuntimeDelta` — single field-level delta recorded during play mode (ADR-0042, ADR-0036).
//!
//! Captured when a `Tunable` field's runtime value differs from its authoring
//! baseline. The `apply_back` workflow presents this as a selectable delta.
//!
//! Moved to `editor-model` in v0.90 PR1 so that `editor-core` (Bevy systems) can
//! write to `EditorSession.runtime_delta_buffer` via the `EditorSessionPort`
//! trait without importing `editor-application` (which would reintroduce the
//! dep that v0.88 PR B / v0.89 PR2a cut).
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
