//! Schema validation and fetcher for component schemas.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A component schema entry from the frontend's schema registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSchema {
    #[serde(rename = "type_id")]
    pub type_id: String,
    #[serde(default)]
    pub fields: Vec<SchemaField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub default: Option<Value>,
}

/// Validates that the schemas JSON has the expected shape.
///
/// The frontend sends an array of `{ type_id: string, fields: [...] }` objects.
/// This validates the shape without checking every field (the frontend owns that data).
pub fn validate_schemas(schemas: &Value) -> Result<(), SchemaValidationError> {
    let arr = schemas
        .as_array()
        .ok_or(SchemaValidationError::NotAnArray)?;

    for item in arr {
        if !item.is_object() {
            return Err(SchemaValidationError::InvalidItem(
                "schema entry must be an object".to_string(),
            ));
        }

        let obj = item.as_object().unwrap();

        // type_id is required
        if !obj.contains_key("type_id") {
            return Err(SchemaValidationError::MissingField("type_id".to_string()));
        }

        if !obj.get("type_id").and_then(|v| v.as_str()).is_some() {
            return Err(SchemaValidationError::InvalidFieldType(
                "type_id".to_string(),
                "string".to_string(),
            ));
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SchemaValidationError {
    #[error("schemas must be a JSON array")]
    NotAnArray,
    #[error("schema entry is invalid: {0}")]
    InvalidItem(String),
    #[error("schema entry missing required field: {0}")]
    MissingField(String),
    #[error("field '{0}' must be of type {1}")]
    InvalidFieldType(String, String),
}

/// Schema fetcher — accepts raw JSON from the frontend and validates it.
#[derive(Debug, Clone)]
pub struct SchemaFetcher;

impl SchemaFetcher {
    /// Validate and return the schemas JSON.
    ///
    /// # Errors
    /// Returns `SchemaValidationError` if the JSON is malformed or missing required fields.
    pub fn fetch(schemas: Value) -> Result<Value, SchemaValidationError> {
        validate_schemas(&schemas)?;
        Ok(schemas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_schemas() {
        let schemas = serde_json::json!([
            {
                "type_id": "editor.Transform2D",
                "fields": [
                    {"name": "translation", "type": "Vec2", "default": {"x": 0.0, "y": 0.0}}
                ]
            }
        ]);
        assert!(SchemaFetcher::fetch(schemas.clone()).is_ok());
        assert_eq!(
            SchemaFetcher::fetch(schemas)
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_missing_type_id() {
        let schemas = serde_json::json!([
            {
                "fields": [{"name": "translation", "type": "Vec2"}]
            }
        ]);
        let result = SchemaFetcher::fetch(schemas);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SchemaValidationError::MissingField(f) if f == "type_id"
        ));
    }

    #[test]
    fn test_not_an_array() {
        let schemas = serde_json::json!({"type_id": "editor.Transform2D"});
        let result = SchemaFetcher::fetch(schemas);
        assert!(matches!(
            result.unwrap_err(),
            SchemaValidationError::NotAnArray
        ));
    }
}
