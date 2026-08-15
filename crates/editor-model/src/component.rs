//! Component types for the editor model.

use serde::{Deserialize, Serialize};

/// A component instance attaching typed values to an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentInstance {
    pub type_id: String,
    #[serde(default)]
    pub values: serde_json::Value,
}
