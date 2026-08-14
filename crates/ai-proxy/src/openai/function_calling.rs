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
    static SCHEMA: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    SCHEMA
        .get_or_init(|| {
            // Hito 4 Order 7: schema has 13 commands (was 10). Loaded from
            // an external JSON file to avoid the `json!` macro recursion
            // limit (the inlined macro exceeded it with 13 commands).
            serde_json::from_str(include_str!("../../data/propose_commands_schema.json"))
                .expect("propose_commands_schema.json is valid")
        })
        .clone()
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
///
/// Hito 4 Order 7 (scene-component-authoring) extends this with
/// `DeleteSceneComponent` + `RenameSceneComponent` (no AI).
pub const FORBIDDEN_AI_COMMANDS: &[&str] = &[
    "DeleteSourceFile",
    "RenameSourceFile",
    "DeleteSceneComponent",
    "RenameSceneComponent",
];

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
    fn test_propose_commands_schema_includes_scene_component_commands() {
        // Hito 4 Order 7 (scene-component-authoring): 3 new commands added.
        let schema = propose_commands_schema();
        let schema_str = serde_json::to_string(&schema).unwrap();
        assert!(schema_str.contains("CreateSceneComponent"));
        assert!(schema_str.contains("UpdateSceneComponentFields"));
        assert!(schema_str.contains("BindSceneToSchema"));
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
        assert_eq!(
            kept.len(),
            2,
            "should keep 2 (CreateEntity + WriteSourceFile)"
        );
        assert_eq!(rejected.len(), 2, "should reject 2 forbidden");
        assert!(rejected.contains(&"DeleteSourceFile".to_string()));
        assert!(rejected.contains(&"RenameSourceFile".to_string()));
    }

    #[test]
    fn test_forbidden_ai_commands_constant() {
        // Per D2 (Order 6): only Create/Write allowed; Delete/Rename forbidden.
        // Per Order 7: DeleteSceneComponent + RenameSceneComponent forbidden.
        assert!(FORBIDDEN_AI_COMMANDS.contains(&"DeleteSourceFile"));
        assert!(FORBIDDEN_AI_COMMANDS.contains(&"RenameSourceFile"));
        assert!(FORBIDDEN_AI_COMMANDS.contains(&"DeleteSceneComponent"));
        assert!(FORBIDDEN_AI_COMMANDS.contains(&"RenameSceneComponent"));
        assert!(!FORBIDDEN_AI_COMMANDS.contains(&"CreateSourceFile"));
        assert!(!FORBIDDEN_AI_COMMANDS.contains(&"WriteSourceFile"));
        assert!(!FORBIDDEN_AI_COMMANDS.contains(&"CreateSceneComponent"));
        assert!(!FORBIDDEN_AI_COMMANDS.contains(&"UpdateSceneComponentFields"));
        assert!(!FORBIDDEN_AI_COMMANDS.contains(&"BindSceneToSchema"));
    }
}
