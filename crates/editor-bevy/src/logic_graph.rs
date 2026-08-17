//! Logic Graph data model for visual behavior authoring.
//!
//! ADR-0047 split complete:
//! - Pure types (NodeId, PortId, NodeTypeId, LogicNodeRole, LogicNode, LogicEdge,
//!   LogicGraphAsset, LogicInstance, helper functions) live in `editor_model::logic_graph`.
//! - `LogicBinding` with `#[derive(Component)]` lives in `bevy_logic_binding.rs`.
//!
//! This module is now a thin re-export wrapper. All pure types are re-exported
//! from `editor_model::logic_graph` so existing call sites are unaffected.
//! The WASM persistence helpers (save/load) remain here as they need Bevy/WASM.

pub use editor_model::logic_graph::{
    LogicEdge, LogicGraphAsset, LogicInstance, LogicNode, LogicNodeRole, NodeId, NodeTypeId,
    PortId, count_logic_bindings, editor_logic_binding_component, find_dangling_edge_nodes,
    find_duplicate_node_id,
};

/// A lightweight catalog entry for LogicGraphAssets — stored in
/// `project.json` alongside `SceneAssetCatalogEntry` (ADR-0008 layout).
/// Parallel to `SceneAssetCatalogEntry` but for behavior graphs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

/// Placeholder for non-WASM targets.
#[cfg(not(target_arch = "wasm32"))]
pub async fn save_logic_graph_body(_asset: &LogicGraphAsset) -> Result<(), String> {
    Ok(())
}

/// Placeholder for non-WASM targets.
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_logic_graph_body(_logical_path: &str) -> Result<LogicGraphAsset, String> {
    Err("load_logic_graph_body not available on non-WASM target".to_string())
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
        use editor_model::ComponentInstance;
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
        use editor_model::ComponentInstance;
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
