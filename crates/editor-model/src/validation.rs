//! Project-wide validation issue types (H2.4 Asset Family collapse).
//!
//! Moved from `editor-bevy::lib` so the asset session state in
//! `editor_model::AssetSessionState` can hold `Vec<ValidationIssue>` without
//! `editor_model` depending on `editor-bevy`. This mirrors the H2.3 pattern
//! for `OperationLog` and `AssetOperationLog`.

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

/// A single project-wide validation issue surfaced by the editor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Unique identifier for this issue (stable across polls).
    pub id: String,
    /// Error = blocks save/export. Warning = non-fatal. Info = advisory.
    pub severity: ValidationSeverity,
    /// Which subsystem generated this issue.
    pub category: ValidationCategory,
    /// Machine-readable issue code (e.g. "orphaned_index", "missing_entity").
    pub code: String,
    /// Human-readable description.
    pub message: String,
    /// StableId of the affected entity, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_entity_id: Option<String>,
    /// asset_id of the affected asset, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_asset_id: Option<String>,
    /// scene_id of the affected scene, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_scene_id: Option<String>,
}

/// Severity classification for [`ValidationIssue`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Subsystem that generated a [`ValidationIssue`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationCategory {
    Catalog,
    Override,
    Export,
    Schema,
    Dirty,
    Logic,
    /// Issues produced during external source import (ADR-0041).
    Import,
    /// Other categories the editor does not classify further.
    #[serde(other)]
    Other,
}
