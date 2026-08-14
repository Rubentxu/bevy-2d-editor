//! Context assembly: system prompt building, token estimation, and scene truncation.
//!
//! Hito 4 Order 6 (`code-aware-ai`) introduced the multi-source context model.
//! See `sources.rs` for the `ContextSource` trait + `TokenBudget` + `Priority`,
//! `source_impls.rs` for the 6 concrete implementations, and `system_prompt.rs`
//! for the `ContextBuilder` orchestrator.

mod scene_truncator;
mod schema_fetcher;
mod source_impls;
pub mod sources;
mod system_prompt;

pub use scene_truncator::{estimate_tokens, truncate_scene_if_over_budget, truncate_to_budget};
pub use schema_fetcher::SchemaFetcher;
pub use sources::{
    CatalogEntry, ComponentRef, ContextSource, LogicGraphRef, NodeRef, Priority, SceneAssetContext,
    SelectedEntity, SourceFileRef, TokenBudget,
};
pub use system_prompt::ContextBuilder;
