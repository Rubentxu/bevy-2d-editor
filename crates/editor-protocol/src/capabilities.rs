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
pub trait WorldApi {
    fn get_workspace(&self) -> Result<WorkspaceSnapshot, DispatchError>;
    fn add_level(&mut self, name: String) -> Result<StableId, DispatchError>;
    fn remove_level(&mut self, id: StableId) -> Result<(), DispatchError>;
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
use editor_model::StableId;

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
