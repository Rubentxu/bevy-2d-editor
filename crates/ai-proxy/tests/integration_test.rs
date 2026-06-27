//! Integration tests for the AI proxy handlers using tower::oneshot.

use axum::{
    body::Body,
    Router,
};
use http_body_util::BodyExt;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

use ai_proxy::context::SchemaFetcher;
use ai_proxy::handlers::propose::{AppState, ProposeRequest};
use ai_proxy::openai::OpenAIClient;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn app_state() -> AppState {
    AppState {
        openai_client: Arc::new(OpenAIClient::new(
            "sk-test-key".to_string(),
            "gpt-4o".to_string(),
        )),
        token_threshold: 10_000,
    }
}

// ---------------------------------------------------------------------------
// Schema validation tests
// ---------------------------------------------------------------------------

#[test]
fn test_validate_schemas_valid() {
    let schemas = json!([
        {
            "type_id": "editor.Transform2D",
            "fields": [
                {"name": "translation", "type": "Vec2"}
            ]
        }
    ]);
    assert!(SchemaFetcher::fetch(schemas.clone()).is_ok());
}

#[test]
fn test_validate_schemas_missing_type_id() {
    let schemas = json!([
        {
            "fields": [{"name": "translation", "type": "Vec2"}]
        }
    ]);
    let result = SchemaFetcher::fetch(schemas);
    assert!(result.is_err());
}

#[test]
fn test_validate_schemas_not_array() {
    let schemas = json!({"type_id": "editor.Transform2D"});
    let result = SchemaFetcher::fetch(schemas);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Context truncation tests
// ---------------------------------------------------------------------------

#[test]
fn test_scene_truncation_large_scene() {
    use ai_proxy::context::truncate_scene_if_over_budget;

    // Build a large scene
    let scene = serde_json::to_string(&serde_json::json!({
        "entities": (0..2000).map(|i| serde_json::json!({
            "id": format!("ent_{}", i),
            "name": format!("Entity {}", i),
            "components": []
        })).collect::<Vec<_>>()
    }))
    .unwrap();

    let schemas = r#"[
        {"type_id": "editor.Transform2D", "fields": [{"name": "x", "type": "f32"}]}
    ]"#;

    let threshold = 5_000;
    let (result, truncated, tokens) =
        truncate_scene_if_over_budget("", schemas, &scene, threshold);

    assert!(truncated, "Scene should be truncated when over budget");
    assert!(
        result.len() < scene.len(),
        "Result should be shorter than original scene"
    );
    assert!(
        tokens <= threshold + 200, // Small tolerance for overhead
        "Token count {} should be within threshold {} (+ overhead)",
        tokens,
        threshold
    );
}

#[test]
fn test_scene_not_truncated_small_scene() {
    use ai_proxy::context::truncate_scene_if_over_budget;

    let scene = r#"{"entities": []}"#;

    let (result, truncated, tokens) =
        truncate_scene_if_over_budget("", "", scene, 10_000);

    assert!(!truncated);
    assert_eq!(result, scene);
    assert!(tokens > 0);
}

// ---------------------------------------------------------------------------
// Token estimation tests
// ---------------------------------------------------------------------------

#[test]
fn test_token_estimation_chars_divided_by_4() {
    use ai_proxy::context::estimate_tokens;

    assert_eq!(estimate_tokens("hello"), 1); // 5 chars / 4
    assert_eq!(estimate_tokens("a".repeat(100).as_str()), 25);
    assert_eq!(estimate_tokens(""), 0);
}

// ---------------------------------------------------------------------------
// ProposeRequest validation tests
// ---------------------------------------------------------------------------

#[test]
fn test_propose_request_missing_prompt() {
    let req = ProposeRequest {
        prompt: None,
        scene_snapshot: Some(serde_json::json!({})),
        schemas: Some(serde_json::json!([])),
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
    };
    assert!(req.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Health endpoint tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_returns_200_ok() {
    use ai_proxy::handlers::health::health_handler;
    use axum::routing::get;

    let app = Router::new().route("/v1/health", get(health_handler));

    let res = app
        .oneshot(
            http::Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("\"status\":\"ok\""));
}
