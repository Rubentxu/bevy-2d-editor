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
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicBinding {
    pub asset_id: String,
    pub version: u32,
}
