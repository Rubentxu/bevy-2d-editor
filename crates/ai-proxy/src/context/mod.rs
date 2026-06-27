//! Context assembly: system prompt building, token estimation, and scene truncation.

mod schema_fetcher;
mod scene_truncator;
mod system_prompt;

pub use schema_fetcher::SchemaFetcher;
pub use scene_truncator::{estimate_tokens, truncate_scene_if_over_budget};
pub use system_prompt::ContextBuilder;
