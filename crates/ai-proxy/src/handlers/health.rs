//! `GET /v1/health` handler.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

/// Health check response body.
#[derive(Serialize)]
struct HealthBody {
    status: &'static str,
}

/// `GET /v1/health` — returns 200 OK indicating the proxy is running.
pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthBody { status: "ok" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        routing::get,
        Router,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_200() {
        let app: Router = Router::new().route("/v1/health", get(health_handler));

        let res = app
            .oneshot(
                http::Request::builder()
                    .uri("/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("\"status\":\"ok\""));
    }
}
