//! Scene Asset Catalog — editor-side metadata index for SceneAssetDocuments.
//!
//! See ADR-0005 §Implementation Direction step 1: Scene Asset Catalog as a
//! first-class Project concept.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::scene_asset::SceneAssetRole;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneAssetCatalog {
    entries: BTreeMap<String, SceneAssetCatalogEntry>,
    #[serde(skip)]
    path_index: BTreeMap<String, String>,
    #[serde(skip)]
    role_index: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetCatalogEntry {
    pub asset_id: String,
    pub logical_path: String,
    pub role: SceneAssetRole,
    pub current_version: u32,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatalogError {
    #[error("duplicate asset_id '{id}'")]
    DuplicateAssetId { id: String },
    #[error("duplicate logical_path '{path}'")]
    DuplicateLogicalPath { path: String },
    #[error("asset_id '{id}' not found")]
    NotFound { id: String },
    #[error("invalid logical path: {reason}")]
    InvalidPath { reason: String },
    #[error("invalid version: new version {new} not greater than current {current}")]
    InvalidVersion { current: u32, new: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogWarning {
    pub code: String,
    pub message: String,
    pub asset_id: Option<String>,
    pub logical_path: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// SceneAssetCatalog impl
// ─────────────────────────────────────────────────────────────────────────────

impl SceneAssetCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_entries(entries: Vec<SceneAssetCatalogEntry>) -> Result<Self, CatalogError> {
        entries.into_iter().fold(Ok(Self::default()), |acc, entry| {
            let mut catalog = acc?;
            catalog.register(entry)?;
            Ok(catalog)
        })
    }

    pub fn register(&mut self, entry: SceneAssetCatalogEntry) -> Result<(), CatalogError> {
        validate_logical_path(&entry.logical_path)?;

        let mut entry = entry;
        let normalized = normalize_logical_path(&entry.logical_path);
        entry.logical_path = normalized.clone();

        if self.entries.contains_key(&entry.asset_id) {
            return Err(CatalogError::DuplicateAssetId {
                id: entry.asset_id.clone(),
            });
        }
        if self.path_index.contains_key(&normalized) {
            return Err(CatalogError::DuplicateLogicalPath { path: normalized });
        }

        entry.tags = dedupe_tags(entry.tags);

        let asset_id = entry.asset_id.clone();
        let role_key = role_key(&entry.role).to_string();

        self.entries.insert(asset_id.clone(), entry);
        self.path_index.insert(normalized, asset_id.clone());
        self.role_index
            .entry(role_key)
            .or_default()
            .insert(asset_id);

        Ok(())
    }

    pub fn unregister(&mut self, asset_id: &str) -> Result<SceneAssetCatalogEntry, CatalogError> {
        let entry = self
            .entries
            .remove(asset_id)
            .ok_or_else(|| CatalogError::NotFound {
                id: asset_id.to_string(),
            })?;

        let normalized = normalize_logical_path(&entry.logical_path);
        self.path_index.remove(&normalized);

        let role_key = role_key(&entry.role).to_string();
        if let Some(set) = self.role_index.get_mut(&role_key) {
            set.remove(asset_id);
            if set.is_empty() {
                self.role_index.remove(&role_key);
            }
        }

        Ok(entry)
    }

    pub fn update_version(&mut self, asset_id: &str, new_version: u32) -> Result<(), CatalogError> {
        let entry = self
            .entries
            .get(asset_id)
            .ok_or_else(|| CatalogError::NotFound {
                id: asset_id.to_string(),
            })?;

        if new_version <= entry.current_version {
            return Err(CatalogError::InvalidVersion {
                current: entry.current_version,
                new: new_version,
            });
        }

        let updated_at = current_unix_millis();

        let entry = self.entries.get_mut(asset_id).unwrap();
        entry.current_version = new_version;
        entry.updated_at = updated_at;

        Ok(())
    }

    pub fn get(&self, asset_id: &str) -> Option<&SceneAssetCatalogEntry> {
        self.entries.get(asset_id)
    }

    pub fn resolve_path(&self, path: &str) -> Option<&str> {
        let normalized = normalize_logical_path(path);
        self.path_index.get(&normalized).map(|s| s.as_str())
    }

    pub fn list_all(&self) -> Vec<&SceneAssetCatalogEntry> {
        self.entries.values().collect()
    }

    pub fn list_by_role(&self, role: SceneAssetRole) -> Vec<&SceneAssetCatalogEntry> {
        let key = role_key(&role).to_string();
        self.role_index
            .get(&key)
            .map(|set| set.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn broken_references<I, S>(&self, references: I) -> Vec<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut seen = BTreeSet::new();
        let mut result = Vec::new();
        for reference in references {
            let key = reference.as_ref();
            if !seen.contains(key) {
                seen.insert(key.to_string());
                if !self.entries.contains_key(key) {
                    result.push(key.to_string());
                }
            }
        }
        result
    }

    pub fn validate_invariants(&self) -> Vec<CatalogWarning> {
        let mut warnings = Vec::new();
        for entry in self.entries.values() {
            let observed = normalize_logical_path(&entry.logical_path);
            if observed != entry.logical_path {
                warnings.push(CatalogWarning {
                    code: "non_normalized_path".to_string(),
                    message: format!(
                        "logical_path '{}' is not normalized (expected '{}')",
                        entry.logical_path, observed
                    ),
                    asset_id: Some(entry.asset_id.clone()),
                    logical_path: Some(entry.logical_path.clone()),
                });
            }
            if !entry.asset_id.starts_with("id_") {
                warnings.push(CatalogWarning {
                    code: "malformed_asset_id".to_string(),
                    message: format!(
                        "asset_id '{}' does not start with 'id_' prefix",
                        entry.asset_id
                    ),
                    asset_id: Some(entry.asset_id.clone()),
                    logical_path: None,
                });
            }
            let deduped = dedupe_tags(entry.tags.clone());
            if deduped != entry.tags {
                warnings.push(CatalogWarning {
                    code: "duplicate_tag".to_string(),
                    message: "tags contain duplicates".to_string(),
                    asset_id: Some(entry.asset_id.clone()),
                    logical_path: None,
                });
            }
        }
        warnings
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public free functions
// ─────────────────────────────────────────────────────────────────────────────

pub fn mint_asset_id() -> String {
    format!("id_{}_{}", current_unix_millis(), random_hex_8())
}

pub fn normalize_logical_path(path: &str) -> String {
    let s = path.trim();
    let s = s.to_lowercase();
    let s = s.replace('\\', "/");
    let s = s.replace("//", "/");
    let s = s.trim_matches('/').to_string();
    s
}

pub fn validate_logical_path(path: &str) -> Result<(), CatalogError> {
    if path.trim().is_empty() {
        return Err(CatalogError::InvalidPath {
            reason: "empty".to_string(),
        });
    }
    let segments: Vec<&str> = path.split('/').collect();
    for seg in segments {
        if seg == ".." || seg == "." {
            return Err(CatalogError::InvalidPath {
                reason: "path traversal not allowed".to_string(),
            });
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers
// ─────────────────────────────────────────────────────────────────────────────

fn role_key(role: &SceneAssetRole) -> &'static str {
    match role {
        SceneAssetRole::Actor => "actor",
        SceneAssetRole::Fragment => "fragment",
        SceneAssetRole::Screen => "screen",
        SceneAssetRole::Level => "level",
        SceneAssetRole::Ui => "ui",
        SceneAssetRole::Effect => "effect",
        SceneAssetRole::Logic => "logic",
    }
}

fn dedupe_tags(tags: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    tags.into_iter()
        .filter(|t| seen.insert(t.clone()))
        .collect()
}

fn current_unix_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
fn random_hex_8() -> String {
    use js_sys::{Date, Math};
    let seed = (Date::now() * 1_000_000.0) as u64 ^ (Math::random() * 1e15) as u64;
    format!("{:016x}", seed & 0xFFFFFFFF)
}

#[cfg(not(target_arch = "wasm32"))]
fn random_hex_8() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let counter = static_counter();
    format!("{:016x}", nanos.wrapping_add(counter) & 0xFFFFFFFF)
}

#[cfg(not(target_arch = "wasm32"))]
fn static_counter() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static C: AtomicU64 = AtomicU64::new(0);
    C.fetch_add(1, Ordering::SeqCst)
}
