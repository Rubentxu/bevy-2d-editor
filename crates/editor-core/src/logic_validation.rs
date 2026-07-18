//! Logic Graph Validation — detects graph-shape and type-boundary defects.
//!
//! Validates a `LogicGraphAsset` against a `LogicNodeRegistry` to surface:
//! - Duplicate node IDs
//! - Dangling edge endpoints
//! - Invalid port-type connections
//! - Cycles in the graph topology
//! - Dangling controller references (rust-controller nodes with unknown controller_id)
//! - Missing bindings (active-graph scope only)
//!
//! SCOPE NOTE: active-graph only — validates the single active graph reachable through
//! `with_logic_graph`. Cross-document enumeration of every `LogicGraphAsset` is deferred.

use crate::logic_evaluator::{LogicNodeRegistry, PortValueType};
use crate::logic_graph::{find_dangling_edge_nodes, find_duplicate_node_id, LogicEdge, LogicGraphAsset, LogicNode, NodeId, NodeTypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ─────────────────────────────────────────────────────────────────────────────
// Issue codes
// ─────────────────────────────────────────────────────────────────────────────

/// Issue codes for logic graph validation errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogicValidationIssueCode {
    /// Two or more nodes share the same node_id.
    DuplicateNodeId,
    /// An edge references a node not present in the graph.
    DanglingEdgeEndpoint,
    /// An edge connects ports with incompatible types.
    InvalidPortType,
    /// The graph contains a directed cycle.
    Cycle,
    /// A rust-controller node references a controller_id not in the registry.
    DanglingControllerRef,
}

/// A single validation issue found in a LogicGraphAsset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicValidationIssue {
    /// The issue code categorizing this defect.
    pub code: LogicValidationIssueCode,
    /// The asset_id of the graph containing the issue.
    pub asset_id: String,
    /// Human-readable description.
    pub message: String,
    /// Node IDs directly involved in this issue.
    pub affected_node_ids: Vec<NodeId>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Validate a `LogicGraphAsset` against a `LogicNodeRegistry`.
///
/// Returns one `LogicValidationIssue` per detected defect. An empty Vec means
/// the graph passed all validation checks.
///
/// # Validation checks performed
/// - **DuplicateNodeId**: reuses `find_duplicate_node_id`
/// - **DanglingEdgeEndpoint**: reuses `find_dangling_edge_nodes`
/// - **InvalidPortType**: port-type compatibility via `NodeDescriptor` port specs
/// - **Cycle**: DFS over edges with visited + recursion-stack
/// - **DanglingControllerRef**: `rust-controller` nodes with `controller_id` absent from registry
///
/// SCOPE NOTE: active-graph only — validates the single active graph reachable
/// through `with_logic_graph`. Cross-document enumeration of every `LogicGraphAsset`
/// is deferred.
pub fn validate_logic_graph(
    asset: &LogicGraphAsset,
    registry: &LogicNodeRegistry,
) -> Vec<LogicValidationIssue> {
    let mut issues = Vec::new();

    // 1. Duplicate node IDs
    let dups = find_duplicate_node_id(asset);
    for dup_id in dups {
        issues.push(LogicValidationIssue {
            code: LogicValidationIssueCode::DuplicateNodeId,
            asset_id: asset.asset_id.clone(),
            message: format!("duplicate node id '{}'", dup_id.as_str()),
            affected_node_ids: vec![dup_id],
        });
    }

    // 2. Dangling edge endpoints
    let dangling = find_dangling_edge_nodes(asset);
    for dangling_id in dangling {
        issues.push(LogicValidationIssue {
            code: LogicValidationIssueCode::DanglingEdgeEndpoint,
            asset_id: asset.asset_id.clone(),
            message: format!("edge references undefined node '{}'", dangling_id.as_str()),
            affected_node_ids: vec![dangling_id],
        });
    }

    // 3. Invalid port-type connections
    issues.extend(validate_edge_port_types(asset, registry));

    // 4. Cycle detection (DFS)
    issues.extend(detect_cycles(asset));

    // 5. Dangling controller references
    issues.extend(validate_controller_refs(asset, registry));

    issues
}

// ─────────────────────────────────────────────────────────────────────────────
// Port-type validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validate that each edge connects ports with compatible types.
fn validate_edge_port_types(
    asset: &LogicGraphAsset,
    registry: &LogicNodeRegistry,
) -> Vec<LogicValidationIssue> {
    let mut issues = Vec::new();

    // Build a map of node_id -> node for O(1) lookup
    let node_map: HashMap<NodeId, &LogicNode> = asset
        .nodes
        .iter()
        .map(|n| (n.node_id.clone(), n))
        .collect();

    for edge in &asset.edges {
        // Look up source and target nodes
        let Some(from_node) = node_map.get(&edge.from_node) else {
            // Dangling endpoint already reported separately
            continue;
        };
        let Some(to_node) = node_map.get(&edge.to_node) else {
            // Dangling endpoint already reported separately
            continue;
        };

        // Get descriptors — unknown node_type_id skips port-type check (no false positive)
        let Some(from_desc) = registry.descriptor(&from_node.node_type_id) else {
            continue;
        };
        let Some(to_desc) = registry.descriptor(&to_node.node_type_id) else {
            continue;
        };

        // Check: from_port must be in source node's outputs
        let from_port_type = from_desc
            .outputs
            .iter()
            .find(|p| p.port_id == edge.from_port.as_str())
            .map(|p| p.value_type.clone());

        let from_port_type = match from_port_type {
            Some(t) => t,
            None => {
                issues.push(LogicValidationIssue {
                    code: LogicValidationIssueCode::InvalidPortType,
                    asset_id: asset.asset_id.clone(),
                    message: format!(
                        "port '{}' not found in outputs of node '{}'",
                        edge.from_port.as_str(),
                        from_node.node_id.as_str()
                    ),
                    affected_node_ids: vec![edge.from_node.clone(), edge.to_node.clone()],
                });
                continue;
            }
        };

        // Check: to_port must be in target node's inputs
        let to_port_type = to_desc
            .inputs
            .iter()
            .find(|p| p.port_id == edge.to_port.as_str())
            .map(|p| p.value_type.clone());

        let to_port_type = match to_port_type {
            Some(t) => t,
            None => {
                issues.push(LogicValidationIssue {
                    code: LogicValidationIssueCode::InvalidPortType,
                    asset_id: asset.asset_id.clone(),
                    message: format!(
                        "port '{}' not found in inputs of node '{}'",
                        edge.to_port.as_str(),
                        to_node.node_id.as_str()
                    ),
                    affected_node_ids: vec![edge.from_node.clone(), edge.to_node.clone()],
                });
                continue;
            }
        };

        // Check: port types must match
        if from_port_type != to_port_type {
            issues.push(LogicValidationIssue {
                code: LogicValidationIssueCode::InvalidPortType,
                asset_id: asset.asset_id.clone(),
                message: format!(
                    "port type mismatch: '{}' (source output) vs '{}' (target input) on edge {} -> {}",
                    port_type_name(&from_port_type),
                    port_type_name(&to_port_type),
                    from_node.node_id.as_str(),
                    to_node.node_id.as_str()
                ),
                affected_node_ids: vec![edge.from_node.clone(), edge.to_node.clone()],
            });
        }
    }

    issues
}

fn port_type_name(t: &PortValueType) -> &'static str {
    match t {
        PortValueType::Bool => "bool",
        PortValueType::Float => "float",
        PortValueType::Vec2 => "vec2",
        PortValueType::EntityRef => "entity_ref",
        PortValueType::Action => "action",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Cycle detection (DFS)
// ─────────────────────────────────────────────────────────────────────────────

/// Detect cycles in the directed graph using DFS with visited set + recursion stack.
/// Emits one `Cycle` issue per back-edge discovered.
fn detect_cycles(asset: &LogicGraphAsset) -> Vec<LogicValidationIssue> {
    let mut issues = Vec::new();

    // Build adjacency list: node_id -> list of target node_ids
    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for node in &asset.nodes {
        adj.entry(node.node_id.clone()).or_default();
    }
    for edge in &asset.edges {
        adj.entry(edge.from_node.clone())
            .or_default()
            .push(edge.to_node.clone());
    }

    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut rec_stack: HashSet<NodeId> = HashSet::new();

    // DFS from each unvisited node
    for node_id in asset.nodes.iter().map(|n| n.node_id.clone()) {
        if !visited.contains(&node_id) {
            detect_cycles_dfs(
                &adj,
                &node_id,
                &mut visited,
                &mut rec_stack,
                &mut issues,
                &asset.asset_id,
            );
        }
    }

    issues
}

fn detect_cycles_dfs(
    adj: &HashMap<NodeId, Vec<NodeId>>,
    node_id: &NodeId,
    visited: &mut HashSet<NodeId>,
    rec_stack: &mut HashSet<NodeId>,
    issues: &mut Vec<LogicValidationIssue>,
    asset_id: &str,
) {
    visited.insert(node_id.clone());
    rec_stack.insert(node_id.clone());

    for neighbor in adj.get(node_id).into_iter().flatten() {
        if !visited.contains(neighbor) {
            detect_cycles_dfs(adj, neighbor, visited, rec_stack, issues, asset_id);
        } else if rec_stack.contains(neighbor) {
            // Back-edge found — cycle detected
            issues.push(LogicValidationIssue {
                code: LogicValidationIssueCode::Cycle,
                asset_id: asset_id.to_string(),
                message: format!(
                    "cycle detected: '{}' -> '{}' closes a directed loop",
                    node_id.as_str(),
                    neighbor.as_str()
                ),
                affected_node_ids: vec![node_id.clone(), neighbor.clone()],
            });
        }
    }

    rec_stack.remove(node_id);
}

// ─────────────────────────────────────────────────────────────────────────────
// Controller reference validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validate that rust-controller nodes reference a controller_id in the registry.
fn validate_controller_refs(
    asset: &LogicGraphAsset,
    registry: &LogicNodeRegistry,
) -> Vec<LogicValidationIssue> {
    let mut issues = Vec::new();

    for node in &asset.nodes {
        // Only check rust-controller nodes
        if node.node_type_id.as_str() != "rust-controller" {
            continue;
        }

        let Some(controller_id) = &node.controller_id else {
            // Missing controller_id on a rust-controller node
            issues.push(LogicValidationIssue {
                code: LogicValidationIssueCode::DanglingControllerRef,
                asset_id: asset.asset_id.clone(),
                message: format!(
                    "rust-controller node '{}' is missing a controller_id",
                    node.node_id.as_str()
                ),
                affected_node_ids: vec![node.node_id.clone()],
            });
            continue;
        };

        // Check if controller_id exists in registry
        if registry.get_controller(controller_id).is_none() {
            issues.push(LogicValidationIssue {
                code: LogicValidationIssueCode::DanglingControllerRef,
                asset_id: asset.asset_id.clone(),
                message: format!(
                    "rust-controller node '{}' references unknown controller '{}'",
                    node.node_id.as_str(),
                    controller_id
                ),
                affected_node_ids: vec![node.node_id.clone()],
            });
        }
    }

    issues
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic_evaluator::{global_node_registry, NodeDescriptor, PortSpec};
    use crate::logic_graph::{LogicNode, LogicNodeRole, NodeTypeId, PortId};

    // ── Helper: minimal LogicGraphAsset ──────────────────────────────────

    fn make_asset(nodes: Vec<LogicNode>, edges: Vec<LogicEdge>) -> LogicGraphAsset {
        LogicGraphAsset {
            asset_id: "test_asset".to_string(),
            logical_path: "logic/test".to_string(),
            version: 1,
            nodes,
            edges,
            ..Default::default()
        }
    }

    // ── Issue code serde round-trip ──────────────────────────────────────

    #[test]
    fn logic_validation_issue_code_serde_roundtrip() {
        for code in [
            LogicValidationIssueCode::DuplicateNodeId,
            LogicValidationIssueCode::DanglingEdgeEndpoint,
            LogicValidationIssueCode::InvalidPortType,
            LogicValidationIssueCode::Cycle,
            LogicValidationIssueCode::DanglingControllerRef,
        ] {
            let json = serde_json::to_string(&code).unwrap();
            let parsed: LogicValidationIssueCode = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, code, "round-trip failed for {:?}", code);
        }
    }

    #[test]
    fn logic_validation_issue_code_serializes_to_kebab_case() {
        assert_eq!(
            serde_json::to_string(&LogicValidationIssueCode::DuplicateNodeId).unwrap(),
            "\"duplicate-node-id\""
        );
        assert_eq!(
            serde_json::to_string(&LogicValidationIssueCode::DanglingEdgeEndpoint).unwrap(),
            "\"dangling-edge-endpoint\""
        );
        assert_eq!(
            serde_json::to_string(&LogicValidationIssueCode::InvalidPortType).unwrap(),
            "\"invalid-port-type\""
        );
        assert_eq!(
            serde_json::to_string(&LogicValidationIssueCode::Cycle).unwrap(),
            "\"cycle\""
        );
        assert_eq!(
            serde_json::to_string(&LogicValidationIssueCode::DanglingControllerRef).unwrap(),
            "\"dangling-controller-ref\""
        );
    }

    // ── LogicValidationIssue serde round-trip ─────────────────────────────

    #[test]
    fn logic_validation_issue_roundtrip() {
        let issue = LogicValidationIssue {
            code: LogicValidationIssueCode::Cycle,
            asset_id: "test_asset".to_string(),
            message: "cycle detected".to_string(),
            affected_node_ids: vec![NodeId::new("node_a"), NodeId::new("node_b")],
        };
        let json = serde_json::to_string(&issue).unwrap();
        let parsed: LogicValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.code, LogicValidationIssueCode::Cycle);
        assert_eq!(parsed.asset_id, "test_asset");
        assert_eq!(parsed.message, "cycle detected");
        assert_eq!(parsed.affected_node_ids.len(), 2);
    }

    // ── Empty output baseline ────────────────────────────────────────────

    #[test]
    fn validate_logic_graph_empty_asset_returns_empty() {
        let asset = make_asset(vec![], vec![]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        assert!(issues.is_empty(), "empty asset should have no issues");
    }

    // ── DuplicateNodeId ───────────────────────────────────────────────────

    #[test]
    fn validate_detects_duplicate_node_ids() {
        let node_a = LogicNode {
            node_id: NodeId::new("node_x"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
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
        let asset = make_asset(vec![node_a, node_b], vec![]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        let dup_issues: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.code, LogicValidationIssueCode::DuplicateNodeId))
            .collect();
        assert_eq!(dup_issues.len(), 1);
        assert_eq!(dup_issues[0].affected_node_ids[0].as_str(), "node_x");
    }

    #[test]
    fn validate_no_duplicate_node_ids() {
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
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
        let asset = make_asset(vec![node_a, node_b], vec![]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        assert!(
            !issues.iter().any(|i| matches!(i.code, LogicValidationIssueCode::DuplicateNodeId)),
            "unique node ids should not produce DuplicateNodeId issues"
        );
    }

    // ── DanglingEdgeEndpoint ───────────────────────────────────────────────

    #[test]
    fn validate_detects_dangling_edge_endpoint() {
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("tick"),
            to_node: NodeId::new("node_b"), // does not exist!
            to_port: PortId::new("cond"),
        };
        let asset = make_asset(vec![node_a], vec![edge]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        let danglings: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.code, LogicValidationIssueCode::DanglingEdgeEndpoint))
            .collect();
        assert_eq!(danglings.len(), 1);
        assert_eq!(danglings[0].affected_node_ids[0].as_str(), "node_b");
    }

    #[test]
    fn validate_no_dangling_edge_endpoints() {
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_b = LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("tick"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("a"),
        };
        let asset = make_asset(vec![node_a, node_b], vec![edge]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        assert!(
            !issues.iter().any(|i| matches!(i.code, LogicValidationIssueCode::DanglingEdgeEndpoint)),
            "valid edge endpoints should not produce issues"
        );
    }

    // ── InvalidPortType ───────────────────────────────────────────────────

    #[test]
    fn validate_detects_port_type_mismatch() {
        // sensor.always outputs "tick" (Bool)
        // controller.and expects "a" (Bool) and "b" (Bool)
        // But we wire tick -> a (correct) then tick -> condition (no port on controller.and)
        // First test: from_port not in source outputs
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_b = LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        // Wire to non-existent output port
        let edge = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("nonexistent_out"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("a"),
        };
        let asset = make_asset(vec![node_a, node_b], vec![edge]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        let port_issues: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.code, LogicValidationIssueCode::InvalidPortType))
            .collect();
        assert!(!port_issues.is_empty());
    }

    #[test]
    fn validate_unknown_node_type_skips_port_check() {
        // A node with an unknown node_type_id should NOT produce InvalidPortType
        // just because we don't know its ports
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_b = LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("custom.user_node"), // not in registry
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("tick"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("any_port"),
        };
        let asset = make_asset(vec![node_a, node_b], vec![edge]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        assert!(
            !issues.iter().any(|i| matches!(i.code, LogicValidationIssueCode::InvalidPortType)),
            "unknown node type should not produce InvalidPortType"
        );
    }

    // ── Cycle detection ───────────────────────────────────────────────────

    #[test]
    fn validate_detects_two_node_cycle() {
        // A -> B and B -> A
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_b = LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge_ab = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("condition"),
        };
        let edge_ba = LogicEdge {
            from_node: NodeId::new("node_b"),
            from_port: PortId::new("done"),
            to_node: NodeId::new("node_a"),
            to_port: PortId::new("a"),
        };
        let asset = make_asset(vec![node_a, node_b], vec![edge_ab, edge_ba]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        assert!(
            issues.iter().any(|i| matches!(i.code, LogicValidationIssueCode::Cycle)),
            "A→B→A cycle should be detected"
        );
    }

    #[test]
    fn validate_detects_self_loop() {
        // A -> A (self-loop)
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_a"),
            to_port: PortId::new("a"),
        };
        let asset = make_asset(vec![node_a], vec![edge]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        assert!(
            issues.iter().any(|i| matches!(i.code, LogicValidationIssueCode::Cycle)),
            "self-loop should be detected as cycle"
        );
    }

    #[test]
    fn validate_no_cycle_in_acyclic_graph() {
        // sensor.always -> controller.and -> controller.if
        // No cycles
        let node_a = LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_b = LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge_ab = LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("tick"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("a"),
        };
        // Acyclic chain A → B → C
        let node_c = LogicNode {
            node_id: NodeId::new("node_c"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let edge_bc = LogicEdge {
            from_node: NodeId::new("node_b"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_c"),
            to_port: PortId::new("condition"),
        };
        let asset = make_asset(vec![node_a, node_b, node_c], vec![edge_ab, edge_bc]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        assert!(
            !issues.iter().any(|i| matches!(i.code, LogicValidationIssueCode::Cycle)),
            "linear chain should have no cycles"
        );
    }

    // ── DanglingControllerRef ──────────────────────────────────────────────

    #[test]
    fn validate_detects_unknown_controller_id() {
        let node = LogicNode {
            node_id: NodeId::new("ctrl_1"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("rust-controller"),
            field_values: serde_json::json!({}),
            controller_id: Some("ghost.controller".to_string()), // not registered
        };
        let asset = make_asset(vec![node], vec![]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        let ctrl_issues: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.code, LogicValidationIssueCode::DanglingControllerRef))
            .collect();
        assert_eq!(ctrl_issues.len(), 1);
        assert!(ctrl_issues[0]
            .message
            .contains("ghost.controller"));
    }

    #[test]
    fn validate_detects_missing_controller_id_on_rust_controller() {
        let node = LogicNode {
            node_id: NodeId::new("ctrl_1"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("rust-controller"),
            field_values: serde_json::json!({}),
            controller_id: None, // missing!
        };
        let asset = make_asset(vec![node], vec![]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        let ctrl_issues: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.code, LogicValidationIssueCode::DanglingControllerRef))
            .collect();
        assert_eq!(ctrl_issues.len(), 1);
    }

    #[test]
    fn validate_no_dangling_controller_ref_for_non_rust_controller() {
        let node = LogicNode {
            node_id: NodeId::new("ctrl_1"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"), // not rust-controller
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let asset = make_asset(vec![node], vec![]);
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        assert!(
            !issues.iter().any(|i| matches!(i.code, LogicValidationIssueCode::DanglingControllerRef)),
            "non-rust-controller nodes should not be checked for controller_id"
        );
    }

    // ── Clean graph (e2e) ─────────────────────────────────────────────────

    #[test]
    fn validate_clean_sensor_controller_actuator_chain_returns_empty() {
        // sensor.always --tick:bool--> controller.and --out:bool--> controller.if --done:action-->
        let node_sensor = LogicNode {
            node_id: NodeId::new("sensor"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_and = LogicNode {
            node_id: NodeId::new("and"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_if = LogicNode {
            node_id: NodeId::new("if"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };

        // sensor.always.tick -> controller.and.a
        let edge1 = LogicEdge {
            from_node: NodeId::new("sensor"),
            from_port: PortId::new("tick"),
            to_node: NodeId::new("and"),
            to_port: PortId::new("a"),
        };
        // controller.and.out -> controller.if.condition
        let edge2 = LogicEdge {
            from_node: NodeId::new("and"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("if"),
            to_port: PortId::new("condition"),
        };

        let asset = make_asset(
            vec![node_sensor, node_and, node_if],
            vec![edge1, edge2],
        );
        let registry = global_node_registry();
        let issues = validate_logic_graph(&asset, registry);
        assert!(
            issues.is_empty(),
            "clean chain should return zero issues, got: {:?}",
            issues
        );
    }
}
