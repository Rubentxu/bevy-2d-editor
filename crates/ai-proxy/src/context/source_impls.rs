//! Concrete `ContextSource` implementations for Hito 4 Order 6.
//!
//! Each source is autonomous: it knows how to serialize its input as
//! prompt text and how to truncate under a shared `TokenBudget`. The
//! `ContextBuilder` orchestrator (in `system_prompt.rs`) composes them.

use super::sources::{
    ContextSource, LogicGraphRef, Priority, SceneAssetContext, SelectedEntity, SourceFileRef,
    TokenBudget,
};
use serde_json::Value;

// ─────────────────────────────────────────────────────────────────────────────
// SceneSnapshotSource — wraps an existing scene Value (refactor target)
// ─────────────────────────────────────────────────────────────────────────────

pub struct SceneSnapshotSource {
    pub json: Value,
}

impl ContextSource for SceneSnapshotSource {
    fn name(&self) -> &'static str { "scene_snapshot" }
    fn priority(&self) -> Priority { Priority::SCENE_SNAPSHOT }
    fn total_chars(&self) -> usize {
        serde_json::to_string(&self.json).map(|s| s.len()).unwrap_or(0)
    }
    fn assemble(&self, budget: &mut TokenBudget) -> String {
        let serialized = serde_json::to_string_pretty(&self.json).unwrap_or_default();
        let truncated = crate::context::truncate_to_budget(&serialized, budget);
        format!("## Current Scene Snapshot\n\n{}", truncated)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SchemasSource — wraps the combined schema registry
// ─────────────────────────────────────────────────────────────────────────────

pub struct SchemasSource {
    pub json: Value,
}

impl ContextSource for SchemasSource {
    fn name(&self) -> &'static str { "schemas" }
    fn priority(&self) -> Priority { Priority::SCHEMAS }
    fn total_chars(&self) -> usize {
        serde_json::to_string(&self.json).map(|s| s.len()).unwrap_or(0)
    }
    fn assemble(&self, budget: &mut TokenBudget) -> String {
        let serialized = serde_json::to_string_pretty(&self.json).unwrap_or_default();
        let truncated = crate::context::truncate_to_budget(&serialized, budget);
        format!("## Available Component Schemas\n\n{}", truncated)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SourceFilesSource — Rust + toml source code (the differentiator)
// ─────────────────────────────────────────────────────────────────────────────

pub struct SourceFilesSource {
    pub files: Vec<SourceFileRef>,
}

impl ContextSource for SourceFilesSource {
    fn name(&self) -> &'static str { "source_files" }
    fn priority(&self) -> Priority { Priority::SOURCE_FILES }
    fn total_chars(&self) -> usize {
        self.files.iter().map(|f| f.content.len() + f.path.len() + 8).sum()
    }
    fn assemble(&self, budget: &mut TokenBudget) -> String {
        if self.files.is_empty() {
            return String::new();
        }
        let mut out = String::from("## Source Files\n\n");
        // Reserve some chars for the per-file header even if content gets truncated.
        let header_overhead: usize = self.files.iter().map(|f| f.path.len() + 16).sum();
        // Try to fit each file's content; truncate the last one if budget exhausted.
        for (i, f) in self.files.iter().enumerate() {
            let header = format!("=== {} ({}) ===\n", f.path, f.id);
            if budget.try_consume(header.len()) {
                out.push_str(&header);
            } else {
                break;
            }
            // Reserve at least 80 chars for content (better to drop a file than
            // emit a 3-char fragment).
            let remaining = budget.remaining();
            if remaining < 80 {
                out.push_str("[skipped: no budget remaining]\n");
                break;
            }
            // Consume up to remaining chars; if content is shorter, take all.
            let take = f.content.len().min(remaining);
            let actual = budget.consume_up_to(take);
            out.push_str(&f.content[..actual]);
            if actual < f.content.len() {
                out.push_str("\n[truncated]\n");
            }
            out.push('\n');
            // If this is the last file and budget was hit, no need to iterate further.
            if i == self.files.len() - 1 && budget.remaining() == 0 {
                break;
            }
        }
        // Suppress unused warning
        let _ = header_overhead;
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LogicGraphsSource — nodes + edges for current scene's logic graph
// ─────────────────────────────────────────────────────────────────────────────

pub struct LogicGraphsSource {
    pub graphs: Vec<LogicGraphRef>,
}

impl ContextSource for LogicGraphsSource {
    fn name(&self) -> &'static str { "logic_graphs" }
    fn priority(&self) -> Priority { Priority::LOGIC_GRAPHS }
    fn total_chars(&self) -> usize {
        self.graphs.iter().map(|g| g.asset_id.len() + 64 + g.nodes.len() * 32 + g.edges.len() * 48).sum()
    }
    fn assemble(&self, budget: &mut TokenBudget) -> String {
        if self.graphs.is_empty() {
            return String::new();
        }
        let mut out = String::from("## Logic Graphs\n\n");
        for g in &self.graphs {
            if budget.remaining() < 40 { break; }
            let header = format!("=== Graph: {} ===\n", g.asset_id);
            if !budget.try_consume(header.len()) { break; }
            out.push_str(&header);
            out.push_str("Nodes:\n");
            for n in &g.nodes {
                let line = format!("  - {} ({}): pos={}\n", n.id, n.r#type, n.position);
                if !budget.try_consume(line.len()) { break; }
                out.push_str(&line);
            }
            out.push_str("Edges:\n");
            for e in &g.edges {
                let line = format!("  - {}:{}\n    -> {}:{}\n", e.from_node, e.from_port, e.to_node, e.to_port);
                if !budget.try_consume(line.len()) { break; }
                out.push_str(&line);
            }
        }
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SceneAssetSource — catalog + (optionally) selected asset body
// ─────────────────────────────────────────────────────────────────────────────

pub struct SceneAssetSource {
    pub ctx: SceneAssetContext,
}

impl ContextSource for SceneAssetSource {
    fn name(&self) -> &'static str { "scene_assets" }
    fn priority(&self) -> Priority { Priority::SCENE_ASSET_CATALOG }
    fn total_chars(&self) -> usize {
        let catalog_chars: usize = self.ctx.catalog.iter()
            .map(|c| c.id.len() + c.name.len() + c.role.len() + 8).sum();
        catalog_chars + self.ctx.selected_body.as_ref().map(|b| b.len()).unwrap_or(0)
    }
    fn assemble(&self, budget: &mut TokenBudget) -> String {
        if self.ctx.catalog.is_empty() && self.ctx.selected_body.is_none() {
            return String::new();
        }
        let mut out = String::from("## Scene Assets\n\n");
        if !self.ctx.catalog.is_empty() {
            out.push_str("### Catalog\n");
            for c in &self.ctx.catalog {
                let line = format!("- id={} name={} role={}\n", c.id, c.name, c.role);
                if !budget.try_consume(line.len()) { break; }
                out.push_str(&line);
            }
        }
        if let Some(body) = &self.ctx.selected_body {
            out.push_str("\n### Selected Asset Body\n");
            let take = body.len().min(budget.remaining());
            let actual = budget.consume_up_to(take);
            out.push_str(&body[..actual]);
            if actual < body.len() {
                out.push_str("\n[truncated]\n");
            }
        }
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SelectedEntitySource — current inspector selection (if any)
// ─────────────────────────────────────────────────────────────────────────────

pub struct SelectedEntitySource {
    pub entity: Option<SelectedEntity>,
}

impl ContextSource for SelectedEntitySource {
    fn name(&self) -> &'static str { "selected_entity" }
    fn priority(&self) -> Priority { Priority::SELECTED_ENTITY }
    fn total_chars(&self) -> usize {
        match &self.entity {
            Some(e) => e.stable_id.len() + e.components.iter().map(|c| c.type_id.len() + 32).sum::<usize>(),
            None => 0,
        }
    }
    fn assemble(&self, budget: &mut TokenBudget) -> String {
        let entity = match &self.entity {
            Some(e) => e,
            None => return String::new(),
        };
        let mut out = format!("## Selected Entity: {}\n\n", entity.stable_id);
        out.push_str("Components:\n");
        for c in &entity.components {
            let val = serde_json::to_string(&c.values).unwrap_or_default();
            let line = format!("  - {}\n    values: {}\n", c.type_id, val);
            if !budget.try_consume(line.len()) { break; }
            out.push_str(&line);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_files_empty_returns_empty_string() {
        let s = SourceFilesSource { files: vec![] };
        let mut b = TokenBudget::new(1000);
        let out = s.assemble(&mut b);
        assert_eq!(out, "");
    }

    #[test]
    fn source_files_single_fits() {
        let s = SourceFilesSource {
            files: vec![SourceFileRef {
                id: "x".to_string(),
                path: "src/x.rs".to_string(),
                content: "fn main() {}".to_string(),
            }],
        };
        let mut b = TokenBudget::new(1000);
        let out = s.assemble(&mut b);
        assert!(out.contains("src/x.rs"));
        assert!(out.contains("fn main"));
    }

    #[test]
    fn source_files_truncates_when_over_budget() {
        let s = SourceFilesSource {
            files: vec![SourceFileRef {
                id: "x".to_string(),
                path: "src/x.rs".to_string(),
                content: "x".repeat(500),
            }],
        };
        // Budget large enough to fit the file header (~22 chars) but < 500
        // chars of content. SourceFilesSource should append "[truncated]".
        let mut b = TokenBudget::new(300);
        let out = s.assemble(&mut b);
        assert!(out.contains("[truncated]"), "expected '[truncated]' marker, got: {}", out);
    }

    #[test]
    fn source_files_two_files_second_truncated() {
        let s = SourceFilesSource {
            files: vec![
                SourceFileRef { id: "a".into(), path: "a.rs".into(), content: "AAAA".into() },
                SourceFileRef { id: "b".into(), path: "b.rs".into(), content: "B".repeat(500) },
            ],
        };
        let mut b = TokenBudget::new(50);
        let out = s.assemble(&mut b);
        assert!(out.contains("a.rs"));
        // Second file likely dropped or truncated
    }

    #[test]
    fn logic_graphs_empty_returns_empty_string() {
        let s = LogicGraphsSource { graphs: vec![] };
        let mut b = TokenBudget::new(1000);
        assert_eq!(s.assemble(&mut b), "");
    }

    #[test]
    fn logic_graphs_single_renders() {
        let s = LogicGraphsSource {
            graphs: vec![LogicGraphRef {
                asset_id: "g1".into(),
                nodes: vec![
                    super::super::sources::NodeRef {
                        id: "n1".into(), r#type: "Input".into(),
                        position: serde_json::json!({"x": 0, "y": 0}),
                    },
                ],
                edges: vec![],
            }],
        };
        let mut b = TokenBudget::new(1000);
        let out = s.assemble(&mut b);
        assert!(out.contains("g1"));
        assert!(out.contains("n1"));
    }

    #[test]
    fn scene_assets_catalog_only() {
        let s = SceneAssetSource {
            ctx: SceneAssetContext {
                catalog: vec![super::super::sources::CatalogEntry {
                    id: "a1".into(), name: "level1".into(), role: "level".into(),
                }],
                selected_body: None,
            },
        };
        let mut b = TokenBudget::new(1000);
        let out = s.assemble(&mut b);
        assert!(out.contains("a1"));
        assert!(out.contains("level1"));
    }

    #[test]
    fn scene_assets_with_selected_body_truncates() {
        let body = "X".repeat(500);
        let s = SceneAssetSource {
            ctx: SceneAssetContext {
                catalog: vec![],
                selected_body: Some(body),
            },
        };
        let mut b = TokenBudget::new(80);
        let out = s.assemble(&mut b);
        assert!(out.contains("[truncated]"));
    }

    #[test]
    fn selected_entity_none_returns_empty() {
        let s = SelectedEntitySource { entity: None };
        let mut b = TokenBudget::new(1000);
        assert_eq!(s.assemble(&mut b), "");
    }

    #[test]
    fn selected_entity_some_renders() {
        let s = SelectedEntitySource {
            entity: Some(SelectedEntity {
                stable_id: "ent_001".into(),
                components: vec![super::super::sources::ComponentRef {
                    type_id: "editor.Transform2D".into(),
                    values: serde_json::json!({"translation": {"x": 1.0}}),
                }],
            }),
        };
        let mut b = TokenBudget::new(1000);
        let out = s.assemble(&mut b);
        assert!(out.contains("ent_001"));
        assert!(out.contains("editor.Transform2D"));
    }

    #[test]
    fn scene_snapshot_truncates() {
        let big = serde_json::json!({"entities": (0..1000).map(|i| serde_json::json!({"id": format!("ent_{}", i), "name": "x"})).collect::<Vec<_>>()});
        let s = SceneSnapshotSource { json: big };
        let mut b = TokenBudget::new(100);
        let out = s.assemble(&mut b);
        assert!(!out.is_empty());
    }
}