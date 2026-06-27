//! Axum router and app state setup.

use std::sync::Arc;
use axum::{
    http::HeaderValue,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::config::AppConfig;
use crate::handlers::{health_handler, propose_handler};
use crate::handlers::propose::AppState;
use crate::openai::OpenAIClient;

/// Build the axum Router with all routes and middleware.
pub fn build_router(config: &AppConfig) -> Router {
    // Create OpenAI client
    let openai_client = Arc::new(OpenAIClient::new(
        config.openai_api_key.clone(),
        config.model.clone(),
    ));

    let app_state = AppState {
        openai_client,
        token_threshold: config.token_threshold,
    };

    // CORS configuration
    let allowed_origins: Vec<HeaderValue> = config
        .allowed_origins
        .iter()
        .filter_map(|origin| {
            origin
                .parse::<HeaderValue>()
                .ok()
        })
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(if allowed_origins.is_empty() {
            tower_http::cors::AllowOrigin::any()
        } else {
            tower_http::cors::AllowOrigin::list(allowed_origins)
        })
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
        .route("/v1/health", get(health_handler))
        .route("/v1/propose", post(propose_handler))
        .layer(cors)
        .with_state(app_state);

    info!(
        port = config.port,
        model = %config.model,
        token_threshold = config.token_threshold,
        allowed_origins = ?config.allowed_origins,
        "AI proxy server configured"
    );

    router
}
