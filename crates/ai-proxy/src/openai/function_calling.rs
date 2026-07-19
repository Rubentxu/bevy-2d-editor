//! JSON schema for the `propose_commands` tool (function-calling) and response parsing.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Command envelope returned by the proxy to the frontend.
///
/// The `command` field matches the `#[serde(tag = "type", rename_all = "PascalCase")]`
/// shape defined in `crates/editor-core/src/command.rs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub command: Value,
    #[serde(default)]
    pub metadata: CommandMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandMetadata {
    #[serde(default)]
    pub authorship: String,
    #[serde(default)]
    pub timestamp: u64,
    #[serde(default)]
    pub rationale: Option<String>,
}

/// JSON schema for the `propose_commands` tool passed to OpenAI's function-calling API.
///
/// This must match the `Command` enum shape from `crates/editor-core/src/command.rs`:
/// `#[serde(tag = "type", rename_all = "PascalCase")]`
pub fn propose_commands_schema() -> Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": "propose_commands",
            "description": "Propose one or more editor commands to fulfill the user's natural language request.",
            "parameters": {
                "type": "object",
                "properties": {
                    "commands": {
                        "type": "array",
                        "description": "Array of editor commands to apply.",
                        "items": {
                            "type": "object",
                            "oneOf": [
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "CreateEntity" },
                                        "id": { "type": "string", "description": "Stable entity ID (use ent_ai_<uuid>)" },
                                        "name": { "type": "string", "description": "Human-readable entity name" },
                                        "components": {
                                            "type": "array",
                                            "description": "Component instances to attach",
                                            "default": [],
                                            "items": {
                                                "type": "object",
                                                "properties": {
                                                    "type_id": { "type": "string" },
                                                    "values": { "type": "object", "description": "Component field values" }
                                                },
                                                "required": ["type_id", "values"]
                                            }
                                        }
                                    },
                                    "required": ["type", "id", "name"]
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "DeleteEntity" },
                                        "id": { "type": "string", "description": "Stable entity ID to delete" }
                                    },
                                    "required": ["type", "id"]
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "AddComponent" },
                                        "entity_id": { "type": "string" },
                                        "type_id": { "type": "string", "description": "Schema type ID e.g. editor.Sprite2D" },
                                        "values": { "type": "object" }
                                    },
                                    "required": ["type", "entity_id", "type_id", "values"]
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "RemoveComponent" },
                                        "entity_id": { "type": "string" },
                                        "type_id": { "type": "string" }
                                    },
                                    "required": ["type", "entity_id", "type_id"]
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "SetComponentField" },
                                        "entity_id": { "type": "string" },
                                        "type_id": { "type": "string" },
                                        "field_path": { "type": "string", "description": "Dotted path e.g. translation.x" },
                                        "value": { "type": "object", "description": "New field value" }
                                    },
                                    "required": ["type", "entity_id", "type_id", "field_path", "value"]
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "ReparentEntity" },
                                        "entity_id": { "type": "string" },
                                        "old_parent": { "type": ["string", "null"] },
                                        "new_parent": { "type": ["string", "null"] }
                                    },
                                    "required": ["type", "entity_id"]
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "RenameEntity" },
                                        "entity_id": { "type": "string" },
                                        "old_name": { "type": ["string", "null"] },
                                        "new_name": { "type": "string" }
                                    },
                                    "required": ["type", "entity_id", "new_name"]
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "Batch" },
                                        "label": { "type": "string" },
                                        "commands": {
                                            "type": "array",
                                            "items": { "$ref": "#" }
                                        }
                                    },
                                    "required": ["type", "label", "commands"]
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "CreateSourceFile" },
                                        "path": { "type": "string", "description": "Project-relative path e.g. 'src/player.rs'" },
                                        "name": { "type": "string", "description": "File name e.g. 'player.rs'" },
                                        "content": { "type": "string", "description": "Full file body" }
                                    },
                                    "required": ["type", "path", "name", "content"]
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": { "type": "string", "const": "WriteSourceFile" },
                                        "id": { "type": "string", "description": "Existing source-file id (from create_source_file response)" },
                                        "content": { "type": "string", "description": "New full file body" }
                                    },
                                    "required": ["type", "id", "content"]
                                }
                            ]
                        }
                    },
                    "rationale": {
                        "type": "string",
                        "description": "Brief explanation of why these commands were proposed."
                    }
                },
                "required": ["commands", "rationale"]
            }
        }
    })
}

/// Parse an OpenAI tool_calls response into a list of CommandEnvelopes.
pub fn parse_tool_calls(
    tool_calls: Vec<serde_json::Value>,
    model: &str,
    rationale: Option<String>,
) -> Vec<CommandEnvelope> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let authorship = format!("agent:{}", model);

    tool_calls
        .into_iter()
        .filter_map(|call| {
            let args = call.get("function")?.get("arguments")?;
            let args_str = args.as_str()?;
            let parsed: Value = serde_json::from_str(args_str).ok()?;

            let commands = parsed.get("commands")?.as_array()?.clone();
            let rationale = parsed
                .get("rationale")
                .and_then(|v| v.as_str())
                .map(String::from)
                .or_else(|| rationale.clone());

            commands
                .into_iter()
                .map(|cmd| CommandEnvelope {
                    command: cmd,
                    metadata: CommandMetadata {
                        authorship: authorship.clone(),
                        timestamp,
                        rationale: rationale.clone(),
                    },
                })
                .collect::<Vec<_>>()
                .into_iter()
                .next()
        })
        .collect()
}

/// Commands that AI is forbidden to emit. Per design decision D2, AI must
/// not be able to delete or rename source files in v1. The frontend applies
/// this check before dispatching commands; this function is exported for
/// the proxy to validate OpenAI responses server-side.
pub const FORBIDDEN_AI_COMMANDS: &[&str] = &["DeleteSourceFile", "RenameSourceFile"];

/// Filter out any commands that the AI is not allowed to emit.
/// Returns `(envelopes, rejected_names)` where `rejected_names` is a list
/// of command type names that were dropped (for warning / telemetry).
pub fn filter_forbidden_commands(
    envelopes: Vec<CommandEnvelope>,
) -> (Vec<CommandEnvelope>, Vec<String>) {
    let mut rejected: Vec<String> = Vec::new();
    let kept: Vec<CommandEnvelope> = envelopes
        .into_iter()
        .filter(|env| {
            let type_name = env
                .command
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if FORBIDDEN_AI_COMMANDS.contains(&type_name) {
                rejected.push(type_name.to_string());
                false
            } else {
                true
            }
        })
        .collect();
    (kept, rejected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_call_into_commands() {
        let tool_calls = vec![serde_json::json!({
            "function": {
                "name": "propose_commands",
                "arguments": r#"{
                    "commands": [
                        {
                            "type": "CreateEntity",
                            "id": "ent_ai_1234",
                            "name": "Player",
                            "components": []
                        }
                    ],
                    "rationale": "Created player entity"
                }"#
            }
        })];

        let envelopes = parse_tool_calls(tool_calls, "gpt-4o", None);
        assert_eq!(envelopes.len(), 1);
        let env = &envelopes[0];
        assert_eq!(env.metadata.authorship, "agent:gpt-4o");
        assert!(env.metadata.rationale.is_some());
    }

    #[test]
    fn test_propose_commands_schema_is_valid_json() {
        let schema = propose_commands_schema();
        let schema_str = serde_json::to_string(&schema).unwrap();
        assert!(schema_str.contains("propose_commands"));
        assert!(schema_str.contains("CreateEntity"));
    }

    #[test]
    fn test_propose_commands_schema_includes_source_file_commands() {
        let schema = propose_commands_schema();
        let schema_str = serde_json::to_string(&schema).unwrap();
        // Hito 4 Order 6: source file commands added.
        assert!(schema_str.contains("CreateSourceFile"));
        assert!(schema_str.contains("WriteSourceFile"));
    }

    #[test]
    fn test_filter_forbidden_commands_drops_delete_source_file() {
        let envelopes = vec![
            CommandEnvelope {
                command: serde_json::json!({"type": "CreateEntity", "id": "ent_ai_x", "name": "X"}),
                metadata: CommandMetadata::default(),
            },
            CommandEnvelope {
                command: serde_json::json!({"type": "DeleteSourceFile", "id": "src_x"}),
                metadata: CommandMetadata::default(),
            },
            CommandEnvelope {
                command: serde_json::json!({"type": "RenameSourceFile", "id": "src_x", "new_name": "y.rs"}),
                metadata: CommandMetadata::default(),
            },
            CommandEnvelope {
                command: serde_json::json!({"type": "WriteSourceFile", "id": "src_x", "content": "fn main() {}"}),
                metadata: CommandMetadata::default(),
            },
        ];
        let (kept, rejected) = filter_forbidden_commands(envelopes);
        assert_eq!(kept.len(), 2, "should keep 2 (CreateEntity + WriteSourceFile)");
        assert_eq!(rejected.len(), 2, "should reject 2 forbidden");
        assert!(rejected.contains(&"DeleteSourceFile".to_string()));
        assert!(rejected.contains(&"RenameSourceFile".to_string()));
    }

    #[test]
    fn test_forbidden_ai_commands_constant() {
        // Per D2: only Create/Write allowed; Delete/Rename forbidden.
        assert!(FORBIDDEN_AI_COMMANDS.contains(&"DeleteSourceFile"));
        assert!(FORBIDDEN_AI_COMMANDS.contains(&"RenameSourceFile"));
        assert!(!FORBIDDEN_AI_COMMANDS.contains(&"CreateSourceFile"));
        assert!(!FORBIDDEN_AI_COMMANDS.contains(&"WriteSourceFile"));
    }
}
