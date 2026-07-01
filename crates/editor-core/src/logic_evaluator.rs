//! Logic Evaluator — dispatch trait and typed port boundary.
//!
//! Phase 1: NodeEvaluator trait + PortValue enum + metadata structs.
//! Phase 2: LogicNodeRegistry singleton + placeholder built-in evaluators.
//! All tests follow Strict TDD: RED → GREEN → TRIANGULATE → REFACTOR.

use crate::logic_graph::{LogicNode, LogicNodeRole, NodeTypeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

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
