//! ContextSource trait + concrete source types for the multi-source context
//! model introduced in Hito 4 Order 6 (`code-aware-ai`).
//!
//! Each context source is autonomous: it knows its priority, how to assemble
//! itself as text, and how to truncate under a shared `TokenBudget`. The
//! `ContextBuilder` orchestrator (in `system_prompt.rs`) iterates over
//! `Vec<Box<dyn ContextSource>>` and fills the budget greedily.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Data types carried by the request (mirror the FE MultiSourceContext shape)
// ─────────────────────────────────────────────────────────────────────────────

/// Reference to a source file visible to the AI. Full text is carried in v1
/// (no chunking); future versions may add chunking based on token budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFileRef {
    /// Stable source-file id (matches `editor.SOURCE_FILE.id`).
    pub id: String,
    /// Project-relative path, e.g. `src/player.rs`.
    pub path: String,
    /// Full file body. Truncated by `SourceFilesSource` if over budget.
    pub content: String,
}

/// Reference to a logic graph (nodes + edges).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicGraphRef {
    /// Asset id of the logic graph (matches `editor.SCENE_ASSET.id`).
    pub asset_id: String,
    pub nodes: Vec<NodeRef>,
    pub edges: Vec<EdgeRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRef {
    pub id: String,
    pub r#type: String,
    pub position: serde_json::Value, // arbitrary JSON for x/y
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRef {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

/// Scene-asset context: full catalog + (optionally) the currently-selected
/// asset's body. Per design decision D4: catalog always, body only for
/// selected asset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneAssetContext {
    #[serde(default)]
    pub catalog: Vec<CatalogEntry>,
    #[serde(default)]
    pub selected_body: Option<String>, // JSON string of the asset
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub role: String,
}

/// Currently-selected entity in the InspectorPanel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedEntity {
    pub stable_id: String,
    pub components: Vec<ComponentRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRef {
    pub type_id: String,
    pub values: serde_json::Value,
}

// ─────────────────────────────────────────────────────────────────────────────
// Priority + budget
// ─────────────────────────────────────────────────────────────────────────────

/// Priority used to order sources when filling the token budget.
/// Higher = included first. Numbers chosen so that domain-critical sources
/// (scene, schemas) are always preferred; derived sources (logic graphs,
/// asset catalog) are dropped first under pressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Priority(pub u32);

impl Priority {
    pub const SCENE_SNAPSHOT: Priority = Priority(100);
    pub const SELECTED_ENTITY: Priority = Priority(90);
    pub const SCHEMAS: Priority = Priority(80);
    pub const SCENE_ASSET_SELECTED: Priority = Priority(60);
    pub const SOURCE_FILES: Priority = Priority(50);
    pub const LOGIC_GRAPHS: Priority = Priority(40);
    pub const SCENE_ASSET_CATALOG: Priority = Priority(30);
}

/// Shared mutable budget. `chars / 4` is the project-wide token heuristic
/// (matches `scene_truncator::estimate_tokens`).
#[derive(Debug, Clone)]
pub struct TokenBudget {
    total_chars: usize,
    used_chars: usize,
}

impl TokenBudget {
    pub fn new(total_chars: usize) -> Self {
        Self { total_chars, used_chars: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.total_chars.saturating_sub(self.used_chars)
    }

    pub fn used(&self) -> usize {
        self.used_chars
    }

    pub fn total(&self) -> usize {
        self.total_chars
    }

    /// Try to consume `n` chars. Returns `true` if the entire amount fits.
    pub fn try_consume(&mut self, n: usize) -> bool {
        if self.used_chars + n <= self.total_chars {
            self.used_chars += n;
            true
        } else {
            false
        }
    }

    /// Consume up to `n` chars. Returns the number actually consumed
    /// (≤ remaining).
    pub fn consume_up_to(&mut self, n: usize) -> usize {
        let take = n.min(self.remaining());
        self.used_chars += take;
        take
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Source trait
// ─────────────────────────────────────────────────────────────────────────────

/// A source of context for the AI: a named, prioritized, self-truncating
/// chunk of text that contributes to the assembled system prompt.
pub trait ContextSource {
    /// Stable identifier (e.g. `"source_files"`, `"scene_snapshot"`).
    fn name(&self) -> &'static str;

    /// Higher priority = included first when filling the budget.
    fn priority(&self) -> Priority;

    /// Total chars this source *would* emit if budget were unlimited.
    /// Used for the context debug view (per-source stats).
    fn total_chars(&self) -> usize;

    /// Assemble the source's text, consuming from `budget`. Returns the
    /// (possibly truncated) text and the number of chars actually emitted.
    ///
    /// Implementations MUST honor the budget: if they cannot fit fully,
    /// they truncate and append `"[truncated]"` so the LLM knows.
    fn assemble(&self, budget: &mut TokenBudget) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_budget_consume_exact() {
        let mut b = TokenBudget::new(100);
        assert!(b.try_consume(60));
        assert_eq!(b.used(), 60);
        assert!(b.try_consume(40));
        assert_eq!(b.used(), 100);
        assert!(!b.try_consume(1));
    }

    #[test]
    fn token_budget_consume_up_to() {
        let mut b = TokenBudget::new(100);
        let taken = b.consume_up_to(150);
        assert_eq!(taken, 100);
        assert_eq!(b.used(), 100);
    }

    #[test]
    fn priority_ordering() {
        assert!(Priority::SCENE_SNAPSHOT > Priority::SELECTED_ENTITY);
        assert!(Priority::SOURCE_FILES > Priority::LOGIC_GRAPHS);
        assert!(Priority::LOGIC_GRAPHS > Priority::SCENE_ASSET_CATALOG);
    }

    #[test]
    fn empty_source_file_ref_deserializes() {
        let json = r#"{"id":"x","path":"a.rs","content":""}"#;
        let sf: SourceFileRef = serde_json::from_str(json).unwrap();
        assert_eq!(sf.id, "x");
    }
}