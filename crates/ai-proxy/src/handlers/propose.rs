//! `POST /v1/propose` handler.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

use crate::context::sources::{
    LogicGraphRef, SceneAssetContext, SelectedEntity, SourceFileRef,
};
use crate::context::{ContextBuilder, SchemaFetcher};
use crate::error::AppError;
use crate::openai::filter_forbidden_commands;
use crate::openai::OpenAIClient;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub openai_client: Arc<OpenAIClient>,
    pub token_threshold: usize,
}

/// Incoming request to the `/v1/propose` endpoint.
///
/// Hito 4 Order 6 (`code-aware-ai`): the four new fields
/// (`source_files`, `logic_graphs`, `scene_assets`, `selected_entity`) are
/// all `#[serde(default)]` to preserve backward compatibility with v0.69.0
/// clients that only send `prompt` + `scene_snapshot` + `schemas`.
#[derive(Debug, Deserialize)]
pub struct ProposeRequest {
    /// Natural language prompt from the user.
    pub prompt: Option<String>,
    /// Current scene document snapshot from the editor.
    pub scene_snapshot: Option<Value>,
    /// Combined component schema registry from the editor.
    pub schemas: Option<Value>,

    // ── Hito 4 Order 6 additions (code-aware-ai) ─────────────────────────
    /// Source files visible to the AI (Rust + toml). Each carries full text.
    #[serde(default)]
    pub source_files: Vec<SourceFileRef>,
    /// Logic graphs currently in scope (current scene only by default).
    #[serde(default)]
    pub logic_graphs: Vec<LogicGraphRef>,
    /// Scene asset context: catalog + (optionally) the currently-selected
    /// asset's full body.
    #[serde(default)]
    pub scene_assets: SceneAssetContext,
    /// Currently-selected entity in the inspector (if any).
    #[serde(default)]
    pub selected_entity: Option<SelectedEntity>,
}

impl ProposeRequest {
    /// Validate required fields.
    pub fn validate(&self) -> Result<(), AppError> {
        if self.prompt.is_none() {
            return Err(AppError::BadRequest("missing required field: 'prompt'".to_string()));
        }
        if self.scene_snapshot.is_none() {
            return Err(AppError::BadRequest(
                "missing required field: 'scene_snapshot'".to_string(),
            ));
        }
        if self.schemas.is_none() {
            return Err(AppError::BadRequest(
                "missing required field: 'schemas'".to_string(),
            ));
        }
        // Validate SourceFileRef shape
        for (i, sf) in self.source_files.iter().enumerate() {
            if sf.id.is_empty() {
                return Err(AppError::BadRequest(format!(
                    "source_files[{}]: 'id' is required and must be non-empty",
                    i
                )));
            }
        }
        // Validate SelectedEntity has at least one component if present
        if let Some(sel) = &self.selected_entity {
            if sel.stable_id.is_empty() {
                return Err(AppError::BadRequest(
                    "selected_entity: 'stable_id' is required and must be non-empty"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Response body for a successful `/v1/propose` call.
#[derive(Debug, Serialize)]
pub struct ProposeResponse {
    pub commands: Vec<serde_json::Value>,
    pub rationale: String,
    pub model: String,
}

/// `POST /v1/propose` — takes a user prompt, scene snapshot, and schemas;
/// calls OpenAI with a assembled system prompt; returns command proposals.
pub async fn propose_handler(
    State(state): State<AppState>,
    Json(req): Json<ProposeRequest>,
) -> Result<Json<ProposeResponse>, AppError> {
    // Validate request
    req.validate()?;

    let prompt = req.prompt.unwrap();
    let scene_snapshot = req.scene_snapshot.unwrap();
    let schemas = req.schemas.unwrap();

    // Validate schemas shape
    let schemas = SchemaFetcher::fetch(schemas)
        .map_err(|e| AppError::BadRequest(format!("invalid schemas: {}", e)))?;

    // Build system prompt (Hito 4 Order 6: pass multi-source context)
    let mut ctx = ContextBuilder::new(EDITOR_DOMAIN)
        .with_schemas(schemas.clone())
        .with_scene(scene_snapshot.clone())
        .with_token_threshold(state.token_threshold)
        .with_source_files(req.source_files.clone())
        .with_logic_graphs(req.logic_graphs.clone())
        .with_scene_assets(req.scene_assets.clone())
        .with_selected_entity(req.selected_entity.clone());

    let system_prompt = ctx.build();
    let was_truncated = ctx.was_truncated();
    let token_count = ctx.token_count();

    info!(
        endpoint = "/v1/propose",
        tokens = token_count,
        truncated = was_truncated,
        "assembling OpenAI request"
    );

    // Call OpenAI
    let (envelopes, model) = state
        .openai_client
        .propose_commands(&system_prompt, &prompt)
        .await?;

    if envelopes.is_empty() {
        warn!(
            endpoint = "/v1/propose",
            model = %model,
            "OpenAI returned no commands"
        );
    }

    // D2 security: filter out commands the AI is forbidden to emit
    // (DeleteSourceFile, RenameSourceFile, DeleteSceneComponent,
    // RenameSceneComponent). Per ADR-0015 §Decision D2 and ADR-0016.
    let (envelopes, rejected) = filter_forbidden_commands(envelopes);
    if !rejected.is_empty() {
        warn!(
            endpoint = "/v1/propose",
            model = %model,
            rejected = ?rejected,
            "filtered forbidden AI commands server-side"
        );
    }

    let commands: Vec<Value> = envelopes.into_iter().map(|e| serde_json::to_value(e).unwrap()).collect();

    // Collect rationale from first command's metadata (all have same rationale)
    let rationale = commands
        .first()
        .and_then(|c| c.get("metadata"))
        .and_then(|m| m.get("rationale"))
        .and_then(|r| r.as_str())
        .unwrap_or("No rationale provided")
        .to_string();

    info!(
        endpoint = "/v1/propose",
        status = %StatusCode::OK.as_u16(),
        commands = commands.len(),
        model = %model,
        tokens = token_count,
        truncated = was_truncated,
        "request completed"
    );

    Ok(Json(ProposeResponse {
        commands,
        rationale,
        model,
    }))
}

/// Editor domain description injected into the system prompt.
/// This is sourced from the project CONTEXT.md and should be kept in sync.
const EDITOR_DOMAIN: &str = r#"# Bevy 2D Editor — Domain Model

## Overview
The Bevy 2D Editor is a browser-based scene editor for Bevy 2D games. It manages a **Project** containing scenes and shared definitions. The editor's source-of-truth document for a scene is a **SceneDocument** stored as stable JSON.

## Core Concepts

### Entity
A logical object inside a scene with an immutable stable ID (prefix `ent_` or `ent_ai_`) and a separate human-facing name. An Entity is NOT a Bevy Entity — it is an editor-level concept.

### Component Schema Registry
The project-global catalog of component types and their field definitions. Schemas define what components an entity can have. Each schema has a `type_id` (e.g. `editor.Transform2D`, `editor.Sprite2D`) and a list of fields with names and types.

### Component Instance
Values attached to an Entity for one component type. References a schema in the Component Schema Registry. Stored as `{ type_id: string, values: object }`.

### SceneDocument
The editor's source-of-truth document for a scene:
```json
{
  "version": "0.1",
  "scene_id": "...",
  "entities": [
    {
      "id": "ent_xxx",
      "name": "Player",
      "components": [
        { "type_id": "editor.Transform2D", "values": { "translation": { "x": 0, "y": 0 } } }
      ]
    }
  ]
}
```

### Command Types
The editor accepts typed commands (not raw JSON patches):
- **CreateEntity** — add entity with stable ID and optional components
- **DeleteEntity** — remove entity
- **AddComponent** — attach component to existing entity
- **RemoveComponent** — detach component from entity
- **SetComponentField** — update one field using dotted `field_path` (e.g. `translation.x`)
- **ReparentEntity** — reparent entity under new parent
- **RenameEntity** — change entity name
- **Batch** — group commands atomically
- **CreateSourceFile** — create a new Rust source file (path, name, content)
- **WriteSourceFile** — overwrite the contents of an existing source file by id
- **CreateSceneComponent** — create a new SceneComponent schema (Bevy 0.19 `#[derive(SceneComponent)]`)
- **UpdateSceneComponentFields** — update an existing SceneComponent schema's fields
- **BindSceneToSchema** — bind a schema to a scene asset (or clear the binding)

NOTE: Deleting or renaming source files or Scene Components via AI is **not** supported in v1 (security). If the user asks to delete/rename, refuse and suggest manual action.

### Stable ID
The immutable identifier for an Entity. Always has prefix `ent_` or `ent_ai_` for AI-created entities. Never changes — this is critical for undo/redo and cross-reference stability.

## Field Types
- `Vec2` — represented as `{ "x": number, "y": number }`
- `Color` — represented as `{ "r": number, "g": number, "b": number, "a": number }` with values 0.0–1.0
- `f32`, `f64` — JSON numbers
- `String` — JSON string
- `bool` — JSON boolean
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::sources::SourceFileRef;

    fn empty_sources() -> Vec<SourceFileRef> { vec![] }
    fn empty_graphs() -> Vec<crate::context::sources::LogicGraphRef> { vec![] }
    fn empty_assets() -> crate::context::sources::SceneAssetContext {
        crate::context::sources::SceneAssetContext::default()
    }

    #[test]
    fn test_propose_request_missing_prompt() {
        let req = ProposeRequest {
            prompt: None,
            scene_snapshot: Some(serde_json::json!({})),
            schemas: Some(serde_json::json!([])),
            source_files: empty_sources(),
            logic_graphs: empty_graphs(),
            scene_assets: empty_assets(),
            selected_entity: None,
        };
        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("prompt"));
    }

    #[test]
    fn test_propose_request_missing_scene() {
        let req = ProposeRequest {
            prompt: Some("test".to_string()),
            scene_snapshot: None,
            schemas: Some(serde_json::json!([])),
            source_files: empty_sources(),
            logic_graphs: empty_graphs(),
            scene_assets: empty_assets(),
            selected_entity: None,
        };
        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("scene_snapshot"));
    }

    #[test]
    fn test_propose_request_missing_schemas() {
        let req = ProposeRequest {
            prompt: Some("test".to_string()),
            scene_snapshot: Some(serde_json::json!({})),
            schemas: None,
            source_files: empty_sources(),
            logic_graphs: empty_graphs(),
            scene_assets: empty_assets(),
            selected_entity: None,
        };
        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("schemas"));
    }

    #[test]
    fn test_propose_request_valid() {
        let req = ProposeRequest {
            prompt: Some("test".to_string()),
            scene_snapshot: Some(serde_json::json!({})),
            schemas: Some(serde_json::json!([])),
            source_files: empty_sources(),
            logic_graphs: empty_graphs(),
            scene_assets: empty_assets(),
            selected_entity: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_propose_request_with_source_files_valid() {
        // Hito 4 Order 6: source_files is optional but if present must have non-empty ids.
        let req = ProposeRequest {
            prompt: Some("test".to_string()),
            scene_snapshot: Some(serde_json::json!({})),
            schemas: Some(serde_json::json!([])),
            source_files: vec![SourceFileRef {
                id: "src_player_rs".to_string(),
                path: "src/player.rs".to_string(),
                content: "fn update() {}".to_string(),
            }],
            logic_graphs: empty_graphs(),
            scene_assets: empty_assets(),
            selected_entity: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_propose_request_rejects_empty_source_file_id() {
        let req = ProposeRequest {
            prompt: Some("test".to_string()),
            scene_snapshot: Some(serde_json::json!({})),
            schemas: Some(serde_json::json!([])),
            source_files: vec![SourceFileRef {
                id: String::new(),
                path: "src/x.rs".to_string(),
                content: String::new(),
            }],
            logic_graphs: empty_graphs(),
            scene_assets: empty_assets(),
            selected_entity: None,
        };
        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("source_files"));
    }
}
