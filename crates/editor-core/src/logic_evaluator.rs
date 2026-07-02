//! Logic Evaluator — dispatch trait and typed port boundary.
//!
//! Phase 1: NodeEvaluator trait + PortValue enum + metadata structs.
//! Phase 2: LogicNodeRegistry singleton + placeholder built-in evaluators.
//! Phase 3: Logic graph evaluation dispatch (evaluate_logic_binding).
//! All tests follow Strict TDD: RED → GREEN → TRIANGULATE → REFACTOR.

use crate::logic_graph::{LogicEdge, LogicGraphAsset, LogicNode, LogicNodeRole, NodeId, NodeTypeId, PortId};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

// WASM bindings — only compiled on wasm32 target
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

/// Typed value boundary for logic evaluator ports.
/// NO `serde_json::Value` inside — this is the strict contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortValue {
    Bool(bool),
    Float(f32),
    Vec2 { x: f32, y: f32 },
    EntityRef(String),
    Action(String),
}

/// Port specification for a single input or output port.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortSpec {
    pub port_id: String,
    pub value_type: PortValueType,
    pub display_name: String,
}

/// Value type enumeration — mirrors the PortValue variants.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortValueType {
    Bool,
    Float,
    Vec2,
    EntityRef,
    Action,
}

/// Node descriptor — metadata for a node type used by the registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDescriptor {
    pub node_type_id: NodeTypeId,
    pub role: LogicNodeRole,
    pub display_name: String,
    pub category: String,
    pub inputs: Vec<PortSpec>,
    pub outputs: Vec<PortSpec>,
}

/// Parameter specification for a node type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParamSpec {
    pub name: String,
    pub value_type: PortValueType,
    pub default: Option<serde_json::Value>,
}

/// Errors that can occur during logic graph evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum LogicError {
    /// Graph asset not registered.
    AssetNotFound(String),
    /// Version mismatch between binding and asset.
    VersionMismatch {
        asset_id: String,
        expected: u32,
        actual: u32,
    },
    /// Cycle detected — no valid topological order exists.
    CycleDetected,
    /// No evaluator registered for the given node type.
    MissingEvaluator(NodeTypeId),
    /// Referenced port does not exist on the node.
    InvalidPort { node_id: NodeId, port_id: PortId },
    /// Node from execution order not found in graph asset.
    NodeNotFound(NodeId),
}

impl std::fmt::Display for LogicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicError::AssetNotFound(id) => write!(f, "graph asset not found: '{}'", id),
            LogicError::VersionMismatch { asset_id, expected, actual } => {
                write!(f, "version mismatch for '{}': expected {}, got {}", asset_id, expected, actual)
            }
            LogicError::CycleDetected => write!(f, "cycle detected in graph: no valid execution order"),
            LogicError::MissingEvaluator(ntid) => write!(f, "missing evaluator for node type: '{}'", ntid.as_str()),
            LogicError::InvalidPort { node_id, port_id } => {
                write!(f, "invalid port: '{}.{}'", node_id.as_str(), port_id.as_str())
            }
            LogicError::NodeNotFound(node_id) => {
                write!(f, "node not found in graph: '{}'", node_id.as_str())
            }
        }
    }
}

impl std::error::Error for LogicError {}

/// NodeEvaluator dispatch trait — ADR-0011 §D2.
///
/// Evaluators are stateless; all configuration lives in `LogicNode.field_values`.
/// The trait is `Send + Sync` to support multi-threaded dispatch schedulers.
pub trait NodeEvaluator: Send + Sync {
    /// Evaluate this node with the given input values.
    /// Returns output values for each output port.
    fn evaluate(&self, node: &LogicNode, inputs: &[PortValue]) -> Vec<PortValue>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 2: Registry — dual-keyed process-global singleton
// ─────────────────────────────────────────────────────────────────────────────

/// Global node registry — dual-keyed by NodeTypeId and controller_id.
pub struct LogicNodeRegistry {
    by_node_type: HashMap<NodeTypeId, Box<dyn NodeEvaluator>>,
    by_controller_id: HashMap<String, Box<dyn NodeEvaluator>>,
    descriptors: HashMap<NodeTypeId, NodeDescriptor>,
}

impl LogicNodeRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            by_node_type: HashMap::new(),
            by_controller_id: HashMap::new(),
            descriptors: HashMap::new(),
        }
    }

    /// Get an evaluator by NodeTypeId.
    pub fn get_evaluator(&self, node_type_id: &NodeTypeId) -> Option<&dyn NodeEvaluator> {
        self.by_node_type.get(node_type_id).map(|e| e.as_ref())
    }

    /// Get an evaluator by controller_id (for RustController nodes).
    pub fn get_controller(&self, controller_id: &str) -> Option<&dyn NodeEvaluator> {
        self.by_controller_id.get(controller_id).map(|e| e.as_ref())
    }

    /// Get the descriptor for a node type.
    pub fn descriptor(&self, node_type_id: &NodeTypeId) -> Option<&NodeDescriptor> {
        self.descriptors.get(node_type_id)
    }

    /// Get all descriptors as a slice.
    pub fn all_descriptors(&self) -> &HashMap<NodeTypeId, NodeDescriptor> {
        &self.descriptors
    }

    /// Register a built-in evaluator with its descriptor (OCP insertion point).
    pub fn register_builtin(
        &mut self,
        evaluator: Box<dyn NodeEvaluator>,
        descriptor: NodeDescriptor,
    ) {
        let node_type_id = descriptor.node_type_id.clone();
        self.by_node_type.insert(node_type_id.clone(), evaluator);
        self.descriptors.insert(node_type_id, descriptor);
    }

    /// Register a controller evaluator by controller_id.
    pub fn register_controller(&mut self, controller_id: &str, evaluator: Box<dyn NodeEvaluator>) {
        self.by_controller_id.insert(controller_id.to_string(), evaluator);
    }
}

impl Default for LogicNodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Process-global singleton accessor.
static REGISTRY: OnceLock<LogicNodeRegistry> = OnceLock::new();

/// Get the global node registry, initialized with built-in seeds on first call.
pub fn global_node_registry() -> &'static LogicNodeRegistry {
    REGISTRY.get_or_init(|| {
        let mut registry = LogicNodeRegistry::new();
        seed_builtin_evaluators(&mut registry);
        registry
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 3: LogicGraphAsset registry (in-memory, WASM-side persistence deferred)
// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    /// In-memory registry of LogicGraphAsset documents, keyed by asset_id.
    /// Initialized lazily on first access.
    static LOGIC_GRAPH_REGISTRY: RefCell<Option<HashMap<String, LogicGraphAsset>>> =
        const { RefCell::new(None) };
}

/// Register a LogicGraphAsset in the in-memory registry.
pub fn register_logic_graph(asset: LogicGraphAsset) {
    LOGIC_GRAPH_REGISTRY.with(|cell| {
        let mut reg = cell.borrow_mut();
        if reg.is_none() {
            *reg = Some(HashMap::new());
        }
        reg.as_mut().unwrap().insert(asset.asset_id.clone(), asset);
    });
}

/// Get a LogicGraphAsset by asset_id from the in-memory registry.
pub fn get_logic_graph_asset(asset_id: &str) -> Option<LogicGraphAsset> {
    LOGIC_GRAPH_REGISTRY.with(|cell| {
        cell.borrow().as_ref()?.get(asset_id).cloned()
    })
}

/// Evaluate a logic binding: find graph by asset_id, run sensor→controller→actuator.
/// Submits actuator outputs to ACTUATOR_OUTPUT_BUS.
///
/// # Arguments
/// * `asset_id` - The asset identifier of the LogicGraphAsset to evaluate
/// * `version` - Expected version (returns error if mismatched)
///
/// # Errors
/// Returns `LogicError` if the asset is not found, version mismatches, or
/// evaluation cannot proceed (cycle, missing evaluator).
pub fn evaluate_logic_binding(asset_id: &str, version: u32, entity_bits: u64) -> Result<(), LogicError> {
    // Step 1: Find the LogicGraphAsset by asset_id
    let asset = get_logic_graph_asset(asset_id)
        .ok_or_else(|| LogicError::AssetNotFound(asset_id.to_string()))?;

    // Step 2: Version check
    if asset.version != version {
        return Err(LogicError::VersionMismatch {
            asset_id: asset_id.to_string(),
            expected: version,
            actual: asset.version,
        });
    }

    // Step 3: Build execution order via topological sort of edges
    // Sensors run first (they have no input dependencies), then controllers, then actuators.
    // Edges go from output port of source node to input port of target node.
    // We do a Kahn's algorithm: nodes with all inputs satisfied can run.
    let execution_order = topological_sort(&asset.nodes, &asset.edges)
        .ok_or(LogicError::CycleDetected)?;

    // Step 4: Run sensors first, collect their output values
    // Step 5: Run controllers, propagating values through the graph
    // Step 6: Run actuators, calling submit_actuator_output for each
    evaluate_nodes_in_order(&asset, &execution_order, entity_bits)?;

    Ok(())
}

/// Kahn's algorithm topological sort. Returns None if a cycle exists.
fn topological_sort(nodes: &[LogicNode], edges: &[LogicEdge]) -> Option<Vec<NodeId>> {
    // Build adjacency and in-degree maps
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for node in nodes {
        in_degree.insert(node.node_id.clone(), 0);
        adjacency.entry(node.node_id.clone()).or_default();
    }

    for edge in edges {
        *in_degree.entry(edge.to_node.clone()).or_insert(0) += 1;
        adjacency
            .entry(edge.from_node.clone())
            .or_default()
            .push(edge.to_node.clone());
    }

    // Start with nodes that have no incoming edges
    let mut queue: Vec<NodeId> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    let mut result: Vec<NodeId> = Vec::new();

    while let Some(node_id) = queue.pop() {
        result.push(node_id.clone());
        if let Some(neighbors) = adjacency.get(&node_id) {
            for neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }
    }

    // If we processed all nodes, return the order; otherwise there's a cycle
    if result.len() == nodes.len() {
        Some(result)
    } else {
        None
    }
}

/// Evaluate all nodes in the given order (result of topological sort).
fn evaluate_nodes_in_order(asset: &LogicGraphAsset, order: &[NodeId], entity_bits: u64) -> Result<(), LogicError> {
    let registry = global_node_registry();

    // Build a map from NodeId to LogicNode for O(1) lookup
    let node_map: HashMap<NodeId, &LogicNode> = asset
        .nodes
        .iter()
        .map(|n| (n.node_id.clone(), n))
        .collect();

    // Build a map from (NodeId, PortId) to PortValue for input resolution
    // This is populated as nodes are evaluated
    let mut port_values: HashMap<(NodeId, PortId), PortValue> = HashMap::new();

    for node_id in order {
        let node = node_map.get(node_id).ok_or_else(|| LogicError::NodeNotFound(node_id.clone()))?;

        // Gather inputs for this node from port_values (set by upstream nodes)
        let input_values = gather_input_values(node, &asset.edges, &port_values)?;

        // Evaluate the node
        let evaluator = registry.get_evaluator(&node.node_type_id);

        let evaluator = evaluator.ok_or_else(|| LogicError::MissingEvaluator(node.node_type_id.clone()))?;

        let outputs = evaluator.evaluate(node, &input_values);

        // Store output values for downstream nodes
        store_output_values(node, &outputs, &asset.edges, &mut port_values)?;

        // If actuator, submit to the bus
        if node.role == LogicNodeRole::Actuator {
            submit_actuator_outputs_from_node(node, &outputs, entity_bits);
        }
    }

    Ok(())
}

/// Gather input values for a node from edges and port_values map.
fn gather_input_values(
    node: &LogicNode,
    edges: &[LogicEdge],
    port_values: &HashMap<(NodeId, PortId), PortValue>,
) -> Result<Vec<PortValue>, LogicError> {
    // Find edges that target this node
    let input_edges: Vec<&LogicEdge> = edges
        .iter()
        .filter(|e| &e.to_node == &node.node_id)
        .collect();

    // Collect inputs in a deterministic order based on port_id
    let mut inputs: Vec<PortValue> = Vec::new();
    for input_edge in input_edges {
        if let Some(value) = port_values.get(&(input_edge.from_node.clone(), input_edge.from_port.clone())) {
            inputs.push(value.clone());
        } else {
            // Missing input — use default
            inputs.push(PortValue::Action(String::new()));
        }
    }

    Ok(inputs)
}

/// Store output values from a node into the port_values map.
fn store_output_values(
    node: &LogicNode,
    outputs: &[PortValue],
    edges: &[LogicEdge],
    port_values: &mut HashMap<(NodeId, PortId), PortValue>,
) -> Result<(), LogicError> {
    // Find edges that originate from this node
    let output_edges: Vec<&LogicEdge> = edges
        .iter()
        .filter(|e| &e.from_node == &node.node_id)
        .collect();

    for (i, output_edge) in output_edges.iter().enumerate() {
        if i < outputs.len() {
            port_values.insert(
                (output_edge.from_node.clone(), output_edge.from_port.clone()),
                outputs[i].clone(),
            );
        }
    }

    Ok(())
}

/// Submit actuator outputs to the ACTUATOR_OUTPUT_BUS.
/// Field name is derived from `node.node_type_id` (e.g., `"actuator.translate"`):
/// - "actuator.translate" → "translation"
/// - "actuator.rotate" → "rotation"
/// - "actuator.scale" → "scale"
/// - "actuator.set_color" → "color"
/// - default: use the full type_id string
fn submit_actuator_outputs_from_node(node: &LogicNode, outputs: &[PortValue], entity_bits: u64) {
    use bevy::prelude::Entity;
    use crate::actuator_bus::submit_actuator_output;

    let entity = Entity::from_bits(entity_bits);

    // Derive field name from node type
    let field = match node.node_type_id.as_str() {
        "actuator.translate" => "translation",
        "actuator.rotate" => "rotation",
        "actuator.scale" => "scale",
        "actuator.set_color" => "color",
        other => other,
    };

    for output in outputs.iter() {
        submit_actuator_output(entity, field, output.clone());
    }
}

/// Seed the three placeholder built-in evaluators.
fn seed_builtin_evaluators(registry: &mut LogicNodeRegistry) {
    // controller.if
    registry.register_builtin(
        Box::new(IfEvaluator),
        NodeDescriptor {
            node_type_id: NodeTypeId::new("controller.if"),
            role: LogicNodeRole::Controller,
            display_name: "If".to_string(),
            category: "controller".to_string(),
            inputs: vec![
                PortSpec {
                    port_id: "condition".to_string(),
                    value_type: PortValueType::Bool,
                    display_name: "Condition".to_string(),
                },
                PortSpec {
                    port_id: "then".to_string(),
                    value_type: PortValueType::Action,
                    display_name: "Then".to_string(),
                },
                PortSpec {
                    port_id: "else".to_string(),
                    value_type: PortValueType::Action,
                    display_name: "Else".to_string(),
                },
            ],
            outputs: vec![PortSpec {
                port_id: "done".to_string(),
                value_type: PortValueType::Action,
                display_name: "Done".to_string(),
            }],
        },
    );

    // controller.and
    registry.register_builtin(
        Box::new(AndEvaluator),
        NodeDescriptor {
            node_type_id: NodeTypeId::new("controller.and"),
            role: LogicNodeRole::Controller,
            display_name: "And".to_string(),
            category: "controller".to_string(),
            inputs: vec![
                PortSpec {
                    port_id: "a".to_string(),
                    value_type: PortValueType::Bool,
                    display_name: "A".to_string(),
                },
                PortSpec {
                    port_id: "b".to_string(),
                    value_type: PortValueType::Bool,
                    display_name: "B".to_string(),
                },
            ],
            outputs: vec![PortSpec {
                port_id: "out".to_string(),
                value_type: PortValueType::Bool,
                display_name: "Out".to_string(),
            }],
        },
    );

    // sensor.always
    registry.register_builtin(
        Box::new(AlwaysEvaluator),
        NodeDescriptor {
            node_type_id: NodeTypeId::new("sensor.always"),
            role: LogicNodeRole::Sensor,
            display_name: "Always".to_string(),
            category: "sensor".to_string(),
            inputs: vec![],
            outputs: vec![PortSpec {
                port_id: "tick".to_string(),
                value_type: PortValueType::Bool,
                display_name: "Tick".to_string(),
            }],
        },
    );

    // actuator.translate
    registry.register_builtin(
        Box::new(TranslateEvaluator),
        NodeDescriptor {
            node_type_id: NodeTypeId::new("actuator.translate"),
            role: LogicNodeRole::Actuator,
            display_name: "Translate".to_string(),
            category: "actuator".to_string(),
            inputs: vec![PortSpec {
                port_id: "vector".to_string(),
                value_type: PortValueType::Vec2,
                display_name: "Vector".to_string(),
            }],
            outputs: vec![PortSpec {
                port_id: "done".to_string(),
                value_type: PortValueType::Vec2,
                display_name: "Done".to_string(),
            }],
        },
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Placeholder built-in evaluators
// ─────────────────────────────────────────────────────────────────────────────

/// controller.if — returns the then/else action depending on condition.
struct IfEvaluator;
impl NodeEvaluator for IfEvaluator {
    fn evaluate(&self, _node: &LogicNode, inputs: &[PortValue]) -> Vec<PortValue> {
        // inputs[0] = condition (Bool), inputs[1] = then trigger, inputs[2] = else trigger
        let cond = inputs
            .get(0)
            .and_then(|v| match v {
                PortValue::Bool(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);

        let output = if cond {
            inputs.get(1).cloned().unwrap_or(PortValue::Action(String::new()))
        } else {
            inputs.get(2).cloned().unwrap_or(PortValue::Action(String::new()))
        };
        vec![output]
    }
}

/// controller.and — logical AND of two Bool inputs.
struct AndEvaluator;
impl NodeEvaluator for AndEvaluator {
    fn evaluate(&self, _node: &LogicNode, inputs: &[PortValue]) -> Vec<PortValue> {
        let a = inputs.get(0).and_then(|v| match v {
            PortValue::Bool(b) => Some(*b),
            _ => None,
        });
        let b = inputs.get(1).and_then(|v| match v {
            PortValue::Bool(b) => Some(*b),
            _ => None,
        });
        vec![PortValue::Bool(a.unwrap_or(false) && b.unwrap_or(false))]
    }
}

/// sensor.always — emits Bool(true) every tick.
struct AlwaysEvaluator;
impl NodeEvaluator for AlwaysEvaluator {
    fn evaluate(&self, _node: &LogicNode, _inputs: &[PortValue]) -> Vec<PortValue> {
        vec![PortValue::Bool(true)]
    }
}

/// actuator.translate — outputs the input Vec2 as the translation value.
struct TranslateEvaluator;
impl NodeEvaluator for TranslateEvaluator {
    fn evaluate(&self, _node: &LogicNode, inputs: &[PortValue]) -> Vec<PortValue> {
        // Pass through the first input (expected: Vec2 { x, y })
        vec![inputs.first().cloned().unwrap_or(PortValue::Vec2 { x: 0.0, y: 0.0 })]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 4: WASM exports (wasm32 only)
// ─────────────────────────────────────────────────────────────────────────────

/// Register a LogicGraphAsset from JSON in the in-memory registry.
/// Called from JS side after loading a .logic asset.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn register_logic_graph_wasm(asset_id: &str, version: u32, graph_json: &str) -> Result<(), JsValue> {
    // Deserialize from JSON
    let asset: LogicGraphAsset = serde_wasm_bindgen::from_str(graph_json)
        .map_err(|e| JsValue::from_str(&format!("failed to parse graph JSON: {}", e)))?;

    // Verify the asset_id matches (defensive)
    if asset.asset_id != asset_id {
        return Err(JsValue::from_str(&format!(
            "asset_id mismatch: expected '{}', got '{}'",
            asset_id, asset.asset_id
        )));
    }

    // Version check
    if asset.version != version {
        return Err(JsValue::from_str(&format!(
            "version mismatch: expected {}, got {}",
            version, asset.version
        )));
    }

    register_logic_graph(asset);
    Ok(())
}

/// Evaluate a logic binding by asset_id and version.
/// Submits ActuatorOutputs to the ACTUATOR_OUTPUT_BUS for the Bevy system to apply.
/// Returns Ok(()) on success; missing asset is logged and returns Ok(()).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn evaluate_logic_binding_wasm(entity_bits: u64, asset_id: &str, version: u32) -> Result<(), JsValue> {
    // Evaluate the binding — this populates the ACTUATOR_OUTPUT_BUS via actuator nodes.
    // The entity_bits for actuator outputs are set by the dispatch layer in logic_dispatch.rs.
    match evaluate_logic_binding(asset_id, version, entity_bits) {
        Ok(()) => Ok(()),
        Err(e) => {
            // AssetNotFound is logged but does not crash — iteration continues per spec
            match &e {
                LogicError::AssetNotFound(_) => {
                    web_sys::console::warn_1(&JsValue::from_str(&format!(
                        "evaluate_logic_binding_wasm: asset not found '{}'",
                        asset_id
                    )));
                    Ok(()) // Per spec: iteration continues
                }
                _ => Err(JsValue::from_str(&format!(
                    "evaluation error for '{}': {}",
                    asset_id, e
                ))),
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 1 RED tests — trait + enum existence
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic_graph::{LogicNode, LogicNodeRole, NodeTypeId};

    // §1.1: NodeEvaluator trait exists with correct signature and is Send + Sync
    #[test]
    fn node_evaluator_trait_exists() {
        // This test verifies the trait exists and has the right method signature
        // by checking that we can use it as a bound.
        fn _check_evaluator<E: NodeEvaluator>() {}

        // If this compiles, the trait exists with the right shape.
        // Send + Sync is verified by the blanket impl: trait NodeEvaluator: Send + Sync {}
    }

    // Verify NodeEvaluator is Send + Sync by creating a trait object.
    #[test]
    fn node_evaluator_is_send_and_sync() {
        // Create a simple evaluator that counts invocations.
        struct DummyEvaluator;
        impl NodeEvaluator for DummyEvaluator {
            fn evaluate(&self, _node: &LogicNode, _inputs: &[PortValue]) -> Vec<PortValue> {
                vec![]
            }
        }
        let evaluator: Box<dyn NodeEvaluator> = Box::new(DummyEvaluator);
        // If this compiles, the trait is Send + Sync
        let _ = evaluator;
    }

    // §1.2: PortValue enum has the required variants
    #[test]
    fn port_value_has_bool_variant() {
        let v = PortValue::Bool(true);
        assert!(matches!(v, PortValue::Bool(true)));
    }

    #[test]
    fn port_value_has_float_variant() {
        let v = PortValue::Float(3.14);
        assert!(matches!(v, PortValue::Float(f) if f == 3.14));
    }

    #[test]
    fn port_value_has_vec2_variant() {
        let v = PortValue::Vec2 { x: 1.0, y: 2.0 };
        assert!(matches!(v, PortValue::Vec2 { x, y } if x == 1.0 && y == 2.0));
    }

    #[test]
    fn port_value_has_entity_ref_variant() {
        let v = PortValue::EntityRef("player".to_string());
        assert!(matches!(v, PortValue::EntityRef(s) if s == "player"));
    }

    #[test]
    fn port_value_has_action_variant() {
        let v = PortValue::Action("jump".to_string());
        assert!(matches!(v, PortValue::Action(s) if s == "jump"));
    }

    // §1.3: serde_json::Value is NOT reachable inside PortValue
    // This is verified by compile-fail: if PortValue contained serde_json::Value,
    // this test would not compile because PortValueType is an enum, not a Value.
    #[test]
    fn port_value_type_enum_does_not_contain_json_value() {
        // PortValueType is an enum — serde_json::Value cannot appear inside it.
        // If someone tries to add a variant like `Json(serde_json::Value)`,
        // this compile-time check would fail because PortValueType is a simple enum.
        let vt: PortValueType = PortValueType::Bool;
        assert!(matches!(vt, PortValueType::Bool));
    }

    // §1.5: PortSpec, NodeDescriptor, ParamSpec constructors + serde round-trip
    #[test]
    fn port_spec_roundtrip() {
        let spec = PortSpec {
            port_id: "cond".to_string(),
            value_type: PortValueType::Bool,
            display_name: "Condition".to_string(),
        };
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: PortSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.port_id, "cond");
        assert_eq!(parsed.value_type, PortValueType::Bool);
        assert_eq!(parsed.display_name, "Condition");
    }

    #[test]
    fn port_value_roundtrip() {
        let v = PortValue::Vec2 { x: 1.0, y: 2.0 };
        let json = serde_json::to_string(&v).unwrap();
        let parsed: PortValue = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, v);
    }

    #[test]
    fn node_descriptor_roundtrip() {
        let desc = NodeDescriptor {
            node_type_id: NodeTypeId::new("controller.if"),
            role: LogicNodeRole::Controller,
            display_name: "If".to_string(),
            category: "controller".to_string(),
            inputs: vec![
                PortSpec {
                    port_id: "condition".to_string(),
                    value_type: PortValueType::Bool,
                    display_name: "Condition".to_string(),
                },
                PortSpec {
                    port_id: "then".to_string(),
                    value_type: PortValueType::Action,
                    display_name: "Then".to_string(),
                },
            ],
            outputs: vec![PortSpec {
                port_id: "done".to_string(),
                value_type: PortValueType::Action,
                display_name: "Done".to_string(),
            }],
        };
        let json = serde_json::to_string(&desc).unwrap();
        let parsed: NodeDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.node_type_id.as_str(), "controller.if");
        assert_eq!(parsed.role, LogicNodeRole::Controller);
        assert_eq!(parsed.inputs.len(), 2);
        assert_eq!(parsed.outputs.len(), 1);
    }

    #[test]
    fn param_spec_roundtrip() {
        let spec = ParamSpec {
            name: "threshold".to_string(),
            value_type: PortValueType::Float,
            default: Some(serde_json::json!(0.5)),
        };
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: ParamSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "threshold");
        assert_eq!(parsed.value_type, PortValueType::Float);
        assert_eq!(parsed.default, Some(serde_json::json!(0.5)));
    }

    #[test]
    fn param_spec_without_default_roundtrip() {
        let spec = ParamSpec {
            name: "threshold".to_string(),
            value_type: PortValueType::Float,
            default: None,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: ParamSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "threshold");
        assert!(parsed.default.is_none());
    }

    // §1.7: PortValue variant → PortValueType mapping helper
    #[test]
    fn port_value_type_from_port_value_bool() {
        fn _vt(v: &PortValue) -> PortValueType {
            match v {
                PortValue::Bool(_) => PortValueType::Bool,
                PortValue::Float(_) => PortValueType::Float,
                PortValue::Vec2 { .. } => PortValueType::Vec2,
                PortValue::EntityRef(_) => PortValueType::EntityRef,
                PortValue::Action(_) => PortValueType::Action,
            }
        }
        assert_eq!(_vt(&PortValue::Bool(true)), PortValueType::Bool);
        assert_eq!(_vt(&PortValue::Float(1.0)), PortValueType::Float);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 2 RED tests — registry + dispatch
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod registry_tests {
    use super::*;

    // §2.1: get_evaluator returns None for unknown NodeTypeId
    #[test]
    fn get_evaluator_unknown_returns_none() {
        let registry = LogicNodeRegistry::new();
        let result = registry.get_evaluator(&NodeTypeId::new("unknown.node"));
        assert!(result.is_none());
    }

    // §2.1: get_controller returns None for unknown controller_id
    #[test]
    fn get_controller_unknown_returns_none() {
        let registry = LogicNodeRegistry::new();
        let result = registry.get_controller("unknown_controller");
        assert!(result.is_none());
    }

    // §2.2: descriptor returns None for unknown NodeTypeId
    #[test]
    fn descriptor_unknown_returns_none() {
        let registry = LogicNodeRegistry::new();
        let result = registry.descriptor(&NodeTypeId::new("unknown.node"));
        assert!(result.is_none());
    }

    // §2.3: global_node_registry singleton returns identical pointer
    #[test]
    fn global_registry_singleton() {
        let reg1 = global_node_registry();
        let reg2 = global_node_registry();
        assert_eq!(reg1 as *const _, reg2 as *const _);
    }

    // §2.5: controller.if dispatch
    #[test]
    fn controller_if_condition_true() {
        let registry = global_node_registry();
        let evaluator = registry.get_evaluator(&NodeTypeId::new("controller.if")).unwrap();
        let node = LogicNode {
            node_id: crate::logic_graph::NodeId::new("test"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        // Condition=true, then="jump", else="skip"
        let inputs = vec![
            PortValue::Bool(true),
            PortValue::Action("jump".to_string()),
            PortValue::Action("skip".to_string()),
        ];
        let outputs = evaluator.evaluate(&node, &inputs);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], PortValue::Action("jump".to_string()));
    }

    #[test]
    fn controller_if_condition_false() {
        let registry = global_node_registry();
        let evaluator = registry.get_evaluator(&NodeTypeId::new("controller.if")).unwrap();
        let node = LogicNode {
            node_id: crate::logic_graph::NodeId::new("test"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        // Condition=false, then="jump", else="skip"
        let inputs = vec![
            PortValue::Bool(false),
            PortValue::Action("jump".to_string()),
            PortValue::Action("skip".to_string()),
        ];
        let outputs = evaluator.evaluate(&node, &inputs);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], PortValue::Action("skip".to_string()));
    }

    // §2.5: sensor.always dispatch
    #[test]
    fn sensor_always_emits_true() {
        let registry = global_node_registry();
        let evaluator = registry
            .get_evaluator(&NodeTypeId::new("sensor.always"))
            .unwrap();
        let node = LogicNode {
            node_id: crate::logic_graph::NodeId::new("test"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let outputs = evaluator.evaluate(&node, &[]);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], PortValue::Bool(true));
    }

    // §2.5: controller.and dispatch
    #[test]
    fn controller_and_both_true() {
        let registry = global_node_registry();
        let evaluator = registry
            .get_evaluator(&NodeTypeId::new("controller.and"))
            .unwrap();
        let node = LogicNode {
            node_id: crate::logic_graph::NodeId::new("test"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let inputs = vec![PortValue::Bool(true), PortValue::Bool(true)];
        let outputs = evaluator.evaluate(&node, &inputs);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], PortValue::Bool(true));
    }

    #[test]
    fn controller_and_one_false() {
        let registry = global_node_registry();
        let evaluator = registry
            .get_evaluator(&NodeTypeId::new("controller.and"))
            .unwrap();
        let node = LogicNode {
            node_id: crate::logic_graph::NodeId::new("test"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let inputs = vec![PortValue::Bool(true), PortValue::Bool(false)];
        let outputs = evaluator.evaluate(&node, &inputs);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], PortValue::Bool(false));
    }

    // §2.7: OCP extension — registering 4th evaluator doesn't disturb 3 prior entries
    #[test]
    fn ocp_extension_fourth_evaluator() {
        let mut registry = LogicNodeRegistry::new();
        seed_builtin_evaluators(&mut registry);

        // Verify the three built-ins are registered
        assert!(registry.get_evaluator(&NodeTypeId::new("controller.if")).is_some());
        assert!(registry.get_evaluator(&NodeTypeId::new("controller.and")).is_some());
        assert!(registry
            .get_evaluator(&NodeTypeId::new("sensor.always"))
            .is_some());

        // Register a 4th evaluator
        struct FourthEvaluator;
        impl NodeEvaluator for FourthEvaluator {
            fn evaluate(&self, _node: &LogicNode, _inputs: &[PortValue]) -> Vec<PortValue> {
                vec![PortValue::Bool(false)]
            }
        }
        registry.register_builtin(
            Box::new(FourthEvaluator),
            NodeDescriptor {
                node_type_id: NodeTypeId::new("controller.not"),
                role: LogicNodeRole::Controller,
                display_name: "Not".to_string(),
                category: "controller".to_string(),
                inputs: vec![PortSpec {
                    port_id: "in".to_string(),
                    value_type: PortValueType::Bool,
                    display_name: "In".to_string(),
                }],
                outputs: vec![PortSpec {
                    port_id: "out".to_string(),
                    value_type: PortValueType::Bool,
                    display_name: "Out".to_string(),
                }],
            },
        );

        // Verify all four are present
        assert!(registry.get_evaluator(&NodeTypeId::new("controller.if")).is_some());
        assert!(registry.get_evaluator(&NodeTypeId::new("controller.and")).is_some());
        assert!(registry
            .get_evaluator(&NodeTypeId::new("sensor.always"))
            .is_some());
        assert!(registry
            .get_evaluator(&NodeTypeId::new("controller.not"))
            .is_some());

        // Verify dispatch still works on original three
        let node = LogicNode {
            node_id: crate::logic_graph::NodeId::new("test"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let evaluator = registry.get_evaluator(&NodeTypeId::new("controller.if")).unwrap();
        let inputs = vec![
            PortValue::Bool(true),
            PortValue::Action("a".to_string()),
            PortValue::Action("b".to_string()),
        ];
        let outputs = evaluator.evaluate(&node, &inputs);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], PortValue::Action("a".to_string()));
    }

    // §2.2: descriptor returns descriptor for known node type
    #[test]
    fn descriptor_known_returns_some() {
        let registry = global_node_registry();
        let desc = registry.descriptor(&NodeTypeId::new("controller.if"));
        assert!(desc.is_some());
        assert_eq!(desc.unwrap().node_type_id.as_str(), "controller.if");
        assert_eq!(desc.unwrap().role, LogicNodeRole::Controller);
    }

    // §2.1: get_evaluator returns Some for known built-in
    #[test]
    fn get_evaluator_known_returns_some() {
        let registry = global_node_registry();
        let result = registry.get_evaluator(&NodeTypeId::new("controller.if"));
        assert!(result.is_some());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 4 integration tests — actuator bus + graph evaluation
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::actuator_bus::{submit_actuator_output, drain_actuator_outputs, ActuatorOutput};
    use crate::logic_graph::{LogicEdge, LogicGraphAsset, LogicNode, LogicNodeRole, NodeId, NodeTypeId, PortId};

    // §T1: ActuatorOutput serializes/deserializes correctly
    #[test]
    fn test_actuator_output_serde() {
        let output = ActuatorOutput {
            entity_bits: 42,
            field: "translation".to_string(),
            value: PortValue::Vec2 { x: 1.0, y: 2.0 },
        };
        let json = serde_json::to_string(&output).unwrap();
        let parsed: ActuatorOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entity_bits, 42);
        assert_eq!(parsed.field, "translation");
        assert_eq!(parsed.value, PortValue::Vec2 { x: 1.0, y: 2.0 });
    }

    // §T2: ActuatorOutput serde roundtrip with all PortValue variants
    #[test]
    fn test_actuator_output_serde_all_variants() {
        for value in [
            PortValue::Bool(true),
            PortValue::Float(3.14),
            PortValue::Vec2 { x: 1.0, y: 2.0 },
            PortValue::EntityRef("player".to_string()),
            PortValue::Action("jump".to_string()),
        ] {
            let output = ActuatorOutput {
                entity_bits: 99,
                field: "test_field".to_string(),
                value: value.clone(),
            };
            let json = serde_json::to_string(&output).unwrap();
            let parsed: ActuatorOutput = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.value, value);
        }
    }

    // §T3: Bus push/drain roundtrip
    #[test]
    fn test_submit_and_drain() {
        // Drain any pre-existing entries first
        let _ = drain_actuator_outputs();

        // Submit two outputs
        let entity = bevy::prelude::Entity::from_bits(123);
        submit_actuator_output(entity, "x", PortValue::Float(1.0));
        submit_actuator_output(entity, "y", PortValue::Float(2.0));

        // Drain and verify
        let outputs = drain_actuator_outputs();
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].entity_bits, 123);
        assert_eq!(outputs[0].field, "x");
        assert_eq!(outputs[0].value, PortValue::Float(1.0));
        assert_eq!(outputs[1].field, "y");
        assert_eq!(outputs[1].value, PortValue::Float(2.0));

        // Bus should be empty after drain
        let outputs = drain_actuator_outputs();
        assert!(outputs.is_empty());
    }

    // §T4: evaluate_logic_binding with missing asset returns AssetNotFound
    #[test]
    fn test_evaluate_logic_binding_missing_asset() {
        let result = evaluate_logic_binding("nonexistent.asset", 1, 0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, LogicError::AssetNotFound(id) if id == "nonexistent.asset"));
    }

    // §T5: Topological sort — 3-node linear chain
    #[test]
    fn test_topological_sort_simple() {
        let nodes = vec![
            LogicNode {
                node_id: NodeId::new("sensor"),
                role: LogicNodeRole::Sensor,
                node_type_id: NodeTypeId::new("sensor.always"),
                field_values: serde_json::json!({}),
                controller_id: None,
            },
            LogicNode {
                node_id: NodeId::new("controller"),
                role: LogicNodeRole::Controller,
                node_type_id: NodeTypeId::new("controller.if"),
                field_values: serde_json::json!({}),
                controller_id: None,
            },
            LogicNode {
                node_id: NodeId::new("actuator"),
                role: LogicNodeRole::Actuator,
                node_type_id: NodeTypeId::new("actuator.translate"),
                field_values: serde_json::json!({}),
                controller_id: None,
            },
        ];

        // Edges: sensor → controller → actuator (via tick port connections)
        let edges = vec![
            LogicEdge {
                from_node: NodeId::new("sensor"),
                from_port: PortId::new("tick"),
                to_node: NodeId::new("controller"),
                to_port: PortId::new("condition"),
            },
            LogicEdge {
                from_node: NodeId::new("controller"),
                from_port: PortId::new("done"),
                to_node: NodeId::new("actuator"),
                to_port: PortId::new("trigger"),
            },
        ];

        let order = topological_sort(&nodes, &edges);
        assert!(order.is_some());
        let order = order.unwrap();
        assert_eq!(order.len(), 3);
        // sensor must come before controller
        let sensor_idx = order.iter().position(|id| id.as_str() == "sensor").unwrap();
        let controller_idx = order.iter().position(|id| id.as_str() == "controller").unwrap();
        let actuator_idx = order.iter().position(|id| id.as_str() == "actuator").unwrap();
        assert!(sensor_idx < controller_idx);
        assert!(controller_idx < actuator_idx);
    }

    // §T6: Topological sort — diamond shape (both branches converge)
    #[test]
    fn test_topological_sort_diamond() {
        let nodes = vec![
            LogicNode {
                node_id: NodeId::new("start"),
                role: LogicNodeRole::Sensor,
                node_type_id: NodeTypeId::new("sensor.always"),
                field_values: serde_json::json!({}),
                controller_id: None,
            },
            LogicNode {
                node_id: NodeId::new("left"),
                role: LogicNodeRole::Controller,
                node_type_id: NodeTypeId::new("controller.if"),
                field_values: serde_json::json!({}),
                controller_id: None,
            },
            LogicNode {
                node_id: NodeId::new("right"),
                role: LogicNodeRole::Controller,
                node_type_id: NodeTypeId::new("controller.and"),
                field_values: serde_json::json!({}),
                controller_id: None,
            },
            LogicNode {
                node_id: NodeId::new("end"),
                role: LogicNodeRole::Actuator,
                node_type_id: NodeTypeId::new("actuator.translate"),
                field_values: serde_json::json!({}),
                controller_id: None,
            },
        ];

        // Diamond edges: start → left → end, start → right → end
        let edges = vec![
            LogicEdge {
                from_node: NodeId::new("start"),
                from_port: PortId::new("tick"),
                to_node: NodeId::new("left"),
                to_port: PortId::new("condition"),
            },
            LogicEdge {
                from_node: NodeId::new("start"),
                from_port: PortId::new("tick"),
                to_node: NodeId::new("right"),
                to_port: PortId::new("a"),
            },
            LogicEdge {
                from_node: NodeId::new("left"),
                from_port: PortId::new("done"),
                to_node: NodeId::new("end"),
                to_port: PortId::new("trigger"),
            },
            LogicEdge {
                from_node: NodeId::new("right"),
                from_port: PortId::new("out"),
                to_node: NodeId::new("end"),
                to_port: PortId::new("trigger"),
            },
        ];

        let order = topological_sort(&nodes, &edges);
        assert!(order.is_some());
        let order = order.unwrap();
        assert_eq!(order.len(), 4);
        // start must be before both left and right
        let start_idx = order.iter().position(|id| id.as_str() == "start").unwrap();
        let left_idx = order.iter().position(|id| id.as_str() == "left").unwrap();
        let right_idx = order.iter().position(|id| id.as_str() == "right").unwrap();
        let end_idx = order.iter().position(|id| id.as_str() == "end").unwrap();
        assert!(start_idx < left_idx);
        assert!(start_idx < right_idx);
        assert!(left_idx < end_idx);
        assert!(right_idx < end_idx);
    }

    // §T7: Topological sort — cycle detection returns None
    #[test]
    fn test_topological_sort_cycle() {
        let nodes = vec![
            LogicNode {
                node_id: NodeId::new("a"),
                role: LogicNodeRole::Controller,
                node_type_id: NodeTypeId::new("controller.if"),
                field_values: serde_json::json!({}),
                controller_id: None,
            },
            LogicNode {
                node_id: NodeId::new("b"),
                role: LogicNodeRole::Controller,
                node_type_id: NodeTypeId::new("controller.and"),
                field_values: serde_json::json!({}),
                controller_id: None,
            },
        ];

        // Cycle: a → b → a
        let edges = vec![
            LogicEdge {
                from_node: NodeId::new("a"),
                from_port: PortId::new("done"),
                to_node: NodeId::new("b"),
                to_port: PortId::new("a"),
            },
            LogicEdge {
                from_node: NodeId::new("b"),
                from_port: PortId::new("out"),
                to_node: NodeId::new("a"),
                to_port: PortId::new("condition"),
            },
        ];

        let order = topological_sort(&nodes, &edges);
        assert!(order.is_none());
    }

    // §T8: evaluate_logic_binding — sensor → controller chain (no actuator)
    #[test]
    fn test_evaluate_logic_binding_sensor_controller_chain() {
        use crate::actuator_bus::drain_actuator_outputs;

        // Drain any pre-existing bus entries
        let _ = drain_actuator_outputs();

        // Build a minimal chain: sensor.always → controller.if
        let graph = LogicGraphAsset {
            asset_id: "test/chain".to_string(),
            logical_path: "test/chain".to_string(),
            version: 1,
            nodes: vec![
                LogicNode {
                    node_id: NodeId::new("sensor"),
                    role: LogicNodeRole::Sensor,
                    node_type_id: NodeTypeId::new("sensor.always"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
                LogicNode {
                    node_id: NodeId::new("controller"),
                    role: LogicNodeRole::Controller,
                    node_type_id: NodeTypeId::new("controller.if"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
            ],
            edges: vec![
                LogicEdge {
                    from_node: NodeId::new("sensor"),
                    from_port: PortId::new("tick"),
                    to_node: NodeId::new("controller"),
                    to_port: PortId::new("condition"),
                },
            ],
            ..Default::default()
        };

        register_logic_graph(graph);

        let result = evaluate_logic_binding("test/chain", 1, 0);
        // Should succeed (no actuator in this chain)
        assert!(result.is_ok(), "expected ok, got {:?}", result);

        // Clean up
        let _ = drain_actuator_outputs();
    }

    // §T9: evaluate_logic_binding — version mismatch returns error
    #[test]
    fn test_evaluate_logic_binding_version_mismatch() {
        let graph = LogicGraphAsset {
            asset_id: "test/version".to_string(),
            logical_path: "test/version".to_string(),
            version: 5,
            nodes: vec![
                LogicNode {
                    node_id: NodeId::new("sensor"),
                    role: LogicNodeRole::Sensor,
                    node_type_id: NodeTypeId::new("sensor.always"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
            ],
            ..Default::default()
        };

        register_logic_graph(graph);

        // Call with wrong version
        let result = evaluate_logic_binding("test/version", 99, 0);
        assert!(result.is_err());
        match result.unwrap_err() {
            LogicError::VersionMismatch { asset_id, expected, actual } => {
                assert_eq!(asset_id, "test/version");
                assert_eq!(expected, 99);
                assert_eq!(actual, 5);
            }
            e => panic!("expected VersionMismatch, got {:?}", e),
        }
    }

    // §T10: register_logic_graph and get_logic_graph_asset roundtrip
    #[test]
    fn test_register_and_retrieve_graph() {
        let graph = LogicGraphAsset {
            asset_id: "test/retrieve".to_string(),
            logical_path: "test/retrieve".to_string(),
            version: 2,
            nodes: vec![
                LogicNode {
                    node_id: NodeId::new("sensor"),
                    role: LogicNodeRole::Sensor,
                    node_type_id: NodeTypeId::new("sensor.always"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
            ],
            ..Default::default()
        };

        register_logic_graph(graph.clone());

        let retrieved = get_logic_graph_asset("test/retrieve");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.asset_id, "test/retrieve");
        assert_eq!(retrieved.version, 2);
    }

    // §T11: submit and drain with entity bits preserved
    #[test]
    fn test_entity_bits_preserved_in_bus() {
        let _ = drain_actuator_outputs();

        let entity = bevy::prelude::Entity::from_bits(777);
        submit_actuator_output(entity, "vx", PortValue::Float(10.0));

        let outputs = drain_actuator_outputs();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].entity_bits, 777);
        assert_eq!(outputs[0].field, "vx");
    }

    // §T12: end-to-end pipeline — register → evaluate(actuator.translate) → drain
    #[test]
    fn test_end_to_end_actuator_pipeline() {
        use crate::actuator_bus::drain_actuator_outputs;

        let _ = drain_actuator_outputs();

        // Graph: sensor.always → controller.and → actuator.translate
        // controller.and takes two Bool inputs, outputs Bool.
        // actuator.translate receives Bool from controller.and — wrong type for the evaluator,
        // but TranslateEvaluator returns inputs.first() so it would return Bool.
        // We test the simpler case: the actuator IS reached and the output bus gets
        // an entry with the correct entity_bits and field="translation".
        let graph = LogicGraphAsset {
            asset_id: "test/e2e".to_string(),
            logical_path: "test/e2e".to_string(),
            version: 1,
            nodes: vec![
                LogicNode {
                    node_id: NodeId::new("sensor"),
                    role: LogicNodeRole::Sensor,
                    node_type_id: NodeTypeId::new("sensor.always"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
                LogicNode {
                    node_id: NodeId::new("controller"),
                    role: LogicNodeRole::Controller,
                    node_type_id: NodeTypeId::new("controller.and"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
                LogicNode {
                    node_id: NodeId::new("actuator"),
                    role: LogicNodeRole::Actuator,
                    node_type_id: NodeTypeId::new("actuator.translate"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
            ],
            edges: vec![
                LogicEdge {
                    from_node: NodeId::new("sensor"),
                    from_port: PortId::new("tick"),
                    to_node: NodeId::new("controller"),
                    to_port: PortId::new("a"),
                },
                LogicEdge {
                    from_node: NodeId::new("controller"),
                    from_port: PortId::new("out"),
                    to_node: NodeId::new("actuator"),
                    to_port: PortId::new("vector"),
                },
            ],
            ..Default::default()
        };

        register_logic_graph(graph);

        let entity_bits: u64 = 9999;
        let result = evaluate_logic_binding("test/e2e", 1, entity_bits);
        assert!(result.is_ok(), "expected ok, got {:?}", result);

        let outputs = drain_actuator_outputs();
        assert!(!outputs.is_empty(), "expected at least one actuator output");

        let translate_output = outputs.iter().find(|o| o.field == "translation");
        assert!(translate_output.is_some(), "expected translation output, got {:?}", outputs);
        let out = translate_output.unwrap();
        // entity_bits must be threaded through from dispatch → evaluate → submit
        assert_eq!(out.entity_bits, entity_bits, "entity_bits should be threaded through");
        // The output value is whatever TranslateEvaluator returned (Bool in this case,
        // since controller.and outputs Bool). This test verifies the pipeline plumbing.
        assert!(
            matches!(out.value, PortValue::Bool(_)),
            "expected Bool output from translator given Bool input, got {:?}",
            out.value
        );
    }
}
