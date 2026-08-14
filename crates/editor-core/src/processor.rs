//! Command processor for the Bevy 2D Editor.
//!
//! Provides `validate(doc, cmd)` and `apply(doc, cmd) -> Result<Command, CommandError>`.
//! Each command captures pre-state explicitly so inverse generation is mechanical.
//! Validation runs before mutation; failed commands leave the document unchanged.
//!
//! Batches apply atomically: on any failure inside a batch, the document is
//! restored from a pre-batch snapshot (CRIT-2 fix).
//!
//! HD-1 context: Some commands need access to external state (asset catalog,
//! asset body cache) for resync. This is provided via `ProcessorContext`,
//! which callers can construct from the live globals via
//! `ProcessorContext::from_globals()` or build explicitly for tests.

use crate::command::{Command, CommandError};
use crate::document::{ComponentInstance, Entity, LocalId, SceneDocument, StableId};
use crate::scene_asset::SceneAssetDocument;
use crate::scene_instance::SceneInstance;
use crate::scene_instance_overrides::{remove_override, resync, upsert_override};

/// Context passed to commands that need to resolve external resources
/// (HD-1 cleanup). Construct via `from_globals()` for production code,
/// or build directly in tests to avoid touching the global thread-locals.
///
/// All fields are optional: when `None`, commands that need the resource
/// silently no-op the resync step (preserving legacy behavior). Future
/// versions will require Some(...) for the affected commands.
#[derive(Debug, Default, Clone)]
pub struct ProcessorContext {
    /// Resolved asset body for the current ReplaceInstanceAsset target,
    /// or None if not applicable / not pre-fetched.
    pub asset_body: Option<SceneAssetDocument>,
}

impl ProcessorContext {
    /// Build a context from the live global thread-locals. Used by
    /// production callers (dispatch_command, tests that go through
    /// the full surface).
    pub fn from_globals(asset_ref: &str) -> Self {
        // Resolve path → asset_id (clone to detach lifetime).
        let asset_id =
            crate::with_asset_catalog(|cat| cat.resolve_path(asset_ref).map(|s| s.to_string()));
        // Resolve asset_id → catalog entry.
        let entry = asset_id.and_then(|id| crate::with_asset_catalog(|cat| cat.get(&id).cloned()));
        // Resolve entry.logical_path → body cache entry.
        let asset_body = entry.and_then(|e| {
            crate::with_asset_body_cache(|cache| cache.get(&e.logical_path).cloned())
        });
        ProcessorContext { asset_body }
    }

    /// Empty context for tests / callers that don't need external resources.
    pub fn empty() -> Self {
        ProcessorContext::default()
    }
}

/// Extract the asset_ref (if any) from a command. Used by `apply` to
/// pre-resolve the ProcessorContext for commands that need asset body
/// access (currently only ReplaceInstanceAsset). Returns an empty string
/// for commands that don't need resolution.
fn asset_ref_of<'a>(cmd: &'a Command) -> &'a str {
    match cmd {
        Command::Noop {} => "",
        Command::ReplaceInstanceAsset { new_asset_ref, .. } => new_asset_ref.as_str(),
        Command::PlaceInstance { asset_ref, .. } => asset_ref.as_str(),
        _ => "",
    }
}

/// Find an entity by id and return a mutable reference.
fn find_entity_mut<'a>(
    doc: &'a mut SceneDocument,
    id: &StableId,
) -> Result<&'a mut Entity, CommandError> {
    doc.entities
        .iter_mut()
        .find(|e| &e.id == id)
        .ok_or_else(|| CommandError::EntityNotFound(id.clone()))
}

/// Find an entity by id and return an immutable reference.
fn find_entity<'a>(doc: &'a SceneDocument, id: &StableId) -> Result<&'a Entity, CommandError> {
    doc.entities
        .iter()
        .find(|e| &e.id == id)
        .ok_or_else(|| CommandError::EntityNotFound(id.clone()))
}

/// Set a field at a dotted path within a JSON object. Returns the old value.
///
/// Path navigation: split on '.', navigate to parent, set leaf.
/// For `"translation.x"` on `{"translation": {"x": 0, "y": 0}}`,
/// the result is `{"translation": {"x": <new>, "y": 0}}`.
///
/// NOTE: This is the same pattern as `asset_command::set_field_path_vec`
/// but accepts a dotted `&str` instead of `Vec<String>`. Per ADR-0007, the
/// two command surfaces stay independent — no logic unification.
fn set_field_path(
    value: &mut serde_json::Value,
    path: &str,
    new: serde_json::Value,
) -> Result<serde_json::Value, CommandError> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() || (parts.len() == 1 && parts[0].is_empty()) {
        return Err(CommandError::FieldNotFound(path.to_string()));
    }
    let mut current = value;
    for part in &parts[..parts.len() - 1] {
        current = current
            .as_object_mut()
            .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?
            .get_mut(*part)
            .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?;
    }
    let leaf = parts.last().unwrap();
    let obj = current
        .as_object_mut()
        .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?;
    let old = obj
        .get(*leaf)
        .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?
        .clone();
    obj.insert(leaf.to_string(), new);
    Ok(old)
}

/// Check if reparenting `entity_id` under `proposed_parent` would create a cycle.
fn would_create_cycle(
    doc: &SceneDocument,
    entity_id: &StableId,
    proposed_parent: &StableId,
) -> bool {
    if entity_id == proposed_parent {
        return true;
    }
    let mut current = Some(proposed_parent.clone());
    while let Some(id) = current {
        if &id == entity_id {
            return true;
        }
        match find_entity(doc, &id) {
            Ok(e) => current = e.parent.clone(),
            Err(_) => return false,
        }
    }
    false
}

/// Validate a command against the document and schema registry, without mutating.
/// Returns `Ok(())` if the command can be applied, or `Err(CommandError)` otherwise.
pub fn validate(doc: &SceneDocument, cmd: &Command) -> Result<(), CommandError> {
    match cmd {
        Command::Noop {} => {}
        Command::CreateEntity { id, .. } => {
            if doc.entities.iter().any(|e| &e.id == id) {
                return Err(CommandError::DuplicateId(id.clone()));
            }
        }
        Command::DeleteEntity { id } => {
            find_entity(doc, id)?;
        }
        Command::AddComponent {
            entity_id, type_id, ..
        } => {
            find_entity(doc, entity_id)?;
            if crate::schema::combined_registry().get(type_id).is_none() {
                return Err(CommandError::UnknownSchema(type_id.clone()));
            }
        }
        Command::RemoveComponent { entity_id, type_id } => {
            let entity = find_entity(doc, entity_id)?;
            if !entity.components.iter().any(|c| &c.type_id == type_id) {
                // No-op: valid but produces self-inverse
            }
        }
        Command::SetComponentField {
            entity_id,
            type_id,
            field_path,
            ..
        } => {
            let entity = find_entity(doc, entity_id)?;
            let component = entity
                .components
                .iter()
                .find(|c| &c.type_id == type_id)
                .ok_or_else(|| CommandError::UnknownSchema(type_id.clone()))?;
            // Verify field path exists
            let parts: Vec<&str> = field_path.split('.').collect();
            let mut current = &component.values;
            for part in &parts {
                current = current
                    .as_object()
                    .and_then(|o| o.get(*part))
                    .ok_or_else(|| CommandError::FieldNotFound(field_path.clone()))?;
            }
        }
        // v0.82 P2 (ADR-0025): validate every targeted entity exists, owns
        // the component, and the field_path exists on at least one of them
        // (the apply path will skip entities missing either the component
        // or the field — see ADR-0025 §D5).
        Command::SetComponentFieldOnMultiple {
            entity_ids,
            type_id,
            field_path,
            ..
        } => {
            if entity_ids.is_empty() {
                return Err(CommandError::InvalidArgument(
                    "SetComponentFieldOnMultiple: empty entity_ids".to_string(),
                ));
            }
            // De-dup check (informational; apply is defensive).
            let mut seen: std::collections::HashSet<&StableId> =
                std::collections::HashSet::with_capacity(entity_ids.len());
            for id in entity_ids {
                if !seen.insert(id) {
                    return Err(CommandError::InvalidArgument(format!(
                        "SetComponentFieldOnMultiple: duplicate entity_id {}",
                        id
                    )));
                }
            }
            // Every entity must exist; we don't require all of them to
            // own the component (apply skips non-owners), but every
            // owner must have the field_path — checked inline below.
            for id in entity_ids {
                let entity = find_entity(doc, id)?;
                if let Some(component) = entity.components.iter().find(|c| &c.type_id == type_id) {
                    let parts: Vec<&str> = field_path.split('.').collect();
                    let mut current = &component.values;
                    for part in &parts {
                        current =
                            current
                                .as_object()
                                .and_then(|o| o.get(*part))
                                .ok_or_else(|| {
                                    CommandError::FieldNotFound(format!(
                                        "{} (entity {})",
                                        field_path, id
                                    ))
                                })?;
                    }
                }
            }
        }
        Command::ReparentEntity {
            entity_id,
            new_parent,
            ..
        } => {
            find_entity(doc, entity_id)?;
            if let Some(new_p) = new_parent {
                find_entity(doc, new_p)?;
                if would_create_cycle(doc, entity_id, new_p) {
                    return Err(CommandError::WouldCreateCycle(entity_id.clone()));
                }
            }
        }
        Command::RenameEntity { entity_id, .. } => {
            find_entity(doc, entity_id)?;
        }
        Command::CreateSceneComponent { .. } => {
            return Err(crate::command::CommandError::Unsupported(
                "CreateSceneComponent must be applied via command_scene_component::apply_create"
                    .to_string(),
            ));
        }
        Command::UpdateSceneComponentFields { .. } => {
            return Err(crate::command::CommandError::Unsupported(
                "UpdateSceneComponentFields must be applied via command_scene_component::apply_update".to_string()
            ));
        }
        Command::BindSceneToSchema { .. } => {
            return Err(crate::command::CommandError::Unsupported(
                "BindSceneToSchema must be applied via command_scene_component::apply_bind"
                    .to_string(),
            ));
        }
        Command::Batch { commands, .. } => {
            // Validate each command in order. Snapshot doc state in memory for
            // accurate validation (later commands see earlier ones).
            let mut temp_doc = doc.clone();
            for (i, c) in commands.iter().enumerate() {
                validate(&temp_doc, c).map_err(|e| CommandError::BatchFailed {
                    index: i,
                    source: Box::new(e),
                })?;
                // Apply to temp doc so subsequent validates see the effect
                apply(&mut temp_doc, c).map_err(|e| CommandError::BatchFailed {
                    index: i,
                    source: Box::new(e),
                })?;
            }
        }
        Command::PlaceInstance { instance_id, .. } => {
            // Instance ID must not already exist
            if doc.instances.contains_key(instance_id) {
                return Err(CommandError::DuplicateId(instance_id.clone()));
            }
        }
        Command::RemoveInstance { instance_id } => {
            // Instance must exist
            if !doc.instances.contains_key(instance_id) {
                return Err(CommandError::InstanceNotFound(instance_id.clone()));
            }
        }
        Command::ReplaceInstanceAsset { instance_id, .. } => {
            // Instance must exist
            if !doc.instances.contains_key(instance_id) {
                return Err(CommandError::InstanceNotFound(instance_id.clone()));
            }
        }
        Command::UpsertOverride { instance_id, .. } => {
            // Instance must exist
            if !doc.instances.contains_key(instance_id) {
                return Err(CommandError::InstanceNotFound(instance_id.clone()));
            }
        }
        Command::RevertOverride { instance_id, .. } => {
            // Instance must exist
            if !doc.instances.contains_key(instance_id) {
                return Err(CommandError::InstanceNotFound(instance_id.clone()));
            }
        }
    }
    Ok(())
}

/// Apply a command to the document, returning the inverse command.
///
/// Validation runs first; if it fails, the document is unchanged.
///
/// Uses `ProcessorContext::from_globals(&cmd.asset_ref_if_any())` to resolve
/// external state. If a custom context is needed (e.g., for tests), use
/// `apply_with_context`.
pub fn apply(doc: &mut SceneDocument, cmd: &Command) -> Result<Command, CommandError> {
    let ctx = ProcessorContext::from_globals(asset_ref_of(cmd));
    apply_with_context(doc, cmd, &ctx)
}

/// Like `apply` but uses an explicit `ProcessorContext`. Tests use this to
/// inject mock asset bodies without touching the global thread-locals.
pub fn apply_with_context(
    doc: &mut SceneDocument,
    cmd: &Command,
    ctx: &ProcessorContext,
) -> Result<Command, CommandError> {
    // Validate before mutating
    validate(doc, cmd)?;

    match cmd {
        Command::Noop {} => Ok(Command::Noop {}),
        Command::CreateEntity {
            id,
            name,
            components,
        } => {
            doc.entities.push(Entity {
                id: id.clone(),
                local_id: LocalId::new(id.as_str()),
                name: name.clone(),
                parent: None,
                components: components.clone(),
            });
            Ok(Command::DeleteEntity { id: id.clone() })
        }
        Command::DeleteEntity { id } => {
            let pos = doc
                .entities
                .iter()
                .position(|e| &e.id == id)
                .ok_or_else(|| CommandError::EntityNotFound(id.clone()))?;
            let removed = doc.entities.remove(pos);
            // Reparent children to root
            for entity in doc.entities.iter_mut() {
                if entity.parent.as_ref() == Some(id) {
                    entity.parent = None;
                }
            }
            Ok(Command::CreateEntity {
                id: removed.id,
                name: removed.name,
                components: removed.components,
            })
        }
        Command::AddComponent {
            entity_id,
            type_id,
            values,
        } => {
            let entity = find_entity_mut(doc, entity_id)?;
            entity.components.push(ComponentInstance {
                type_id: type_id.clone(),
                values: values.clone(),
            });
            Ok(Command::RemoveComponent {
                entity_id: entity_id.clone(),
                type_id: type_id.clone(),
            })
        }
        Command::RemoveComponent {
            entity_id,
            type_id,
        } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let pos = entity.components.iter().position(|c| &c.type_id == type_id);
            match pos {
                Some(p) => {
                    let removed = entity.components.remove(p);
                    Ok(Command::AddComponent {
                        entity_id: entity_id.clone(),
                        type_id: removed.type_id,
                        values: removed.values,
                    })
                }
                None => {
                    // No-op: inverse is self
                    Ok(Command::RemoveComponent {
                        entity_id: entity_id.clone(),
                        type_id: type_id.clone(),
                    })
                }
            }
        }
        Command::SetComponentField {
            entity_id,
            type_id,
            field_path,
            value,
        } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let component = entity
                .components
                .iter_mut()
                .find(|c| &c.type_id == type_id)
                .ok_or_else(|| CommandError::UnknownSchema(type_id.clone()))?;
            let old_value = set_field_path(&mut component.values, field_path, value.clone())?;
            Ok(Command::SetComponentField {
                entity_id: entity_id.clone(),
                type_id: type_id.clone(),
                field_path: field_path.clone(),
                value: old_value,
            })
        }
        // v0.82 P2 (ADR-0025): multi-entity field write. Delegates the
        // apply + inverse collection to the existing Batch machinery by
        // wrapping the fan-out as a single `Command::Batch` of per-entity
        // `SetComponentField`s and recursing. Each inner apply captures
        // its own pre-state so partial-failure rollback and per-entity
        // undo both work for free. Validation (empty entity_ids,
        // de-duplication) is handled in `validate` above; here we only
        // need to construct the inner batch and recurse.
        Command::SetComponentFieldOnMultiple {
            entity_ids,
            type_id,
            field_path,
            value,
        } => {
            // De-dup while preserving order (validate should have done
            // this already, but apply must remain defensive — the batch
            // would otherwise apply the same inner command twice).
            let mut seen: Vec<StableId> = Vec::with_capacity(entity_ids.len());
            for id in entity_ids {
                if !seen.iter().any(|s| s == id) {
                    seen.push(id.clone());
                }
            }
            let inner: Vec<Command> = seen
                .into_iter()
                .map(|id| Command::SetComponentField {
                    entity_id: id,
                    type_id: type_id.clone(),
                    field_path: field_path.clone(),
                    value: value.clone(),
                })
                .collect();
            // Inverse must restore each entity independently. Reuse the
            // same Batch shape so the OperationLog / UI see a single
            // multi-edit entry with one label.
            let batch = Command::Batch {
                label: format!("Multi-set field {}.{}", type_id, field_path),
                commands: inner,
            };
            // Recursive call: `apply` clones the doc for rollback via
            // the existing Batch path. If any inner command fails,
            // `apply` returns the partial inverse collected up to the
            // failure point; caller treats the whole apply as failed.
            let _inverse_batch = apply(doc, &batch)?;
            // Outer inverse mirrors the input shape exactly so a
            // re-dispatch of the original (e.g. on redo) reproduces the
            // same fan-out.
            Ok(Command::SetComponentFieldOnMultiple {
                entity_ids: entity_ids.clone(),
                type_id: type_id.clone(),
                field_path: field_path.clone(),
                value: value.clone(),
            })
        }
        Command::ReparentEntity {
            entity_id,
            old_parent: _,
            new_parent,
        } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let actual_old = entity.parent.clone();
            entity.parent = new_parent.clone();
            // Inverse: new_parent should point back to actual_old
            Ok(Command::ReparentEntity {
                entity_id: entity_id.clone(),
                old_parent: actual_old.clone(),
                new_parent: actual_old,
            })
        }
        Command::RenameEntity {
            entity_id,
            old_name: _,
            new_name,
        } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let actual_old = entity.name.clone();
            entity.name = new_name.clone();
            Ok(Command::RenameEntity {
                entity_id: entity_id.clone(),
                old_name: Some(actual_old.clone()),
                new_name: actual_old,
            })
        }
        Command::CreateSceneComponent { .. } => {
            Err(crate::command::CommandError::Unsupported(
                "CreateSceneComponent must be applied via command_scene_component::apply_create".to_string()
            ))
        }
        Command::UpdateSceneComponentFields { .. } => {
            Err(crate::command::CommandError::Unsupported(
                "UpdateSceneComponentFields must be applied via command_scene_component::apply_update".to_string()
            ))
        }
        Command::BindSceneToSchema { .. } => {
            Err(crate::command::CommandError::Unsupported(
                "BindSceneToSchema must be applied via command_scene_component::apply_bind".to_string()
            ))
        }
        Command::Batch { commands, .. } => {
            // CRIT-2: snapshot the doc before the batch. On any command failure,
            // restore the snapshot in O(1) instead of applying inverses in
            // sequence. Inverse-by-inverse rollback silently discards errors
            // via `let _ = apply(...)`, so a failing inverse would leave the
            // doc partially mutated. Snapshot/restore guarantees the atomic
            // contract regardless of inverse success.
            let snapshot = doc.clone();
            let mut inverses: Vec<Command> = Vec::new();
            for (i, c) in commands.iter().enumerate() {
                match apply(doc, c) {
                    Ok(inv) => inverses.push(inv),
                    Err(e) => {
                        *doc = snapshot;
                        return Err(CommandError::BatchFailed {
                            index: i,
                            source: Box::new(e),
                        });
                    }
                }
            }
            // Inverse batch: reverse the order of inverses
            inverses.reverse();
            Ok(Command::Batch {
                label: "inverse".to_string(),
                commands: inverses,
            })
        }
        Command::PlaceInstance {
            instance_id,
            asset_ref,
            asset_version,
            id_map,
            instance_components,
            component_overrides,
            orphaned_component_overrides,
        } => {
            // S15: PlaceInstance applies and inverts
            let instance = SceneInstance {
                instance_id: instance_id.clone(),
                asset_ref: asset_ref.clone(),
                asset_version_seen: *asset_version,
                id_map: id_map.clone(),
                instance_components: instance_components.clone(),
                component_overrides: component_overrides.clone(),
                orphaned_component_overrides: orphaned_component_overrides.clone(),
            };
            doc.instances.insert(instance_id.clone(), instance);
            // Inverse is RemoveInstance
            Ok(Command::RemoveInstance {
                instance_id: instance_id.clone(),
            })
        }
        Command::RemoveInstance { instance_id } => {
            // S16: RemoveInstance applies and inverts
            // Capture pre-state for inverse
            let removed = doc
                .instances
                .remove(instance_id)
                .ok_or_else(|| CommandError::InstanceNotFound(instance_id.clone()))?;
            // Inverse is PlaceInstance restoring the full captured state
            Ok(Command::PlaceInstance {
                instance_id: removed.instance_id.clone(),
                asset_ref: removed.asset_ref.clone(),
                asset_version: removed.asset_version_seen,
                id_map: removed.id_map.clone(),
                instance_components: removed.instance_components.clone(),
                component_overrides: removed.component_overrides.clone(),
                orphaned_component_overrides: removed.orphaned_component_overrides.clone(),
            })
        }
        Command::ReplaceInstanceAsset {
            instance_id,
            new_asset_ref,
            new_asset_version,
            captured_old: _,
        } => {
            // S17: ReplaceInstanceAsset applies and inverts
            // Capture current state for inverse BEFORE mutating
            let old_instance = doc
                .instances
                .get(instance_id)
                .cloned()
                .ok_or_else(|| CommandError::InstanceNotFound(instance_id.clone()))?;
            // Update the instance
            let instance = doc
                .instances
                .get_mut(instance_id)
                .ok_or_else(|| CommandError::InstanceNotFound(instance_id.clone()))?;
            instance.asset_ref = new_asset_ref.clone();
            instance.asset_version_seen = *new_asset_version;

            // S17: Run resync to reclassify overrides according to new asset schema
            // HD-1: read asset body from ProcessorContext instead of the global
            // catalog/cache. The context was pre-resolved by `apply` via
            // `ProcessorContext::from_globals(new_asset_ref)`.
            if let Some(asset) = ctx.asset_body.as_ref() {
                let _report = resync(asset, instance, *new_asset_version);
            }

            // Inverse is ReplaceInstanceAsset restoring captured old state
            // Clone individually to avoid partial move
            Ok(Command::ReplaceInstanceAsset {
                instance_id: instance_id.clone(),
                new_asset_ref: old_instance.asset_ref.clone(),
                new_asset_version: old_instance.asset_version_seen,
                captured_old: Some(old_instance),
            })
        }
        Command::UpsertOverride {
            instance_id,
            target_local_id,
            component_type_id,
            field_path,
            value,
        } => {
            let instance = doc
                .instances
                .get_mut(instance_id)
                .ok_or_else(|| CommandError::InstanceNotFound(instance_id.clone()))?;

            // Build the incoming patch
            let patch = crate::scene_instance::ComponentOverride {
                target_local_id: target_local_id.clone(),
                component_type_id: component_type_id.clone(),
                field_path: field_path.clone(),
                value: value.clone(),
                status: crate::scene_instance::ComponentOverrideStatus::Active,
            };

            // Check for existing override at same key to capture for inverse
            let existing = instance.component_overrides.iter().find(|p| {
                p.target_local_id == *target_local_id
                    && p.component_type_id == *component_type_id
                    && p.field_path == *field_path
            }).cloned();

            // Apply the upsert (replaces or appends)
            upsert_override(instance, patch);

            // Inverse: if there was a prior override, restore it via UpsertOverride;
            // otherwise remove it via RevertOverride
            match existing {
                Some(old_patch) => Ok(Command::UpsertOverride {
                    instance_id: instance_id.clone(),
                    target_local_id: old_patch.target_local_id,
                    component_type_id: old_patch.component_type_id,
                    field_path: old_patch.field_path,
                    value: old_patch.value,
                }),
                None => Ok(Command::RevertOverride {
                    instance_id: instance_id.clone(),
                    target_local_id: target_local_id.clone(),
                    component_type_id: component_type_id.clone(),
                    field_path: field_path.clone(),
                }),
            }
        }
        Command::RevertOverride {
            instance_id,
            target_local_id,
            component_type_id,
            field_path,
        } => {
            let instance = doc
                .instances
                .get_mut(instance_id)
                .ok_or_else(|| CommandError::InstanceNotFound(instance_id.clone()))?;

            // Idempotent remove: capture if present
            let removed = remove_override(
                instance,
                target_local_id.clone(),
                component_type_id.clone(),
                field_path.clone(),
            );

            match removed {
                Some(patch) => {
                    // Inverse: re-insert via UpsertOverride
                    Ok(Command::UpsertOverride {
                        instance_id: instance_id.clone(),
                        target_local_id: patch.target_local_id,
                        component_type_id: patch.component_type_id,
                        field_path: patch.field_path,
                        value: patch.value,
                    })
                }
                None => {
                    // No-op: inverse is self
                    Ok(Command::RevertOverride {
                        instance_id: instance_id.clone(),
                        target_local_id: target_local_id.clone(),
                        component_type_id: component_type_id.clone(),
                        field_path: field_path.clone(),
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ComponentInstance;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn empty_doc() -> SceneDocument {
        SceneDocument {
            version: "0.1".to_string(),
            scene_id: "test".to_string(),
            name: "Test".to_string(),
            entities: vec![],
            instances: BTreeMap::new(),
        }
    }

    fn entity_with_components(id: &str, name: &str, components: Vec<ComponentInstance>) -> Entity {
        Entity {
            id: StableId::new(id),
            local_id: LocalId::new(id),
            name: name.to_string(),
            parent: None,
            components,
        }
    }

    fn transform2d(x: f32, y: f32) -> ComponentInstance {
        ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: json!({
                "translation": {"x": x, "y": y},
                "rotation": 0.0,
                "scale": {"x": 1.0, "y": 1.0}
            }),
        }
    }

    // ===== CreateEntity =====

    #[test]
    fn test_create_entity_adds_fresh_entity() {
        let mut doc = empty_doc();
        let cmd = Command::CreateEntity {
            id: StableId::new("ent_new"),
            name: "Foo".to_string(),
            components: vec![],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(doc.entities[0].id.as_str(), "ent_new");

        // Inverse should be DeleteEntity
        match inverse {
            Command::DeleteEntity { id } => assert_eq!(id.as_str(), "ent_new"),
            _ => panic!("Wrong inverse variant"),
        }
    }

    #[test]
    fn test_create_entity_rejects_duplicate_id() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_dup", "Foo", vec![]));
        let cmd = Command::CreateEntity {
            id: StableId::new("ent_dup"),
            name: "Bar".to_string(),
            components: vec![],
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(CommandError::DuplicateId(_))));
        assert_eq!(doc.entities.len(), 1); // unchanged
    }

    // ===== DeleteEntity =====

    #[test]
    fn test_delete_entity_removes_leaf() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::DeleteEntity {
            id: StableId::new("ent_01"),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 0);
        // Inverse is CreateEntity
        assert!(matches!(inverse, Command::CreateEntity { .. }));
    }

    #[test]
    fn test_delete_entity_reparents_children_to_root() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("parent", "Parent", vec![]));
        doc.entities.push(Entity {
            id: StableId::new("child"),
            local_id: LocalId::new("child"),
            name: "Child".to_string(),
            parent: Some(StableId::new("parent")),
            components: vec![],
        });
        let cmd = Command::DeleteEntity {
            id: StableId::new("parent"),
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(doc.entities[0].parent, None); // child reparented to root
    }

    #[test]
    fn test_delete_entity_missing_fails() {
        let mut doc = empty_doc();
        let cmd = Command::DeleteEntity {
            id: StableId::new("ent_missing"),
        };
        assert!(matches!(
            apply(&mut doc, &cmd),
            Err(CommandError::EntityNotFound(_))
        ));
    }

    // ===== AddComponent =====

    #[test]
    fn test_add_component_with_valid_schema() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::AddComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            values: json!({"translation": {"x": 1.0, "y": 2.0}, "rotation": 0.0, "scale": {"x": 1.0, "y": 1.0}}),
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].components.len(), 1);
        assert_eq!(doc.entities[0].components[0].type_id, "editor.Transform2D");
    }

    #[test]
    fn test_add_component_unknown_schema_rejected() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::AddComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Bogus".to_string(),
            values: json!({}),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(CommandError::UnknownSchema(_))));
        assert!(doc.entities[0].components.is_empty());
    }

    #[test]
    fn test_add_component_preserves_unknown_fields() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::AddComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            values: json!({"translation": {"x": 0.0, "y": 0.0}, "rotation": 0.0, "scale": {"x": 1.0, "y": 1.0}, "future_field": 42}),
        };
        apply(&mut doc, &cmd).unwrap();
        let val = &doc.entities[0].components[0].values;
        assert_eq!(val.get("future_field").unwrap(), &json!(42));
    }

    // ===== RemoveComponent =====

    #[test]
    fn test_remove_component_removes_existing() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components(
            "ent_01",
            "Foo",
            vec![transform2d(0.0, 0.0)],
        ));
        let cmd = Command::RemoveComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
        };
        apply(&mut doc, &cmd).unwrap();
        assert!(doc.entities[0].components.is_empty());
    }

    #[test]
    fn test_remove_component_absent_is_noop() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::RemoveComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Sprite2D".to_string(),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        // No-op: inverse is self
        assert!(matches!(inverse, Command::RemoveComponent { .. }));
        assert!(doc.entities[0].components.is_empty());
    }

    // ===== SetComponentField =====

    #[test]
    fn test_set_component_field_simple_path() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components(
            "ent_01",
            "Foo",
            vec![transform2d(0.0, 0.0)],
        ));
        let cmd = Command::SetComponentField {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            field_path: "translation.x".to_string(),
            value: json!(100.0),
        };
        apply(&mut doc, &cmd).unwrap();
        let val = &doc.entities[0].components[0].values;
        assert_eq!(val["translation"]["x"], json!(100.0));
        assert_eq!(val["translation"]["y"], json!(0.0));
    }

    #[test]
    fn test_set_component_field_nested_path() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components(
            "ent_01",
            "Foo",
            vec![ComponentInstance {
                type_id: "editor.Sprite2D".to_string(),
                values: json!({"color": {"r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0}}),
            }],
        ));
        let cmd = Command::SetComponentField {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Sprite2D".to_string(),
            field_path: "color.r".to_string(),
            value: json!(0.5),
        };
        apply(&mut doc, &cmd).unwrap();
        let val = &doc.entities[0].components[0].values;
        assert_eq!(val["color"]["r"], json!(0.5));
        assert_eq!(val["color"]["g"], json!(1.0));
    }

    #[test]
    fn test_set_component_field_missing_fails() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components(
            "ent_01",
            "Foo",
            vec![transform2d(0.0, 0.0)],
        ));
        let cmd = Command::SetComponentField {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            field_path: "nonexistent.field".to_string(),
            value: json!(42),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(CommandError::FieldNotFound(_))));
    }

    // ===== ReparentEntity =====

    #[test]
    fn test_reparent_entity_valid() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("A", "A", vec![]));
        doc.entities.push(entity_with_components("B", "B", vec![]));
        let cmd = Command::ReparentEntity {
            entity_id: StableId::new("A"),
            old_parent: None,
            new_parent: Some(StableId::new("B")),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].parent, Some(StableId::new("B")));

        // Inverse points back: new_parent = what old_parent was (None, was root)
        match inverse {
            Command::ReparentEntity {
                old_parent,
                new_parent,
                ..
            } => {
                assert_eq!(old_parent, None);
                assert_eq!(new_parent, None);
            }
            _ => panic!("Wrong inverse"),
        }
    }

    #[test]
    fn test_reparent_entity_cycle_rejected() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("A", "A", vec![]));
        doc.entities.push(entity_with_components("B", "B", vec![]));
        doc.entities.push(Entity {
            id: StableId::new("C"),
            local_id: LocalId::new("C"),
            name: "C".to_string(),
            parent: Some(StableId::new("B")),
            components: vec![],
        });
        // B is child of A's chain? No, B is root. C is child of B.
        // Setting A's parent to C would create cycle: A → C → B (root, stops)
        // Actually that's not a cycle. Let's create a real cycle: A under C, then C under A.
        let cmd = Command::ReparentEntity {
            entity_id: StableId::new("A"),
            old_parent: None,
            new_parent: Some(StableId::new("C")),
        };
        // A -> C is fine (C is not A's descendant). Let's test self-parenting.
        let cmd_self = Command::ReparentEntity {
            entity_id: StableId::new("A"),
            old_parent: None,
            new_parent: Some(StableId::new("A")),
        };
        let result = apply(&mut doc, &cmd_self);
        assert!(matches!(result, Err(CommandError::WouldCreateCycle(_))));

        // Real cycle: A -> C -> B and then B -> A
        let _ = apply(&mut doc, &cmd); // A's parent = C
        let cmd_cycle = Command::ReparentEntity {
            entity_id: StableId::new("B"),
            old_parent: None,
            new_parent: Some(StableId::new("A")),
        };
        let result = apply(&mut doc, &cmd_cycle);
        assert!(matches!(result, Err(CommandError::WouldCreateCycle(_))));
    }

    // ===== RenameEntity =====

    #[test]
    fn test_rename_entity_updates_name_preserves_id() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01J", "Player", vec![]));
        let cmd = Command::RenameEntity {
            entity_id: StableId::new("ent_01J"),
            old_name: None,
            new_name: "PlayerSpawn".to_string(),
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].name, "PlayerSpawn");
        assert_eq!(doc.entities[0].id.as_str(), "ent_01J");
    }

    // ===== Batch =====

    #[test]
    fn test_batch_applies_all_commands() {
        let mut doc = empty_doc();
        let cmd = Command::Batch {
            label: "test-batch".to_string(),
            commands: vec![
                Command::CreateEntity {
                    id: StableId::new("ent_01"),
                    name: "Foo".to_string(),
                    components: vec![],
                },
                Command::CreateEntity {
                    id: StableId::new("ent_02"),
                    name: "Bar".to_string(),
                    components: vec![],
                },
            ],
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 2);
    }

    #[test]
    fn test_batch_atomic_rollback_on_failure() {
        let mut doc = empty_doc();
        let cmd = Command::Batch {
            label: "test-batch".to_string(),
            commands: vec![
                Command::CreateEntity {
                    id: StableId::new("ent_01"),
                    name: "Foo".to_string(),
                    components: vec![],
                },
                Command::AddComponent {
                    entity_id: StableId::new("ent_01"),
                    type_id: "editor.Bogus".to_string(),
                    values: json!({}),
                },
            ],
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(CommandError::BatchFailed { .. })));
        // Document should be rolled back: 0 entities
        assert_eq!(doc.entities.len(), 0);
    }

    #[test]
    fn test_batch_inverse_reverses_order() {
        let mut doc = empty_doc();
        let cmd = Command::Batch {
            label: "test".to_string(),
            commands: vec![
                Command::CreateEntity {
                    id: StableId::new("A"),
                    name: "A".to_string(),
                    components: vec![],
                },
                Command::CreateEntity {
                    id: StableId::new("B"),
                    name: "B".to_string(),
                    components: vec![],
                },
                Command::CreateEntity {
                    id: StableId::new("C"),
                    name: "C".to_string(),
                    components: vec![],
                },
            ],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        match inverse {
            Command::Batch { commands, .. } => {
                assert_eq!(commands.len(), 3);
                // First inverse should undo C (the last applied)
                match &commands[0] {
                    Command::DeleteEntity { id } => assert_eq!(id.as_str(), "C"),
                    _ => panic!("Wrong first inverse"),
                }
                match &commands[2] {
                    Command::DeleteEntity { id } => assert_eq!(id.as_str(), "A"),
                    _ => panic!("Wrong last inverse"),
                }
            }
            _ => panic!("Wrong inverse variant"),
        }
    }

    // ===== Roundtrip =====

    #[test]
    fn test_forward_inverse_roundtrip_create_entity() {
        let mut doc = empty_doc();
        let original_len = doc.entities.len();
        let cmd = Command::CreateEntity {
            id: StableId::new("ent_01"),
            name: "Foo".to_string(),
            components: vec![],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), original_len + 1);
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.entities.len(), original_len);
        assert_eq!(doc, empty_doc());
    }

    #[test]
    fn test_forward_inverse_roundtrip_set_field() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components(
            "ent_01",
            "Foo",
            vec![transform2d(50.0, 50.0)],
        ));
        let cmd = Command::SetComponentField {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            field_path: "translation.x".to_string(),
            value: json!(999.0),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(
            doc.entities[0].components[0].values["translation"]["x"],
            json!(999.0)
        );
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(
            doc.entities[0].components[0].values["translation"]["x"],
            json!(50.0)
        );
    }

    #[test]
    fn test_forward_inverse_roundtrip_reparent() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("A", "A", vec![]));
        doc.entities.push(entity_with_components("B", "B", vec![]));
        let cmd = Command::ReparentEntity {
            entity_id: StableId::new("A"),
            old_parent: None,
            new_parent: Some(StableId::new("B")),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].parent, Some(StableId::new("B")));
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.entities[0].parent, None);
    }

    // ===== Validation leaves doc unchanged =====

    #[test]
    fn test_failed_validation_leaves_doc_unchanged() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01", "Foo", vec![]));
        let snapshot_before = doc.clone();
        let cmd = Command::AddComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Bogus".to_string(),
            values: json!({}),
        };
        let _ = apply(&mut doc, &cmd);
        assert_eq!(doc, snapshot_before);
    }

    #[test]
    fn test_field_path_helper_simple() {
        let mut v = json!({"a": 1});
        let old = set_field_path(&mut v, "a", json!(99)).unwrap();
        assert_eq!(old, json!(1));
        assert_eq!(v["a"], json!(99));
    }

    #[test]
    fn test_field_path_helper_nested() {
        let mut v = json!({"a": {"b": {"c": 1}}});
        let old = set_field_path(&mut v, "a.b.c", json!(42)).unwrap();
        assert_eq!(old, json!(1));
        assert_eq!(v["a"]["b"]["c"], json!(42));
    }

    #[test]
    fn test_field_path_helper_missing() {
        let mut v = json!({"a": 1});
        let result = set_field_path(&mut v, "b", json!(42));
        assert!(matches!(result, Err(CommandError::FieldNotFound(_))));
    }

    // ===== UpsertOverride =====

    fn make_instance(instance_id: &str) -> (SceneInstance, SceneDocument) {
        let inst = SceneInstance {
            instance_components: vec![],
            instance_id: StableId::new(instance_id),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: Default::default(),
            component_overrides: vec![],
            orphaned_component_overrides: vec![],
        };
        let mut doc = empty_doc();
        doc.instances
            .insert(StableId::new(instance_id), inst.clone());
        (inst, doc)
    }

    // S1 — Upsert inserts into empty overrides
    #[test]
    fn test_upsert_override_inserts_into_empty() {
        let (_, mut doc) = make_instance("inst_1");
        let cmd = Command::UpsertOverride {
            instance_id: StableId::new("inst_1"),
            target_local_id: crate::scene_asset::LocalId::new("root"),
            component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
            value: serde_json::json!("cannon.png"),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(
            doc.instances
                .get(&StableId::new("inst_1"))
                .unwrap()
                .component_overrides
                .len(),
            1
        );
        // Inverse should be RevertOverride since there was no prior override
        assert!(matches!(inverse, Command::RevertOverride { .. }));
    }

    // S2 — Upsert forward/inverse roundtrip (replaces same-key override)
    #[test]
    fn test_forward_inverse_roundtrip_upsert_override() {
        let (_, mut doc) = make_instance("inst_1");
        // Pre-populate with an override
        doc.instances
            .get_mut(&StableId::new("inst_1"))
            .unwrap()
            .component_overrides
            .push(crate::scene_instance::ComponentOverride {
                target_local_id: crate::scene_asset::LocalId::new("root"),
                component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
                field_path: vec!["asset".to_string()],
                value: serde_json::json!("cannon.png"),
                status: crate::scene_instance::ComponentOverrideStatus::Active,
            });

        let cmd = Command::UpsertOverride {
            instance_id: StableId::new("inst_1"),
            target_local_id: crate::scene_asset::LocalId::new("root"),
            component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
            value: serde_json::json!("enemy.png"),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();

        // Verify new value is stored
        assert_eq!(
            doc.instances
                .get(&StableId::new("inst_1"))
                .unwrap()
                .component_overrides[0]
                .value,
            serde_json::json!("enemy.png")
        );

        // Apply inverse — should restore original value
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(
            doc.instances
                .get(&StableId::new("inst_1"))
                .unwrap()
                .component_overrides[0]
                .value,
            serde_json::json!("cannon.png")
        );
    }

    // ===== RevertOverride =====

    // S3 — Revert removes the matching override
    #[test]
    fn test_revert_override_removes_matching() {
        let (_, mut doc) = make_instance("inst_1");
        doc.instances
            .get_mut(&StableId::new("inst_1"))
            .unwrap()
            .component_overrides
            .push(crate::scene_instance::ComponentOverride {
                target_local_id: crate::scene_asset::LocalId::new("root"),
                component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
                field_path: vec!["asset".to_string()],
                value: serde_json::json!("cannon.png"),
                status: crate::scene_instance::ComponentOverrideStatus::Active,
            });

        let cmd = Command::RevertOverride {
            instance_id: StableId::new("inst_1"),
            target_local_id: crate::scene_asset::LocalId::new("root"),
            component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert!(
            doc.instances
                .get(&StableId::new("inst_1"))
                .unwrap()
                .component_overrides
                .is_empty()
        );

        // Inverse should re-insert
        assert!(matches!(inverse, Command::UpsertOverride { .. }));
    }

    // S4 — Revert of absent override is no-op
    #[test]
    fn test_revert_override_noop() {
        let (_, mut doc) = make_instance("inst_1");
        let cmd = Command::RevertOverride {
            instance_id: StableId::new("inst_1"),
            target_local_id: crate::scene_asset::LocalId::new("root"),
            component_type_id: crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            field_path: vec!["asset".to_string()],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        // No override was present — still no override
        assert!(
            doc.instances
                .get(&StableId::new("inst_1"))
                .unwrap()
                .component_overrides
                .is_empty()
        );
        // Inverse is self (noop)
        assert!(matches!(inverse, Command::RevertOverride { .. }));
    }
}
