//! Logic Command processor for Logic Graph Authoring mode.
//!
//! A separate command surface for mutating `LogicGraphAsset` documents.
//! Uses `NodeId` and `PortId` for node/edge identity.
//!
//! ## Design
//! - Mechanical inverse generation (mirrors `asset_command.rs`)
//! - `field_path: Vec<String>` for unambiguous field addressing
//! - `LogicOperationLog` mirrors `AssetOperationLog` for per-graph undo/redo
//!
//! ## Inverse table (design)
//! | Forward          | Inverse                              |
//! |------------------|--------------------------------------|
//! | `AddNode`       | `RemoveNode { node_id }`             |
//! | `RemoveNode`     | `AddNode { full captured node }`      |
//! | `ConnectPorts`   | `DisconnectPorts { full captured edge }` |
//! | `DisconnectPorts`| `ConnectPorts { from/to/from_port/to_port }` |
//! | `SetNodeField`   | `SetNodeField { old value at field_path }` |
//! | `Batch`          | `Batch { reversed inverses }`         |

use crate::logic_graph::{LogicEdge, LogicGraphAsset, LogicNode, LogicNodeRole, NodeId, NodeTypeId, PortId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─────────────────────────────────────────────────────────────────────────
// LogicCommand enum
// ─────────────────────────────────────────────────────────────────────────

/// Typed command enum for Logic Graph document mutations.
///
/// Uses `#[serde(tag = "type")]` so each variant serializes as
/// `{"type": "AddNode", ...}` — self-describing and extensible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum LogicCommand {
    /// Add a new node to the graph.
    AddNode {
        node_id: NodeId,
        role: LogicNodeRole,
        node_type_id: NodeTypeId,
        #[serde(default)]
        field_values: serde_json::Value,
        #[serde(default)]
        controller_id: Option<String>,
    },
    /// Remove a node from the graph.
    RemoveNode {
        node_id: NodeId,
    },
    /// Connect two nodes via their ports.
    ConnectPorts {
        from_node: NodeId,
        from_port: PortId,
        to_node: NodeId,
        to_port: PortId,
    },
    /// Disconnect two nodes (identified by their port endpoints).
    DisconnectPorts {
        from_node: NodeId,
        from_port: PortId,
        to_node: NodeId,
        to_port: PortId,
    },
    /// Update a field on a node.
    SetNodeField {
        node_id: NodeId,
        field_path: Vec<String>,
        value: serde_json::Value,
    },
    /// Group multiple commands into a single atomic history entry.
    Batch {
        label: String,
        commands: Vec<LogicCommand>,
    },
}

// ─────────────────────────────────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum LogicCommandError {
    #[error("node not found: {0:?}")]
    NodeNotFound(NodeId),

    #[error("duplicate node_id: {0:?}")]
    DuplicateNodeId(NodeId),

    #[error("edge not found: from_node={0:?} from_port={1:?} to_node={2:?} to_port={3:?}")]
    EdgeNotFound(NodeId, PortId, NodeId, PortId),

    #[error("batch failed at {index}: {source}")]
    BatchFailed {
        index: usize,
        #[source]
        source: Box<LogicCommandError>,
    },

    #[error("JSON error: {0}")]
    JsonError(String),

    /// Returned when a mutation is attempted on a built-in immutable recipe.
    #[error("built-in recipe '{0}' is immutable and cannot be modified")]
    RecipeImmutable(String),
}

impl From<serde_json::Error> for LogicCommandError {
    fn from(e: serde_json::Error) -> Self {
        LogicCommandError::JsonError(e.to_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────
// LogicProcessor
// ─────────────────────────────────────────────────────────────────────────

/// Find a mutable node by NodeId.
fn find_node_mut<'a>(
    doc: &'a mut LogicGraphAsset,
    node_id: &NodeId,
) -> Result<&'a mut LogicNode, LogicCommandError> {
    doc.nodes
        .iter_mut()
        .find(|n| n.node_id == *node_id)
        .ok_or_else(|| LogicCommandError::NodeNotFound(node_id.clone()))
}

/// Find a node by NodeId (immutable).
fn find_node<'a>(
    doc: &'a LogicGraphAsset,
    node_id: &NodeId,
) -> Result<&'a LogicNode, LogicCommandError> {
    doc.nodes
        .iter()
        .find(|n| n.node_id == *node_id)
        .ok_or_else(|| LogicCommandError::NodeNotFound(node_id.clone()))
}

/// Set a field at a `Vec<String>` path within a JSON object. Returns the old value.
///
/// Path navigation: split on segments, navigate to parent, set leaf.
pub fn set_field_path_vec(
    value: &mut serde_json::Value,
    path: &[String],
    new: serde_json::Value,
) -> Result<serde_json::Value, LogicCommandError> {
    if path.is_empty() {
        return Err(LogicCommandError::JsonError("Empty field path".to_string()));
    }
    let mut current = value;
    for part in &path[..path.len() - 1] {
        current = current
            .as_object_mut()
            .ok_or_else(|| LogicCommandError::JsonError(format!("Cannot navigate through non-object at '{}'", part)))?
            .get_mut(part)
            .ok_or_else(|| LogicCommandError::JsonError(format!("Field not found: '{}'", part)))?;
    }
    let leaf = path.last().unwrap();
    let obj = current
        .as_object_mut()
        .ok_or_else(|| LogicCommandError::JsonError(format!("Cannot set field on non-object at '{}'", leaf)))?;
    let old = obj
        .get(leaf)
        .ok_or_else(|| LogicCommandError::JsonError(format!("Field not found: '{}'", leaf)))?
        .clone();
    obj.insert(leaf.clone(), new);
    Ok(old)
}

/// Apply a LogicCommand to a LogicGraphAsset, returning the inverse command.
pub fn apply(
    doc: &mut LogicGraphAsset,
    cmd: &LogicCommand,
) -> Result<LogicCommand, LogicCommandError> {
    // Immutability guard: reject any mutation on a built-in recipe.
    // This mirrors the `is_builtin_type` + `CannotRegisterBuiltin` pattern in schema.rs.
    if doc.builtin {
        return Err(LogicCommandError::RecipeImmutable(doc.asset_id.clone()));
    }

    match cmd {
        LogicCommand::AddNode {
            node_id,
            role,
            node_type_id,
            field_values,
            controller_id,
        } => {
            // Check duplicate
            if doc.nodes.iter().any(|n| n.node_id == *node_id) {
                return Err(LogicCommandError::DuplicateNodeId(node_id.clone()));
            }
            doc.nodes.push(LogicNode {
                node_id: node_id.clone(),
                role: *role,
                node_type_id: node_type_id.clone(),
                field_values: field_values.clone(),
                controller_id: controller_id.clone(),
            });
            Ok(LogicCommand::RemoveNode {
                node_id: node_id.clone(),
            })
        }

        LogicCommand::RemoveNode { node_id } => {
            let pos = doc
                .nodes
                .iter()
                .position(|n| n.node_id == *node_id)
                .ok_or_else(|| LogicCommandError::NodeNotFound(node_id.clone()))?;
            let removed = doc.nodes.remove(pos);

            // Remove any edges connected to this node
            doc.edges.retain(|e| e.from_node != *node_id && e.to_node != *node_id);

            Ok(LogicCommand::AddNode {
                node_id: removed.node_id,
                role: removed.role,
                node_type_id: removed.node_type_id,
                field_values: removed.field_values,
                controller_id: removed.controller_id,
            })
        }

        LogicCommand::ConnectPorts {
            from_node,
            from_port,
            to_node,
            to_port,
        } => {
            // Reject if either node doesn't exist
            find_node(doc, from_node)?;
            find_node(doc, to_node)?;

            // Check for duplicate edge
            let is_duplicate = doc.edges.iter().any(|e| {
                e.from_node == *from_node
                    && e.from_port == *from_port
                    && e.to_node == *to_node
                    && e.to_port == *to_port
            });

            if !is_duplicate {
                doc.edges.push(LogicEdge {
                    from_node: from_node.clone(),
                    from_port: from_port.clone(),
                    to_node: to_node.clone(),
                    to_port: to_port.clone(),
                });
            }

            Ok(LogicCommand::DisconnectPorts {
                from_node: from_node.clone(),
                from_port: from_port.clone(),
                to_node: to_node.clone(),
                to_port: to_port.clone(),
            })
        }

        LogicCommand::DisconnectPorts {
            from_node,
            from_port,
            to_node,
            to_port,
        } => {
            // Reject if either node doesn't exist (unknown node)
            find_node(doc, from_node)?;
            find_node(doc, to_node)?;

            let pos = doc.edges.iter().position(|e| {
                e.from_node == *from_node
                    && e.from_port == *from_port
                    && e.to_node == *to_node
                    && e.to_port == *to_port
            });

            match pos {
                Some(p) => {
                    let removed = doc.edges.remove(p);
                    Ok(LogicCommand::ConnectPorts {
                        from_node: removed.from_node,
                        from_port: removed.from_port,
                        to_node: removed.to_node,
                        to_port: removed.to_port,
                    })
                }
                None => {
                    // Edge not found — return self as inverse (no-op)
                    Ok(LogicCommand::DisconnectPorts {
                        from_node: from_node.clone(),
                        from_port: from_port.clone(),
                        to_node: to_node.clone(),
                        to_port: to_port.clone(),
                    })
                }
            }
        }

        LogicCommand::SetNodeField {
            node_id,
            field_path,
            value,
        } => {
            let node = find_node_mut(doc, node_id)?;
            let old_value = set_field_path_vec(&mut node.field_values, field_path, value.clone())?;
            Ok(LogicCommand::SetNodeField {
                node_id: node_id.clone(),
                field_path: field_path.clone(),
                value: old_value,
            })
        }

        LogicCommand::Batch { label: _, commands } => {
            let mut inverses: Vec<LogicCommand> = Vec::new();
            for (i, c) in commands.iter().enumerate() {
                match apply(doc, c) {
                    Ok(inv) => inverses.push(inv),
                    Err(e) => {
                        // Rollback: apply inverses in reverse
                        for inv in inverses.iter().rev() {
                            let _ = apply(doc, inv);
                        }
                        return Err(LogicCommandError::BatchFailed {
                            index: i,
                            source: Box::new(e),
                        });
                    }
                }
            }
            inverses.reverse();
            Ok(LogicCommand::Batch {
                label: "inverse".to_string(),
                commands: inverses,
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// LogicOperationLog
// ─────────────────────────────────────────────────────────────────────────

/// Single entry in the logic operation log: forward command, inverse, and metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicLogEntry {
    pub forward: LogicCommand,
    pub inverse: LogicCommand,
}

/// Append-only history with cursor-based undo/redo for logic commands.
#[derive(Debug, Clone)]
pub struct LogicOperationLog {
    entries: Vec<LogicLogEntry>,
    cursor: isize,
    max_size: usize,
}

impl LogicOperationLog {
    /// Create a new empty log with default max size (1000 entries).
    pub fn new() -> Self {
        Self::with_max_size(1000)
    }

    /// Const constructor for use in `thread_local!` initializers.
    pub const fn new_const() -> Self {
        Self {
            entries: Vec::new(),
            cursor: -1,
            max_size: 1000,
        }
    }

    /// Create a new empty log with custom max size.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            cursor: -1,
            max_size,
        }
    }

    /// Record a forward command and its inverse after apply.
    pub fn record(&mut self, forward: &LogicCommand, inverse: LogicCommand) {
        // Truncate redo branch
        if self.cursor < self.entries.len() as isize - 1 {
            let keep = (self.cursor + 1) as usize;
            self.entries.truncate(keep);
        }
        self.entries.push(LogicLogEntry {
            forward: forward.clone(),
            inverse,
        });
        while self.entries.len() > self.max_size {
            self.entries.remove(0);
            self.cursor -= 1;
        }
        self.cursor = self.entries.len() as isize - 1;
    }

    /// Apply the inverse of the entry at the cursor, moving the cursor back.
    pub fn undo(&mut self, doc: &mut LogicGraphAsset) -> Result<(), LogicCommandError> {
        if !self.can_undo() {
            return Err(LogicCommandError::JsonError("Nothing to undo".to_string()));
        }
        let entry = &self.entries[self.cursor as usize];
        apply(doc, &entry.inverse)?;
        self.cursor -= 1;
        Ok(())
    }

    /// Apply the forward of the entry after the cursor, moving forward.
    pub fn redo(&mut self, doc: &mut LogicGraphAsset) -> Result<(), LogicCommandError> {
        if !self.can_redo() {
            return Err(LogicCommandError::JsonError("Nothing to redo".to_string()));
        }
        self.cursor += 1;
        let entry = &self.entries[self.cursor as usize];
        apply(doc, &entry.forward)?;
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.cursor >= 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor < self.entries.len() as isize - 1
    }

    pub fn get_log_size(&self) -> usize {
        self.entries.len()
    }

    pub fn get_cursor(&self) -> isize {
        self.cursor
    }

    /// Returns true if there are un-saved changes.
    pub fn is_dirty(&self) -> bool {
        self.cursor >= 0
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.cursor = -1;
    }
}

impl Default for LogicOperationLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic_graph::{LogicNodeRole, NodeTypeId};

    fn empty_graph() -> LogicGraphAsset {
        LogicGraphAsset {
            asset_id: "test_graph".to_string(),
            logical_path: "logic/test".to_string(),
            version: 1,
            ..Default::default()
        }
    }

    // ── AddNode tests ───────────────────────────────────────────────────────

    #[test]
    fn test_add_node_applies_and_inverts() {
        let mut doc = empty_graph();
        let cmd = LogicCommand::AddNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({"key": "Space"}),
            controller_id: None,
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.nodes.len(), 1);
        match inverse {
            LogicCommand::RemoveNode { node_id } => assert_eq!(node_id.as_str(), "node_a"),
            _ => panic!("Expected RemoveNode inverse"),
        }
    }

    #[test]
    fn test_add_node_duplicate_error() {
        let mut doc = empty_graph();
        let cmd = LogicCommand::AddNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        apply(&mut doc, &cmd).unwrap();
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::DuplicateNodeId(_))));
    }

    #[test]
    fn test_add_node_reapply_inverse_leaves_empty() {
        let mut doc = empty_graph();
        let cmd = LogicCommand::AddNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.nodes.len(), 1);
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.nodes.len(), 0);
    }

    // ── RemoveNode tests ────────────────────────────────────────────────────

    #[test]
    fn test_remove_node_inverse_contains_full_node() {
        let mut doc = empty_graph();
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({"key": "Space"}),
            controller_id: None,
        });

        let cmd = LogicCommand::RemoveNode {
            node_id: NodeId::new("node_a"),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.nodes.len(), 0);

        match inverse {
            LogicCommand::AddNode { node_id, role, node_type_id, field_values, controller_id } => {
                assert_eq!(node_id.as_str(), "node_a");
                assert_eq!(role, LogicNodeRole::Sensor);
                assert_eq!(node_type_id.as_str(), "sensor.key_down");
                assert_eq!(field_values["key"], "Space");
                assert!(controller_id.is_none());
            }
            _ => panic!("Expected AddNode inverse"),
        }
    }

    #[test]
    fn test_remove_node_unknown_error() {
        let mut doc = empty_graph();
        let cmd = LogicCommand::RemoveNode {
            node_id: NodeId::new("node_unknown"),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::NodeNotFound(_))));
    }

    #[test]
    fn test_remove_node_removes_connected_edges() {
        let mut doc = empty_graph();
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("actuator.jump"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });
        doc.edges.push(LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("in"),
        });

        let cmd = LogicCommand::RemoveNode {
            node_id: NodeId::new("node_a"),
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(doc.edges.len(), 0);
    }

    // ── ConnectPorts / DisconnectPorts tests ─────────────────────────────────

    #[test]
    fn test_connect_ports_applies_and_inverts() {
        let mut doc = empty_graph();
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("actuator.jump"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });

        let cmd = LogicCommand::ConnectPorts {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("in"),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.edges.len(), 1);

        match inverse {
            LogicCommand::DisconnectPorts { from_node, from_port, to_node, to_port } => {
                assert_eq!(from_node.as_str(), "node_a");
                assert_eq!(from_port.as_str(), "out");
                assert_eq!(to_node.as_str(), "node_b");
                assert_eq!(to_port.as_str(), "in");
            }
            _ => panic!("Expected DisconnectPorts inverse"),
        }
    }

    #[test]
    fn test_connect_ports_deduplicates() {
        let mut doc = empty_graph();
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("actuator.jump"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });

        let cmd = LogicCommand::ConnectPorts {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("in"),
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.edges.len(), 1);

        // Connect again — should not add duplicate
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.edges.len(), 1);
    }

    #[test]
    fn test_connect_ports_unknown_node_error() {
        let mut doc = empty_graph();
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });

        let cmd = LogicCommand::ConnectPorts {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_unknown"),
            to_port: PortId::new("in"),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::NodeNotFound(_))));
    }

    #[test]
    fn test_disconnect_ports_removes_edge() {
        let mut doc = empty_graph();
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_b"),
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("actuator.jump"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });
        doc.edges.push(LogicEdge {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("in"),
        });

        let cmd = LogicCommand::DisconnectPorts {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_b"),
            to_port: PortId::new("in"),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.edges.len(), 0);

        match inverse {
            LogicCommand::ConnectPorts { from_node, from_port, to_node, to_port } => {
                assert_eq!(from_node.as_str(), "node_a");
                assert_eq!(to_node.as_str(), "node_b");
            }
            _ => panic!("Expected ConnectPorts inverse"),
        }
    }

    #[test]
    fn test_disconnect_ports_unknown_node_error() {
        let mut doc = empty_graph();
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        });

        let cmd = LogicCommand::DisconnectPorts {
            from_node: NodeId::new("node_a"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_unknown"),
            to_port: PortId::new("in"),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::NodeNotFound(_))));
    }

    // ── SetNodeField tests ──────────────────────────────────────────────────

    #[test]
    fn test_set_node_field_inverse_restores_old_value() {
        let mut doc = empty_graph();
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({"threshold": 0.5}),
            controller_id: None,
        });

        let cmd = LogicCommand::SetNodeField {
            node_id: NodeId::new("node_a"),
            field_path: vec!["threshold".to_string()],
            value: serde_json::json!(0.9),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.nodes[0].field_values["threshold"], serde_json::json!(0.9));

        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.nodes[0].field_values["threshold"], serde_json::json!(0.5));
    }

    #[test]
    fn test_set_node_field_nested_path() {
        let mut doc = empty_graph();
        doc.nodes.push(LogicNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({"config": {"threshold": 0.5, "enabled": true}}),
            controller_id: None,
        });

        let cmd = LogicCommand::SetNodeField {
            node_id: NodeId::new("node_a"),
            field_path: vec!["config".to_string(), "threshold".to_string()],
            value: serde_json::json!(0.8),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.nodes[0].field_values["config"]["threshold"], serde_json::json!(0.8));

        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.nodes[0].field_values["config"]["threshold"], serde_json::json!(0.5));
    }

    #[test]
    fn test_set_node_field_unknown_node_error() {
        let mut doc = empty_graph();
        let cmd = LogicCommand::SetNodeField {
            node_id: NodeId::new("node_unknown"),
            field_path: vec!["threshold".to_string()],
            value: serde_json::json!(0.9),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::NodeNotFound(_))));
    }

    // ── Batch tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_batch_inverse_reverses_order() {
        let mut doc = empty_graph();
        let cmd = LogicCommand::Batch {
            label: "test".to_string(),
            commands: vec![
                LogicCommand::AddNode {
                    node_id: NodeId::new("node_a"),
                    role: LogicNodeRole::Sensor,
                    node_type_id: NodeTypeId::new("sensor.key_down"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
                LogicCommand::AddNode {
                    node_id: NodeId::new("node_b"),
                    role: LogicNodeRole::Actuator,
                    node_type_id: NodeTypeId::new("actuator.jump"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
            ],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.nodes.len(), 2);

        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.nodes.len(), 0);
    }

    // ── set_field_path_vec tests ─────────────────────────────────────────────

    #[test]
    fn test_set_field_path_vec_simple() {
        let mut v = serde_json::json!({"a": 1});
        let old = set_field_path_vec(&mut v, &["a".to_string()], serde_json::json!(99)).unwrap();
        assert_eq!(old, serde_json::json!(1));
        assert_eq!(v["a"], serde_json::json!(99));
    }

    #[test]
    fn test_set_field_path_vec_nested() {
        let mut v = serde_json::json!({"a": {"b": {"c": 1}}});
        let old = set_field_path_vec(&mut v, &["a".to_string(), "b".to_string(), "c".to_string()], serde_json::json!(42)).unwrap();
        assert_eq!(old, serde_json::json!(1));
        assert_eq!(v["a"]["b"]["c"], serde_json::json!(42));
    }

    // ── LogicOperationLog tests ─────────────────────────────────────────────

    #[test]
    fn test_logic_operation_log_record_and_undo() {
        let mut log = LogicOperationLog::new_const();
        let mut doc = empty_graph();

        let cmd = LogicCommand::AddNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        log.record(&cmd, inverse);

        assert!(log.can_undo());
        assert!(!log.can_redo());

        log.undo(&mut doc).unwrap();
        assert_eq!(doc.nodes.len(), 0);
        assert!(!log.can_undo());
        assert!(log.can_redo());
    }

    #[test]
    fn test_logic_operation_log_redo() {
        let mut log = LogicOperationLog::new_const();
        let mut doc = empty_graph();

        let cmd = LogicCommand::AddNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        log.record(&cmd, inverse);

        log.undo(&mut doc).unwrap();
        assert_eq!(doc.nodes.len(), 0);

        log.redo(&mut doc).unwrap();
        assert_eq!(doc.nodes.len(), 1);
    }

    #[test]
    fn test_logic_operation_log_is_dirty() {
        let log = LogicOperationLog::new_const();
        assert!(!log.is_dirty());
    }

    #[test]
    fn test_logic_operation_log_clear() {
        let mut log = LogicOperationLog::new_const();
        let mut doc = empty_graph();

        let cmd = LogicCommand::AddNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        log.record(&cmd, inverse);

        log.clear();
        assert!(!log.can_undo());
        assert!(!log.can_redo());
        assert!(!log.is_dirty());
    }

    #[test]
    fn test_logic_operation_log_undo_redo_roundtrip() {
        let mut log = LogicOperationLog::new_const();
        let mut doc = empty_graph();

        let cmd = LogicCommand::AddNode {
            node_id: NodeId::new("node_a"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.key_down"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        log.record(&cmd, inverse);

        log.undo(&mut doc).unwrap();
        assert!(!log.can_undo());
        assert!(log.can_redo());

        log.redo(&mut doc).unwrap();
        assert!(log.can_undo());
        assert!(!log.can_redo());
        assert_eq!(doc.nodes.len(), 1);
    }

    // ── RecipeImmutable guard tests ─────────────────────────────────────────

    fn builtin_graph() -> LogicGraphAsset {
        LogicGraphAsset {
            asset_id: "lga_recipe_jump".to_string(),
            logical_path: "recipes/platformer_jump".to_string(),
            version: 1,
            builtin: true,
            nodes: vec![LogicNode {
                node_id: NodeId::new("sensor_1"),
                role: LogicNodeRole::Sensor,
                node_type_id: NodeTypeId::new("sensor.key_pressed"),
                field_values: serde_json::json!({}),
                controller_id: None,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn add_node_rejected_on_builtin() {
        let mut doc = builtin_graph();
        let cmd = LogicCommand::AddNode {
            node_id: NodeId::new("node_new"),
            role: LogicNodeRole::Actuator,
            node_type_id: NodeTypeId::new("actuator.jump"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::RecipeImmutable(_))));
    }

    #[test]
    fn remove_node_rejected_on_builtin() {
        let mut doc = builtin_graph();
        let cmd = LogicCommand::RemoveNode {
            node_id: NodeId::new("sensor_1"),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::RecipeImmutable(_))));
    }

    #[test]
    fn connect_ports_rejected_on_builtin() {
        let mut doc = builtin_graph();
        let cmd = LogicCommand::ConnectPorts {
            from_node: NodeId::new("sensor_1"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_new"),
            to_port: PortId::new("in"),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::RecipeImmutable(_))));
    }

    #[test]
    fn disconnect_ports_rejected_on_builtin() {
        let mut doc = builtin_graph();
        // No edge exists, but we should hit the builtin guard before the edge check
        let cmd = LogicCommand::DisconnectPorts {
            from_node: NodeId::new("sensor_1"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("node_unknown"),
            to_port: PortId::new("in"),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::RecipeImmutable(_))));
    }

    #[test]
    fn set_node_field_rejected_on_builtin() {
        let mut doc = builtin_graph();
        let cmd = LogicCommand::SetNodeField {
            node_id: NodeId::new("sensor_1"),
            field_path: vec!["key".to_string()],
            value: serde_json::json!("Space"),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::RecipeImmutable(_))));
    }

    #[test]
    fn batch_rollback_on_builtin() {
        let mut doc = builtin_graph();
        let cmd = LogicCommand::Batch {
            label: "try mutating builtin".to_string(),
            commands: vec![
                LogicCommand::AddNode {
                    node_id: NodeId::new("node_new"),
                    role: LogicNodeRole::Actuator,
                    node_type_id: NodeTypeId::new("actuator.jump"),
                    field_values: serde_json::json!({}),
                    controller_id: None,
                },
            ],
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(LogicCommandError::RecipeImmutable(_))));
        // Batch is atomic — document must be unchanged
        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(doc.nodes[0].node_id.as_str(), "sensor_1");
    }
}
