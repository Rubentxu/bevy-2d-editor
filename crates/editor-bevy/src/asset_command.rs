//! Asset Command processor for Scene Asset Authoring mode.
//!
//! H2.4 (Asset Family collapse): the command enum, error type, log entry, and
//! operation log live canonically in `editor_model::asset_operation_log`. This
//! module re-exports them and keeps the Bevy-side **application logic** that
//! actually mutates a `SceneAssetDocument`.
//!
//! The dependency inversion introduced by H2.4 mirrors the one in `operation_log`:
//!
//! - `AssetOperationLog::undo` / `redo` take a `&dyn AssetApplyCommandFn`
//!   callback so the log type can stay pure.
//! - `AssetProcessorApply` is the production callback. It delegates to the
//!   Bevy-side `apply` function below.
//!
//! The kernel (`transaction_bridge::AssetCommandApplier`) also calls into
//! `apply` directly via the `apply as asset_apply` import in
//! `transaction_bridge.rs` — both paths share the same single source of
//! application logic, so the inverse generation cannot drift between the
//! `dispatch_asset_command` path and the `undo_asset` / `redo_asset` path.

pub use editor_model::asset_operation_log::{
    AssetApplyCommandFn, AssetCommand, AssetCommandError, AssetLogEntry, AssetOperationLog,
};

use crate::scene_asset::{LevelLayer, SceneAssetEntity};
use editor_model::auto_layer::AutoLayer;
use editor_model::ids::{LayerId, SceneAssetLocalId};
use editor_model::scene_asset::SceneAssetDocument;
use editor_model::tileset::{TileCoord, TileRef};
use std::collections::BTreeMap;

// ─────────────────────────────────────────────────────────────────────────
// AssetProcessor
// ─────────────────────────────────────────────────────────────────────────

/// Find a mutable entity by `local_id` (string key).
fn find_entity_mut<'a>(
    doc: &'a mut SceneAssetDocument,
    local_id: &str,
) -> Result<&'a mut SceneAssetEntity, AssetCommandError> {
    let key = SceneAssetLocalId(local_id.to_string());
    doc.entities
        .iter_mut()
        .find(|e| e.local_id == key)
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
                local_id: SceneAssetLocalId(local_id.clone()),
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
            let pos = doc
                .entities
                .iter()
                .position(|e| e.local_id.as_str() == local_id)
                .ok_or_else(|| AssetCommandError::EntityNotFound(local_id.clone()))?;
            let removed = doc.entities.remove(pos);

            // v0.90 PR6: rewritten the v0.86-era ponytail marker. The original
            // marker pointed to a non-scheduled "Validation Center Capability 4"
            // with no concrete trigger. Validation Center is still not in any
            // current roadmap cycle, so the relationship-cleanup work remains
            // deferred. The dangling-relationship lint that the original
            // marker promised to add in that cycle is not yet implemented;
            // consumers should treat the warning as a known limitation.

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
            entity.components.push(editor_model::ComponentInstance {
                type_id: type_id.clone(),
                values: values.clone(),
            });
            Ok(AssetCommand::RemoveComponent {
                local_id: local_id.clone(),
                type_id: type_id.clone(),
            })
        }

        AssetCommand::RemoveComponent { local_id, type_id } => {
            let entity = find_entity_mut(doc, local_id)?;
            let pos = entity.components.iter().position(|c| c.type_id == *type_id);
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
            let old_value = set_field_path_vec(&mut component.values, field_path, value.clone())?;
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
            use rand::SeedableRng;

            // ── ALL immutable data collection (before any mutable borrow) ──────────────
            // Find AutoLayer index.
            let auto_idx = doc
                .layers
                .iter()
                .position(
                    |l| matches!(l, LevelLayer::Auto(al) if al.id.as_str() == layer_id.as_str()),
                )
                .ok_or_else(|| AssetCommandError::LayerNotFound(layer_id.0.clone()))?;

            let (source_layer_id, pre_cached, pre_source_gen) = match &doc.layers[auto_idx] {
                LevelLayer::Auto(al) => (
                    al.source_layer_id.clone(),
                    al.cached.clone(),
                    al.source_generation,
                ),
                _ => return Err(AssetCommandError::LayerNotFound(layer_id.0.clone())),
            };

            // Find source TileLayer and clone it so we can drop the borrow.
            let source_clone = {
                let source_idx = doc
                    .layers
                    .iter()
                    .position(|l| matches!(l, LevelLayer::Tile(tl) if tl.id.as_str() == source_layer_id.as_str()))
                    .ok_or_else(|| AssetCommandError::SourceLayerNotFound(source_layer_id.0.clone()))?;
                match &doc.layers[source_idx] {
                    LevelLayer::Tile(tl) => tl.clone(),
                    _ => {
                        return Err(AssetCommandError::SourceLayerNotFound(
                            source_layer_id.0.clone(),
                        ));
                    }
                }
            }; // <-- all immutable borrows end here

            // ── Mutable phase ──────────────────────────────────────────────────────────
            let al_mut = match &mut doc.layers[auto_idx] {
                LevelLayer::Auto(al) => al as &mut AutoLayer,
                _ => return Err(AssetCommandError::LayerNotFound(layer_id.0.clone())),
            };

            // Differentiate undo vs redo/forward using length comparison:
            // - If old_cached.len() != current cached.len() → UNDO (restore saved values)
            // - If lengths match → FORWARD or REDO (regenerate)
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
            let tile_layer_id = LayerId::new(layer_id_str.clone());
            let layer = doc
                .layers
                .iter_mut()
                .find(|l| matches!(l, LevelLayer::Tile(tl) if tl.id == tile_layer_id))
                .ok_or_else(|| AssetCommandError::LayerNotFound(layer_id_str.clone()))?;
            let tl = match layer {
                LevelLayer::Tile(tl) => tl,
                _ => return Err(AssetCommandError::LayerNotFound(layer_id_str)),
            };
            let captured_old = tl.get_tile(&coord).cloned().or_else(|| old_tile.clone());
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
            let tile_layer_id = LayerId::new(layer_id_str.clone());
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
            let auto_layer_id = LayerId::new(layer_id_str.clone());
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
            let auto_layer_id = LayerId::new(layer_id_str.clone());
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
            let auto_layer_id = LayerId::new(layer_id_str.clone());
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
            let captured = removed_rule
                .clone()
                .unwrap_or_else(|| al.rules[*index].clone());
            al.rules.remove(*index);
            Ok(AssetCommand::AddAutoRule {
                layer_id: layer_id.clone(),
                rule: captured,
            })
        }

        AssetCommand::SetIntGridCell {
            layer_id,
            coord,
            old_value,
            value,
        } => {
            // IntGrid cells are stored on IntGridLayer directly; the apply
            // path delegates to `paint_cell` for the typed lookup.
            use editor_model::int_grid::{IntGridLayer, IntGridLayerId};
            let layer_id_str = layer_id.as_str().to_string();
            let target = IntGridLayerId::new(layer_id_str.clone());
            let layer = doc
                .layers
                .iter_mut()
                .find(|l| matches!(l, LevelLayer::IntGrid(il) if il.id == target))
                .ok_or_else(|| AssetCommandError::LayerNotFound(layer_id_str.clone()))?;
            let il = match layer {
                LevelLayer::IntGrid(il) => il as &mut IntGridLayer,
                _ => return Err(AssetCommandError::LayerNotFound(layer_id_str)),
            };
            let prior = old_value
                .clone()
                .or_else(|| il.get_cell(coord.x, coord.y).map(|c| c.value));
            il.paint_cell(coord.x, coord.y, *value, None);
            Ok(AssetCommand::SetIntGridCell {
                layer_id: layer_id.clone(),
                coord: coord.clone(),
                old_value: prior,
                value: *value,
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
// AssetProcessorApply — AssetApplyCommandFn impl
// ─────────────────────────────────────────────────────────────────────────

/// Production `AssetApplyCommandFn` callback for `AssetOperationLog::undo` / `redo`.
///
/// H2.4: lives in `editor-bevy` (the Bevy side) and delegates to the local
/// `apply` function so the log type in `editor_model` stays pure.
///
/// `transaction_bridge::AssetCommandApplier` calls `apply` directly for the
/// kernel path — both paths share the same single source of application
/// logic.
pub struct AssetProcessorApply;

impl AssetApplyCommandFn for AssetProcessorApply {
    fn apply(
        &self,
        doc: &mut SceneAssetDocument,
        cmd: &AssetCommand,
    ) -> Result<AssetCommand, AssetCommandError> {
        apply(doc, cmd)
    }
}
