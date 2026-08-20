//! Bevy-specific LogicBinding component.
//!
//! ADR-0047 split: `LogicBinding` with `#[derive(Component)]` stays in `editor-core`
//! because it requires Bevy. The 8 pure types (NodeId, PortId, NodeTypeId,
//! LogicNodeRole, LogicNode, LogicEdge, LogicGraphAsset, LogicInstance) live in
//! `editor_model::logic_graph`.

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

/// Bevy component attached to entities that have a LogicBinding.
///
/// This component is inserted by `spawn_preview_entity` when it encounters
/// an `editor.LogicBinding` component. The `logic_evaluation_system`
/// queries for this component to find all logic-bound entities and evaluate their graphs.
///
/// ## Dirty Tracking (cycle 2)
///
/// - `dirty`: set to `true` when the binding needs re-evaluation. Cleared after
///   `dispatch_dirty_bindings` processes the binding.
/// - `binding_version`: incremented by every `LogicOperation` (Bind, Unbind,
///   SetBindingFieldOverride). Starts at 1 when `spawn_preview_entity` first creates
///   the component. A value of 0 means "never evaluated" and is skipped by the
///   dispatcher.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicBinding {
    pub asset_id: String,
    pub version: u32,
    /// Whether this binding needs re-evaluation by the dispatch scheduler.
    /// Set to `true` by `mark_bindings_dirty` on sensor events or by
    /// `apply_*` functions on LogicOperations. Cleared by `dispatch_dirty_bindings`
    /// after evaluation.
    pub dirty: bool,
    /// Monotonically increasing version counter bumped by every LogicOperation.
    /// Starts at 1 on first spawn. A binding with `binding_version == 0` is
    /// skipped by the dispatcher (never evaluated before).
    pub binding_version: u64,
}

impl Default for LogicBinding {
    fn default() -> Self {
        Self {
            asset_id: String::new(),
            version: 0,
            dirty: false,
            binding_version: 0,
        }
    }
}
