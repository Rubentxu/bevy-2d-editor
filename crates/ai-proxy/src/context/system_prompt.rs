//! System prompt builder for the AI assistant.

use serde_json::Value;

/// Builder for the full system prompt sent to OpenAI.
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    /// The editor domain description from CONTEXT.md.
    domain_description: String,
    /// Combined schema registry JSON.
    schemas_json: String,
    /// Scene snapshot JSON.
    scene_json: String,
    /// Token threshold for truncation.
    token_threshold: usize,
    /// Whether the scene was truncated.
    truncated: bool,
    /// Final token count after assembly and potential truncation.
    token_count: usize,
}

impl ContextBuilder {
    /// Create a new ContextBuilder.
    pub fn new(domain_description: impl Into<String>) -> Self {
        Self {
            domain_description: domain_description.into(),
            schemas_json: String::new(),
            scene_json: String::new(),
            token_threshold: 10_000,
            truncated: false,
            token_count: 0,
        }
    }

    /// Set the combined schemas JSON.
    pub fn with_schemas(mut self, schemas: Value) -> Self {
        self.schemas_json = serde_json::to_string_pretty(&schemas).unwrap_or_default();
        self
    }

    /// Set the scene snapshot JSON.
    pub fn with_scene(mut self, scene: Value) -> Self {
        self.scene_json = serde_json::to_string_pretty(&scene).unwrap_or_default();
        self
    }

    /// Set the token threshold.
    pub fn with_token_threshold(mut self, threshold: usize) -> Self {
        self.token_threshold = threshold;
        self
    }

    /// Assemble the full system prompt.
    pub fn build(&mut self) -> String {
        // Assemble all parts
        let mut parts = vec![
            self.domain_description.clone(),
            "\n\n## Available Component Schemas\n\n".to_string(),
            self.schemas_json.clone(),
            "\n\n## Current Scene Snapshot\n\n".to_string(),
            self.scene_json.clone(),
            "\n\n## Instructions\n\n".to_string(),
            SYSTEM_INSTRUCTIONS.to_string(),
        ];

        let assembled = parts.join("");
        let tokens = super::scene_truncator::estimate_tokens(&assembled);
        self.token_count = tokens;

        // If over budget, truncate scene
        if tokens > self.token_threshold {
            self.truncated = true;
            // Truncate scene JSON and reassemble
            let schemas_part = format!(
                "{}\n\n## Available Component Schemas\n\n{}\n\n## Current Scene Snapshot\n\n",
                self.domain_description, self.schemas_json
            );
            let schemas_tokens = super::scene_truncator::estimate_tokens(&schemas_part);
            let instructions_tokens =
                super::scene_truncator::estimate_tokens(SYSTEM_INSTRUCTIONS);

            // Budget for scene = threshold - schemas - instructions
            let scene_budget = self.token_threshold.saturating_sub(schemas_tokens + instructions_tokens);
            let scene_chars = scene_budget * 4; // chars/4 heuristic

            let truncated_scene: String = self
                .scene_json
                .chars()
                .take(scene_chars)
                .collect();
            let truncated_scene =
                if truncated_scene.ends_with('}') || truncated_scene.ends_with(']') {
                    truncated_scene
                } else {
                    // Try to find a good cut point (last complete object)
                    truncated_scene
                        .rfind('}')
                        .map(|i| &truncated_scene[..=i])
                        .unwrap_or(&truncated_scene)
                        .to_string()
                };

            parts = vec![
                self.domain_description.clone(),
                "\n\n## Available Component Schemas\n\n".to_string(),
                self.schemas_json.clone(),
                "\n\n## Current Scene Snapshot (TRUNCATED)\n\n".to_string(),
                truncated_scene,
                "\n\n## Instructions\n\n".to_string(),
                SYSTEM_INSTRUCTIONS.to_string(),
            ];
            let assembled = parts.join("");
            self.token_count = super::scene_truncator::estimate_tokens(&assembled);
        }

        assembled
    }

    /// Returns `true` if the scene was truncated.
    pub fn was_truncated(&self) -> bool {
        self.truncated
    }

    /// Returns the final token count after assembly.
    pub fn token_count(&self) -> usize {
        self.token_count
    }
}

/// Static instruction text injected into every system prompt.
const SYSTEM_INSTRUCTIONS: &str = r#"You are an AI assistant for a Bevy 2D game editor.

Your role is to translate natural language requests from the user into precise editor commands.

## Command Format
All commands MUST be valid JSON matching the schema provided. Use the `propose_commands` tool to respond.
Every command MUST use the `type` field as a discriminator (e.g. `{"type": "CreateEntity", ...}`).

## Entity IDs
- Always generate stable entity IDs with the prefix `ent_ai_` followed by a UUID, e.g. `ent_ai_a1b2c3d4`.
- Never reuse an ID from a previous command in the same response.
- Do NOT use IDs from the scene snapshot unless explicitly referencing an existing entity.

## Schema Compliance
- Use exact `type_id` values from the schema registry (e.g. `editor.Transform2D`, `editor.Sprite2D`).
- Field names and types must match the schema exactly.
- Provide default values for optional fields only when needed.

## Command Types
- **CreateEntity**: Create a new entity with optional components.
- **DeleteEntity**: Remove an entity by ID.
- **AddComponent**: Add a component to an existing entity.
- **RemoveComponent**: Remove a component from an entity.
- **SetComponentField**: Update a specific field using dotted `field_path` (e.g. `translation.x`).
- **ReparentEntity**: Move an entity under a new parent (old_parent is auto-captured).
- **RenameEntity**: Change an entity's name.
- **Batch**: Group multiple commands atomically.

## Response Rules
1. Always use the `propose_commands` tool.
2. Return exactly what the user asked for — no extra entities or modifications.
3. Keep commands minimal and focused.
4. Include a brief `rationale` explaining the changes.
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_system_prompt_includes_domain() {
        let builder = ContextBuilder::new("Bevy 2D Editor v0.1");
        let mut b = builder.with_schemas(json!([{"type_id": "editor.Transform2D"}]));
        let prompt = b.build();
        assert!(prompt.contains("Bevy 2D Editor v0.1"));
    }

    #[test]
    fn test_system_prompt_includes_schemas() {
        let schemas = json!([
            {
                "type_id": "editor.Transform2D",
                "fields": [
                    {"name": "translation", "type": "Vec2"}
                ]
            }
        ]);
        let builder = ContextBuilder::new("Test domain");
        let prompt = builder.with_schemas(schemas).build();
        assert!(prompt.contains("editor.Transform2D"));
        assert!(prompt.contains("translation"));
    }

    #[test]
    fn test_was_truncated_set_when_over_budget() {
        let scene_large = serde_json::json!({
            "entities": (0..1000).map(|i| serde_json::json!({"id": format!("ent_{}", i), "name": format!("Entity {}", i)})).collect::<Vec<_>>()
        });
        let mut builder = ContextBuilder::new("Test");
        builder = builder
            .with_schemas(json!([]))
            .with_scene(scene_large)
            .with_token_threshold(100); // Very low threshold
        builder.build();
        assert!(builder.was_truncated());
    }

    #[test]
    fn test_not_truncated_under_budget() {
        let scene = serde_json::json!({"entities": []});
        let mut builder = ContextBuilder::new("Test");
        builder = builder
            .with_schemas(json!([]))
            .with_scene(scene)
            .with_token_threshold(100_000); // High threshold
        builder.build();
        assert!(!builder.was_truncated());
    }
}
