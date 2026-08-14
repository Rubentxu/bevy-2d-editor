//! Unified error type for the AI proxy with HTTP status mapping.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("OPENAI_API_KEY not configured")]
    MissingApiKey,

    #[error("OpenAI API error: {0}")]
    OpenAIError(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, ErrorBody { error: msg.clone() })
            }
            AppError::MissingApiKey => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorBody {
                    error: "OPENAI_API_KEY not configured".to_string(),
                },
            ),
            AppError::OpenAIError(msg) => (
                StatusCode::BAD_GATEWAY,
                ErrorBody {
                    error: format!("OpenAI API error: {}", msg),
                },
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorBody { error: msg.clone() },
            ),
        };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_api_key_status() {
        let err = AppError::MissingApiKey;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_bad_request_status() {
        let err = AppError::BadRequest("missing field 'prompt'".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_openai_error_status() {
        let err = AppError::OpenAIError("timeout".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_internal_error_status() {
        let err = AppError::Internal("unexpected".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
