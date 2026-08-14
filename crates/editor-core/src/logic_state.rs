//! HIGH-1 phase 2: logic-graph state sub-module.
//!
//! Owns the LOGIC_GRAPH_DOC (active graph being edited), the
//! LOGIC_OPERATION_LOG (per-graph undo/redo), and the LOGIC_GRAPH_CATALOG
//! (catalog of all logic graph assets persisted in OPFS).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::logic_command::LogicOperationLog;
use crate::logic_graph::{LogicGraphAsset, LogicGraphCatalogEntry};

thread_local! {
    /// Logic Graph document: the active logic graph being edited.
    pub static LOGIC_GRAPH_DOC: RefCell<Option<LogicGraphAsset>> = const { RefCell::new(None) };
    /// Logic operation log: per-graph undo/redo history.
    pub static LOGIC_OPERATION_LOG: RefCell<LogicOperationLog> = const { RefCell::new(LogicOperationLog::new_const()) };
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
        let normalized = crate::scene_asset_catalog::normalize_logical_path(&entry.logical_path);
        self.path_index.insert(normalized, entry.asset_id.clone());
        self.entries.insert(entry.asset_id.clone(), entry);
        Ok(())
    }

    /// Remove an entry by asset_id. Returns the removed entry.
    pub fn unregister(&mut self, asset_id: &str) -> Option<LogicGraphCatalogEntry> {
        if let Some(entry) = self.entries.remove(asset_id) {
            let normalized =
                crate::scene_asset_catalog::normalize_logical_path(&entry.logical_path);
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
                crate::scene_asset_catalog::normalize_logical_path(&entry.logical_path);
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
    LOGIC_OPERATION_LOG.with(|cell| f(&*cell.borrow()))
}

/// Get a mutable borrowed reference to the LogicOperationLog.
pub fn with_logic_log_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut LogicOperationLog) -> R,
{
    LOGIC_OPERATION_LOG.with(|cell| f(&mut *cell.borrow_mut()))
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
}
