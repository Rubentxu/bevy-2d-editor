//! Component types for the editor model.

use serde::{Deserialize, Serialize};

/// A component instance attaching typed values to an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentInstance {
    /// Fully-qualified type identifier (e.g. "editor.Transform2D").
    pub type_id: String,
    /// Serialized field values as JSON.
    #[serde(default)]
    pub values: serde_json::Value,
}
