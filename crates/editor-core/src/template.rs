//! Entity Templates for the Bevy 2D Editor.
//!
//! Implements Hito 0 §6.7: reusable editor-owned templates that can
//! instantiate trees of entities with fresh global StableIds. Templates
//! use local IDs internally; on instantiation, the editor mints fresh
//! StableIds in the Scene.
//!
//! Architecture:
//! - `EntityTemplate` + `TemplateEntity` types (flat Vec with parent references)
//! - `validate()` rejects cycles, multi-root, dangling refs, unknown schemas
//! - `instantiate()` walks the tree, mints fresh IDs, builds SceneDocument entities
//! - In-memory cache (`TEMPLATE_CACHE`) avoids repeated OPFS reads
//! - Counter-based ID minting for uniqueness

use crate::document::{ComponentInstance, Entity, SceneDocument, StableId};
use crate::schema;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// A reusable entity template that can instantiate a tree of entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityTemplate {
    pub template_id: String,
    pub display_name: String,
    pub version: String,
    pub entities: Vec<TemplateEntity>,
}

/// A single entity inside a template. Uses `local_id` for parent references
/// within the template. `local_id` is NOT a StableId — it's a template-local
/// reference that never appears in SceneDocuments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateEntity {
    /// Template-local ID (e.g., "root", "child_1"). References within the
    /// template use this. Never appears in SceneDocument as StableId.
    pub local_id: String,
    pub name: String,
    /// `None` for root entity. `Some(local_id)` for child entity referencing
    /// another entity's local_id within the same template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_local_id: Option<String>,
    pub components: Vec<ComponentInstance>,
}

/// Errors returned by template operations.
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Template must have exactly one root, found {0}")]
    MultipleRoots(usize),

    #[error("Template has no entities")]
    EmptyTemplate,

    #[error("Template contains a cycle through '{0}'")]
    Cycle(String),

    #[error("Parent local_id '{0}' not found in template")]
    DanglingParent(String),

    #[error("Unknown component schema: {0}")]
    UnknownSchema(String),

    #[error("Template not loaded: {0}")]
    NotLoaded(String),

    #[error("JSON parse error: {0}")]
    Parse(String),
}

/// Validate a template: cycle-free, exactly one root, valid parent references,
/// known component schemas.
pub fn validate(template: &EntityTemplate) -> Result<(), TemplateError> {
    if template.entities.is_empty() {
        return Err(TemplateError::EmptyTemplate);
    }

    // Build local_id → index map
    let map: HashMap<&str, usize> = template
        .entities
        .iter()
        .enumerate()
        .map(|(i, e)| (e.local_id.as_str(), i))
        .collect();

    // Count roots (entities with no parent)
    let root_count = template
        .entities
        .iter()
        .filter(|e| e.parent_local_id.is_none())
        .count();

    if root_count != 1 {
        return Err(TemplateError::MultipleRoots(root_count));
    }

    // Dangling parent check
    for entity in &template.entities {
        if let Some(parent) = &entity.parent_local_id {
            if !map.contains_key(parent.as_str()) {
                return Err(TemplateError::DanglingParent(parent.clone()));
            }
        }
    }

    // Cycle detection: walk parent chain from each entity, ensure no back-edge
    for entity in &template.entities {
        let mut visited: HashSet<String> = HashSet::new();
        let mut current_local = entity.local_id.clone();
        loop {
            visited.insert(current_local.clone());
            let parent = map
                .get(current_local.as_str())
                .and_then(|i| template.entities.get(*i))
                .and_then(|e| e.parent_local_id.clone());
            match parent {
                None => break,
                Some(p) => {
                    if visited.contains(&p) {
                        return Err(TemplateError::Cycle(p));
                    }
                    current_local = p;
                }
            }
        }
    }

    // Component schema validation
    let registry = schema::combined_registry();
    for entity in &template.entities {
        for component in &entity.components {
            if registry.get(&component.type_id).is_none() {
                return Err(TemplateError::UnknownSchema(component.type_id.clone()));
            }
        }
    }

    Ok(())
}

/// Counter-based ID minter: `ent_<timestamp_ms>_<counter>`.
/// Provides uniqueness across rapid instantiations without external deps.
thread_local! {
    static ID_COUNTER: Cell<u64> = const { Cell::new(0) };
}

pub fn mint_stable_id() -> StableId {
    let counter = ID_COUNTER.with(|c| {
        let n = c.get() + 1;
        c.set(n);
        n
    });
    // In WASM, std::time::SystemTime may panic due to no system clock.
    // Use a fallback: just the counter (sufficient for unique IDs within session).
    #[cfg(not(target_arch = "wasm32"))]
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    #[cfg(target_arch = "wasm32")]
    let suffix = 0u128;
    StableId::new(format!("ent_{:x}_{:x}", suffix, counter))
}

// ─────────────────────────────────────────────────────────────────────────────
// In-memory template cache
// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    static TEMPLATE_CACHE: RefCell<HashMap<String, EntityTemplate>> = RefCell::new(HashMap::new());
}

pub fn cache_template(template: EntityTemplate) {
    TEMPLATE_CACHE.with(|c| {
        c.borrow_mut()
            .insert(template.template_id.clone(), template);
    });
}

pub fn get_cached_template(template_id: &str) -> Option<EntityTemplate> {
    TEMPLATE_CACHE.with(|c| c.borrow().get(template_id).cloned())
}

pub fn remove_cached_template(template_id: &str) -> Option<EntityTemplate> {
    TEMPLATE_CACHE.with(|c| c.borrow_mut().remove(template_id))
}

pub fn clear_template_cache() {
    TEMPLATE_CACHE.with(|c| c.borrow_mut().clear());
}

/// Instantiate a template into the scene. Mints fresh StableIds for each
/// template entity, builds Entity values, sets parent references, applies
/// target_parent to the root entity if specified. Returns the minted IDs.
pub fn instantiate(
    template: &EntityTemplate,
    target_parent: Option<&StableId>,
    doc: &mut SceneDocument,
) -> Result<Vec<StableId>, TemplateError> {
    // Mint fresh IDs
    let local_to_minted: HashMap<String, StableId> = template
        .entities
        .iter()
        .map(|e| (e.local_id.clone(), mint_stable_id()))
        .collect();

    // Build all entities with parent = None initially
    let mut minted_entities: Vec<Entity> = template
        .entities
        .iter()
        .map(|te| {
            let id = local_to_minted.get(&te.local_id).unwrap().clone();
            Entity {
                id,
                name: te.name.clone(),
                parent: None,
                components: te.components.clone(),
            }
        })
        .collect();

    // Set parent references
    for (i, te) in template.entities.iter().enumerate() {
        if let Some(parent_local) = &te.parent_local_id {
            if let Some(parent_id) = local_to_minted.get(parent_local) {
                minted_entities[i].parent = Some(parent_id.clone());
            }
        }
    }

    // Apply target_parent to root (the entity with no parent)
    if let Some(target) = target_parent {
        for entity in minted_entities.iter_mut() {
            if entity.parent.is_none() {
                entity.parent = Some(target.clone());
                break;
            }
        }
    }

    let minted_ids: Vec<StableId> = minted_entities.iter().map(|e| e.id.clone()).collect();
    doc.entities.extend(minted_entities);
    Ok(minted_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{ComponentInstance, Entity};
    use serde_json::json;

    fn empty_doc() -> SceneDocument {
        SceneDocument {
            version: "0.1".to_string(),
            scene_id: "test".to_string(),
            name: "Test".to_string(),
            entities: vec![],
        }
    }

    fn template_entity(local_id: &str, parent: Option<&str>, type_id: &str) -> TemplateEntity {
        TemplateEntity {
            local_id: local_id.to_string(),
            name: local_id.to_string(),
            parent_local_id: parent.map(String::from),
            components: vec![ComponentInstance {
                type_id: type_id.to_string(),
                values: json!({"translation": {"x": 0.0, "y": 0.0}}),
            }],
        }
    }

    fn simple_template_single_root() -> EntityTemplate {
        EntityTemplate {
            template_id: "simple".to_string(),
            display_name: "Simple".to_string(),
            version: "0.1".to_string(),
            entities: vec![template_entity("root", None, "editor.Transform2D")],
        }
    }

    fn simple_template_tree() -> EntityTemplate {
        EntityTemplate {
            template_id: "tree".to_string(),
            display_name: "Tree".to_string(),
            version: "0.1".to_string(),
            entities: vec![
                template_entity("root", None, "editor.Transform2D"),
                template_entity("child1", Some("root"), "editor.Transform2D"),
                template_entity("child2", Some("root"), "editor.Transform2D"),
            ],
        }
    }

    // ===== Serialization =====

    #[test]
    fn test_entity_template_single_root_serialization() {
        let t = simple_template_single_root();
        let json = serde_json::to_string(&t).unwrap();
        let rt: EntityTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(t, rt);
    }

    #[test]
    fn test_entity_template_tree_serialization() {
        let t = simple_template_tree();
        let json = serde_json::to_string(&t).unwrap();
        let rt: EntityTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(t, rt);
    }

    // ===== Validation =====

    #[test]
    fn test_validate_empty_template_fails() {
        let t = EntityTemplate {
            template_id: "empty".to_string(),
            display_name: "Empty".to_string(),
            version: "0.1".to_string(),
            entities: vec![],
        };
        assert!(matches!(validate(&t), Err(TemplateError::EmptyTemplate)));
    }

    #[test]
    fn test_validate_multiple_roots_fails() {
        let t = EntityTemplate {
            template_id: "multi".to_string(),
            display_name: "Multi".to_string(),
            version: "0.1".to_string(),
            entities: vec![
                template_entity("a", None, "editor.Transform2D"),
                template_entity("b", None, "editor.Transform2D"),
            ],
        };
        assert!(matches!(validate(&t), Err(TemplateError::MultipleRoots(2))));
    }

    #[test]
    fn test_validate_dangling_parent_fails() {
        // Need a root + child with dangling parent ref
        let t = EntityTemplate {
            template_id: "dangling".to_string(),
            display_name: "Dangling".to_string(),
            version: "0.1".to_string(),
            entities: vec![
                template_entity("root", None, "editor.Transform2D"),
                template_entity("child", Some("nonexistent"), "editor.Transform2D"),
            ],
        };
        assert!(matches!(
            validate(&t),
            Err(TemplateError::DanglingParent(_))
        ));
    }

    #[test]
    fn test_validate_cycle_detected() {
        // 3-entity cycle where each points to next (no root exists)
        // Validation rejects this first as MultipleRoots, then Cycle
        let t = EntityTemplate {
            template_id: "cycle".to_string(),
            display_name: "Cycle".to_string(),
            version: "0.1".to_string(),
            entities: vec![
                template_entity("a", Some("c"), "editor.Transform2D"),
                template_entity("b", Some("a"), "editor.Transform2D"),
                template_entity("c", Some("b"), "editor.Transform2D"),
            ],
        };
        // Either MultipleRoots or Cycle should be the error
        let result = validate(&t);
        assert!(matches!(
            result,
            Err(TemplateError::MultipleRoots(_)) | Err(TemplateError::Cycle(_))
        ));
    }

    #[test]
    fn test_validate_unknown_schema_fails() {
        let t = EntityTemplate {
            template_id: "unknown".to_string(),
            display_name: "Unknown".to_string(),
            version: "0.1".to_string(),
            entities: vec![template_entity("root", None, "game.UnknownSchema")],
        };
        assert!(matches!(
            validate(&t),
            Err(TemplateError::UnknownSchema(_))
        ));
    }

    #[test]
    fn test_validate_valid_template_succeeds() {
        let t = simple_template_tree();
        assert!(validate(&t).is_ok());
    }

    // ===== Instantiation =====

    #[test]
    fn test_instantiate_single_root() {
        let mut doc = empty_doc();
        let template = simple_template_single_root();
        let minted = instantiate(&template, None, &mut doc).unwrap();
        assert_eq!(minted.len(), 1);
        assert_eq!(doc.entities.len(), 1);
        assert!(doc.entities[0].parent.is_none());
        // ID is fresh, not in template
        let minted_id = minted[0].as_str();
        assert!(minted_id.starts_with("ent_"));
        assert_ne!(minted_id, "root");
    }

    #[test]
    fn test_instantiate_tree() {
        let mut doc = empty_doc();
        let template = simple_template_tree();
        let minted = instantiate(&template, None, &mut doc).unwrap();
        assert_eq!(minted.len(), 3);
        assert_eq!(doc.entities.len(), 3);

        // Find root in minted_entities
        let root_id = doc
            .entities
            .iter()
            .find(|e| e.parent.is_none())
            .unwrap()
            .id
            .as_str()
            .to_string();
        // Two entities should have root as parent
        let children_count = doc
            .entities
            .iter()
            .filter(|e| e.parent.as_ref().map(|p| p.as_str().to_string()) == Some(root_id.clone()))
            .count();
        assert_eq!(children_count, 2);
    }

    #[test]
    fn test_instantiate_with_target_parent() {
        let mut doc = empty_doc();
        // Add a pre-existing entity
        doc.entities.push(Entity {
            id: StableId::new("existing_entity"),
            name: "Existing".to_string(),
            parent: None,
            components: vec![],
        });
        let template = simple_template_single_root();
        let target = StableId::new("existing_entity");
        let minted = instantiate(&template, Some(&target), &mut doc).unwrap();
        // The minted entity should have parent = existing_entity
        let minted_entity = doc
            .entities
            .iter()
            .find(|e| e.id == minted[0])
            .unwrap();
        assert_eq!(minted_entity.parent.as_ref().unwrap().as_str(), "existing_entity");
    }

    #[test]
    fn test_instantiate_twice_different_ids() {
        let mut doc = empty_doc();
        let template = simple_template_single_root();
        let minted1 = instantiate(&template, None, &mut doc).unwrap();
        let minted2 = instantiate(&template, None, &mut doc).unwrap();
        assert_ne!(minted1[0], minted2[0]);
        assert_eq!(doc.entities.len(), 2);
    }

    // ===== ID Minter =====

    #[test]
    fn test_mint_stable_id_unique() {
        let id1 = mint_stable_id();
        let id2 = mint_stable_id();
        let id3 = mint_stable_id();
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_mint_stable_id_format() {
        let id = mint_stable_id();
        let s = id.as_str();
        assert!(s.starts_with("ent_"));
        assert!(s.contains('_'));
    }

    // ===== Cache =====

    #[test]
    fn test_template_cache_basic() {
        clear_template_cache();
        assert!(get_cached_template("nonexistent").is_none());
        cache_template(simple_template_single_root());
        assert!(get_cached_template("simple").is_some());
        let removed = remove_cached_template("simple");
        assert!(removed.is_some());
        assert!(get_cached_template("simple").is_none());
    }
}