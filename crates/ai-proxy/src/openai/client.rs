//! OpenAI Chat Completions client using reqwest.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::function_calling::{parse_tool_calls, propose_commands_schema, CommandEnvelope};
use crate::error::AppError;

/// OpenAI API endpoint for chat completions.
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// Request body sent to OpenAI's /chat/completions endpoint.
#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    tools: Vec<Value>,
    #[serde(rename = "tool_choice")]
    tool_choice: ToolChoice,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum ToolChoice {
    Auto,
}

/// OpenAI API response body.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    // The model that was used
    #[serde(default)]
    model: Option<String>,
    /// Choices array — we use the first choice's message.
    #[serde(default)]
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    #[serde(default)]
    tool_calls: Vec<Value>,
    #[serde(default)]
    content: Option<String>,
}

/// Client for calling the OpenAI Chat Completions API with function-calling.
#[derive(Clone)]
pub struct OpenAIClient {
    http: Client,
    api_key: String,
    model: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client.
    pub fn new(api_key: String, model: String) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("reqwest Client must be buildable");
        Self {
            http,
            api_key,
            model,
        }
    }

    /// Call OpenAI with a user prompt and return parsed `CommandEnvelope`s.
    ///
    /// # Errors
    /// Returns `AppError::OpenAIError` on network failure, HTTP non-2xx, or API error.
    pub async fn propose_commands(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<(Vec<CommandEnvelope>, String), AppError> {
        let tool_schema = propose_commands_schema();

        let request_body = ChatRequest {
            model: &self.model,
            messages: vec![
                Message {
                    role: "system",
                    content: system_prompt,
                },
                Message {
                    role: "user",
                    content: user_prompt,
                },
            ],
            tools: vec![tool_schema],
            tool_choice: ToolChoice::Auto,
        };

        let response = self
            .http
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::OpenAIError(format!("request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::OpenAIError(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }

        let chat_resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::OpenAIError(format!("failed to parse response: {}", e)))?;

        let model = chat_resp.model.unwrap_or_else(|| self.model.clone());

        let choice = chat_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| AppError::OpenAIError("no choices in response".to_string()))?;

        let tool_calls = choice.message.tool_calls;

        // Extract rationale from content if available
        let rationale = choice.message.content.filter(|c| !c.is_empty());

        let envelopes = parse_tool_calls(tool_calls, &model, rationale);
        Ok((envelopes, model))
    }
}

#[cfg(test)]
mod tests {
    // Integration tests using wiremock are in the integration test file.
    // Unit tests for the client itself are minimal since it requires network.

    use super::*;

    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::new("sk-test".to_string(), "gpt-4o".to_string());
        assert_eq!(client.model, "gpt-4o");
    }
}
