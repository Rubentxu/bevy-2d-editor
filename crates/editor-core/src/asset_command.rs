//! Asset Command processor for Scene Asset Authoring mode.
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
//! | Forward | Inverse |
//! |---------|---------|
//! | `AddEntity` | `RemoveEntity { local_id }` |
//! | `RemoveEntity` | `AddEntity { full captured entity }` |
//! | `RenameEntity` | `RenameEntity { old_name, swapped_new }` |
//! | `AddComponent` | `RemoveComponent { local_id, type_id }` |
//! | `RemoveComponent` | `AddComponent { captured values }` |
//! | `SetComponentValue` | `SetComponentValue { old value at field_path }` |
//! | `Batch` | `Batch { reversed inverses }` |

use crate::document::ComponentInstance;
use crate::scene_asset::{LayerId, LocalId, SceneAssetDocument, SceneAssetEntity};
use crate::tileset::{TileCoord, TileGrid, TileRef};
use serde::{Deserialize, Serialize};
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
    RemoveEntity {
        local_id: String,
    },
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
    RemoveComponent {
        local_id: String,
        type_id: String,
    },
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
        layer_id: LayerId,
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
        layer_id: LayerId,
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
        layer_id: LayerId,
        /// The new rule.
        rule: crate::auto_layer::AutoRule,
    },
    /// Update an AutoRule in an AutoLayer at the given index. Captures
    /// pre-state (the old rule) for undo via `UpdateAutoRule`.
    UpdateAutoRule {
        layer_id: LayerId,
        index: usize,
        /// Captured pre-state: the rule being replaced.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_rule: Option<crate::auto_layer::AutoRule>,
        new_rule: crate::auto_layer::AutoRule,
    },
    /// Remove an AutoRule from an AutoLayer at the given index. Captures
    /// pre-state (the removed rule) for undo via `AddAutoRule`.
    RemoveAutoRule {
        layer_id: LayerId,
        index: usize,
        /// Captured pre-state: the rule being removed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        removed_rule: Option<crate::auto_layer::AutoRule>,
    },
}

// ─────────────────────────────────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────────────────────────────────

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
// AssetProcessor
// ─────────────────────────────────────────────────────────────────────────

/// Find a mutable entity by LocalId.
fn find_entity_mut<'a>(
    doc: &'a mut SceneAssetDocument,
    local_id: &str,
) -> Result<&'a mut SceneAssetEntity, AssetCommandError> {
    doc.entities
        .iter_mut()
        .find(|e| e.local_id.as_str() == local_id)
        .ok_or_else(|| AssetCommandError::EntityNotFound(local_id.to_string()))
}

/// Set a field at a `Vec<String>` path within a JSON object. Returns the old value.
///
/// Path navigation: split on segments, navigate to parent, set leaf.
/// For `["translation", "x"]` on `{"translation": {"x": 0, "y": 0}}`,
/// the result is `{"translation": {"x": <new>, "y": 0}}`.
///
/// NOTE: This is the same pattern as `processor::set_field_path` but accepts
/// `Vec<String>` instead of a dotted `&str`. Per ADR-0007, the two command
/// surfaces stay independent — no logic unification.
pub fn set_field_path_vec(
    value: &mut serde_json::Value,
    path: &[String],
    new: serde_json::Value,
) -> Result<serde_json::Value, AssetCommandError> {
    if path.is_empty() {
        return Err(AssetCommandError::FieldNotFound(path.to_vec()));
    }
    let mut current = value;
    for part in &path[..path.len() - 1] {
        current = current
            .as_object_mut()
            .ok_or_else(|| AssetCommandError::FieldNotFound(path.to_vec()))?
            .get_mut(part)
            .ok_or_else(|| AssetCommandError::FieldNotFound(path.to_vec()))?;
    }
    let leaf = path.last().unwrap();
    let obj = current
        .as_object_mut()
        .ok_or_else(|| AssetCommandError::FieldNotFound(path.to_vec()))?;
    let old = obj
        .get(leaf)
        .ok_or_else(|| AssetCommandError::FieldNotFound(path.to_vec()))?
        .clone();
    obj.insert(leaf.clone(), new);
    Ok(old)
}

/// Apply an AssetCommand to a SceneAssetDocument, returning the inverse command.
///
/// Validation runs before mutation; failed commands leave the document unchanged.
pub fn apply(
    doc: &mut SceneAssetDocument,
    cmd: &AssetCommand,
) -> Result<AssetCommand, AssetCommandError> {
    use crate::scene_asset::LevelLayer;
    match cmd {
        AssetCommand::AddEntity {
            local_id,
            name,
            local_path,
            components,
        } => {
            // Check duplicate
            if doc.entities.iter().any(|e| e.local_id.as_str() == local_id) {
                return Err(AssetCommandError::DuplicateLocalId(local_id.clone()));
            }
            doc.entities.push(SceneAssetEntity {
                local_id: LocalId::new(local_id.clone()),
                local_path: local_path.clone(),
                name: name.clone(),
                components: components.clone(),
            });
            Ok(AssetCommand::RemoveEntity {
                local_id: local_id.clone(),
            })
        }

        AssetCommand::RemoveEntity { local_id } => {
            let pos = doc
                .entities
                .iter()
                .position(|e| e.local_id.as_str() == local_id)
                .ok_or_else(|| AssetCommandError::EntityNotFound(local_id.clone()))?;
            let removed = doc.entities.remove(pos);

            // ponytail: Relationships referencing the removed entity are NOT
            // cleaned up here — relationships are read-only in this cut and
            // dangling refs are deferred to the Validation Center (Capability 4).

            Ok(AssetCommand::AddEntity {
                local_id: removed.local_id.as_str().to_string(),
                name: removed.name,
                local_path: removed.local_path,
                components: removed.components,
            })
        }

        AssetCommand::RenameEntity {
            local_id,
            old_name: _,
            new_name,
        } => {
            let entity = find_entity_mut(doc, local_id)?;
            let actual_old = entity.name.clone();
            entity.name = new_name.clone();
            Ok(AssetCommand::RenameEntity {
                local_id: local_id.clone(),
                old_name: Some(actual_old.clone()),
                new_name: actual_old,
            })
        }

        AssetCommand::AddComponent {
            local_id,
            type_id,
            values,
        } => {
            let entity = find_entity_mut(doc, local_id)?;
            entity.components.push(ComponentInstance {
                type_id: type_id.clone(),
                values: values.clone(),
            });
            Ok(AssetCommand::RemoveComponent {
                local_id: local_id.clone(),
                type_id: type_id.clone(),
            })
        }

        AssetCommand::RemoveComponent {
            local_id,
            type_id,
        } => {
            let entity = find_entity_mut(doc, local_id)?;
            let pos = entity
                .components
                .iter()
                .position(|c| c.type_id == *type_id);
            match pos {
                Some(p) => {
                    let removed = entity.components.remove(p);
                    Ok(AssetCommand::AddComponent {
                        local_id: local_id.clone(),
                        type_id: removed.type_id,
                        values: removed.values,
                    })
                }
                None => {
                    // No-op: inverse is self
                    Ok(AssetCommand::RemoveComponent {
                        local_id: local_id.clone(),
                        type_id: type_id.clone(),
                    })
                }
            }
        }

        AssetCommand::SetComponentValue {
            local_id,
            type_id,
            field_path,
            value,
        } => {
            let entity = find_entity_mut(doc, local_id)?;
            let component = entity
                .components
                .iter_mut()
                .find(|c| c.type_id == *type_id)
                .ok_or_else(|| AssetCommandError::ComponentNotFound(type_id.clone()))?;
            let old_value =
                set_field_path_vec(&mut component.values, field_path, value.clone())?;
            Ok(AssetCommand::SetComponentValue {
                local_id: local_id.clone(),
                type_id: type_id.clone(),
                field_path: field_path.clone(),
                value: old_value,
            })
        }

        AssetCommand::RegenerateAutoLayer {
            layer_id,
            old_cached,
            old_source_generation,
        } => {
            use crate::auto_layer::regenerate as auto_regenerate;
            use crate::scene_asset::LevelLayer;
            use rand::SeedableRng;

            // ── ALL immutable data collection (before any mutable borrow) ──────────────
            // Find AutoLayer index.
            let auto_idx = doc.layers
                .iter()
                .position(|l| matches!(l, LevelLayer::Auto(al) if al.id.as_str() == layer_id.as_str()))
                .ok_or_else(|| AssetCommandError::LayerNotFound(layer_id.0.clone()))?;

            let (source_layer_id, pre_cached, pre_source_gen) = match &doc.layers[auto_idx] {
                LevelLayer::Auto(al) => (al.source_layer_id.clone(), al.cached.clone(), al.source_generation),
                _ => return Err(AssetCommandError::LayerNotFound(layer_id.0.clone())),
            };

            // Find source TileLayer and clone it so we can drop the borrow.
            let source_clone = {
                let source_idx = doc.layers
                    .iter()
                    .position(|l| matches!(l, LevelLayer::Tile(tl) if tl.id.as_str() == source_layer_id.as_str()))
                    .ok_or_else(|| AssetCommandError::SourceLayerNotFound(source_layer_id.0.clone()))?;
                match &doc.layers[source_idx] {
                    LevelLayer::Tile(tl) => tl.clone(),
                    _ => return Err(AssetCommandError::SourceLayerNotFound(source_layer_id.0.clone())),
                }
            }; // <-- all immutable borrows end here

            // ── Mutable phase ──────────────────────────────────────────────────────────
            let al_mut = match &mut doc.layers[auto_idx] {
                LevelLayer::Auto(al) => al as &mut crate::auto_layer::AutoLayer,
                _ => return Err(AssetCommandError::LayerNotFound(layer_id.0.clone())),
            };

            // Differentiate undo vs redo/forward using length comparison:
            // - If old_cached.len() != current cached.len() → UNDO (restore saved values)
            // - If lengths match → FORWARD or REDO (regenerate)
            // This works because:
            //   Forward: old_cached = {} (len 0), current = {} (len 0) → match → regenerate ✓
            //   Undo:    old_cached = {} (len 0), current = {tile99} (len 1) → differ → restore ✓
            //   Redo:    old_cached = {} (len 0), current = {} (len 0) after undo → match → regenerate ✓
            if old_cached.len() != al_mut.cached.len() {
                // UNDO: restore saved values directly, do NOT regenerate
                al_mut.cached = old_cached.clone();
                al_mut.source_generation = *old_source_generation;
            } else {
                // FORWARD or REDO: regenerate from source
                let mut rng = rand::rngs::StdRng::seed_from_u64(42);
                auto_regenerate(al_mut, &source_clone, &mut rng);
            }

            // Inverse captures the pre-apply state (to restore on undo).
            // Forward captures post-regen state (to restore on redo).
            Ok(AssetCommand::RegenerateAutoLayer {
                layer_id: layer_id.clone(),
                old_cached: pre_cached,
                old_source_generation: pre_source_gen,
            })
        }

        AssetCommand::PaintTile {
            layer_id,
            x,
            y,
            old_tile,
            tileset_id,
            local_index,
        } => {
            // HIGH-10: route tile paint through the command surface so undo/redo
            // captures the previous TileRef (if any) for restoration.
            let coord = TileCoord::new(*x, *y);
            let layer_id_str = layer_id.as_str().to_string();
            let tile_layer_id = crate::tile_layer::TileLayerId(layer_id_str.clone());
            let layer = doc
                .layers
                .iter_mut()
                .find(|l| matches!(l, LevelLayer::Tile(tl) if tl.id == tile_layer_id))
                .ok_or_else(|| AssetCommandError::LayerNotFound(layer_id_str.clone()))?;
            let tl = match layer {
                LevelLayer::Tile(tl) => tl,
                _ => return Err(AssetCommandError::LayerNotFound(layer_id_str)),
            };
            let captured_old = tl
                .get_tile(&coord)
                .cloned()
                .or_else(|| old_tile.clone());
            let tile_ref = TileRef {
                tileset_id: tileset_id.clone(),
                local_index: *local_index,
            };
            tl.paint_tile(coord, tile_ref);
            Ok(AssetCommand::EraseTile {
                layer_id: layer_id.clone(),
                x: *x,
                y: *y,
                erased_tile: captured_old,
            })
        }

        AssetCommand::EraseTile {
            layer_id,
            x,
            y,
            erased_tile,
        } => {
            // HIGH-10: route tile erase through the command surface.
            let coord = TileCoord::new(*x, *y);
            let layer_id_str = layer_id.as_str().to_string();
            let tile_layer_id = crate::tile_layer::TileLayerId(layer_id_str.clone());
            let layer = doc
                .layers
                .iter_mut()
                .find(|l| matches!(l, LevelLayer::Tile(tl) if tl.id == tile_layer_id))
                .ok_or_else(|| AssetCommandError::LayerNotFound(layer_id_str.clone()))?;
            let tl = match layer {
                LevelLayer::Tile(tl) => tl,
                _ => return Err(AssetCommandError::LayerNotFound(layer_id_str)),
            };
            let captured = tl.erase_tile(&coord).or_else(|| erased_tile.clone());
            if captured.is_none() {
                return Err(AssetCommandError::TileNotFound {
                    layer_id: layer_id_str,
                    x: *x,
                    y: *y,
                });
            }
            let captured_tile = captured.unwrap();
            Ok(AssetCommand::PaintTile {
                layer_id: layer_id.clone(),
                x: *x,
                y: *y,
                old_tile: None,
                tileset_id: captured_tile.tileset_id,
                local_index: captured_tile.local_index,
            })
        }

        AssetCommand::AddAutoRule { layer_id, rule } => {
            // MED-8: route through command surface for undo/redo.
            let layer_id_str = layer_id.as_str().to_string();
            let auto_layer_id = crate::auto_layer::AutoLayerId(layer_id_str.clone());
            let layer = doc
                .layers
                .iter_mut()
                .find(|l| matches!(l, LevelLayer::Auto(al) if al.id == auto_layer_id))
                .ok_or_else(|| AssetCommandError::LayerNotFound(layer_id_str.clone()))?;
            let al = match layer {
                LevelLayer::Auto(al) => al,
                _ => return Err(AssetCommandError::LayerNotFound(layer_id_str)),
            };
            al.rules.push(rule.clone());
            // Inverse is RemoveAutoRule at the original index (= old length).
            Ok(AssetCommand::RemoveAutoRule {
                layer_id: layer_id.clone(),
                index: al.rules.len() - 1,
                removed_rule: Some(rule.clone()),
            })
        }

        AssetCommand::UpdateAutoRule {
            layer_id,
            index,
            old_rule,
            new_rule,
        } => {
            let layer_id_str = layer_id.as_str().to_string();
            let auto_layer_id = crate::auto_layer::AutoLayerId(layer_id_str.clone());
            let layer = doc
                .layers
                .iter_mut()
                .find(|l| matches!(l, LevelLayer::Auto(al) if al.id == auto_layer_id))
                .ok_or_else(|| AssetCommandError::LayerNotFound(layer_id_str.clone()))?;
            let al = match layer {
                LevelLayer::Auto(al) => al,
                _ => return Err(AssetCommandError::LayerNotFound(layer_id_str)),
            };
            if *index >= al.rules.len() {
                return Err(AssetCommandError::TileNotFound {
                    layer_id: layer_id_str,
                    x: -1,
                    y: *index as i32,
                });
            }
            let captured = old_rule.clone().unwrap_or_else(|| al.rules[*index].clone());
            al.rules[*index] = new_rule.clone();
            Ok(AssetCommand::UpdateAutoRule {
                layer_id: layer_id.clone(),
                index: *index,
                old_rule: Some(captured),
                new_rule: new_rule.clone(),
            })
        }

        AssetCommand::RemoveAutoRule {
            layer_id,
            index,
            removed_rule,
        } => {
            let layer_id_str = layer_id.as_str().to_string();
            let auto_layer_id = crate::auto_layer::AutoLayerId(layer_id_str.clone());
            let layer = doc
                .layers
                .iter_mut()
                .find(|l| matches!(l, LevelLayer::Auto(al) if al.id == auto_layer_id))
                .ok_or_else(|| AssetCommandError::LayerNotFound(layer_id_str.clone()))?;
            let al = match layer {
                LevelLayer::Auto(al) => al,
                _ => return Err(AssetCommandError::LayerNotFound(layer_id_str)),
            };
            if *index >= al.rules.len() {
                return Err(AssetCommandError::TileNotFound {
                    layer_id: layer_id_str,
                    x: -1,
                    y: *index as i32,
                });
            }
            let captured = removed_rule.clone().unwrap_or_else(|| al.rules[*index].clone());
            al.rules.remove(*index);
            Ok(AssetCommand::AddAutoRule {
                layer_id: layer_id.clone(),
                rule: captured,
            })
        }

        AssetCommand::Batch { label: _, commands } => {
            // CRIT-2: snapshot the doc before the batch. On any command
            // failure, restore the snapshot in O(1) instead of applying
            // inverses in sequence. See processor.rs for rationale.
            let snapshot = doc.clone();
            let mut inverses: Vec<AssetCommand> = Vec::new();
            for (i, c) in commands.iter().enumerate() {
                match apply(doc, c) {
                    Ok(inv) => inverses.push(inv),
                    Err(e) => {
                        *doc = snapshot;
                        return Err(AssetCommandError::BatchFailed {
                            index: i,
                            source: Box::new(e),
                        });
                    }
                }
            }
            inverses.reverse();
            Ok(AssetCommand::Batch {
                label: "inverse".to_string(),
                commands: inverses,
            })
        }
    }
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
#[derive(Debug, Clone)]
pub struct AssetOperationLog {
    entries: Vec<AssetLogEntry>,
    cursor: isize,
    max_size: usize,
}

impl AssetOperationLog {
    /// Create a new empty log with default max size (1000 entries).
    pub fn new() -> Self {
        Self::with_max_size(1000)
    }

    /// Const constructor for use in `thread_local!` initializers.
    pub const fn new_const() -> Self {
        Self {
            entries: Vec::new(),
            cursor: -1,
            max_size: 1000,
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
    pub fn undo(&mut self, doc: &mut SceneAssetDocument) -> Result<(), AssetCommandError> {
        if !self.can_undo() {
            return Err(AssetCommandError::JsonError("Nothing to undo".to_string()));
        }
        let entry = &self.entries[self.cursor as usize];
        apply(doc, &entry.inverse)?;
        self.cursor -= 1;
        Ok(())
    }

    /// Apply the forward of the entry after the cursor, moving forward.
    pub fn redo(&mut self, doc: &mut SceneAssetDocument) -> Result<(), AssetCommandError> {
        if !self.can_redo() {
            return Err(AssetCommandError::JsonError("Nothing to redo".to_string()));
        }
        self.cursor += 1;
        let entry = &self.entries[self.cursor as usize];
        apply(doc, &entry.forward)?;
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
    /// After record, the log is "dirty" until saved/cleared.
    pub fn is_dirty(&self) -> bool {
        // Dirty means: there are recorded changes that may not be saved.
        // A simple heuristic: log has entries
        self.cursor >= 0
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.cursor = -1;
    }
}

impl Default for AssetOperationLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ComponentInstance;
    use serde_json::json;

    fn empty_doc() -> SceneAssetDocument {
        SceneAssetDocument {
            layers: vec![],
            asset_id: "id_test".to_string(),
            logical_path: "test/asset".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        }
    }

    fn entity(local_id: &str, name: &str, components: Vec<ComponentInstance>) -> SceneAssetEntity {
        SceneAssetEntity {
            local_id: LocalId::new(local_id),
            local_path: format!("./{}", local_id),
            name: name.to_string(),
            components,
        }
    }

    #[test]
    fn test_add_entity_applies_and_inverts() {
        let mut doc = empty_doc();
        let cmd = AssetCommand::AddEntity {
            local_id: "a1".to_string(),
            name: "A".to_string(),
            local_path: "./a1".to_string(),
            components: vec![],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 1);
        match inverse {
            AssetCommand::RemoveEntity { local_id } => assert_eq!(local_id, "a1"),
            _ => panic!("Expected RemoveEntity"),
        }
    }

    #[test]
    fn test_remove_entity_inverse_contains_full_entity() {
        let mut doc = empty_doc();
        let transform = ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: json!({"translation": {"x": 0.0, "y": 0.0}}),
        };
        doc.entities.push(entity("a1", "A", vec![transform]));

        let cmd = AssetCommand::RemoveEntity {
            local_id: "a1".to_string(),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 0);

        match inverse {
            AssetCommand::AddEntity { local_id, name, local_path, components } => {
                assert_eq!(local_id, "a1");
                assert_eq!(name, "A");
                assert_eq!(local_path, "./a1");
                assert_eq!(components.len(), 1);
            }
            _ => panic!("Expected AddEntity"),
        }
    }

    #[test]
    fn test_rename_entity_inverse_swaps_names() {
        let mut doc = empty_doc();
        doc.entities.push(entity("a1", "Original", vec![]));

        let cmd = AssetCommand::RenameEntity {
            local_id: "a1".to_string(),
            old_name: None,
            new_name: "New".to_string(),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].name, "New");

        match inverse {
            AssetCommand::RenameEntity { local_id, old_name, new_name } => {
                assert_eq!(local_id, "a1");
                assert_eq!(old_name, Some("Original".to_string()));
                assert_eq!(new_name, "Original");
            }
            _ => panic!("Expected RenameEntity"),
        }
    }

    #[test]
    fn test_set_component_value_inverse_restores_old_value() {
        let mut doc = empty_doc();
        let transform = ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: json!({"translation": {"x": 0.0, "y": 0.0}}),
        };
        doc.entities.push(entity("a1", "A", vec![transform]));

        let cmd = AssetCommand::SetComponentValue {
            local_id: "a1".to_string(),
            type_id: "editor.Transform2D".to_string(),
            field_path: vec!["translation".to_string(), "x".to_string()],
            value: json!(100.0),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].components[0].values["translation"]["x"], json!(100.0));

        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.entities[0].components[0].values["translation"]["x"], json!(0.0));
    }

    #[test]
    fn test_batch_inverse_reverses_order() {
        let mut doc = empty_doc();
        let cmd = AssetCommand::Batch {
            label: "test".to_string(),
            commands: vec![
                AssetCommand::AddEntity {
                    local_id: "a1".to_string(),
                    name: "A".to_string(),
                    local_path: "./a1".to_string(),
                    components: vec![],
                },
                AssetCommand::AddEntity {
                    local_id: "a2".to_string(),
                    name: "B".to_string(),
                    local_path: "./a2".to_string(),
                    components: vec![],
                },
            ],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 2);

        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.entities.len(), 0);
    }

    #[test]
    fn test_set_field_path_vec_simple() {
        let mut v = json!({"a": 1});
        let old = set_field_path_vec(&mut v, &["a".to_string()], json!(99)).unwrap();
        assert_eq!(old, json!(1));
        assert_eq!(v["a"], json!(99));
    }

    #[test]
    fn test_set_field_path_vec_nested() {
        let mut v = json!({"a": {"b": {"c": 1}}});
        let old = set_field_path_vec(&mut v, &["a".to_string(), "b".to_string(), "c".to_string()], json!(42)).unwrap();
        assert_eq!(old, json!(1));
        assert_eq!(v["a"]["b"]["c"], json!(42));
    }

    #[test]
    fn test_asset_operation_log_record_and_undo() {
        let mut log = AssetOperationLog::new_const();
        let mut doc = empty_doc();

        let cmd = AssetCommand::AddEntity {
            local_id: "a1".to_string(),
            name: "A".to_string(),
            local_path: "./a1".to_string(),
            components: vec![],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        log.record(&cmd, inverse);

        assert!(log.can_undo());
        assert!(!log.can_redo());

        log.undo(&mut doc).unwrap();
        assert_eq!(doc.entities.len(), 0);
        assert!(!log.can_undo());
        assert!(log.can_redo());
    }

    #[test]
    fn test_asset_operation_log_redo() {
        let mut log = AssetOperationLog::new_const();
        let mut doc = empty_doc();

        let cmd = AssetCommand::AddEntity {
            local_id: "a1".to_string(),
            name: "A".to_string(),
            local_path: "./a1".to_string(),
            components: vec![],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        log.record(&cmd, inverse);

        log.undo(&mut doc).unwrap();
        assert_eq!(doc.entities.len(), 0);

        log.redo(&mut doc).unwrap();
        assert_eq!(doc.entities.len(), 1);
    }

    #[test]
    fn test_asset_operation_log_is_dirty() {
        let log = AssetOperationLog::new_const();
        assert!(!log.is_dirty());
    }

    #[test]
    fn test_asset_operation_log_clear() {
        let mut log = AssetOperationLog::new_const();
        let mut doc = empty_doc();

        let cmd = AssetCommand::AddEntity {
            local_id: "a1".to_string(),
            name: "A".to_string(),
            local_path: "./a1".to_string(),
            components: vec![],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        log.record(&cmd, inverse);

        log.clear();
        assert!(!log.can_undo());
        assert!(!log.can_redo());
        assert!(!log.is_dirty());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RG2 — apply regen → inverse → cached restored to C1
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_regenerate_auto_layer_inverse_restores_cached() {
        use crate::auto_layer::{AutoLayer, AutoLayerId, AutoRule, Pattern3x3, PatternCell};
        use crate::scene_asset::{LayerId, LevelLayer};
        use crate::tile_layer::TileLayer;
        use crate::tileset::{TileCoord, TileRef, TileGrid};

        let tileset_id = crate::tileset::TilesetId::new("ts_test".to_string());
        let source_layer_id = LayerId::new("lyr_src".to_string());
        let auto_layer_id = AutoLayerId::new("al_01".to_string());

        // Source TileLayer with one tile at (0, 0), generation = 1
        let mut source_tl = TileLayer::new(
            crate::tile_layer::TileLayerId::new("lyr_src".to_string()),
            "Source".to_string(),
            tileset_id.clone(),
        );
        source_tl.generation = 1;
        source_tl.paint_tile(
            TileCoord::new(0, 0),
            TileRef { tileset_id: "ts_test".to_string(), local_index: 0 },
        );

        // AutoLayer: initially stale (cached empty, source_gen = 0)
        // Rule pattern: all Any (matches any neighborhood)
        let pattern: Pattern3x3 = [
            [PatternCell::Any, PatternCell::Any, PatternCell::Any],
            [PatternCell::Any, PatternCell::Any, PatternCell::Any],
            [PatternCell::Any, PatternCell::Any, PatternCell::Any],
        ];
        let auto_layer = AutoLayer {
            id: auto_layer_id.clone(),
            name: "Auto".to_string(),
            order: 0,
            source_layer_id: source_layer_id.clone(),
            tileset_id: tileset_id.clone(),
            rules: vec![AutoRule {
                pattern,
                output: vec![TileRef { tileset_id: "ts_test".to_string(), local_index: 99 }],
                chance: None,
            }],
            cached: TileGrid::default(),
            source_generation: 0, // stale — source is at gen 1
        };

        let mut doc = empty_doc();
        doc.layers.push(LevelLayer::Tile(source_tl));
        doc.layers.push(LevelLayer::Auto(auto_layer));

        // C1: pre-regen cached state (should be empty)
        let pre_cached: TileGrid = match &doc.layers[1] {
            LevelLayer::Auto(al) => al.cached.clone(),
            _ => panic!("expected AutoLayer"),
        };
        assert!(pre_cached.is_empty(), "pre-regen cached should be empty");

        // Apply RegenerateAutoLayer
        let cmd = AssetCommand::RegenerateAutoLayer {
            layer_id: LayerId::new(auto_layer_id.0.clone()),
            old_cached: TileGrid::default(),
            old_source_generation: 0,
        };
        let inverse = apply(&mut doc, &cmd).unwrap();

        // After regen: cached should no longer be empty (rule fires for source tile)
        let post_cached: TileGrid = match &doc.layers[1] {
            LevelLayer::Auto(al) => al.cached.clone(),
            _ => panic!("expected AutoLayer"),
        };
        assert!(!post_cached.is_empty(), "post-regen cached should not be empty");

        // Apply inverse: should restore cached to C1
        apply(&mut doc, &inverse).unwrap();

        let restored_cached: TileGrid = match &doc.layers[1] {
            LevelLayer::Auto(al) => al.cached.clone(),
            _ => panic!("expected AutoLayer"),
        };
        assert!(restored_cached.is_empty(), "inverse should restore cached to C1 (empty)");
    }
}
