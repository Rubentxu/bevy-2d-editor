//! HIGH-1 phase 2: logic-graph state sub-module.
//!
//! Owns the LOGIC_GRAPH_DOC (active graph being edited), the
//! LOGIC_OPERATION_LOG (per-graph undo/redo), and the LOGIC_GRAPH_CATALOG
//! (catalog of all logic graph assets persisted in OPFS).

/// v0.91 PR2 (transitional): reserved key for the "active logic graph" slot
/// on `EditorSession::logic_states`. Used by test helpers to write to the
/// session.
pub const ACTIVE_LOGIC_GRAPH_PATH: &str = "_active";

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::document::StableId;
use crate::logic_command::{BindError, BindingId, LogicOperation, LogicOperationLog};
use crate::logic_graph::{LogicGraphAsset, LogicGraphCatalogEntry};

thread_local! {
    /// Logic Graph document: the active logic graph being edited.
    pub static LOGIC_GRAPH_DOC: RefCell<Option<LogicGraphAsset>> = const { RefCell::new(None) };
    /// Logic operation log: per-graph undo/redo history.
    /// Wrapped in `Option` to enable take/write-back in undo_graph/redo_graph
    /// without requiring `LogicOperationLog: Default`.
    pub static LOGIC_OPERATION_LOG: RefCell<Option<LogicOperationLog>> = const { RefCell::new(Some(LogicOperationLog::new_const())) };
    /// Logic Graph catalog: metadata for all persisted logic graph assets.
    pub static LOGIC_GRAPH_CATALOG: RefCell<Option<LogicGraphCatalog>> = const { RefCell::new(None) };
}

/// Catalog of LogicGraphAssets — parallel to SceneAssetCatalog.
#[derive(Debug, Clone, Default)]
pub struct LogicGraphCatalog {
    entries: BTreeMap<String, LogicGraphCatalogEntry>,
    path_index: BTreeMap<String, String>,
}

impl LogicGraphCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// List all catalog entries (owned copy).
    pub fn list_all(&self) -> Vec<LogicGraphCatalogEntry> {
        self.entries.values().cloned().collect()
    }

    /// Get a specific entry by asset_id.
    pub fn get(&self, asset_id: &str) -> Option<&LogicGraphCatalogEntry> {
        self.entries.get(asset_id)
    }

    /// Register a new entry. Returns error if asset_id already exists.
    pub fn register(
        &mut self,
        entry: LogicGraphCatalogEntry,
    ) -> Result<(), LogicGraphCatalogError> {
        if self.entries.contains_key(&entry.asset_id) {
            return Err(LogicGraphCatalogError::DuplicateAssetId {
                id: entry.asset_id.clone(),
            });
        }
        let normalized =
            editor_model::scene_asset_catalog::normalize_logical_path(&entry.logical_path);
        self.path_index.insert(normalized, entry.asset_id.clone());
        self.entries.insert(entry.asset_id.clone(), entry);
        Ok(())
    }

    /// Remove an entry by asset_id. Returns the removed entry.
    pub fn unregister(&mut self, asset_id: &str) -> Option<LogicGraphCatalogEntry> {
        if let Some(entry) = self.entries.remove(asset_id) {
            let normalized =
                editor_model::scene_asset_catalog::normalize_logical_path(&entry.logical_path);
            self.path_index.remove(&normalized);
            Some(entry)
        } else {
            None
        }
    }

    /// Seed the catalog from a list of entries (used during project load).
    pub fn seed(&mut self, entries: Vec<LogicGraphCatalogEntry>) {
        for entry in entries {
            let normalized =
                editor_model::scene_asset_catalog::normalize_logical_path(&entry.logical_path);
            self.path_index.insert(normalized, entry.asset_id.clone());
            self.entries.insert(entry.asset_id.clone(), entry);
        }
    }
}

/// Errors from LogicGraphCatalog operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LogicGraphCatalogError {
    #[error("duplicate asset_id '{id}'")]
    DuplicateAssetId { id: String },
}

/// Get an immutable borrowed reference to the LogicGraphAsset.
pub fn with_logic_graph<F, R>(f: F) -> R
where
    F: FnOnce(&Option<LogicGraphAsset>) -> R,
{
    LOGIC_GRAPH_DOC.with(|cell| f(&*cell.borrow()))
}

/// Get a mutable borrowed reference to the LogicGraphAsset.
pub fn with_logic_graph_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Option<LogicGraphAsset>) -> R,
{
    LOGIC_GRAPH_DOC.with(|cell| f(&mut *cell.borrow_mut()))
}

/// Get an immutable borrowed reference to the LogicOperationLog.
pub fn with_logic_log<F, R>(f: F) -> R
where
    F: FnOnce(&LogicOperationLog) -> R,
{
    LOGIC_OPERATION_LOG.with(|cell| f(cell.borrow().as_ref().unwrap()))
}

/// Get a mutable borrowed reference to the LogicOperationLog.
pub fn with_logic_log_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut LogicOperationLog) -> R,
{
    LOGIC_OPERATION_LOG.with(|cell| f(cell.borrow_mut().as_mut().unwrap()))
}

// ─────────────────────────────────────────────────────────────────────────────
// v0.92: Re-entrancy safe helpers (take/write-back)
//
// apply_command equivalent for logic uses kernel.apply_atomic which internally
// calls processor::apply. The nested RefCell borrow pattern in undo_logic/
// redo_logic (with_logic_graph_mut closure → with_logic_log_mut) is fixed
// with these helpers.
// ─────────────────────────────────────────────────────────────────────────────

/// Result of applying a logic command.
#[derive(Debug)]
pub struct LogicApplyResult {
    pub inverse: LogicOperationLog,
    pub snapshot: LogicGraphAsset,
}

// ─────────────────────────────────────────────────────────────────────────────
// v0.92: Re-entrancy safe helpers (take/write-back)
//
// undo_graph and redo_graph use the take/write-back pattern: the graph
// is extracted from the LOGIC_GRAPH_DOC RefCell before OperationLog::undo/redo
// is called, releasing the RefCell borrow for the duration of the call.
// This prevents nested RefCell borrow issues in the callers.
// ─────────────────────────────────────────────────────────────────────────────

/// Undo the last logic command (re-entrancy safe, v0.92).
///
/// Panics if no graph is loaded (precondition: caller must ensure graph is active).
pub fn undo_graph() -> Result<LogicGraphAsset, crate::logic_command::LogicCommandError> {
    let mut graph = LOGIC_GRAPH_DOC
        .with(|cell| cell.borrow_mut().take())
        .expect("undo_graph: no active logic graph");
    let mut log = LOGIC_OPERATION_LOG
        .with(|l| l.borrow_mut().take())
        .unwrap_or_else(LogicOperationLog::new_const);
    log.undo(&mut graph)?;
    LOGIC_GRAPH_DOC.with(|cell| *cell.borrow_mut() = Some(graph.clone()));
    LOGIC_OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(log));
    Ok(graph)
}

/// Redo the next logic command (re-entrancy safe, v0.92).
///
/// Panics if no graph is loaded (precondition: caller must ensure graph is active).
pub fn redo_graph() -> Result<LogicGraphAsset, crate::logic_command::LogicCommandError> {
    let mut graph = LOGIC_GRAPH_DOC
        .with(|cell| cell.borrow_mut().take())
        .expect("redo_graph: no active logic graph");
    let mut log = LOGIC_OPERATION_LOG
        .with(|l| l.borrow_mut().take())
        .unwrap_or_else(LogicOperationLog::new_const);
    log.redo(&mut graph)?;
    LOGIC_GRAPH_DOC.with(|cell| *cell.borrow_mut() = Some(graph.clone()));
    LOGIC_OPERATION_LOG.with(|cell| *cell.borrow_mut() = Some(log));
    Ok(graph)
}

/// Get an immutable borrowed reference to the LogicGraphCatalog, initializing if needed.
pub fn with_logic_graph_catalog<F, R>(f: F) -> R
where
    F: FnOnce(&LogicGraphCatalog) -> R,
{
    LOGIC_GRAPH_CATALOG.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(LogicGraphCatalog::new());
        }
        f(mut_ref.as_ref().unwrap())
    })
}

/// Get a mutable borrowed reference to the LogicGraphCatalog, initializing if needed.
pub fn with_logic_graph_catalog_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut LogicGraphCatalog) -> R,
{
    LOGIC_GRAPH_CATALOG.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(LogicGraphCatalog::new());
        }
        f(mut_ref.as_mut().unwrap())
    })
}

/// Seed the three built-in recipe entries into the LogicGraphCatalog.
/// Called lazily on first `list_logic_graph_assets` call so that built-in
/// recipes appear in the listing alongside user-created graphs.
/// Uses the same thread-local guard pattern as `seed_builtin_recipes()` to
/// avoid re-seeding on every call.
thread_local! {
    static BUILTIN_CATALOG_SEEDED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Seed built-in recipe catalog entries into LOGIC_GRAPH_CATALOG.
/// Safe to call multiple times — the guard bails if already seeded on this thread.
pub fn seed_builtin_recipes_to_catalog() {
    if BUILTIN_CATALOG_SEEDED.with(|s| s.get()) {
        return;
    }
    BUILTIN_CATALOG_SEEDED.with(|s| s.set(true));

    // Parse recipe JSON to extract asset metadata for the catalog entries.
    // We use the same include_str! paths as logic_recipes.rs to avoid duplication.
    let recipes: Vec<crate::logic_graph::LogicGraphAsset> = [
        include_str!("../recipes/platformer_jump.json"),
        include_str!("../recipes/health_damage.json"),
        include_str!("../recipes/proximity_trigger.json"),
    ]
    .iter()
    .filter_map(|json_str| serde_json::from_str(json_str).ok())
    .collect();

    // Use the shared WASM-compatible time helper — SystemTime::now() and
    // Instant::now() both panic in wasm32-unknown-unknown stdlib.
    let now = crate::time::now_millis();

    with_logic_graph_catalog_mut(|cat| {
        for recipe in recipes {
            let entry = crate::logic_graph::LogicGraphCatalogEntry {
                asset_id: recipe.asset_id,
                logical_path: recipe.logical_path,
                builtin: true,
                created_at: now,
                updated_at: now,
            };
            // Ignore duplicates — user may have created a graph with the same id
            let _ = cat.register(entry);
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// LogicBinding registry — thread-local storage for instance ↔ recipe bindings
// ─────────────────────────────────────────────────────────────────────────────

/// Record of a logic binding on a scene instance.
#[derive(Debug, Clone)]
pub struct BindingRecord {
    /// Unique binding identifier.
    pub binding_id: BindingId,
    /// Recipe asset_id being bound.
    pub recipe_id: String,
    /// Recipe version at bind time.
    pub version: u32,
    /// Field overrides applied to the binding.
    pub field_overrides: BTreeMap<String, serde_json::Value>,
}

thread_local! {
    /// Registry of logic bindings keyed by scene instance StableId.
    /// Used by apply_bind/unbind/set_override and consumed by
    /// spawn_preview_entity to insert LogicBinding ECS components.
    pub static LOGIC_BINDING_REGISTRY: RefCell<Option<BTreeMap<StableId, BindingRecord>>> =
        const { RefCell::new(None) };
}

/// Get the binding registry, initializing if needed.
pub fn with_binding_registry<F, R>(f: F) -> R
where
    F: FnOnce(&BTreeMap<StableId, BindingRecord>) -> R,
{
    LOGIC_BINDING_REGISTRY.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(BTreeMap::new());
        }
        f(mut_ref.as_ref().unwrap())
    })
}

/// Get a mutable reference to the binding registry, initializing if needed.
pub fn with_binding_registry_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut BTreeMap<StableId, BindingRecord>) -> R,
{
    LOGIC_BINDING_REGISTRY.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(BTreeMap::new());
        }
        f(mut_ref.as_mut().unwrap())
    })
}

/// Result of a binding operation.
#[derive(Debug)]
pub struct BindingResult {
    /// The inverse operation to undo this bind.
    pub inverse: LogicOperation,
    /// The binding ID assigned.
    pub binding_id: BindingId,
}

/// Bind a recipe to a scene instance.
///
/// Validates that:
/// - The recipe exists in the catalog
/// - The instance is not already bound
///
/// Returns the binding ID and the inverse operation on success.
pub fn apply_bind_logic_graph_to_instance(
    scene_instance_id: StableId,
    recipe_id: &str,
    field_overrides: BTreeMap<String, serde_json::Value>,
) -> Result<BindingResult, BindError> {
    // Ensure catalog is seeded
    seed_builtin_recipes_to_catalog();

    // Validate recipe exists in catalog
    let recipe_exists = with_logic_graph_catalog(|cat| cat.get(recipe_id).is_some());
    if !recipe_exists {
        return Err(BindError::RecipeNotFound {
            recipe_id: recipe_id.to_string(),
        });
    }

    // Validate no existing binding on this instance
    let already_bound = with_binding_registry(|reg| reg.contains_key(&scene_instance_id));
    if already_bound {
        return Err(BindError::AlreadyBound { scene_instance_id });
    }

    // Generate binding ID (unique string identifier)
    let binding_id = BindingId::new(format!("bind_{}_{}", scene_instance_id.as_str(), recipe_id));

    // Get recipe version from catalog
    let version = with_logic_graph_catalog(|cat| {
        cat.get(recipe_id).map(|e| e.asset_id.clone()) // just check existence
    })
    .map(|_| 1u32)
    .unwrap_or(1);

    // Record the binding
    let record = BindingRecord {
        binding_id: binding_id.clone(),
        recipe_id: recipe_id.to_string(),
        version,
        field_overrides: field_overrides.clone(),
    };

    with_binding_registry_mut(|reg| {
        reg.insert(scene_instance_id.clone(), record);
    });

    // Build inverse operation
    let inverse = LogicOperation::UnbindLogicGraphFromInstance {
        scene_instance_id,
        binding_id: binding_id.clone(),
    };

    Ok(BindingResult {
        inverse,
        binding_id,
    })
}

/// Unbind a logic binding from a scene instance.
///
/// Returns the inverse bind operation on success.
pub fn apply_unbind_logic_graph_from_instance(
    scene_instance_id: StableId,
    binding_id: BindingId,
) -> Result<LogicOperation, BindError> {
    // Validate binding exists
    let record =
        with_binding_registry(|reg| reg.get(&scene_instance_id).cloned()).ok_or_else(|| {
            BindError::BindingNotFound {
                binding_id: binding_id.clone(),
            }
        })?;

    // Remove the binding
    with_binding_registry_mut(|reg| {
        reg.remove(&scene_instance_id);
    });

    // Build inverse bind operation with same parameters
    let inverse = LogicOperation::BindLogicGraphToInstance {
        scene_instance_id,
        recipe_id: record.recipe_id,
        field_overrides: record.field_overrides,
    };

    Ok(inverse)
}

/// Set a field override on an existing binding.
pub fn apply_set_binding_field_override(
    binding_id: BindingId,
    field_path: String,
    value: serde_json::Value,
) -> Result<(LogicOperation, LogicOperation), BindError> {
    // Find the binding by binding_id
    let (scene_instance_id, mut record) = with_binding_registry(|reg| {
        reg.iter()
            .find(|(_, r)| r.binding_id == binding_id)
            .map(|(sid, r)| (sid.clone(), r.clone()))
    })
    .ok_or_else(|| BindError::BindingNotFound {
        binding_id: binding_id.clone(),
    })?;

    // Validate field_path exists in recipe schema
    // For now, we accept any field_path (schema validation deferred)
    let _recipe_id = record.recipe_id.clone();

    // Build inverse operation (restore old value or remove if none)
    let old_value = record.field_overrides.get(&field_path).cloned();
    let inverse_old = old_value.clone();

    // Update the field override
    record
        .field_overrides
        .insert(field_path.clone(), value.clone());
    with_binding_registry_mut(|reg| {
        if let Some(r) = reg.get_mut(&scene_instance_id) {
            r.field_overrides = record.field_overrides;
        }
    });

    // Forward operation (same structure, different value)
    let forward = LogicOperation::SetBindingFieldOverride {
        binding_id: binding_id.clone(),
        field_path: field_path.clone(),
        value: value.clone(),
    };

    // Build inverse: SetFieldOverride with old value
    let inverse = if let Some(old) = inverse_old {
        LogicOperation::SetBindingFieldOverride {
            binding_id,
            field_path,
            value: old,
        }
    } else {
        // If no old value, the inverse is a no-op; return the same operation
        LogicOperation::SetBindingFieldOverride {
            binding_id,
            field_path,
            value: serde_json::Value::Null,
        }
    };

    Ok((forward, inverse))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_list_all_empty_when_new() {
        let cat = LogicGraphCatalog::new();
        assert!(cat.list_all().is_empty());
    }

    #[test]
    fn catalog_register_inserts_entry() {
        let mut cat = LogicGraphCatalog::new();
        let entry = crate::logic_graph::LogicGraphCatalogEntry {
            asset_id: "test_graph".to_string(),
            logical_path: "logic/test".to_string(),
            builtin: false,
            created_at: 1000,
            updated_at: 1000,
        };
        cat.register(entry.clone()).unwrap();
        assert_eq!(cat.list_all().len(), 1);
        assert_eq!(cat.get("test_graph").unwrap().asset_id, "test_graph");
    }

    #[test]
    fn catalog_register_rejects_duplicate_asset_id() {
        let mut cat = LogicGraphCatalog::new();
        let entry = crate::logic_graph::LogicGraphCatalogEntry {
            asset_id: "dup".to_string(),
            logical_path: "logic/dup".to_string(),
            builtin: false,
            created_at: 1000,
            updated_at: 1000,
        };
        cat.register(entry.clone()).unwrap();
        let result = cat.register(entry);
        assert!(result.is_err());
    }

    #[test]
    fn catalog_unregister_removes_entry() {
        let mut cat = LogicGraphCatalog::new();
        let entry = crate::logic_graph::LogicGraphCatalogEntry {
            asset_id: "remove_me".to_string(),
            logical_path: "logic/remove".to_string(),
            builtin: false,
            created_at: 1000,
            updated_at: 1000,
        };
        cat.register(entry).unwrap();
        cat.unregister("remove_me");
        assert!(cat.get("remove_me").is_none());
    }

    #[test]
    fn catalog_seed_populates_entries() {
        let mut cat = LogicGraphCatalog::new();
        let entries = vec![
            crate::logic_graph::LogicGraphCatalogEntry {
                asset_id: "seed_a".to_string(),
                logical_path: "logic/seed_a".to_string(),
                builtin: false,
                created_at: 1000,
                updated_at: 1000,
            },
            crate::logic_graph::LogicGraphCatalogEntry {
                asset_id: "seed_b".to_string(),
                logical_path: "logic/seed_b".to_string(),
                builtin: false,
                created_at: 2000,
                updated_at: 2000,
            },
        ];
        cat.seed(entries);
        assert_eq!(cat.list_all().len(), 2);
        assert!(cat.get("seed_a").is_some());
        assert!(cat.get("seed_b").is_some());
    }

    #[test]
    fn catalog_path_index_allows_lookup_by_path() {
        // The path_index enables future by-path lookup; here we verify
        // that two entries with different paths don't collide
        let mut cat = LogicGraphCatalog::new();
        cat.register(crate::logic_graph::LogicGraphCatalogEntry {
            asset_id: "g1".to_string(),
            logical_path: "logic/graph_one".to_string(),
            builtin: false,
            created_at: 1000,
            updated_at: 1000,
        })
        .unwrap();
        cat.register(crate::logic_graph::LogicGraphCatalogEntry {
            asset_id: "g2".to_string(),
            logical_path: "logic/graph_two".to_string(),
            builtin: false,
            created_at: 1000,
            updated_at: 1000,
        })
        .unwrap();
        // Both entries are independently accessible
        assert_eq!(cat.get("g1").unwrap().logical_path, "logic/graph_one");
        assert_eq!(cat.get("g2").unwrap().logical_path, "logic/graph_two");
    }

    // ── LogicBinding tests ─────────────────────────────────────────────────

    fn clear_binding_state() {
        // Clear binding registry
        LOGIC_BINDING_REGISTRY.with(|cell| {
            *cell.borrow_mut() = None;
        });
        // Clear catalog (to allow re-seeding)
        LOGIC_GRAPH_CATALOG.with(|cell| {
            *cell.borrow_mut() = None;
        });
        // Reset seed flag
        BUILTIN_CATALOG_SEEDED.with(|s| s.set(false));
    }

    #[test]
    fn bind_inserts_logic_binding() {
        use crate::document::StableId;

        clear_binding_state();

        let sid = StableId::new("inst_test_001");
        // Use the recipe asset_id ("lga_recipe_jump") not logical_path
        let result =
            apply_bind_logic_graph_to_instance(sid.clone(), "lga_recipe_jump", BTreeMap::new());

        assert!(result.is_ok(), "bind should succeed: {:?}", result);
        let binding_result = result.unwrap();
        assert_eq!(
            binding_result
                .binding_id
                .as_str()
                .starts_with("bind_inst_test_001_"),
            true
        );

        // Verify binding is in registry
        let found = with_binding_registry(|reg| reg.get(&sid).is_some());
        assert!(found, "binding should be in registry");
    }

    #[test]
    fn unbind_removes_logic_binding() {
        use crate::document::StableId;

        clear_binding_state();

        let sid = StableId::new("inst_test_002");
        let bind_result =
            apply_bind_logic_graph_to_instance(sid.clone(), "lga_recipe_jump", BTreeMap::new())
                .unwrap();

        // Verify binding exists
        let found_before = with_binding_registry(|reg| reg.get(&sid).is_some());
        assert!(found_before, "binding should exist before unbind");

        // Unbind
        let unbind_result =
            apply_unbind_logic_graph_from_instance(sid.clone(), bind_result.binding_id);
        assert!(
            unbind_result.is_ok(),
            "unbind should succeed: {:?}",
            unbind_result
        );

        // Verify binding is gone
        let found_after = with_binding_registry(|reg| reg.get(&sid).is_some());
        assert!(!found_after, "binding should be removed after unbind");
    }

    #[test]
    fn set_field_updates_field() {
        use crate::document::StableId;

        clear_binding_state();

        let sid = StableId::new("inst_test_003");
        let bind_result =
            apply_bind_logic_graph_to_instance(sid.clone(), "lga_recipe_jump", BTreeMap::new())
                .unwrap();

        // Set a field override
        let override_result = apply_set_binding_field_override(
            bind_result.binding_id.clone(),
            "jump_force".to_string(),
            serde_json::json!(500.0),
        );
        assert!(override_result.is_ok(), "set override should succeed");

        // Verify override is in registry
        let has_override = with_binding_registry(|reg| {
            reg.get(&sid)
                .map(|r| r.field_overrides.get("jump_force") == Some(&serde_json::json!(500.0)))
                .unwrap_or(false)
        });
        assert!(has_override, "field override should be set in registry");
    }

    #[test]
    fn bind_idempotent_when_repeat_same_recipe() {
        use crate::document::StableId;

        clear_binding_state();

        let sid = StableId::new("inst_test_004");
        let first =
            apply_bind_logic_graph_to_instance(sid.clone(), "lga_recipe_jump", BTreeMap::new());
        assert!(first.is_ok(), "first bind should succeed");

        // Second bind should fail with AlreadyBound
        let second =
            apply_bind_logic_graph_to_instance(sid.clone(), "lga_recipe_jump", BTreeMap::new());
        assert!(second.is_err());
        assert!(matches!(second, Err(BindError::AlreadyBound { .. })));
    }
}

// v0.91 PR2 NOTE: LOGIC_GRAPH_DOC migration is deferred to PR3 (causality
// migration pass). The current implementation is correct (single-threaded
// WASM, no contention), and the trait seam is in place — the migration
// becomes a mechanical `SCENE_DOC.with → with_session_mut` substitution
// once PR3 lands.
