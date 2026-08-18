//! Logic Graph data model for visual behavior authoring.
//!
//! Mirrors the Scene Asset document model but carries `nodes` and `edges`
//! for a visual node/edge graph. Distinct from a Bevy runtime scene.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::component::ComponentInstance;

/// Opaque stable identity of a node inside a LogicGraphAsset.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub String);

impl NodeId {
    /// Construct a new NodeId from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    /// Borrow the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque stable identity of a port on a LogicNode.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PortId(pub String);

impl PortId {
    /// Construct a new PortId from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    /// Borrow the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque identity of a node type (e.g. "sensor.key_down", "rust-controller").
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeTypeId(pub String);

impl NodeTypeId {
    /// Construct a new NodeTypeId from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    /// Borrow the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Role of a LogicNode in the Sensor → Controller → Actuator flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicNodeRole {
    /// Emits events or values (e.g. key press, collision, timer).
    Sensor,
    /// Makes decisions (e.g. if, gate, compare, math).
    Controller,
    /// Produces side-effects (e.g. apply impulse, set animation, spawn).
    Actuator,
}

/// One node in a LogicGraphAsset graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicNode {
    /// Unique identifier for this node within the graph.
    pub node_id: NodeId,
    /// Role in the Sensor → Controller → Actuator flow.
    pub role: LogicNodeRole,
    /// Type of the node (e.g. "sensor.key_down", "rust-controller").
    pub node_type_id: NodeTypeId,
    /// Per-instance field values for this node.
    #[serde(default)]
    pub field_values: serde_json::Value,
    /// For `RustController` nodes, the resolved controller identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller_id: Option<String>,
}

/// A directed edge connecting two LogicNodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicEdge {
    /// Source node of the edge.
    pub from_node: NodeId,
    /// Source port on the source node.
    pub from_port: PortId,
    /// Destination node of the edge.
    pub to_node: NodeId,
    /// Destination port on the destination node.
    pub to_port: PortId,
}

/// Editor-owned durable authoring document for a logic graph asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicGraphAsset {
    /// Stable identifier for this asset.
    pub asset_id: String,
    /// Logical project path.
    pub logical_path: String,
    /// Monotonically increasing version number.
    pub version: u32,
    /// Whether this is a built-in recipe (not user-authored).
    #[serde(default)]
    pub builtin: bool,
    /// All nodes in this graph.
    #[serde(default)]
    pub nodes: Vec<LogicNode>,
    /// All directed edges in this graph.
    #[serde(default)]
    pub edges: Vec<LogicEdge>,
    /// Unknown JSON fields preserved for forward compatibility (ADR-0046 rule 2).
    #[serde(default, flatten)]
    pub extension_data: BTreeMap<String, serde_json::Value>,
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
            extension_data: BTreeMap::new(),
        }
    }
}

/// A lightweight catalog entry for LogicGraphAssets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicGraphCatalogEntry {
    /// Stable identifier for this asset.
    pub asset_id: String,
    /// Logical project path.
    pub logical_path: String,
    /// Whether this is a built-in recipe.
    #[serde(default)]
    pub builtin: bool,
    /// Unix timestamp (ms) when this asset was created.
    #[serde(default)]
    pub created_at: u64,
    /// Unix timestamp (ms) when this asset was last modified.
    #[serde(default)]
    pub updated_at: u64,
}

/// Binding payload for a LogicInstance — placed use of a LogicGraphAsset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicInstance {
    /// ID of the LogicGraphAsset being instantiated.
    pub asset_id: String,
    /// Version of the LogicGraphAsset at the time of binding.
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
    let node_ids: std::collections::HashSet<_> =
        asset.nodes.iter().map(|n| n.node_id.clone()).collect();
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
        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.nodes[0].node_id, node_a.node_id);
        assert_eq!(parsed.edges[0].from_node, edge.from_node);
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
            node_id: NodeId::new("node_x"),
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
            to_node: NodeId::new("node_b"),
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
    fn count_logic_bindings_counts_editor_logic_binding() {
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
    fn builtin_field_defaults_to_false_when_absent() {
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
}
