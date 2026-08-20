//! Capability API trait definitions for the typed editor backend.
//! Per ADR-0034 and typed-editor-backend-delta spec.
//!
//! Implementations live in:
//!   - editor-wasm (WASM adapter)
//!   - editor-application (test doubles)
//!   - editor-bevy (RuntimeApi only)

/// Scene authoring capability — entity CRUD and field mutations.
/// Corresponds to SceneApi in typed-editor-backend spec.
pub trait SceneApi {
    // Entity lifecycle
    fn create_entity(&mut self) -> Result<StableId, DispatchError>;
    fn delete_entity(&mut self, id: StableId) -> Result<(), DispatchError>;
    fn duplicate_entity(&mut self, id: StableId) -> Result<StableId, DispatchError>;

    // Field mutations
    fn set_field(
        &mut self,
        entity: StableId,
        component: String,
        field: String,
        value: serde_json::Value,
    ) -> Result<(), DispatchError>;
    fn get_field(
        &self,
        entity: StableId,
        component: String,
        field: String,
    ) -> Result<serde_json::Value, DispatchError>;
}

/// Scene asset authoring capability — place, replace, validate overrides.
/// Corresponds to SceneAssetApi in typed-editor-backend spec.
pub trait SceneAssetApi {
    fn place_instance(
        &mut self,
        asset_ref: String,
        position: [f32; 2],
    ) -> Result<StableId, DispatchError>;
    fn replace_asset(
        &mut self,
        instance_id: StableId,
        new_asset_ref: String,
    ) -> Result<(), DispatchError>;
    fn validate_overrides(&self, instance_id: StableId) -> Result<ValidationReport, DispatchError>;
    fn get_override_status(&self, instance_id: StableId) -> Result<OverrideStatus, DispatchError>;
}

/// World workspace capability — topology and navigation per ADR-0037.
///
/// Expanded from the 3-method stub at `capabilities.rs:50–54`. The new
/// shape mirrors `SceneAssetApi` (level mutation) plus the validation
/// extension (LDtk-faithful topology rules per spec §ww-validation).
pub trait WorldApi {
    /// Snapshot the active world (or the only one) for canvas read.
    fn get_workspace(&self) -> Result<WorldSummary, DispatchError>;

    /// Add a level ref to the world. Returns the new `level_id`.
    /// ADR-0037 + workspace cap: errors with `WorkspaceTooLarge` past 100.
    fn add_level_to_world(
        &mut self,
        asset_ref: String,
        position: [f32; 2],
    ) -> Result<String, DispatchError>;

    /// Remove a level ref and all incident links.
    fn remove_level_from_world(&mut self, level_id: String) -> Result<(), DispatchError>;

    /// Insert a one-way or bidirectional link between two level refs.
    fn connect_levels(
        &mut self,
        from: String,
        to: String,
        direction: LinkDirection,
        kind: WorldLinkKind,
    ) -> Result<String, DispatchError>;

    /// Move a level ref to a new world-space position.
    fn place_level(&mut self, level_id: String, position: [f32; 2]) -> Result<(), DispatchError>;

    /// Replace the active `LayoutPolicy`.
    fn set_layout_policy(&mut self, policy: LayoutPolicy) -> Result<(), DispatchError>;

    /// Return the list of level ids unreachable from the world entry.
    /// Forwarded to `validate_topology` (Warning severity, not Error).
    fn find_unreachable(&self) -> Result<Vec<String>, DispatchError>;

    /// Compute a non-binding layout proposal for the current `levels`
    /// list under the active `LayoutPolicy` (positions are returned,
    /// not applied). Used by the canvas toolbar to snap alignment.
    fn layout_world_proposal(&self) -> Result<Vec<[f32; 2]>, DispatchError>;

    /// Set per-level streaming policy.
    fn set_streaming_policy(
        &mut self,
        level_id: String,
        policy: StreamingPolicy,
    ) -> Result<(), DispatchError>;

    /// Run topology validation. Returns the full `Vec<TopologyIssue>`
    /// so the Validation Center can dedupe / cluster.
    fn validate_topology(&self) -> Result<Vec<TopologyIssue>, DispatchError>;
}

/// Logic graph authoring capability — nodes and edges.
/// Corresponds to LogicApi in typed-editor-backend spec.
pub trait LogicApi {
    fn add_node(
        &mut self,
        graph_id: StableId,
        node_type: String,
        position: [f32; 2],
    ) -> Result<StableId, DispatchError>;
    fn connect_ports(
        &mut self,
        edge_id: StableId,
        from: PortRef,
        to: PortRef,
    ) -> Result<(), DispatchError>;
    fn set_node_field(
        &mut self,
        node_id: StableId,
        field: String,
        value: serde_json::Value,
    ) -> Result<(), DispatchError>;
    fn delete_node(&mut self, node_id: StableId) -> Result<(), DispatchError>;
}

/// Runtime control capability — play mode and runtime deltas.
/// Corresponds to RuntimeApi in typed-editor-backend spec.
pub trait RuntimeApi {
    fn enter_play_mode(&mut self) -> Result<(), DispatchError>;
    fn exit_play_mode(&mut self) -> Result<(), DispatchError>;
    fn get_runtime_deltas(&self) -> Result<Vec<RuntimeDelta>, DispatchError>;
    fn rebuild_preview_world(&mut self) -> Result<RebuildReport, DispatchError>;
}

/// Code authoring capability — source file CRUD per ADR-0043.
/// Corresponds to CodeApi in typed-editor-backend spec.
pub trait CodeApi {
    fn create_source_file(&mut self, path: String, content: String) -> Result<(), DispatchError>;
    fn write_source_file(&mut self, path: String, content: String) -> Result<(), DispatchError>;
    // D2: delete and rename are FORBIDDEN in v1
}

/// Validation and diagnostics capability.
pub trait ValidationApi {
    fn get_validation_issues(&self) -> Result<ValidationReport, DispatchError>;
    fn get_resync_reports(&self, instance_id: StableId)
    -> Result<Vec<ResyncReport>, DispatchError>;

    /// Topology issues for the given world. Returns an empty Vec when
    /// the world has no validation problems.
    fn get_topology_issues(&self, world_id: StableId) -> Result<Vec<TopologyIssue>, DispatchError>;
}

/// Change submission and approval capability.
/// Corresponds to ChangeApi in typed-editor-backend spec.
pub trait ChangeApi {
    fn submit_pending_change_set(&mut self) -> Result<ChangeSetSummary, DispatchError>;
    fn approve_change_set(&mut self, id: StableId) -> Result<ApplyReceipt, DispatchError>;
    fn reject_change_set(&mut self, id: StableId, reason: String) -> Result<(), DispatchError>;
    fn get_change_history(
        &self,
        scope: HistoryScope,
    ) -> Result<Vec<ChangeSetSummary>, DispatchError>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Supporting types used by the capability traits
// ─────────────────────────────────────────────────────────────────────────────

use crate::dispatch_error::DispatchError;
pub use editor_model::StableId;
use editor_model::world::{
    LayoutPolicy, LinkDirection, StreamingPolicy, WorldId, WorldLevelRef, WorldLink, WorldLinkKind,
};

/// Reference to a port on a logic node.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortRef {
    pub node_id: StableId,
    pub port_id: String,
}

/// Snapshot of the world workspace topology.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceSnapshot {
    pub levels: Vec<LevelSummary>,
}

/// Summary of a level in the workspace.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LevelSummary {
    pub id: StableId,
    pub name: String,
    pub entity_count: usize,
}

/// Override validation report.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidationReport {
    pub active: usize,
    pub stale: usize,
    pub orphaned: usize,
    pub conflict: usize,
}

/// Status of overrides for a scene instance.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OverrideStatus {
    pub active: usize,
    pub stale: usize,
    pub orphaned: usize,
    pub conflict: usize,
}

/// A runtime delta between authoring baseline and runtime value.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeDelta {
    pub entity_id: StableId,
    pub component: String,
    pub field: String,
    pub author_value: serde_json::Value,
    pub runtime_value: serde_json::Value,
}

/// Report from rebuilding the preview world.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RebuildReport {
    pub rebuild_count: u32,
    pub duration_ms: u64,
}

/// Summary of a change set.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChangeSetSummary {
    pub id: StableId,
    pub label: String,
    pub origin: String,
    pub author: String,
    pub timestamp: u64,
    pub operation_count: usize,
}

/// Receipt from applying a change set.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ApplyReceipt {
    pub change_set_id: StableId,
    pub applied_at: u64,
    pub operations_applied: usize,
}

/// Scope for change history queries.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HistoryScope {
    pub entity_id: Option<StableId>,
    pub scene_id: Option<String>,
    pub limit: Option<usize>,
}

/// Resync report for a scene instance.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResyncReport {
    pub instance_id: StableId,
    pub stale_overrides: Vec<StaleOverride>,
}

/// A stale override entry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StaleOverride {
    pub local_id: String,
    pub component_type_id: String,
    pub field_path: Vec<String>,
    pub reason: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// World Workspace DTOs (ADR-0037)
// ─────────────────────────────────────────────────────────────────────────────

/// Snapshot of a world for the canvas / frontend consumption.
/// Replaces the old `WorkspaceSnapshot` / `LevelSummary` pair.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorldSummary {
    pub id: WorldId,
    pub world_id: String,
    pub name: String,
    pub layout_policy: LayoutPolicy,
    pub levels: Vec<WorldLevelRef>,
    pub links: Vec<WorldLink>,
    pub current_version: u32,
    pub updated_at: u64,
}

/// Summary of a world link for lightweight canvas rendering.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorldLinkSummary {
    pub id: String,
    pub from: String,
    pub to: String,
    pub direction: LinkDirection,
    pub kind: WorldLinkKind,
}

/// Issue code for topology validation errors.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyIssueCode {
    /// Level unreachable from the world entry.
    Unreachable,
    /// Reciprocal link mismatch (A→B but B has no link to A).
    InvalidReciprocal,
    /// LDtk neighbour reference points to a level not in the world.
    MissingNeighbour,
    /// WorldLevelRef.asset_ref does not resolve in SceneAssetCatalog.
    MissingLevelRef,
    /// Cycle detected in the world's link graph (new in GRAPH-005).
    Cycle,
}

/// Severity level for topology issues.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologySeverity {
    Warning,
    Error,
}

/// A single topology validation issue.
///
/// LDtk-faithful severity matrix:
/// - `Unreachable` → Warning
/// - `InvalidReciprocal` → Warning
/// - `MissingNeighbour` → Warning
/// - `MissingLevelRef` → Error
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TopologyIssue {
    pub code: TopologyIssueCode,
    pub world_id: String,
    pub level_id: Option<String>,
    pub link_id: Option<String>,
    pub severity: TopologySeverity,
    pub message: String,
}
