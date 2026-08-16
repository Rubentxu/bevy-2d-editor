//! Scene Instance Override Resolution
//!
//! Pure-functions module for the override lifecycle contracted by
//! ADR-0005 §Overrides and §Versioning and Resync.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::document::StableId;
use crate::scene_asset::{LocalId, SceneAssetDocument, SceneAssetEntity};
use crate::scene_instance::{ComponentOverride, ComponentOverrideStatus, SceneInstance};
use editor_model::ComponentInstance;

// ---------------------------------------------------------------------------
// Public Types
// ---------------------------------------------------------------------------

/// Result of merging a `SceneAssetDocument` with an instance's overrides.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResolvedScene {
    pub entities: BTreeMap<LocalId, ResolvedEntity>,
    pub id_map: BTreeMap<LocalId, StableId>,
    pub minted_stable_ids: BTreeSet<StableId>,
    pub unresolved: Vec<ComponentOverride>,
}

/// One entity inside a resolved scene (projection of `SceneAssetEntity`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResolvedEntity {
    pub local_id: LocalId,
    pub local_path: String,
    pub name: String,
    pub components: Vec<ComponentInstance>,
}

/// Summary of what happened during a `resync` call.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ResyncReport {
    pub active: usize,
    pub orphaned: usize,
    pub stale: usize,
    pub conflict: usize,
    pub rebound: usize,
}

/// An issue found by `validate_overrides`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OverrideIssue {
    pub code: String,
    pub patch: ComponentOverride,
    pub message: String,
}

/// Errors that can occur in `effective_values`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OverrideError {
    #[error("empty asset: no entities")]
    EmptyAsset,
    #[error("multiple roots: only single-root assets supported in spike")]
    MultipleRoots,
}

// ---------------------------------------------------------------------------
// Private Helpers
// ---------------------------------------------------------------------------

fn find_entity<'a>(
    asset: &'a SceneAssetDocument,
    local_id: &LocalId,
) -> Option<&'a SceneAssetEntity> {
    asset.entities.iter().find(|e| &e.local_id == local_id)
}

fn find_component<'a>(
    entity: &'a SceneAssetEntity,
    type_id: &str,
) -> Option<&'a ComponentInstance> {
    entity.components.iter().find(|c| c.type_id == type_id)
}

/// Walk a field-path through a JSON value.
///
/// Segments are traversed in order; returns the value at the terminal segment,
/// or `None` if any segment is missing or the path is empty.
fn walk_field_path<'a>(
    root: &'a serde_json::Value,
    segments: &[String],
) -> Option<&'a serde_json::Value> {
    if segments.is_empty() {
        return Some(root);
    }
    let mut current: &serde_json::Value = root;
    for seg in segments.iter().take(segments.len().saturating_sub(1)) {
        current = current.as_object()?.get(seg)?;
    }
    let last = segments.last()?;
    Some(current.as_object()?.get(last)?)
}

/// Walk a field-path through a JSON value (mutable, for insertion).
///
/// Returns the mutable reference to the terminal value, or `None` if the path
/// cannot be fully traversed.
fn walk_field_path_mut<'a>(
    root: &'a mut serde_json::Value,
    segments: &[String],
) -> Option<&'a mut serde_json::Value> {
    if segments.is_empty() {
        return Some(root);
    }
    let mut current: &mut serde_json::Value = root;
    for seg in segments.iter().take(segments.len().saturating_sub(1)) {
        current = current.as_object_mut()?.get_mut(seg)?;
    }
    let last = segments.last()?;
    Some(current.as_object_mut()?.get_mut(last)?)
}

/// Walk `field_path[1..]` inside `values` and insert `value` at the terminal key.
fn apply_field_path(
    component: &mut ComponentInstance,
    field_path: &[String],
    value: serde_json::Value,
) -> Result<(), ()> {
    if field_path.is_empty() {
        return Ok(());
    }
    let target = walk_field_path_mut(&mut component.values, field_path);
    match target {
        Some(t) => {
            *t = value;
            Ok(())
        }
        None => Err(()),
    }
}

/// Returns true when the two JSON values have different `serde_json` kinds.
fn detect_kind_mismatch(existing: &serde_json::Value, patch: &serde_json::Value) -> bool {
    json_kind(existing) != json_kind(patch)
}

/// `serde_json::Value` kind as a static string.
fn json_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// ---------------------------------------------------------------------------
// Public Functions
// ---------------------------------------------------------------------------

/// Pure re-classification of override patches against a given asset.
pub fn classify_overrides(
    asset: &SceneAssetDocument,
    patches: &[ComponentOverride],
) -> Vec<ComponentOverride> {
    patches
        .iter()
        .map(|patch| {
            let entity = match find_entity(asset, &patch.target_local_id) {
                Some(e) => e,
                None => {
                    return ComponentOverride {
                        status: ComponentOverrideStatus::Orphaned,
                        ..patch.clone()
                    };
                }
            };

            let type_id = patch.component_type_id.as_str();
            let component = match find_component(entity, type_id) {
                Some(c) => c,
                None => {
                    return ComponentOverride {
                        status: ComponentOverrideStatus::Orphaned,
                        ..patch.clone()
                    };
                }
            };

            // Walk field_path to check field existence inside the target component.
            if !patch.field_path.is_empty() {
                if walk_field_path(&component.values, &patch.field_path).is_none() {
                    return ComponentOverride {
                        status: ComponentOverrideStatus::Stale,
                        ..patch.clone()
                    };
                }
            }

            // Kind mismatch check
            let existing_value: Option<serde_json::Value> = if !patch.field_path.is_empty() {
                walk_field_path(&component.values, &patch.field_path).cloned()
            } else {
                Some(component.values.clone())
            };

            if let Some(existing) = existing_value {
                if detect_kind_mismatch(&existing, &patch.value) {
                    return ComponentOverride {
                        status: ComponentOverrideStatus::Conflict,
                        ..patch.clone()
                    };
                }
            }

            ComponentOverride {
                status: ComponentOverrideStatus::Active,
                ..patch.clone()
            }
        })
        .collect()
}

/// Mint a fresh `StableId` for every entity in the asset.
pub fn mint_id_map(
    asset: &SceneAssetDocument,
    mint: &mut dyn FnMut() -> StableId,
) -> BTreeMap<LocalId, StableId> {
    asset
        .entities
        .iter()
        .map(|e| (e.local_id.clone(), mint()))
        .collect()
}

/// Extend an existing `id_map` with new entities from the asset (non-destructive).
pub fn reconcile_id_map(
    asset: &SceneAssetDocument,
    existing: &BTreeMap<LocalId, StableId>,
    mint: &mut dyn FnMut() -> StableId,
) -> BTreeMap<LocalId, StableId> {
    let mut result = existing.clone();
    for entity in &asset.entities {
        if !result.contains_key(&entity.local_id) {
            result.insert(entity.local_id.clone(), mint());
        }
    }
    result
}

/// Try to rebind an orphaned component override by exact `target_local_id` match (spike).
pub fn try_rebind(asset: &SceneAssetDocument, orphaned: &ComponentOverride) -> Option<LocalId> {
    find_entity(asset, &orphaned.target_local_id).map(|e| e.local_id.clone())
}

/// Read-only issue scan for a `SceneInstance` against its asset.
pub fn validate_overrides(
    asset: &SceneAssetDocument,
    instance: &SceneInstance,
) -> Vec<OverrideIssue> {
    let mut issues = Vec::new();

    for patch in instance
        .component_overrides
        .iter()
        .chain(instance.orphaned_component_overrides.iter())
    {
        let entity = match find_entity(asset, &patch.target_local_id) {
            Some(e) => e,
            None => {
                issues.push(OverrideIssue {
                    code: "missing_entity".to_string(),
                    patch: patch.clone(),
                    message: format!(
                        "Override targets entity {} which does not exist in asset",
                        patch.target_local_id.as_str()
                    ),
                });
                continue;
            }
        };

        let type_id = patch.component_type_id.as_str();
        let component = match find_component(entity, type_id) {
            Some(c) => c,
            None => {
                issues.push(OverrideIssue {
                    code: "missing_component".to_string(),
                    patch: patch.clone(),
                    message: format!(
                        "Override field_path[0] '{}' does not match any component on entity '{}'",
                        type_id, entity.name
                    ),
                });
                continue;
            }
        };

        // Duplicate field check
        let field_key = patch.field_path.first().map(|s| s.as_str()).unwrap_or("");
        let dupe_count = instance
            .component_overrides
            .iter()
            .filter(|p| {
                p.target_local_id == patch.target_local_id
                    && p.component_type_id == patch.component_type_id
                    && p.field_path == patch.field_path
            })
            .count();
        if dupe_count > 1 {
            issues.push(OverrideIssue {
                code: "duplicate_field".to_string(),
                patch: patch.clone(),
                message: format!(
                    "Duplicate override for field '{}' on entity '{}'",
                    field_key, entity.name
                ),
            });
        }

        // Walk field path for missing_field check
        if !patch.field_path.is_empty() {
            if walk_field_path(&component.values, &patch.field_path).is_none() {
                issues.push(OverrideIssue {
                    code: "missing_field".to_string(),
                    patch: patch.clone(),
                    message: format!(
                        "Field path {:?} does not resolve in component '{}'",
                        &patch.field_path, type_id
                    ),
                });
            }
        }

        // Type conflict check
        let terminal: Option<serde_json::Value> = if !patch.field_path.is_empty() {
            walk_field_path(&component.values, &patch.field_path).cloned()
        } else {
            Some(component.values.clone())
        };

        if let Some(existing) = terminal {
            if detect_kind_mismatch(&existing, &patch.value) {
                issues.push(OverrideIssue {
                    code: "type_conflict".to_string(),
                    patch: patch.clone(),
                    message: format!(
                        "Override value kind '{}' does not match existing value kind '{}'",
                        json_kind(&patch.value),
                        json_kind(&existing)
                    ),
                });
            }
        }
    }

    issues
}

/// Compute effective values: read-only merge of asset + active overrides.
/// Validates that a patch can be applied to a resolved entity.
///
/// Returns `Ok(())` if the patch is valid, or `Err(patch)` if it cannot be applied.
fn validate_patch_target(
    resolved_entity: &ResolvedEntity,
    patch: &ComponentOverride,
) -> Result<(), ComponentOverride> {
    let type_id = patch.component_type_id.as_str();
    let component = match resolved_entity
        .components
        .iter()
        .find(|c| c.type_id == type_id)
    {
        Some(c) => c,
        None => return Err(patch.clone()),
    };

    // Walk field_path to verify path inside the target component.
    if !patch.field_path.is_empty() {
        if walk_field_path(&component.values, &patch.field_path).is_none() {
            return Err(patch.clone());
        }
    }

    // Kind check
    let terminal: Option<&serde_json::Value> = if !patch.field_path.is_empty() {
        walk_field_path(&component.values, &patch.field_path)
    } else {
        Some(&component.values)
    };

    if let Some(existing) = terminal {
        if detect_kind_mismatch(existing, &patch.value) {
            return Err(patch.clone());
        }
    }

    Ok(())
}

/// Applies a validated patch to a resolved entity's component.
fn apply_patch_to_resolved_entity(
    resolved_entity: &mut ResolvedEntity,
    patch: &ComponentOverride,
) -> Result<(), ComponentOverride> {
    let type_id = patch.component_type_id.as_str();
    let component = match resolved_entity
        .components
        .iter_mut()
        .find(|c| c.type_id == type_id)
    {
        Some(c) => c,
        None => return Err(patch.clone()),
    };

    let mut comp = component.clone();
    if apply_field_path(&mut comp, &patch.field_path, patch.value.clone()).is_ok() {
        *component = comp;
        Ok(())
    } else {
        Err(patch.clone())
    }
}

/// Compute the read-only effective value of a `SceneInstance`: the
/// merge of asset defaults with the instance's active component
/// overrides. The `mint` callback supplies a fresh `StableId` for
/// each entity that doesn't have one. Returns `ResolvedScene`
/// (the merged document) or `OverrideError::EmptyAsset` if the asset
/// has no entities.
pub fn effective_values(
    asset: &SceneAssetDocument,
    instance: &SceneInstance,
    mint: &mut dyn FnMut() -> StableId,
) -> Result<ResolvedScene, OverrideError> {
    if asset.entities.is_empty() {
        return Err(OverrideError::EmptyAsset);
    }

    let entities: BTreeMap<LocalId, ResolvedEntity> = asset
        .entities
        .iter()
        .map(|e| {
            (
                e.local_id.clone(),
                ResolvedEntity {
                    local_id: e.local_id.clone(),
                    local_path: e.local_path.clone(),
                    name: e.name.clone(),
                    components: e.components.clone(),
                },
            )
        })
        .collect();

    let mut unresolved = Vec::new();
    let mut resolved_entities: BTreeMap<LocalId, ResolvedEntity> = entities;

    for patch in &instance.component_overrides {
        if patch.status == ComponentOverrideStatus::Orphaned {
            continue;
        }

        let resolved_entity = match resolved_entities.get_mut(&patch.target_local_id) {
            Some(e) => e,
            None => {
                unresolved.push(patch.clone());
                continue;
            }
        };

        // Validate the patch target
        if validate_patch_target(resolved_entity, patch).is_err() {
            unresolved.push(patch.clone());
            continue;
        }

        // Apply the validated patch
        if apply_patch_to_resolved_entity(resolved_entity, patch).is_err() {
            unresolved.push(patch.clone());
        }
    }

    let id_map = mint_id_map(asset, mint);
    let minted_stable_ids: BTreeSet<StableId> = id_map.values().cloned().collect();

    Ok(ResolvedScene {
        entities: resolved_entities,
        id_map,
        minted_stable_ids,
        unresolved,
    })
}

// ---------------------------------------------------------------------------
// Override Mutation Helpers
// ---------------------------------------------------------------------------

/// Insert or replace a component override on a `SceneInstance`.
///
/// Key is `(target_local_id, component_type_id, field_path)`. If an override
/// with the same key already exists, it is replaced. Otherwise the patch is
/// appended. The patch's `status` is forced to `Active` regardless of input.
///
/// This function does NOT mutate `id_map` or `instance_components`.
pub fn upsert_override(inst: &mut SceneInstance, mut patch: ComponentOverride) {
    patch.status = ComponentOverrideStatus::Active;
    if let Some(pos) = inst.component_overrides.iter().position(|p| {
        p.target_local_id == patch.target_local_id
            && p.component_type_id == patch.component_type_id
            && p.field_path == patch.field_path
    }) {
        inst.component_overrides[pos] = patch;
    } else {
        inst.component_overrides.push(patch);
    }
}

/// Remove a component override from a `SceneInstance` by key.
///
/// Returns the removed `ComponentOverride` if one matched the key, or `None`
/// if no override matched (idempotent). Does not touch `orphaned_component_overrides`.
pub fn remove_override(
    inst: &mut SceneInstance,
    target_local_id: LocalId,
    component_type_id: crate::schema::ComponentTypeId,
    field_path: Vec<String>,
) -> Option<ComponentOverride> {
    let pos = inst.component_overrides.iter().position(|p| {
        p.target_local_id == target_local_id
            && p.component_type_id == component_type_id
            && p.field_path == field_path
    });
    pos.map(|p| inst.component_overrides.remove(p))
}

/// A flat projection of a stored override, used by the inspector to render
/// per-field indicators without re-walking the asset schema.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FieldOverrideEntry {
    pub local_id: LocalId,
    pub component_type_id: crate::schema::ComponentTypeId,
    pub field_path: Vec<String>,
    pub status: ComponentOverrideStatus,
}

/// Build a flat index of every stored override (active + orphaned) on a `SceneInstance`.
///
/// This is a read-only projection used by `override_field_status_wasm` to supply
/// per-field override indicators to the inspector UI.
pub fn field_override_index(inst: &SceneInstance) -> Vec<FieldOverrideEntry> {
    inst.component_overrides
        .iter()
        .chain(inst.orphaned_component_overrides.iter())
        .map(|p| FieldOverrideEntry {
            local_id: p.target_local_id.clone(),
            component_type_id: p.component_type_id.clone(),
            field_path: p.field_path.clone(),
            status: p.status,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Override Mutation Helpers — end
// ---------------------------------------------------------------------------

/// Re-validate overrides when the asset version has changed.
pub fn resync(
    asset: &SceneAssetDocument,
    instance: &mut SceneInstance,
    new_asset_version: u32,
) -> ResyncReport {
    instance.asset_version_seen = new_asset_version;

    let mut report = ResyncReport::default();
    let mut new_overrides: Vec<ComponentOverride> = Vec::new();
    let mut new_orphaned: Vec<ComponentOverride> = Vec::new();

    let classified = classify_overrides(asset, &instance.component_overrides);

    for (patch, classified_patch) in instance.component_overrides.iter().zip(classified.iter()) {
        match (patch.status, classified_patch.status) {
            (ComponentOverrideStatus::Active, ComponentOverrideStatus::Orphaned) => {
                let mut p = classified_patch.clone();
                p.status = ComponentOverrideStatus::Orphaned;
                new_orphaned.push(p);
                report.orphaned += 1;
            }
            (ComponentOverrideStatus::Active, ComponentOverrideStatus::Stale) => {
                let mut p = classified_patch.clone();
                p.status = ComponentOverrideStatus::Stale;
                new_overrides.push(p);
                report.stale += 1;
            }
            (ComponentOverrideStatus::Active, ComponentOverrideStatus::Conflict) => {
                let mut p = classified_patch.clone();
                p.status = ComponentOverrideStatus::Conflict;
                new_overrides.push(p);
                report.conflict += 1;
            }
            (ComponentOverrideStatus::Active, ComponentOverrideStatus::Active) => {
                new_overrides.push(classified_patch.clone());
                report.active += 1;
            }
            (_, ComponentOverrideStatus::Orphaned) => {
                let mut p = classified_patch.clone();
                p.status = ComponentOverrideStatus::Orphaned;
                new_orphaned.push(p);
                report.orphaned += 1;
            }
            (_, ComponentOverrideStatus::Stale) => {
                new_overrides.push(classified_patch.clone());
                report.stale += 1;
            }
            (_, ComponentOverrideStatus::Conflict) => {
                new_overrides.push(classified_patch.clone());
                report.conflict += 1;
            }
            (_, ComponentOverrideStatus::Active) => {
                new_overrides.push(classified_patch.clone());
                report.active += 1;
            }
        }
    }

    let mut still_orphaned: Vec<ComponentOverride> = Vec::new();
    for orphan in instance.orphaned_component_overrides.iter() {
        if let Some(new_id) = try_rebind(asset, orphan) {
            let mut rebound_patch = orphan.clone();
            rebound_patch.target_local_id = new_id;
            rebound_patch.status = ComponentOverrideStatus::Active;
            new_overrides.push(rebound_patch);
            report.rebound += 1;
        } else {
            still_orphaned.push(orphan.clone());
        }
    }

    let mut counter = 0u32;
    let mut mint = || {
        counter += 1;
        StableId::new(format!("sid_{}", counter))
    };
    instance.id_map = reconcile_id_map(asset, &instance.id_map, &mut mint);

    instance.component_overrides = new_overrides;
    instance.orphaned_component_overrides = new_orphaned;
    instance.orphaned_component_overrides.extend(still_orphaned);

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mint(counter: &mut u32) -> impl FnMut() -> StableId + '_ {
        move || {
            *counter += 1;
            StableId::new(format!("sid_{}", *counter))
        }
    }

    fn component_override(
        target_local_id: &str,
        component_type_id: &str,
        field_path: Vec<&str>,
        value: serde_json::Value,
        status: ComponentOverrideStatus,
    ) -> ComponentOverride {
        ComponentOverride {
            target_local_id: LocalId::new(target_local_id),
            component_type_id: crate::schema::ComponentTypeId::new(component_type_id),
            field_path: field_path.into_iter().map(str::to_string).collect(),
            value,
            status,
        }
    }

    /// Base fixture for `SceneAssetDocument` with common defaults.
    fn fixture_asset() -> SceneAssetDocument {
        SceneAssetDocument {
            layers: vec![],
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        }
    }

    /// Base fixture for `SceneInstance` with common defaults.
    fn fixture_instance() -> SceneInstance {
        SceneInstance {
            instance_components: vec![],
            instance_id: StableId::new("inst_1"),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: Default::default(),
            component_overrides: vec![],
            orphaned_component_overrides: vec![],
        }
    }

    /// Build a `SceneAssetDocument` base fixture with one entity.
    fn fixture_asset_with_entity(
        local_id: &str,
        local_path: &str,
        name: &str,
        component_type_id: &str,
        component_values: serde_json::Value,
    ) -> SceneAssetDocument {
        let mut asset = fixture_asset();
        asset.entities.push(SceneAssetEntity {
            local_id: LocalId::new(local_id),
            local_path: local_path.to_string(),
            name: name.to_string(),
            components: vec![ComponentInstance {
                type_id: component_type_id.to_string(),
                values: component_values,
            }],
        });
        asset
    }

    /// Build a `SceneAssetDocument` base fixture with multiple entities (empty components).
    fn fixture_asset_with_entities(entities: &[(&str, &str, &str)]) -> SceneAssetDocument {
        let mut asset = fixture_asset();
        for (local_id, local_path, name) in entities {
            asset.entities.push(SceneAssetEntity {
                local_id: LocalId::new(*local_id),
                local_path: (*local_path).to_string(),
                name: (*name).to_string(),
                components: vec![],
            });
        }
        asset
    }

    /// Build a `SceneAssetDocument` base fixture with multiple entities with components.
    fn fixture_asset_with_entities_and_components(
        entities: &[(&str, &str, &str, &str, serde_json::Value)],
    ) -> SceneAssetDocument {
        let mut asset = fixture_asset();
        for (local_id, local_path, name, component_type_id, component_values) in entities {
            asset.entities.push(SceneAssetEntity {
                local_id: LocalId::new(*local_id),
                local_path: (*local_path).to_string(),
                name: (*name).to_string(),
                components: vec![ComponentInstance {
                    type_id: (*component_type_id).to_string(),
                    values: component_values.clone(),
                }],
            });
        }
        asset
    }

    /// Build a `SceneInstance` base fixture with one component override.
    fn fixture_instance_with_override(
        target_local_id: &str,
        component_type_id: &str,
        field_path: Vec<&str>,
        value: serde_json::Value,
        status: ComponentOverrideStatus,
    ) -> SceneInstance {
        let mut inst = fixture_instance();
        inst.component_overrides.push(component_override(
            target_local_id,
            component_type_id,
            field_path,
            value,
            status,
        ));
        inst
    }

    #[test]
    fn test_classify_overrides_active() {
        let asset = fixture_asset_with_entity(
            "root",
            "root",
            "Root",
            "editor.Sprite2D",
            serde_json::json!({"asset": "player.png"}),
        );

        let patch = component_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );

        let result = classify_overrides(&asset, std::slice::from_ref(&patch));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, ComponentOverrideStatus::Active);
    }

    #[test]
    fn test_classify_overrides_orphaned_short_form() {
        let asset = fixture_asset_with_entity(
            "root",
            "root",
            "Root",
            "editor.Sprite2D",
            serde_json::json!({"asset": "player.png"}),
        );

        let patch = component_override(
            "root",
            "Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );

        let result = classify_overrides(&asset, std::slice::from_ref(&patch));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, ComponentOverrideStatus::Orphaned);
    }

    #[test]
    fn test_mint_id_map() {
        let asset = fixture_asset_with_entities(&[("a", "a", "A"), ("b", "b", "B")]);

        let mut counter = 0u32;
        let id_map = {
            let mut mint = make_mint(&mut counter);
            mint_id_map(&asset, &mut mint)
        };

        assert_eq!(id_map.len(), 2);
        assert!(id_map.contains_key(&LocalId::new("a")));
        assert!(id_map.contains_key(&LocalId::new("b")));
        assert_eq!(counter, 2);
    }

    #[test]
    fn test_reconcile_id_map_preserves_existing() {
        let asset = fixture_asset_with_entities(&[("a", "a", "A"), ("b", "b", "B")]);

        let existing: BTreeMap<LocalId, StableId> =
            vec![(LocalId::new("a"), StableId::new("existing_a"))]
                .into_iter()
                .collect();

        let mut counter = 0u32;
        let result = {
            let mut mint = make_mint(&mut counter);
            reconcile_id_map(&asset, &existing, &mut mint)
        };

        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get(&LocalId::new("a")).map(|s| s.as_str()),
            Some("existing_a")
        );
        assert!(result.get(&LocalId::new("b")).is_some());
    }

    #[test]
    fn test_try_rebind_exact_match() {
        let asset = fixture_asset_with_entity(
            "new_abc",
            "root/player/weapons/cannon",
            "Cannon",
            "",
            serde_json::Value::Null,
        );

        // Orphan's target_local_id matches the asset entity's local_id — exact rebind succeeds
        let orphan = component_override(
            "new_abc",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Orphaned,
        );

        let result = try_rebind(&asset, &orphan);
        assert_eq!(result, Some(LocalId::new("new_abc")));
    }

    #[test]
    fn test_resync_preserves_active_on_rename() {
        let mut asset = fixture_asset_with_entity(
            "abc",
            "abc",
            "Cannon",
            "editor.Sprite2D",
            serde_json::json!({"asset": "player.png"}),
        );
        asset.version = 2;

        let mut instance = fixture_instance_with_override(
            "abc",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );
        instance.id_map = vec![(LocalId::new("abc"), StableId::new("ent_a"))]
            .into_iter()
            .collect();
        instance.asset_version_seen = 1;

        let report = resync(&asset, &mut instance, 2);

        assert_eq!(report.active, 1);
        assert_eq!(report.orphaned, 0);
        assert_eq!(report.stale, 0);
        assert_eq!(report.conflict, 0);
        assert_eq!(report.rebound, 0);
        assert_eq!(instance.asset_version_seen, 2);
        assert_eq!(instance.component_overrides.len(), 1);
        assert_eq!(
            instance.component_overrides[0].status,
            ComponentOverrideStatus::Active
        );
    }

    #[test]
    fn test_resync_moves_to_orphaned_on_entity_removed() {
        let mut asset_v2 = fixture_asset();
        asset_v2.version = 2;

        let mut instance = fixture_instance_with_override(
            "abc",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );
        instance.asset_version_seen = 1;

        let report = resync(&asset_v2, &mut instance, 2);

        assert_eq!(report.active, 0);
        assert_eq!(report.orphaned, 1);
        assert_eq!(instance.component_overrides.len(), 0);
        assert_eq!(instance.orphaned_component_overrides.len(), 1);
        assert_eq!(
            instance.orphaned_component_overrides[0].status,
            ComponentOverrideStatus::Orphaned
        );
        assert_eq!(
            instance.orphaned_component_overrides[0].target_local_id,
            LocalId::new("abc")
        );
    }

    #[test]
    fn test_resync_marks_stale_on_field_rename() {
        let mut asset_v2 = fixture_asset_with_entity(
            "root",
            "root",
            "Root",
            "editor.Sprite2D",
            serde_json::json!({"image": "player.png"}),
        );
        asset_v2.version = 2;

        let mut instance = fixture_instance_with_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );
        instance.asset_version_seen = 1;

        let report = resync(&asset_v2, &mut instance, 2);

        assert_eq!(report.stale, 1);
        assert_eq!(report.active, 0);
        assert_eq!(instance.component_overrides.len(), 1);
        assert_eq!(
            instance.component_overrides[0].status,
            ComponentOverrideStatus::Stale
        );
    }

    #[test]
    fn test_resync_marks_conflict_on_type_change() {
        let mut asset_v2 = fixture_asset_with_entity(
            "player",
            "player",
            "Player",
            "editor.Health",
            serde_json::json!({"current": "full"}),
        );
        asset_v2.version = 2;

        let mut instance = fixture_instance_with_override(
            "player",
            "editor.Health",
            vec!["current"],
            serde_json::json!(42),
            ComponentOverrideStatus::Active,
        );
        instance.asset_version_seen = 1;

        let report = resync(&asset_v2, &mut instance, 2);

        assert_eq!(report.conflict, 1);
        assert_eq!(instance.component_overrides.len(), 1);
        assert_eq!(
            instance.component_overrides[0].status,
            ComponentOverrideStatus::Conflict
        );
    }

    #[test]
    fn test_resync_rebinds_via_local_id() {
        let mut asset_v2 = fixture_asset();
        asset_v2.version = 2;

        let mut instance = fixture_instance_with_override(
            "abc",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );
        instance.asset_version_seen = 1;
        instance.id_map = vec![(LocalId::new("abc"), StableId::new("ent_a"))]
            .into_iter()
            .collect();

        resync(&asset_v2, &mut instance, 2);
        assert_eq!(instance.orphaned_component_overrides.len(), 1);

        let mut asset_v3 = fixture_asset_with_entity(
            "abc",
            "weapons/cannon",
            "Cannon",
            "editor.Sprite2D",
            serde_json::json!({"asset": "player.png"}),
        );
        asset_v3.version = 3;

        let report = resync(&asset_v3, &mut instance, 3);

        assert_eq!(report.rebound, 1);
        assert_eq!(instance.component_overrides.len(), 1);
        assert_eq!(
            instance.component_overrides[0].status,
            ComponentOverrideStatus::Active
        );
        assert_eq!(
            instance.component_overrides[0].target_local_id,
            LocalId::new("abc")
        );
    }

    #[test]
    fn test_effective_values_minimal() {
        let asset = fixture_asset_with_entity(
            "root",
            "root",
            "Root",
            "editor.Sprite2D",
            serde_json::json!({"asset": "player.png"}),
        );

        let instance = fixture_instance_with_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );

        let mut counter = 0u32;
        let result = {
            let mut mint = make_mint(&mut counter);
            effective_values(&asset, &instance, &mut mint)
        };

        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.entities.len(), 1);
        assert_eq!(resolved.id_map.len(), 1);
        assert!(resolved.unresolved.is_empty());
    }

    #[test]
    fn test_effective_values_no_overrides() {
        let asset = fixture_asset_with_entities_and_components(&[
            (
                "a",
                "a",
                "A",
                "editor.Sprite2D",
                serde_json::json!({"asset": "a.png"}),
            ),
            (
                "b",
                "b",
                "B",
                "editor.Sprite2D",
                serde_json::json!({"asset": "b.png"}),
            ),
        ]);

        let instance = fixture_instance();

        let resolved = {
            let mut counter = 0u32;
            let mut mint = make_mint(&mut counter);
            effective_values(&asset, &instance, &mut mint).unwrap()
        };
        assert_eq!(resolved.entities.len(), 2);
        assert!(resolved.unresolved.is_empty());
        assert_eq!(resolved.id_map.len(), 2);
    }

    #[test]
    fn test_validate_overrides_missing_entity() {
        let asset = fixture_asset_with_entity("root", "root", "Root", "", serde_json::Value::Null);

        let instance = fixture_instance_with_override(
            "nonexistent",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );

        let issues = validate_overrides(&asset, &instance);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].code, "missing_entity");
    }

    // ===== upsert_override / remove_override / field_override_index =====

    // S11 — Upsert appends to empty overrides
    #[test]
    fn test_upsert_override_appends_to_empty() {
        let mut inst = fixture_instance();
        let patch = component_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Stale, // input status ignored, forced Active
        );
        upsert_override(&mut inst, patch);
        assert_eq!(inst.component_overrides.len(), 1);
        assert_eq!(
            inst.component_overrides[0].status,
            ComponentOverrideStatus::Active
        );
        assert_eq!(
            inst.component_overrides[0].value,
            serde_json::Value::String("cannon.png".to_string())
        );
    }

    // S11 — Upsert replaces a same-key override
    #[test]
    fn test_upsert_override_replaces_same_key() {
        let mut inst = fixture_instance_with_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );
        let patch = component_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("enemy.png".to_string()),
            ComponentOverrideStatus::Active,
        );
        upsert_override(&mut inst, patch);
        assert_eq!(inst.component_overrides.len(), 1);
        assert_eq!(
            inst.component_overrides[0].value,
            serde_json::Value::String("enemy.png".to_string())
        );
        assert_eq!(
            inst.component_overrides[0].status,
            ComponentOverrideStatus::Active
        );
    }

    // S12 — Remove returns the captured patch
    #[test]
    fn test_remove_override_returns_captured() {
        let mut inst = fixture_instance_with_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );
        let result = remove_override(
            &mut inst,
            LocalId::new("root"),
            crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            vec!["asset".to_string()],
        );
        assert!(result.is_some());
        let removed = result.unwrap();
        assert_eq!(
            removed.value,
            serde_json::Value::String("cannon.png".to_string())
        );
        assert!(inst.component_overrides.is_empty());
    }

    // S13 — Remove of absent override returns None
    #[test]
    fn test_remove_override_absent_is_noop() {
        let mut inst = fixture_instance_with_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );
        let result = remove_override(
            &mut inst,
            LocalId::new("root"),
            crate::schema::ComponentTypeId::new("editor.Sprite2D"),
            vec!["nonexistent".to_string()], // different field_path
        );
        assert!(result.is_none());
        assert_eq!(inst.component_overrides.len(), 1); // unchanged
    }

    // field_override_index — covers component_overrides
    #[test]
    fn test_field_override_index_active_overrides() {
        let inst = fixture_instance_with_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Active,
        );
        let index = field_override_index(&inst);
        assert_eq!(index.len(), 1);
        assert_eq!(index[0].local_id, LocalId::new("root"));
        assert_eq!(index[0].component_type_id.as_str(), "editor.Sprite2D");
        assert_eq!(index[0].field_path, vec!["asset"]);
        assert_eq!(index[0].status, ComponentOverrideStatus::Active);
    }

    // field_override_index — covers orphaned_component_overrides
    #[test]
    fn test_field_override_index_includes_orphaned() {
        let mut inst = fixture_instance_with_override(
            "root",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("cannon.png".to_string()),
            ComponentOverrideStatus::Orphaned,
        );
        // Add a second orphaned override
        inst.orphaned_component_overrides.push(component_override(
            "other",
            "editor.Transform2D",
            vec!["translation", "x"],
            serde_json::json!(1.0),
            ComponentOverrideStatus::Orphaned,
        ));
        let index = field_override_index(&inst);
        // component_overrides has 1 (orphaned), orphaned_component_overrides has 1
        assert_eq!(index.len(), 2);
    }

    // field_override_index — ordering is preserved
    #[test]
    fn test_field_override_index_preserves_order() {
        let mut inst = fixture_instance();
        inst.component_overrides.push(component_override(
            "a",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("a.png".to_string()),
            ComponentOverrideStatus::Active,
        ));
        inst.component_overrides.push(component_override(
            "b",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("b.png".to_string()),
            ComponentOverrideStatus::Stale,
        ));
        inst.orphaned_component_overrides.push(component_override(
            "c",
            "editor.Sprite2D",
            vec!["asset"],
            serde_json::Value::String("c.png".to_string()),
            ComponentOverrideStatus::Orphaned,
        ));
        let index = field_override_index(&inst);
        assert_eq!(index.len(), 3);
        assert_eq!(index[0].local_id, LocalId::new("a"));
        assert_eq!(index[1].local_id, LocalId::new("b"));
        assert_eq!(index[2].local_id, LocalId::new("c"));
    }
}
