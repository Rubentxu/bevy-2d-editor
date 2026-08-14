//! Application configuration loaded from environment variables.

use thiserror::Error;

const DEFAULT_PORT: u16 = 11435;
const DEFAULT_MODEL: &str = "gpt-4o";
const DEFAULT_TOKEN_THRESHOLD: usize = 10_000;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("OPENAI_API_KEY environment variable is not set")]
    MissingApiKey,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub openai_api_key: String,
    pub port: u16,
    pub model: String,
    pub token_threshold: usize,
    pub allowed_origins: Vec<String>,
}

impl AppConfig {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    /// Returns `ConfigError::MissingApiKey` if `OPENAI_API_KEY` is not set.
    pub fn from_env() -> Result<Self, ConfigError> {
        let openai_api_key =
            std::env::var("OPENAI_API_KEY").map_err(|_| ConfigError::MissingApiKey)?;

        let port: u16 = std::env::var("PORT")
            .unwrap_or_else(|_| DEFAULT_PORT.to_string())
            .parse()
            .unwrap_or(DEFAULT_PORT);

        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        let token_threshold: usize = std::env::var("TOKEN_THRESHOLD")
            .unwrap_or_else(|_| DEFAULT_TOKEN_THRESHOLD.to_string())
            .parse()
            .unwrap_or(DEFAULT_TOKEN_THRESHOLD);

        // CORS: comma-separated list of allowed origins
        let allowed_origins: Vec<String> = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            openai_api_key,
            port,
            model,
            token_threshold,
            allowed_origins,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loads_defaults() {
        // Clear relevant env vars
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("PORT");
        std::env::remove_var("OPENAI_MODEL");
        std::env::remove_var("TOKEN_THRESHOLD");
        std::env::remove_var("ALLOWED_ORIGINS");

        // Should fail because OPENAI_API_KEY is required
        let result = AppConfig::from_env();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::MissingApiKey));
    }
}
