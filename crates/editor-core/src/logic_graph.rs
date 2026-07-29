//! Logic Graph data model for visual behavior authoring.
//!
//! Mirrors the Scene Asset document model but carries `nodes` and `edges`
//! for a visual node/edge graph. Distinct from a Bevy runtime scene.

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::document::ComponentInstance;

// ─────────────────────────────────────────────────────────────────────────────
// Opaque ID newtypes — mirror LocalId pattern (scene_asset.rs:13-26)
// ─────────────────────────────────────────────────────────────────────────────

/// Opaque stable identity of a node inside a LogicGraphAsset.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque stable identity of a port on a LogicNode.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PortId(pub String);

impl PortId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque identity of a node type (e.g. "sensor.key_down", "rust-controller").
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeTypeId(pub String);

impl NodeTypeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Graph data model types
// ─────────────────────────────────────────────────────────────────────────────

/// Role of a LogicNode in the Sensor → Controller → Actuator flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicNodeRole {
    Sensor,
    Controller,
    Actuator,
}

/// One node in a LogicGraphAsset graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicNode {
    pub node_id: NodeId,
    pub role: LogicNodeRole,
    pub node_type_id: NodeTypeId,
    #[serde(default)]
    pub field_values: serde_json::Value,
    /// Present when `node_type_id` is `"rust-controller"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller_id: Option<String>,
}

/// A directed edge connecting two LogicNodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicEdge {
    pub from_node: NodeId,
    pub from_port: PortId,
    pub to_node: NodeId,
    pub to_port: PortId,
}

/// Editor-owned durable authoring document for a logic graph asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicGraphAsset {
    pub asset_id: String,
    pub logical_path: String,
    pub version: u32,
    /// True for built-in immutable recipes (e.g. `recipes/platformer_jump`).
    /// User-authored assets always have `builtin: false` (the serde default).
    #[serde(default)]
    pub builtin: bool,
    #[serde(default)]
    pub nodes: Vec<LogicNode>,
    #[serde(default)]
    pub edges: Vec<LogicEdge>,
}

impl Default for LogicGraphAsset {
    fn default() -> Self {
        Self {
            asset_id: String::new(),
            logical_path: String::new(),
            version: 0,
            builtin: false,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

/// A lightweight catalog entry for LogicGraphAssets — stored in
/// `project.json` alongside `SceneAssetCatalogEntry` (ADR-0008 layout).
/// Parallel to `SceneAssetCatalogEntry` but for behavior graphs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicGraphCatalogEntry {
    pub asset_id: String,
    pub logical_path: String,
    /// Built-in immutable recipe (e.g. `recipes/platformer_jump`).
    #[serde(default)]
    pub builtin: bool,
    /// Unix timestamp ms when the asset was first created.
    #[serde(default)]
    pub created_at: u64,
    /// Unix timestamp ms when the asset was last saved.
    #[serde(default)]
    pub updated_at: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// LogicInstance binding payload
// ─────────────────────────────────────────────────────────────────────────────

/// Binding payload for a LogicInstance — placed use of a LogicGraphAsset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicInstance {
    pub asset_id: String,
    pub version: u32,
}

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

/// Project a LogicInstance to a ComponentInstance with type_id "editor.LogicBinding".
pub fn editor_logic_binding_component(instance: &LogicInstance) -> ComponentInstance {
    ComponentInstance {
        type_id: "editor.LogicBinding".to_string(),
        values: serde_json::json!({
            "asset_id": instance.asset_id,
            "version": instance.version,
        }),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OPFS persistence helpers (parallel to scene_asset persistence patterns)
// ─────────────────────────────────────────────────────────────────────────────

/// Save a LogicGraphAsset body to OPFS at `logic_graphs/<logical_path>.logic.json`.
/// The catalog entry must be saved separately first (ADR-0019: catalog-first).
#[cfg(target_arch = "wasm32")]
pub async fn save_logic_graph_body(asset: &LogicGraphAsset) -> Result<(), String> {
    let path = crate::persistence::logic_graph_path(&asset.logical_path);
    let json = serde_json::to_string(asset).map_err(|e| e.to_string())?;
    crate::js_save_file(&path, &json).await
}

/// Load a LogicGraphAsset body from OPFS by logical_path.
#[cfg(target_arch = "wasm32")]
pub async fn load_logic_graph_body(logical_path: &str) -> Result<LogicGraphAsset, String> {
    let path = crate::persistence::logic_graph_path(logical_path);
    let json = crate::js_load_file(&path).await?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

/// Delete a LogicGraphAsset body from OPFS by logical_path.
#[cfg(not(target_arch = "wasm32"))]
pub async fn save_logic_graph_body(_asset: &LogicGraphAsset) -> Result<(), String> {
    Ok(())
}

/// Placeholder for non-WASM targets.
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_logic_graph_body(_logical_path: &str) -> Result<LogicGraphAsset, String> {
    Err("load_logic_graph_body not available on non-WASM target".to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Pure shape helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Find NodeIds that appear more than once in a LogicGraphAsset.
pub fn find_duplicate_node_id(asset: &LogicGraphAsset) -> Vec<NodeId> {
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<NodeId, usize> = BTreeMap::new();
    for node in &asset.nodes {
        *counts.entry(node.node_id.clone()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .filter(|(_, c)| *c > 1)
        .map(|(id, _)| id)
        .collect()
}

/// Find NodeIds referenced by edges but not defined in the node list.
pub fn find_dangling_edge_nodes(asset: &LogicGraphAsset) -> Vec<NodeId> {
    let node_ids: std::collections::HashSet<_> = asset
        .nodes
        .iter()
        .map(|n| n.node_id.clone())
        .collect();
    let mut dangling: Vec<NodeId> = Vec::new();
    for edge in &asset.edges {
        if !node_ids.contains(&edge.to_node) && !dangling.contains(&edge.to_node) {
            dangling.push(edge.to_node.clone());
        }
        if !node_ids.contains(&edge.from_node) && !dangling.contains(&edge.from_node) {
            dangling.push(edge.from_node.clone());
        }
    }
    dangling
}

/// Count how many ComponentInstances have type_id "editor.LogicBinding".
pub fn count_logic_bindings(components: &[ComponentInstance]) -> usize {
    components
        .iter()
        .filter(|c| c.type_id == "editor.LogicBinding")
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_populated_asset_preserves_ids() {
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({"key": "Space"}),
            controller_id: None,
        };
        let node_b = LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("actuator.jump"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("in"),
        };
        let asset = LogicGraphAsset {
            asset_id: "lga_jump".to_string(),
            logical_path: "logic/jump".to_string(),
            version: 1,
            nodes: vec![node_a.clone(), node_b.clone()],
            edges: vec![edge.clone()],
            ..Default::default()
        };

        let json = serde_json::to_string(&asset).unwrap();
        let parsed: LogicGraphAsset = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.asset_id, "lga_jump");
        assert_eq!(parsed.logical_path, "logic/jump");
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.nodes[0].node_id, node_a.node_id);
        assert_eq!(parsed.nodes[0].role, node_a.role);
        assert_eq!(parsed.nodes[0].node_type_id, node_a.node_type_id);
        assert_eq!(parsed.edges[0].from_node, edge.from_node);
        assert_eq!(parsed.edges[0].from_port, edge.from_port);
    }

    #[test]
    fn empty_asset_serializes_with_empty_vectors() {
        let asset = LogicGraphAsset {
            asset_id: "lga_empty".to_string(),
            logical_path: "logic/empty".to_string(),
            version: 1,
            ..Default::default()
        };
        let json = serde_json::to_string(&asset).unwrap();
        assert!(json.contains("\"nodes\":[]"));
        assert!(json.contains("\"edges\":[]"));

        let parsed: LogicGraphAsset = serde_json::from_str(&json).unwrap();
        assert!(parsed.nodes.is_empty());
        assert!(parsed.edges.is_empty());
    }

    #[test]
    fn node_carries_role_type_id_and_field_values() {
        let node = LogicNode {
            node_id: NodeId::new("sensor_1"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({"condition": "health < 0"}),
            controller_id: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: LogicNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, LogicNodeRole::Controller);
        assert_eq!(parsed.node_type_id.as_str(), "controller.if");
        assert_eq!(parsed.field_values["condition"], "health < 0");
    }

    #[test]
    fn rust_controller_node_carries_controller_id_when_present() {
        let node = LogicNode {
            node_id: NodeId::new("ctrl_1"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("rust-controller"),
            field_values: serde_json::json!({}),
            controller_id: Some("if_controller".to_string()),
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: LogicNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.controller_id.as_deref(), Some("if_controller"));
    }

    #[test]
    fn logic_instance_round_trips_asset_id() {
        let instance = LogicInstance {
            asset_id: "lga_jump".to_string(),
            version: 1,
        };
        let json = serde_json::to_string(&instance).unwrap();
        let parsed: LogicInstance = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.asset_id, "lga_jump");
        assert_eq!(parsed.version, 1);
    }

    #[test]
    fn logic_instance_projects_to_editor_logic_binding_component() {
        use crate::document::ComponentInstance;
        let instance = LogicInstance {
            asset_id: "lga_jump".to_string(),
            version: 3,
        };
        let component = editor_logic_binding_component(&instance);
        assert_eq!(component.type_id, "editor.LogicBinding");
        assert_eq!(component.values["asset_id"], "lga_jump");
        assert_eq!(component.values["version"], 3);
    }

    #[test]
    fn find_duplicate_node_id_detects_duplicates() {
        let node_a = LogicNode {
            node_id: NodeId::new("node_x"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_b = LogicNode {
            node_id: NodeId::new("node_x"), // duplicate!
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("actuator.jump"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let asset = LogicGraphAsset {
            asset_id: "lga_test".to_string(),
            logical_path: "logic/test".to_string(),
            version: 1,
            nodes: vec![node_a, node_b],
            ..Default::default()
        };
        let dups = find_duplicate_node_id(&asset);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].as_str(), "node_x");
    }

    #[test]
    fn find_duplicate_node_id_returns_empty_when_unique() {
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_b = LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("actuator.jump"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let asset = LogicGraphAsset {
            asset_id: "lga_test".to_string(),
            logical_path: "logic/test".to_string(),
            version: 1,
            nodes: vec![node_a, node_b],
            ..Default::default()
        };
        let dups = find_duplicate_node_id(&asset);
        assert!(dups.is_empty());
    }

    #[test]
    fn find_dangling_edge_nodes_detects_missing_target() {
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_b"), // does not exist!
            to_port: PortId::new("in"),
        };
        let asset = LogicGraphAsset {
            asset_id: "lga_test".to_string(),
            logical_path: "logic/test".to_string(),
            version: 1,
            nodes: vec![node_a],
            edges: vec![edge],
            ..Default::default()
        };
        let dangling = find_dangling_edge_nodes(&asset);
        assert_eq!(dangling.len(), 1);
        assert_eq!(dangling[0].as_str(), "node_b");
    }

    #[test]
    fn find_dangling_edge_nodes_returns_empty_when_all_valid() {
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_b = LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("actuator.jump"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("in"),
        };
        let asset = LogicGraphAsset {
            asset_id: "lga_test".to_string(),
            logical_path: "logic/test".to_string(),
            version: 1,
            nodes: vec![node_a, node_b],
            edges: vec![edge],
            ..Default::default()
        };
        let dangling = find_dangling_edge_nodes(&asset);
        assert!(dangling.is_empty());
    }

    #[test]
    fn count_logic_bindings_counts_editor_logic_binding() {
        use crate::document::ComponentInstance;
        let components = vec![
            ComponentInstance {
                type_id: "editor.Transform2D".to_string(),
                values: serde_json::json!({}),
            },
            ComponentInstance {
                type_id: "editor.LogicBinding".to_string(),
                values: serde_json::json!({"asset_id": "lga_1"}),
            },
            ComponentInstance {
                type_id: "editor.Name".to_string(),
                values: serde_json::json!({"name": "Bob"}),
            },
            ComponentInstance {
                type_id: "editor.LogicBinding".to_string(),
                values: serde_json::json!({"asset_id": "lga_2"}),
            },
        ];
        let count = count_logic_bindings(&components);
        assert_eq!(count, 2);
    }

    #[test]
    fn count_logic_bindings_returns_zero_when_none() {
        use crate::document::ComponentInstance;
        let components = vec![
            ComponentInstance {
                type_id: "editor.Transform2D".to_string(),
                values: serde_json::json!({}),
            },
            ComponentInstance {
                type_id: "editor.Name".to_string(),
                values: serde_json::json!({"name": "Bob"}),
            },
        ];
        let count = count_logic_bindings(&components);
        assert_eq!(count, 0);
    }

    // ── builtin field tests ───────────────────────────────────────────────────

    #[test]
    fn builtin_field_defaults_to_false_when_absent() {
        // JSON without `builtin` field should deserialize to builtin == false
        let json = r#"{
            "asset_id": "lga_jump",
            "logical_path": "logic/jump",
            "version": 1,
            "nodes": [],
            "edges": []
        }"#;
        let parsed: LogicGraphAsset = serde_json::from_str(json).unwrap();
        assert!(!parsed.builtin);
    }

    #[test]
    fn builtin_field_round_trips_true() {
        let asset = LogicGraphAsset {
            asset_id: "lga_recipe_jump".to_string(),
            logical_path: "recipes/platformer_jump".to_string(),
            version: 1,
            builtin: true,
            ..Default::default()
        };
        let json = serde_json::to_string(&asset).unwrap();
        let parsed: LogicGraphAsset = serde_json::from_str(&json).unwrap();
        assert!(parsed.builtin);
    }

    #[test]
    fn builtin_field_round_trips_false() {
        let asset = LogicGraphAsset {
            asset_id: "lga_user".to_string(),
            logical_path: "logic/my_graph".to_string(),
            version: 1,
            builtin: false,
            ..Default::default()
        };
        let json = serde_json::to_string(&asset).unwrap();
        let parsed: LogicGraphAsset = serde_json::from_str(&json).unwrap();
        assert!(!parsed.builtin);
    }
}
