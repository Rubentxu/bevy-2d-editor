//! Asset Command surface for Scene Asset Authoring mode (H2.4 Asset Family).
//!
//! A separate command surface (per ADR-0007) for mutating `SceneAssetDocument`.
//! Uses `LocalId` instead of `StableId` — assets are isolated authoring documents
//! with no parent/child hierarchy in the scene sense.
//!
//! ## Design
//! - Mechanical inverse generation (same pattern as `processor.rs`)
//! - `field_path: Vec<String>` for unambiguous component field addressing (D2)
//! - `AssetOperationLog` mirrors `OperationLog` for per-asset undo/redo
//!
//! ## Inverse table (design §5)
//!
//! | Forward | Inverse |
//! |---------|---------|
//! | `AddEntity` | `RemoveEntity { local_id }` |
//! | `RemoveEntity` | `AddEntity { full captured entity }` |
//! | `RenameEntity` | `RenameEntity { old_name, swapped_new }` |
//! | `AddComponent` | `RemoveComponent { local_id, type_id }` |
//! | `RemoveComponent` | `AddComponent { captured values }` |
//! | `SetComponentValue` | `SetComponentValue { old value at field_path }` |
//! | `Batch` | `Batch { reversed inverses }` |
//!
//! ## H2.4 — Dependency inversion (matches `operation_log`)
//!
//! `AssetOperationLog::undo` / `redo` no longer call a free `apply` function
//! directly; they take a `&dyn AssetApplyCommandFn` callback so the log can
//! stay in `editor_model` while the Bevy-side application logic lives in
//! `editor_bevy::asset_command::AssetProcessorApply`.
//!
//! Tests in this module use a `TestAssetApply` stub defined at the bottom of
//! the file.

#![allow(missing_docs)]

use crate::ComponentInstance;
use crate::auto_layer::{AutoLayerId, AutoRule};
use crate::ids::{LayerId, SceneAssetLocalId};
use crate::scene_asset::{SceneAssetDocument, SceneAssetEntity};
use crate::tile_layer::TileLayerId;
use crate::tileset::{TileCoord, TileGrid, TileRef};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

// ─────────────────────────────────────────────────────────────────────────
// AssetCommand enum
// ─────────────────────────────────────────────────────────────────────────

/// Typed command enum for Scene Asset document mutations.
///
/// Uses `#[serde(tag = "type")]` so each variant serializes as
/// `{"type": "AddEntity", ...}` — self-describing and extensible.
///
/// Mirror of `Command` but for `SceneAssetDocument` with `LocalId` identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum AssetCommand {
    /// Add a new entity to the asset document.
    AddEntity {
        local_id: String,
        name: String,
        local_path: String,
        #[serde(default)]
        components: Vec<ComponentInstance>,
    },
    /// Remove an entity from the asset document.
    RemoveEntity { local_id: String },
    /// Change an entity's human-readable name.
    RenameEntity {
        local_id: String,
        /// Captured pre-state: the name before the rename.
        /// The processor populates this if caller leaves it as None.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_name: Option<String>,
        new_name: String,
    },
    /// Attach a new component instance to an existing entity.
    AddComponent {
        local_id: String,
        type_id: String,
        #[serde(default)]
        values: serde_json::Value,
    },
    /// Remove a component instance from an entity.
    RemoveComponent { local_id: String, type_id: String },
    /// Update one field of a component instance.
    /// `field_path` is `Vec<String>` for unambiguous dot-separated names.
    SetComponentValue {
        local_id: String,
        type_id: String,
        /// Array of field names: `["translation", "x"]` for `values.translation.x`.
        field_path: Vec<String>,
        value: serde_json::Value,
    },
    /// Group multiple commands into a single atomic history entry.
    Batch {
        label: String,
        commands: Vec<AssetCommand>,
    },
    /// Regenerate an AutoLayer's cached tile grid from its source TileLayer.
    ///
    /// Captures `cached` and `source_generation` for the inverse so undo
    /// can restore the pre-regeneration state.
    RegenerateAutoLayer {
        /// The layer being regenerated (identifies which LevelLayer::Auto).
        layer_id: LayerId,
        /// Captured pre-regeneration cached grid for undo.
        #[serde(default)]
        old_cached: TileGrid,
        /// Captured pre-regeneration source_generation for undo.
        #[serde(default)]
        old_source_generation: u64,
    },
    /// Paint a tile at (x, y) on a TileLayer. Captures the previous TileRef
    /// (if any) for undo via `EraseTile`.
    PaintTile {
        layer_id: TileLayerId,
        x: i32,
        y: i32,
        /// Captured pre-state: the TileRef previously at this coord,
        /// or None if the coord was empty. The processor populates this
        /// if the caller leaves it as None.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_tile: Option<TileRef>,
        tileset_id: String,
        local_index: u32,
    },
    /// Erase a tile at (x, y) on a TileLayer. Captures the erased TileRef
    /// for undo via `PaintTile`.
    EraseTile {
        layer_id: TileLayerId,
        x: i32,
        y: i32,
        /// Captured pre-state: the TileRef that was erased.
        /// The processor populates this if the caller leaves it as None.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        erased_tile: Option<TileRef>,
    },
    /// Add an AutoRule to an AutoLayer. Captures the pre-existing rules
    /// length for undo via `RemoveAutoRule`.
    AddAutoRule {
        layer_id: AutoLayerId,
        /// The new rule.
        rule: AutoRule,
    },
    /// Update an AutoRule in an AutoLayer at the given index. Captures
    /// pre-state (the old rule) for undo via `UpdateAutoRule`.
    UpdateAutoRule {
        layer_id: AutoLayerId,
        index: usize,
        /// Captured pre-state: the rule being replaced.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_rule: Option<AutoRule>,
        new_rule: AutoRule,
    },
    /// Remove an AutoRule from an AutoLayer at the given index. Captures
    /// pre-state (the removed rule) for undo via `AddAutoRule`.
    RemoveAutoRule {
        layer_id: AutoLayerId,
        index: usize,
        /// Captured pre-state: the rule being removed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        removed_rule: Option<AutoRule>,
    },
    /// Set / replace a layer-level IntGrid cell value.
    SetIntGridCell {
        layer_id: LayerId,
        coord: TileCoord,
        /// Captured pre-state: the cell value before this write. The processor
        /// populates this if the caller leaves it as None.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_value: Option<i32>,
        value: i32,
    },
}

/// Errors returned by [`AssetCommand`] application / undo / redo.
#[derive(Debug, Error)]
pub enum AssetCommandError {
    #[error("entity not found: {0}")]
    EntityNotFound(String),

    #[error("duplicate local_id: {0}")]
    DuplicateLocalId(String),

    #[error("component not found: {0}")]
    ComponentNotFound(String),

    #[error("field not found: {0:?}")]
    FieldNotFound(Vec<String>),

    #[error("batch failed at {index}: {source}")]
    BatchFailed {
        index: usize,
        #[source]
        source: Box<AssetCommandError>,
    },

    #[error("JSON error: {0}")]
    JsonError(String),

    #[error("layer not found: {0}")]
    LayerNotFound(String),

    #[error("no tile at ({x}, {y}) on layer {layer_id}")]
    TileNotFound { layer_id: String, x: i32, y: i32 },

    #[error("source layer not found for auto layer: {0}")]
    SourceLayerNotFound(String),
}

impl From<serde_json::Error> for AssetCommandError {
    fn from(e: serde_json::Error) -> Self {
        AssetCommandError::JsonError(e.to_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────
// AssetApplyCommandFn callback trait (H2.4 dependency inversion)
// ─────────────────────────────────────────────────────────────────────────
//
// Mirrors `operation_log::ApplyCommandFn`. Breaks the dependency
// `AssetOperationLog` would otherwise have on `editor_bevy::asset_command::apply`,
// so the log type can live canonically in `editor_model`.

/// Callback trait that applies an [`AssetCommand`] to a [`SceneAssetDocument`].
///
/// `editor_bevy` provides a concrete impl (`AssetProcessorApply`) that
/// delegates to its Bevy-side application logic; tests in this module use
/// the `TestAssetApply` stub.
pub trait AssetApplyCommandFn {
    /// Apply `cmd` to `doc`, returning the inverse command (or an error).
    fn apply(
        &self,
        doc: &mut SceneAssetDocument,
        cmd: &AssetCommand,
    ) -> Result<AssetCommand, AssetCommandError>;
}

// ─────────────────────────────────────────────────────────────────────────
// AssetOperationLog
// ─────────────────────────────────────────────────────────────────────────

/// Single entry in the asset operation log: forward command, inverse, and metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetLogEntry {
    pub forward: AssetCommand,
    pub inverse: AssetCommand,
}

/// Append-only history with cursor-based undo/redo for asset commands.
///
/// H2.4: `undo` and `redo` take a `&dyn AssetApplyCommandFn` callback so the
/// type can live in `editor_model` without depending on Bevy processor code.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AssetOperationLog {
    entries: Vec<AssetLogEntry>,
    cursor: isize,
    max_size: usize,
}

impl AssetOperationLog {
    /// Default max log size (matches `OperationLog`).
    pub const DEFAULT_MAX_LOG_SIZE: usize = 1000;

    /// Create a new empty log with default max size.
    pub fn new() -> Self {
        Self::with_max_size(Self::DEFAULT_MAX_LOG_SIZE)
    }

    /// Const constructor preserved for source-compatibility even though
    /// H2.4 removes the asset thread_locals.
    pub const fn new_const() -> Self {
        Self {
            entries: Vec::new(),
            cursor: -1,
            max_size: Self::DEFAULT_MAX_LOG_SIZE,
        }
    }

    /// Create a new empty log with custom max size.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            cursor: -1,
            max_size,
        }
    }

    /// Record a forward command and its inverse after apply.
    pub fn record(&mut self, forward: &AssetCommand, inverse: AssetCommand) {
        // Truncate redo branch
        if self.cursor < self.entries.len() as isize - 1 {
            let keep = (self.cursor + 1) as usize;
            self.entries.truncate(keep);
        }
        self.entries.push(AssetLogEntry {
            forward: forward.clone(),
            inverse,
        });
        while self.entries.len() > self.max_size {
            self.entries.remove(0);
            self.cursor -= 1;
        }
        self.cursor = self.entries.len() as isize - 1;
    }

    /// Apply the inverse of the entry at the cursor, moving the cursor back.
    pub fn undo(
        &mut self,
        doc: &mut SceneAssetDocument,
        apply_fn: &dyn AssetApplyCommandFn,
    ) -> Result<(), AssetCommandError> {
        if !self.can_undo() {
            return Err(AssetCommandError::JsonError("Nothing to undo".to_string()));
        }
        let entry = self.entries[self.cursor as usize].clone();
        apply_fn.apply(doc, &entry.inverse)?;
        self.cursor -= 1;
        Ok(())
    }

    /// Apply the forward of the entry after the cursor, moving forward.
    pub fn redo(
        &mut self,
        doc: &mut SceneAssetDocument,
        apply_fn: &dyn AssetApplyCommandFn,
    ) -> Result<(), AssetCommandError> {
        if !self.can_redo() {
            return Err(AssetCommandError::JsonError("Nothing to redo".to_string()));
        }
        self.cursor += 1;
        let entry = self.entries[self.cursor as usize].clone();
        apply_fn.apply(doc, &entry.forward)?;
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.cursor >= 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor < self.entries.len() as isize - 1
    }

    pub fn get_log_size(&self) -> usize {
        self.entries.len()
    }

    pub fn get_cursor(&self) -> isize {
        self.cursor
    }

    /// Returns true if there are un-saved changes (entries beyond cursor).
    pub fn is_dirty(&self) -> bool {
        self.cursor >= 0
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.cursor = -1;
    }

    /// Snapshot all entries (H2.4 — used by recent-change-sets polling).
    pub fn snapshot_entries(&self) -> Vec<AssetLogEntry> {
        self.entries.clone()
    }
}

/// Counts of override statuses reported by the resync workflow.
///
/// H2.4: moved from `editor-bevy::scene_instance_overrides` so that
/// `AssetSessionState` (in `editor_model`) can hold
/// `Vec<(StableId, ResyncReport)>` without a back-reference to the Bevy
/// crate. Mirrors the H2.3 migration pattern for the asset operation log.
#[allow(missing_docs)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResyncReport {
    /// Number of overrides currently in active (consistent) state.
    pub active: usize,
    /// Number of overrides whose target asset entity is missing.
    pub orphaned: usize,
    /// Number of overrides whose target field path no longer resolves.
    pub stale: usize,
    /// Number of overrides in conflict with the current asset value.
    pub conflict: usize,
    /// Number of overrides that were rebound to a new asset field this cycle.
    pub rebound: usize,
}

// ─────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_asset::SceneAssetMetadata;
    use crate::scene_asset::SceneAssetRole;

    /// Minimal apply callback that handles AddEntity / RemoveEntity /
    /// RenameEntity / AddComponent / RemoveComponent / SetComponentValue.
    /// Mirrors the production `AssetProcessorApply` shape closely enough for
    /// undo/redo tests; production logic that depends on Bevy sub-systems
    /// (auto-layer regeneration, etc.) is not exercised here.
    struct TestAssetApply;

    impl AssetApplyCommandFn for TestAssetApply {
        fn apply(
            &self,
            doc: &mut SceneAssetDocument,
            cmd: &AssetCommand,
        ) -> Result<AssetCommand, AssetCommandError> {
            match cmd {
                AssetCommand::AddEntity {
                    local_id,
                    name,
                    local_path,
                    components,
                } => {
                    let id = SceneAssetLocalId(local_id.clone());
                    if doc.entities.iter().any(|e| e.local_id == id) {
                        return Err(AssetCommandError::DuplicateLocalId(local_id.clone()));
                    }
                    doc.entities.push(SceneAssetEntity {
                        local_id: id,
                        local_path: local_path.clone(),
                        name: name.clone(),
                        components: components.clone(),
                        extension_data: BTreeMap::new(),
                    });
                    Ok(AssetCommand::RemoveEntity {
                        local_id: local_id.clone(),
                    })
                }
                AssetCommand::RemoveEntity { local_id } => {
                    let id = SceneAssetLocalId(local_id.clone());
                    let idx = doc
                        .entities
                        .iter()
                        .position(|e| e.local_id == id)
                        .ok_or_else(|| AssetCommandError::EntityNotFound(local_id.clone()))?;
                    let removed = doc.entities.remove(idx);
                    Ok(AssetCommand::AddEntity {
                        local_id: removed.local_id.0.clone(),
                        name: removed.name.clone(),
                        local_path: removed.local_path.clone(),
                        components: removed.components.clone(),
                    })
                }
                AssetCommand::RenameEntity {
                    local_id, new_name, ..
                } => {
                    let id = SceneAssetLocalId(local_id.clone());
                    let entity = doc
                        .entities
                        .iter_mut()
                        .find(|e| e.local_id == id)
                        .ok_or_else(|| AssetCommandError::EntityNotFound(local_id.clone()))?;
                    let old_name = entity.name.clone();
                    let original = old_name.clone();
                    entity.name = new_name.clone();
                    Ok(AssetCommand::RenameEntity {
                        local_id: local_id.clone(),
                        old_name: Some(old_name),
                        new_name: original,
                    })
                }
                AssetCommand::AddComponent {
                    local_id,
                    type_id,
                    values,
                } => {
                    let id = SceneAssetLocalId(local_id.clone());
                    let entity = doc
                        .entities
                        .iter_mut()
                        .find(|e| e.local_id == id)
                        .ok_or_else(|| AssetCommandError::EntityNotFound(local_id.clone()))?;
                    entity.components.push(ComponentInstance {
                        type_id: type_id.clone(),
                        values: values.clone(),
                    });
                    Ok(AssetCommand::RemoveComponent {
                        local_id: local_id.clone(),
                        type_id: type_id.clone(),
                    })
                }
                AssetCommand::RemoveComponent { local_id, type_id } => {
                    let id = SceneAssetLocalId(local_id.clone());
                    let entity = doc
                        .entities
                        .iter_mut()
                        .find(|e| e.local_id == id)
                        .ok_or_else(|| AssetCommandError::EntityNotFound(local_id.clone()))?;
                    let idx = entity
                        .components
                        .iter()
                        .position(|c| c.type_id == *type_id)
                        .ok_or_else(|| AssetCommandError::ComponentNotFound(type_id.clone()))?;
                    let captured = entity.components.remove(idx);
                    Ok(AssetCommand::AddComponent {
                        local_id: local_id.clone(),
                        type_id: type_id.clone(),
                        values: captured.values,
                    })
                }
                AssetCommand::SetComponentValue {
                    local_id,
                    type_id,
                    field_path,
                    value,
                } => {
                    let id = SceneAssetLocalId(local_id.clone());
                    let entity = doc
                        .entities
                        .iter_mut()
                        .find(|e| e.local_id == id)
                        .ok_or_else(|| AssetCommandError::EntityNotFound(local_id.clone()))?;
                    let component = entity
                        .components
                        .iter_mut()
                        .find(|c| c.type_id == *type_id)
                        .ok_or_else(|| AssetCommandError::ComponentNotFound(type_id.clone()))?;
                    let prior =
                        set_field_path_vec(&mut component.values, field_path, value.clone());
                    Ok(AssetCommand::SetComponentValue {
                        local_id: local_id.clone(),
                        type_id: type_id.clone(),
                        field_path: field_path.clone(),
                        value: prior,
                    })
                }
                _ => Ok(AssetCommand::Batch {
                    label: "noop".to_string(),
                    commands: vec![],
                }),
            }
        }
    }

    /// Dotted-path setter used by the test stub. Mirrors the production
    /// `editor_bevy::asset_command::set_field_path_vec`.
    fn set_field_path_vec(
        root: &mut serde_json::Value,
        path: &[String],
        new: serde_json::Value,
    ) -> serde_json::Value {
        let mut cur = root;
        for seg in path.iter().take(path.len().saturating_sub(1)) {
            match cur {
                serde_json::Value::Object(map) => {
                    let entry = map.entry(seg.clone()).or_insert(serde_json::Value::Null);
                    cur = entry;
                }
                _ => return new,
            }
        }
        if let Some(last) = path.last() {
            if let serde_json::Value::Object(map) = cur {
                let old = map.get(last).cloned().unwrap_or(serde_json::Value::Null);
                map.insert(last.clone(), new.clone());
                return old;
            }
        }
        new
    }

    fn empty_doc() -> SceneAssetDocument {
        SceneAssetDocument {
            layers: vec![],
            asset_id: "id_test".to_string(),
            logical_path: "test/asset".to_string(),
            role: SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            extension_data: BTreeMap::new(),
        }
    }

    fn entity(local_id: &str, name: &str, components: Vec<ComponentInstance>) -> SceneAssetEntity {
        SceneAssetEntity {
            local_id: SceneAssetLocalId(local_id.to_string()),
            local_path: format!("./{}", local_id),
            name: name.to_string(),
            components,
            extension_data: BTreeMap::new(),
        }
    }

    #[test]
    fn add_entity_applies_and_inverts_via_log() {
        let mut doc = empty_doc();
        let cmd = AssetCommand::AddEntity {
            local_id: "a1".to_string(),
            name: "A".to_string(),
            local_path: "./a1".to_string(),
            components: vec![],
        };
        let mut log = AssetOperationLog::new();
        let inverse = TestAssetApply.apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 1);
        log.record(&cmd, inverse);
        log.undo(&mut doc, &TestAssetApply).unwrap();
        assert_eq!(doc.entities.len(), 0);
        log.redo(&mut doc, &TestAssetApply).unwrap();
        assert_eq!(doc.entities.len(), 1);
    }

    #[test]
    fn rename_entity_round_trip() {
        let mut doc = empty_doc();
        doc.entities.push(entity("a1", "Original", vec![]));
        let cmd = AssetCommand::RenameEntity {
            local_id: "a1".to_string(),
            old_name: None,
            new_name: "Renamed".to_string(),
        };
        let mut log = AssetOperationLog::new();
        let inverse = TestAssetApply.apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].name, "Renamed");
        log.record(&cmd, inverse);
        log.undo(&mut doc, &TestAssetApply).unwrap();
        assert_eq!(doc.entities[0].name, "Original");
    }

    #[test]
    fn cannot_undo_when_empty() {
        let mut doc = empty_doc();
        let mut log = AssetOperationLog::new();
        let err = log.undo(&mut doc, &TestAssetApply).unwrap_err();
        assert!(matches!(err, AssetCommandError::JsonError(_)));
    }

    #[test]
    fn max_size_evicts_oldest() {
        let mut log = AssetOperationLog::with_max_size(2);
        let cmd = AssetCommand::Batch {
            label: "noop".to_string(),
            commands: vec![],
        };
        log.record(&cmd, cmd.clone());
        log.record(&cmd, cmd.clone());
        log.record(&cmd, cmd.clone());
        assert_eq!(log.get_log_size(), 2);
        assert_eq!(log.get_cursor(), 1);
    }
}
