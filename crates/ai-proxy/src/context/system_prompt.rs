//! System prompt builder for the AI assistant.
//!
//! Hito 4 Order 6 (`code-aware-ai`): refactored from a single-purpose builder
//! to a multi-source orchestrator. Each `ContextSource` is autonomous; the
//! builder composes them under a shared `TokenBudget` using greedy priority
//! fill (highest-priority sources first).

use serde_json::Value;

use super::source_impls::{
    LogicGraphsSource, SceneAssetSource, SceneSnapshotSource, SchemasSource, SelectedEntitySource,
    SourceFilesSource,
};
use super::sources::{
    ContextSource, LogicGraphRef, SceneAssetContext, SelectedEntity, SourceFileRef, TokenBudget,
};

/// Multi-source context builder. Composes prompt sections from heterogeneous
/// sources (scene, schemas, source files, logic graphs, scene assets, selected
/// entity) under a shared token budget.
///
/// Priority order (higher first):
/// 1. SceneSnapshot (P=100)
/// 2. SelectedEntity (P=90)
/// 3. Schemas (P=80)
/// 4. SceneAsset.selected_body (P=60)
/// 5. SourceFiles (P=50)
/// 6. LogicGraphs (P=40)
/// 7. SceneAsset.catalog (P=30)
///
/// Builder methods (`with_*`) return `Self` to preserve the existing fluent
/// API; `build()` returns the assembled prompt string.
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    domain_description: String,
    schemas: Option<Value>,
    scene: Option<Value>,
    source_files: Vec<SourceFileRef>,
    logic_graphs: Vec<LogicGraphRef>,
    scene_assets: SceneAssetContext,
    selected_entity: Option<SelectedEntity>,
    token_threshold_chars: usize,
    truncated: bool,
    token_count: usize,
}

impl ContextBuilder {
    /// Create a new ContextBuilder with the editor domain description.
    pub fn new(domain_description: impl Into<String>) -> Self {
        Self {
            domain_description: domain_description.into(),
            schemas: None,
            scene: None,
            source_files: Vec::new(),
            logic_graphs: Vec::new(),
            scene_assets: SceneAssetContext::default(),
            selected_entity: None,
            token_threshold_chars: 40_000, // 10k tokens × 4 chars/token
            truncated: false,
            token_count: 0,
        }
    }

    /// Set the combined schemas JSON.
    pub fn with_schemas(mut self, schemas: Value) -> Self {
        self.schemas = Some(schemas);
        self
    }

    /// Set the scene snapshot JSON.
    pub fn with_scene(mut self, scene: Value) -> Self {
        self.scene = Some(scene);
        self
    }

    /// Set the source files visible to the AI.
    pub fn with_source_files(mut self, files: Vec<SourceFileRef>) -> Self {
        self.source_files = files;
        self
    }

    /// Set the logic graphs visible to the AI.
    pub fn with_logic_graphs(mut self, graphs: Vec<LogicGraphRef>) -> Self {
        self.logic_graphs = graphs;
        self
    }

    /// Set the scene-asset context (catalog + selected body).
    pub fn with_scene_assets(mut self, assets: SceneAssetContext) -> Self {
        self.scene_assets = assets;
        self
    }

    /// Set the currently-selected entity (if any).
    pub fn with_selected_entity(mut self, entity: Option<SelectedEntity>) -> Self {
        self.selected_entity = entity;
        self
    }

    /// Set the token threshold (in tokens; converted to chars internally via × 4).
    pub fn with_token_threshold(mut self, threshold_tokens: usize) -> Self {
        self.token_threshold_chars = threshold_tokens * 4;
        self
    }

    /// Assemble the full system prompt.
    pub fn build(&mut self) -> String {
        // 1. Reserve chars for the domain description + system instructions (always preserved).
        let instructions_chars =
            self.domain_description.chars().count() + SYSTEM_INSTRUCTIONS.chars().count() + 64;
        let budget_for_sources =
            self.token_threshold_chars.saturating_sub(instructions_chars);
        let mut budget = TokenBudget::new(budget_for_sources);

        // 2. Compose sources (only those with data).
        let sources: Vec<Box<dyn ContextSource>> = self.compose_sources();

        // 3. Greedy priority fill: sort by priority desc, assemble each.
        let mut sorted_sources = sources;
        sorted_sources.sort_by_key(|s| std::cmp::Reverse(s.priority()));

        let mut sections: Vec<String> = Vec::new();
        let mut any_truncated = false;
        for src in &sorted_sources {
            let total = src.total_chars();
            let text = src.assemble(&mut budget);
            if text.is_empty() {
                continue;
            }
            // If we wanted to emit total chars but used fewer, mark truncated.
            if total > text.len() && text.contains("[truncated]") {
                any_truncated = true;
            }
            sections.push(text);
        }

        // 4. Final assembly: domain + sections + instructions.
        let mut out = String::with_capacity(self.token_threshold_chars);
        out.push_str(&self.domain_description);
        out.push_str("\n\n");
        for sec in sections {
            out.push_str(&sec);
            out.push_str("\n\n");
        }
        out.push_str("## Instructions\n\n");
        out.push_str(SYSTEM_INSTRUCTIONS);

        // 5. Update telemetry
        self.token_count = out.chars().count() / 4;
        self.truncated = any_truncated || out.chars().count() > self.token_threshold_chars;
        out
    }

    fn compose_sources(&self) -> Vec<Box<dyn ContextSource>> {
        let mut v: Vec<Box<dyn ContextSource>> = Vec::new();
        if let Some(scene) = &self.scene {
            v.push(Box::new(SceneSnapshotSource { json: scene.clone() }));
        }
        if self.selected_entity.is_some() {
            v.push(Box::new(SelectedEntitySource { entity: self.selected_entity.clone() }));
        }
        if let Some(schemas) = &self.schemas {
            v.push(Box::new(SchemasSource { json: schemas.clone() }));
        }
        if !self.source_files.is_empty() {
            v.push(Box::new(SourceFilesSource { files: self.source_files.clone() }));
        }
        if !self.logic_graphs.is_empty() {
            v.push(Box::new(LogicGraphsSource { graphs: self.logic_graphs.clone() }));
        }
        if !self.scene_assets.catalog.is_empty() || self.scene_assets.selected_body.is_some() {
            v.push(Box::new(SceneAssetSource { ctx: self.scene_assets.clone() }));
        }
        v
    }

    pub fn was_truncated(&self) -> bool {
        self.truncated
    }

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
- **CreateSourceFile**: Create a new Rust source file (path, name, content).
- **WriteSourceFile**: Overwrite the contents of an existing source file by id.

NOTE: Deleting or renaming source files via AI is **not** supported in v1 (security). If the user asks to delete/rename a file, refuse and suggest manual action.

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
        let mut builder = ContextBuilder::new("Bevy 2D Editor v0.1")
            .with_schemas(json!([{"type_id": "editor.Transform2D"}]))
            .with_scene(json!({"entities": []}));
        let prompt = builder.build();
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
        let mut builder = ContextBuilder::new("Test domain")
            .with_schemas(schemas)
            .with_scene(json!({"entities": []}));
        let prompt = builder.build();
        assert!(prompt.contains("editor.Transform2D"));
        assert!(prompt.contains("translation"));
    }

    #[test]
    fn test_was_truncated_set_when_over_budget() {
        let scene_large = json!({
            "entities": (0..1000).map(|i| json!({"id": format!("ent_{}", i), "name": format!("Entity {}", i)})).collect::<Vec<_>>()
        });
        let mut builder = ContextBuilder::new("Test")
            .with_schemas(json!([]))
            .with_scene(scene_large)
            .with_token_threshold(50);
        builder.build();
        assert!(builder.was_truncated());
    }

    #[test]
    fn test_not_truncated_under_budget() {
        let mut builder = ContextBuilder::new("Test")
            .with_schemas(json!([]))
            .with_scene(json!({"entities": []}))
            .with_token_threshold(10_000);
        builder.build();
        assert!(!builder.was_truncated());
    }

    #[test]
    fn test_source_files_included_in_prompt() {
        let mut builder = ContextBuilder::new("Test")
            .with_schemas(json!([]))
            .with_scene(json!({}))
            .with_source_files(vec![SourceFileRef {
                id: "src_x_rs".into(),
                path: "src/x.rs".into(),
                content: "fn main() {}".into(),
            }])
            .with_token_threshold(10_000);
        let prompt = builder.build();
        assert!(prompt.contains("Source Files"));
        assert!(prompt.contains("src/x.rs"));
        assert!(prompt.contains("fn main"));
    }

    #[test]
    fn test_selected_entity_included_when_present() {
        let mut builder = ContextBuilder::new("Test")
            .with_schemas(json!([]))
            .with_scene(json!({}))
            .with_selected_entity(Some(SelectedEntity {
                stable_id: "ent_001".into(),
                components: vec![],
            }))
            .with_token_threshold(10_000);
        let prompt = builder.build();
        assert!(prompt.contains("Selected Entity"));
        assert!(prompt.contains("ent_001"));
    }

    #[test]
    fn test_priority_order_respected_under_pressure() {
        // With a very tight budget, Scene (P=100) should win over
        // SourceFiles (P=50).
        let big_scene = json!({
            "entities": (0..1000).map(|i| json!({"id": format!("ent_{}", i), "name": "x"})).collect::<Vec<_>>()
        });
        let mut builder = ContextBuilder::new("Test")
            .with_schemas(json!([]))
            .with_scene(big_scene)
            .with_source_files(vec![SourceFileRef {
                id: "x".into(),
                path: "x.rs".into(),
                content: "fn drop_me() {}".into(),
            }])
            .with_token_threshold(60);
        let prompt = builder.build();
        // Scene is always present (highest priority); source file should be
        // truncated or dropped when budget is tight.
        assert!(prompt.contains("Scene Snapshot"));
    }
}