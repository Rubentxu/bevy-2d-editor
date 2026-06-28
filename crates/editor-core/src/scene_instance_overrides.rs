//! Scene Instance Override Resolution
//!
//! Pure-functions module for the override lifecycle contracted by
//! ADR-0005 §Overrides and §Versioning and Resync.

use std::collections::{BTreeMap, BTreeSet};

use crate::document::{ComponentInstance, StableId};
use crate::scene_asset::{LocalId, SceneAssetDocument, SceneAssetEntity};
use crate::scene_instance::{OverridePatch, OverrideStatus, SceneInstance};

// ---------------------------------------------------------------------------
// Public Types
// ---------------------------------------------------------------------------

/// Result of merging a `SceneAssetDocument` with an instance's overrides.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedScene {
    pub entities: BTreeMap<LocalId, ResolvedEntity>,
    pub id_map: BTreeMap<LocalId, StableId>,
    pub minted_stable_ids: BTreeSet<StableId>,
    pub unresolved: Vec<OverridePatch>,
}

/// One entity inside a resolved scene (projection of `SceneAssetEntity`).
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedEntity {
    pub local_id: LocalId,
    pub local_path: String,
    pub name: String,
    pub components: Vec<ComponentInstance>,
}

/// Summary of what happened during a `resync` call.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResyncReport {
    pub active: usize,
    pub orphaned: usize,
    pub stale: usize,
    pub conflict: usize,
    pub rebound: usize,
}

/// An issue found by `validate_overrides`.
#[derive(Debug, Clone, PartialEq)]
pub struct OverrideIssue {
    pub code: String,
    pub patch: OverridePatch,
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

/// Walk `field_path[1..]` inside `values` and insert `value` at the terminal key.
fn apply_field_path(
    component: &mut ComponentInstance,
    field_path: &[String],
    value: serde_json::Value,
) -> Result<(), ()> {
    if field_path.is_empty() {
        return Ok(());
    }
    if field_path.len() == 1 {
        component
            .values
            .as_object_mut()
            .ok_or(())?
            .insert(field_path[0].clone(), value);
        Ok(())
    } else {
        let mut current = &mut component.values;
        for seg in field_path.iter().take(field_path.len() - 1) {
            let next = current.as_object_mut().and_then(|m| m.get_mut(seg));
            match next {
                Some(v) => current = v,
                None => return Err(()),
            }
        }
        let last = field_path.last().ok_or(())?;
        current
            .as_object_mut()
            .ok_or(())?
            .insert(last.clone(), value);
        Ok(())
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

/// Build a map from `local_path` suffix → `LocalId` (scaffolded, unused in spike).
#[allow(dead_code)]
fn build_path_index(asset: &SceneAssetDocument) -> BTreeMap<String, LocalId> {
    let mut idx = BTreeMap::new();
    for e in &asset.entities {
        idx.insert(e.local_path.clone(), e.local_id.clone());
    }
    idx
}

/// Check if `orphan_path` could rebind to `candidate_path` via suffix match (scaffolded).
#[allow(dead_code)]
fn suffix_match(orphan_path: &str, candidate_path: &str) -> bool {
    orphan_path.ends_with(candidate_path) || candidate_path.ends_with(orphan_path)
}

// ---------------------------------------------------------------------------
// Public Functions
// ---------------------------------------------------------------------------

/// Pure re-classification of override patches against a given asset.
pub fn classify_overrides(
    asset: &SceneAssetDocument,
    patches: &[OverridePatch],
) -> Vec<OverridePatch> {
    patches
        .iter()
        .map(|patch| {
            let entity = match find_entity(asset, &patch.target_local_id) {
                Some(e) => e,
                None => {
                    return OverridePatch {
                        status: OverrideStatus::Orphaned,
                        ..patch.clone()
                    };
                }
            };

            let type_id = patch.field_path.first().map(|s| s.as_str()).unwrap_or("");
            let component = match find_component(entity, type_id) {
                Some(c) => c,
                None => {
                    return OverridePatch {
                        status: OverrideStatus::Orphaned,
                        ..patch.clone()
                    };
                }
            };

            // Walk field_path[1..] to check field existence
            if patch.field_path.len() > 1 {
                let segments: Vec<&str> = patch
                    .field_path
                    .iter()
                    .skip(1)
                    .map(|s| s.as_str())
                    .collect();
                let mut current: Option<&serde_json::Value> = Some(&component.values);
                let mut is_stale = false;

                for (i, seg) in segments.iter().enumerate() {
                    match current {
                        Some(serde_json::Value::Object(map)) => {
                            if i < segments.len() - 1 {
                                current = map.get(*seg);
                            } else {
                                if !map.contains_key(*seg) {
                                    is_stale = true;
                                }
                                break;
                            }
                        }
                        _ => {
                            is_stale = true;
                            break;
                        }
                    }
                }

                if is_stale {
                    return OverridePatch {
                        status: OverrideStatus::Stale,
                        ..patch.clone()
                    };
                }
            }

            // Kind mismatch check
            let existing_value: Option<serde_json::Value> = if patch.field_path.len() > 1 {
                let segments: Vec<&str> = patch
                    .field_path
                    .iter()
                    .skip(1)
                    .map(|s| s.as_str())
                    .collect();
                let mut current: Option<&serde_json::Value> = Some(&component.values);
                for (i, seg) in segments.iter().enumerate() {
                    match current {
                        Some(serde_json::Value::Object(map)) => {
                            if i < segments.len() - 1 {
                                current = map.get(*seg);
                            } else {
                                current = map.get(*seg);
                                break;
                            }
                        }
                        _ => {
                            current = None;
                            break;
                        }
                    }
                }
                current.cloned()
            } else {
                Some(component.values.clone())
            };

            if let Some(existing) = existing_value {
                if detect_kind_mismatch(&existing, &patch.value) {
                    return OverridePatch {
                        status: OverrideStatus::Conflict,
                        ..patch.clone()
                    };
                }
            }

            OverridePatch {
                status: OverrideStatus::Active,
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

/// Try to rebind an orphaned patch by exact `target_local_id` match (spike).
pub fn try_rebind(asset: &SceneAssetDocument, orphaned: &OverridePatch) -> Option<LocalId> {
    find_entity(asset, &orphaned.target_local_id).map(|e| e.local_id.clone())
}

/// Read-only issue scan for a `SceneInstance` against its asset.
pub fn validate_overrides(
    asset: &SceneAssetDocument,
    instance: &SceneInstance,
) -> Vec<OverrideIssue> {
    let mut issues = Vec::new();

    for patch in instance
        .overrides
        .iter()
        .chain(instance.orphaned_overrides.iter())
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

        let type_id = patch.field_path.first().map(|s| s.as_str()).unwrap_or("");
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
        let field_key = patch.field_path.get(1).map(|s| s.as_str()).unwrap_or("");
        let dupe_count = instance
            .overrides
            .iter()
            .filter(|p| {
                p.target_local_id == patch.target_local_id
                    && p.field_path.get(1) == patch.field_path.get(1)
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
        if patch.field_path.len() > 1 {
            let segments: Vec<&str> = patch
                .field_path
                .iter()
                .skip(1)
                .map(|s| s.as_str())
                .collect();
            let mut current: Option<&serde_json::Value> = Some(&component.values);
            let mut found = true;

            for (i, seg) in segments.iter().enumerate() {
                match current {
                    Some(serde_json::Value::Object(map)) => {
                        if i < segments.len() - 1 {
                            current = map.get(*seg);
                        } else if !map.contains_key(*seg) {
                            found = false;
                            break;
                        }
                    }
                    _ => {
                        found = false;
                        break;
                    }
                }
            }

            if !found {
                issues.push(OverrideIssue {
                    code: "missing_field".to_string(),
                    patch: patch.clone(),
                    message: format!(
                        "Field path {:?} does not resolve in component '{}'",
                        &patch.field_path[1..],
                        type_id
                    ),
                });
            }
        }

        // Type conflict check
        let terminal: Option<serde_json::Value> = if patch.field_path.len() > 1 {
            let segments: Vec<&str> = patch
                .field_path
                .iter()
                .skip(1)
                .map(|s| s.as_str())
                .collect();
            let mut current: Option<&serde_json::Value> = Some(&component.values);
            for (i, seg) in segments.iter().enumerate() {
                match current {
                    Some(serde_json::Value::Object(map)) => {
                        if i < segments.len() - 1 {
                            current = map.get(*seg);
                        } else {
                            current = map.get(*seg);
                            break;
                        }
                    }
                    _ => {
                        current = None;
                        break;
                    }
                }
            }
            current.cloned()
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

    for patch in &instance.overrides {
        if patch.status == OverrideStatus::Orphaned {
            continue;
        }

        let resolved_entity = match resolved_entities.get_mut(&patch.target_local_id) {
            Some(e) => e,
            None => {
                unresolved.push(patch.clone());
                continue;
            }
        };

        let type_id = patch.field_path.first().map(|s| s.as_str()).unwrap_or("");
        let component = match resolved_entity
            .components
            .iter_mut()
            .find(|c| c.type_id == type_id)
        {
            Some(c) => c,
            None => {
                unresolved.push(patch.clone());
                continue;
            }
        };

        // Walk field_path[1..] to verify path
        if patch.field_path.len() > 1 {
            let segments: Vec<&str> = patch
                .field_path
                .iter()
                .skip(1)
                .map(|s| s.as_str())
                .collect();
            let mut current: Option<&serde_json::Value> = Some(&component.values);
            let mut valid_path = true;

            for (i, seg) in segments.iter().enumerate() {
                match current {
                    Some(serde_json::Value::Object(map)) => {
                        if i < segments.len() - 1 {
                            current = map.get(*seg);
                        } else if !map.contains_key(*seg) {
                            valid_path = false;
                            break;
                        }
                    }
                    _ => {
                        valid_path = false;
                        break;
                    }
                }
            }

            if !valid_path {
                unresolved.push(patch.clone());
                continue;
            }
        }

        // Kind check
        let terminal: Option<serde_json::Value> = if patch.field_path.len() > 1 {
            let segments: Vec<&str> = patch
                .field_path
                .iter()
                .skip(1)
                .map(|s| s.as_str())
                .collect();
            let mut current: Option<&serde_json::Value> = Some(&component.values);
            for (i, seg) in segments.iter().enumerate() {
                match current {
                    Some(serde_json::Value::Object(map)) => {
                        if i < segments.len() - 1 {
                            current = map.get(*seg);
                        } else {
                            current = map.get(*seg);
                            break;
                        }
                    }
                    _ => {
                        current = None;
                        break;
                    }
                }
            }
            current.cloned()
        } else {
            Some(component.values.clone())
        };

        if let Some(existing) = terminal {
            if detect_kind_mismatch(&existing, &patch.value) {
                unresolved.push(patch.clone());
                continue;
            }
        }

        // Apply patch
        let mut comp = component.clone();
        if apply_field_path(&mut comp, &patch.field_path[1..], patch.value.clone()).is_ok() {
            *component = comp;
        } else {
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

/// Re-validate overrides when the asset version has changed.
pub fn resync(
    asset: &SceneAssetDocument,
    instance: &mut SceneInstance,
    new_asset_version: u32,
) -> ResyncReport {
    instance.asset_version_seen = new_asset_version;

    let mut report = ResyncReport::default();
    let mut new_overrides: Vec<OverridePatch> = Vec::new();
    let mut new_orphaned: Vec<OverridePatch> = Vec::new();

    let classified = classify_overrides(asset, &instance.overrides);

    for (patch, classified_patch) in instance.overrides.iter().zip(classified.iter()) {
        match (patch.status, classified_patch.status) {
            (OverrideStatus::Active, OverrideStatus::Orphaned) => {
                let mut p = classified_patch.clone();
                p.status = OverrideStatus::Orphaned;
                new_orphaned.push(p);
                report.orphaned += 1;
            }
            (OverrideStatus::Active, OverrideStatus::Stale) => {
                let mut p = classified_patch.clone();
                p.status = OverrideStatus::Stale;
                new_overrides.push(p);
                report.stale += 1;
            }
            (OverrideStatus::Active, OverrideStatus::Conflict) => {
                let mut p = classified_patch.clone();
                p.status = OverrideStatus::Conflict;
                new_overrides.push(p);
                report.conflict += 1;
            }
            (OverrideStatus::Active, OverrideStatus::Active) => {
                new_overrides.push(classified_patch.clone());
                report.active += 1;
            }
            (_, OverrideStatus::Orphaned) => {
                let mut p = classified_patch.clone();
                p.status = OverrideStatus::Orphaned;
                new_orphaned.push(p);
                report.orphaned += 1;
            }
            (_, OverrideStatus::Stale) => {
                new_overrides.push(classified_patch.clone());
                report.stale += 1;
            }
            (_, OverrideStatus::Conflict) => {
                new_overrides.push(classified_patch.clone());
                report.conflict += 1;
            }
            (_, OverrideStatus::Active) => {
                new_overrides.push(classified_patch.clone());
                report.active += 1;
            }
        }
    }

    let mut still_orphaned: Vec<OverridePatch> = Vec::new();
    for orphan in instance.orphaned_overrides.iter() {
        if let Some(new_id) = try_rebind(asset, orphan) {
            let mut rebound_patch = orphan.clone();
            rebound_patch.target_local_id = new_id;
            rebound_patch.status = OverrideStatus::Active;
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

    instance.overrides = new_overrides;
    instance.orphaned_overrides = new_orphaned;
    instance.orphaned_overrides.extend(still_orphaned);

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

    #[test]
    fn test_classify_overrides_active() {
        let asset = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("root"),
                local_path: "root".to_string(),
                name: "Root".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Sprite2D".to_string(),
                    values: serde_json::json!({"asset": "player.png"}),
                }],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let patch = OverridePatch {
            target_local_id: LocalId::new("root"),
            field_path: vec!["editor.Sprite2D".to_string(), "asset".to_string()],
            value: serde_json::Value::String("cannon.png".to_string()),
            status: OverrideStatus::Active,
        };

        let result = classify_overrides(&asset, std::slice::from_ref(&patch));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, OverrideStatus::Active);
    }

    #[test]
    fn test_classify_overrides_orphaned_short_form() {
        let asset = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("root"),
                local_path: "root".to_string(),
                name: "Root".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Sprite2D".to_string(),
                    values: serde_json::json!({"asset": "player.png"}),
                }],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let patch = OverridePatch {
            target_local_id: LocalId::new("root"),
            field_path: vec!["Sprite2D".to_string(), "asset".to_string()],
            value: serde_json::Value::String("cannon.png".to_string()),
            status: OverrideStatus::Active,
        };

        let result = classify_overrides(&asset, std::slice::from_ref(&patch));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, OverrideStatus::Orphaned);
    }

    #[test]
    fn test_mint_id_map() {
        let asset = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![
                SceneAssetEntity {
                    local_id: LocalId::new("a"),
                    local_path: "a".to_string(),
                    name: "A".to_string(),
                    components: vec![],
                },
                SceneAssetEntity {
                    local_id: LocalId::new("b"),
                    local_path: "b".to_string(),
                    name: "B".to_string(),
                    components: vec![],
                },
            ],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

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
        let asset = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![
                SceneAssetEntity {
                    local_id: LocalId::new("a"),
                    local_path: "a".to_string(),
                    name: "A".to_string(),
                    components: vec![],
                },
                SceneAssetEntity {
                    local_id: LocalId::new("b"),
                    local_path: "b".to_string(),
                    name: "B".to_string(),
                    components: vec![],
                },
            ],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

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
        let asset = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("new_abc"),
                local_path: "root/player/weapons/cannon".to_string(),
                name: "Cannon".to_string(),
                components: vec![],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let orphan = OverridePatch {
            target_local_id: LocalId::new("old_abc"),
            field_path: vec!["editor.Sprite2D".to_string(), "asset".to_string()],
            value: serde_json::Value::String("cannon.png".to_string()),
            status: OverrideStatus::Orphaned,
        };

        let result = try_rebind(&asset, &orphan);
        assert_eq!(result, Some(LocalId::new("new_abc")));
    }

    #[test]
    fn test_resync_preserves_active_on_rename() {
        let asset = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 2,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("abc"),
                local_path: "abc".to_string(),
                name: "Cannon".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Sprite2D".to_string(),
                    values: serde_json::json!({"asset": "player.png"}),
                }],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let mut instance = SceneInstance {
            instance_id: StableId::new("inst_1"),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: vec![(LocalId::new("abc"), StableId::new("ent_a"))]
                .into_iter()
                .collect(),
            overrides: vec![OverridePatch {
                target_local_id: LocalId::new("abc"),
                field_path: vec!["editor.Sprite2D".to_string(), "asset".to_string()],
                value: serde_json::Value::String("cannon.png".to_string()),
                status: OverrideStatus::Active,
            }],
            orphaned_overrides: vec![],
        };

        let report = resync(&asset, &mut instance, 2);

        assert_eq!(report.active, 1);
        assert_eq!(report.orphaned, 0);
        assert_eq!(report.stale, 0);
        assert_eq!(report.conflict, 0);
        assert_eq!(report.rebound, 0);
        assert_eq!(instance.asset_version_seen, 2);
        assert_eq!(instance.overrides.len(), 1);
        assert_eq!(instance.overrides[0].status, OverrideStatus::Active);
    }

    #[test]
    fn test_resync_moves_to_orphaned_on_entity_removed() {
        let asset_v2 = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 2,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let mut instance = SceneInstance {
            instance_id: StableId::new("inst_1"),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: Default::default(),
            overrides: vec![OverridePatch {
                target_local_id: LocalId::new("abc"),
                field_path: vec!["editor.Sprite2D".to_string(), "asset".to_string()],
                value: serde_json::Value::String("cannon.png".to_string()),
                status: OverrideStatus::Active,
            }],
            orphaned_overrides: vec![],
        };

        let report = resync(&asset_v2, &mut instance, 2);

        assert_eq!(report.active, 0);
        assert_eq!(report.orphaned, 1);
        assert_eq!(instance.overrides.len(), 0);
        assert_eq!(instance.orphaned_overrides.len(), 1);
        assert_eq!(
            instance.orphaned_overrides[0].status,
            OverrideStatus::Orphaned
        );
        assert_eq!(
            instance.orphaned_overrides[0].target_local_id,
            LocalId::new("abc")
        );
    }

    #[test]
    fn test_resync_marks_stale_on_field_rename() {
        let asset_v2 = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 2,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("root"),
                local_path: "root".to_string(),
                name: "Root".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Sprite2D".to_string(),
                    values: serde_json::json!({"image": "player.png"}),
                }],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let mut instance = SceneInstance {
            instance_id: StableId::new("inst_1"),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: Default::default(),
            overrides: vec![OverridePatch {
                target_local_id: LocalId::new("root"),
                field_path: vec!["editor.Sprite2D".to_string(), "asset".to_string()],
                value: serde_json::Value::String("cannon.png".to_string()),
                status: OverrideStatus::Active,
            }],
            orphaned_overrides: vec![],
        };

        let report = resync(&asset_v2, &mut instance, 2);

        assert_eq!(report.stale, 1);
        assert_eq!(report.active, 0);
        assert_eq!(instance.overrides.len(), 1);
        assert_eq!(instance.overrides[0].status, OverrideStatus::Stale);
    }

    #[test]
    fn test_resync_marks_conflict_on_type_change() {
        let asset_v2 = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 2,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("player"),
                local_path: "player".to_string(),
                name: "Player".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Health".to_string(),
                    values: serde_json::json!({"current": "full"}),
                }],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let mut instance = SceneInstance {
            instance_id: StableId::new("inst_1"),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: Default::default(),
            overrides: vec![OverridePatch {
                target_local_id: LocalId::new("player"),
                field_path: vec!["editor.Health".to_string(), "current".to_string()],
                value: serde_json::json!(42),
                status: OverrideStatus::Active,
            }],
            orphaned_overrides: vec![],
        };

        let report = resync(&asset_v2, &mut instance, 2);

        assert_eq!(report.conflict, 1);
        assert_eq!(instance.overrides.len(), 1);
        assert_eq!(instance.overrides[0].status, OverrideStatus::Conflict);
    }

    #[test]
    fn test_resync_rebinds_via_local_id() {
        let asset_v2 = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 2,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let mut instance = SceneInstance {
            instance_id: StableId::new("inst_1"),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: vec![(LocalId::new("abc"), StableId::new("ent_a"))]
                .into_iter()
                .collect(),
            overrides: vec![OverridePatch {
                target_local_id: LocalId::new("abc"),
                field_path: vec!["editor.Sprite2D".to_string(), "asset".to_string()],
                value: serde_json::Value::String("cannon.png".to_string()),
                status: OverrideStatus::Active,
            }],
            orphaned_overrides: vec![],
        };

        resync(&asset_v2, &mut instance, 2);
        assert_eq!(instance.orphaned_overrides.len(), 1);

        let asset_v3 = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 3,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("abc"),
                local_path: "weapons/cannon".to_string(),
                name: "Cannon".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Sprite2D".to_string(),
                    values: serde_json::json!({"asset": "player.png"}),
                }],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let report = resync(&asset_v3, &mut instance, 3);

        assert_eq!(report.rebound, 1);
        assert_eq!(instance.overrides.len(), 1);
        assert_eq!(instance.overrides[0].status, OverrideStatus::Active);
        assert_eq!(instance.overrides[0].target_local_id, LocalId::new("abc"));
    }

    #[test]
    fn test_effective_values_minimal() {
        let asset = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("root"),
                local_path: "root".to_string(),
                name: "Root".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Sprite2D".to_string(),
                    values: serde_json::json!({"asset": "player.png"}),
                }],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let instance = SceneInstance {
            instance_id: StableId::new("inst_1"),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: Default::default(),
            overrides: vec![OverridePatch {
                target_local_id: LocalId::new("root"),
                field_path: vec!["editor.Sprite2D".to_string(), "asset".to_string()],
                value: serde_json::Value::String("cannon.png".to_string()),
                status: OverrideStatus::Active,
            }],
            orphaned_overrides: vec![],
        };

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
        let asset = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![
                SceneAssetEntity {
                    local_id: LocalId::new("a"),
                    local_path: "a".to_string(),
                    name: "A".to_string(),
                    components: vec![ComponentInstance {
                        type_id: "editor.Sprite2D".to_string(),
                        values: serde_json::json!({"asset": "a.png"}),
                    }],
                },
                SceneAssetEntity {
                    local_id: LocalId::new("b"),
                    local_path: "b".to_string(),
                    name: "B".to_string(),
                    components: vec![ComponentInstance {
                        type_id: "editor.Sprite2D".to_string(),
                        values: serde_json::json!({"asset": "b.png"}),
                    }],
                },
            ],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let instance = SceneInstance {
            instance_id: StableId::new("inst_1"),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: Default::default(),
            overrides: vec![],
            orphaned_overrides: vec![],
        };

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
        let asset = SceneAssetDocument {
            asset_id: "asset_1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("root"),
                local_path: "root".to_string(),
                name: "Root".to_string(),
                components: vec![],
            }],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
        };

        let instance = SceneInstance {
            instance_id: StableId::new("inst_1"),
            asset_ref: crate::scene_asset::AssetReference::new("assets/test"),
            asset_version_seen: 1,
            id_map: Default::default(),
            overrides: vec![OverridePatch {
                target_local_id: LocalId::new("nonexistent"),
                field_path: vec!["editor.Sprite2D".to_string(), "asset".to_string()],
                value: serde_json::Value::String("cannon.png".to_string()),
                status: OverrideStatus::Active,
            }],
            orphaned_overrides: vec![],
        };

        let issues = validate_overrides(&asset, &instance);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].code, "missing_entity");
    }
}
