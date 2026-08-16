//! Causality edges — typed provenance links on [`PreviewProvenance`].
//
//! §6: Each [`PreviewProvenance`] MUST carry ≥ 3 [`CausalityEdge`] entries
//! from distinct [`CausalityEdgeKind`] categories.

use serde::{Deserialize, Serialize};

/// The semantic role of a single provenance link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalityEdgeKind {
    /// The entity is defined by this edge's target.
    Definition,
    /// The entity is instantiated from this edge's target.
    Instance,
    /// The entity carries a component override from this edge's target.
    Override,
    /// The entity participates in a logic-graph via this edge's target.
    Logic,
    /// The entity's source file is tracked by this edge's target.
    Source,
}

/// One provenance link attached to a [`PreviewProvenance`].
///
/// Records that the annotated entity has a causal relationship to another
/// editor entity identified by `target_stable_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalityEdge {
    /// The semantic kind of this edge.
    pub edge_kind: CausalityEdgeKind,
    /// Stable ID of the target entity this edge points to.
    pub target_stable_id: String,
}
