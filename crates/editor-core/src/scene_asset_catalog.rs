//! Scene Asset Catalog — editor-side metadata index for SceneAssetDocuments.
//!
//! See ADR-0005 §Implementation Direction step 1: Scene Asset Catalog as a
//! first-class Project concept.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::scene_asset::SceneAssetRole;

/// Helper for serde: deserializes only `entries`, then rebuilds indices.
/// The `#[serde(skip)]` on path_index and role_index means they come out empty
/// from the derived Deserialize, so we rebuild them post-deserialization.
impl<'de> Deserialize<'de> for SceneAssetCatalog {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            entries: BTreeMap<String, SceneAssetCatalogEntry>,
        }
        let helper = Helper::deserialize(deserializer)?;
        let mut catalog = SceneAssetCatalog {
            entries: helper.entries,
            path_index: BTreeMap::new(),
            role_index: BTreeMap::new(),
        };
        catalog.rebuild_indices();
        Ok(catalog)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SceneAssetCatalog {
    entries: BTreeMap<String, SceneAssetCatalogEntry>,
    path_index: BTreeMap<String, String>,
    role_index: BTreeMap<String, BTreeSet<String>>,
}

impl Serialize for SceneAssetCatalog {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SceneAssetCatalog", 1)?;
        state.serialize_field("entries", &self.entries)?;
        state.end()
    }
}

/// One entry in the SceneAssetCatalog: metadata for a single scene asset.
///
/// Contains the stable `asset_id`, the user-facing `logical_path`,
/// the asset's role, current schema version, optional tags, and the
/// `created_at` / `updated_at` Unix timestamps (milliseconds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetCatalogEntry {
    pub asset_id: String,
    pub logical_path: String,
    pub role: SceneAssetRole,
    pub current_version: u32,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    /// Optional OPFS `resources/<path>` reference used by the Asset
    /// Browser to render an inline 64×64 preview. `None` when the
    /// asset has no associated preview texture.
    ///
    /// `#[serde(default)]` covers deserialise: old JSON literals
    /// without this field load as `None`.
    /// `#[serde(skip_serializing_if = "Option::is_none")]` covers
    /// serialise: most entries are `None` and the field is omitted
    /// from the on-disk JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_resource: Option<String>,
}

/// Errors produced by SceneAssetCatalog operations (insert, remove, update).
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

/// A non-fatal warning emitted by SceneAssetCatalog operations
/// (e.g., orphaned entries, path mismatches). Surfaced to the UI via
/// `get_asset_catalog_warnings`.
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

    /// Rebuild path_index and role_index from current entries.
    /// Call after deserialization or when indices are stale.
    fn rebuild_indices(&mut self) {
        self.path_index.clear();
        self.role_index.clear();
        for (asset_id, entry) in &self.entries {
            let normalized = normalize_logical_path(&entry.logical_path);
            self.path_index.insert(normalized, asset_id.clone());
            let role_key = role_key(&entry.role).to_string();
            self.role_index
                .entry(role_key)
                .or_default()
                .insert(asset_id.clone());
        }
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
        let created_at = {
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

            entry.created_at
        };

        // Use at least created_at + 1 to guarantee updated_at > created_at,
        // even when register and update happen in the same millisecond.
        let updated_at = current_unix_millis().max(created_at + 1);

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

/// Generate a fresh, unique asset_id. Combines the current Unix
/// timestamp (millis) with 8 hex chars of randomness so collisions
/// across rapid successive calls are extremely unlikely.
pub fn mint_asset_id() -> String {
    format!("id_{}_{}", current_unix_millis(), random_hex_8())
}

/// Normalize a user-supplied logical path: trim, lowercase, replace
/// backslashes with forward slashes, collapse repeated slashes, and
/// strip leading/trailing slashes.
pub fn normalize_logical_path(path: &str) -> String {
    let s = path.trim();
    let s = s.to_lowercase();
    let s = s.replace('\\', "/");
    let s = s.replace("//", "/");
    let s = s.trim_matches('/').to_string();
    s
}

/// Validate that `path` is acceptable as a Scene Asset logical path:
/// non-empty after trim and contains no illegal characters.
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
    crate::time::now_millis()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_asset::SceneAssetRole;

    fn entry(asset_id: &str, logical_path: &str, role: SceneAssetRole) -> SceneAssetCatalogEntry {
        SceneAssetCatalogEntry {
            asset_id: asset_id.to_string(),
            logical_path: logical_path.to_string(),
            role,
            current_version: 1,
            tags: vec![],
            created_at: 1000,
            updated_at: 1000,
            preview_resource: None,
        }
    }

    /// Spec scenario from opfs-catalog-persistence cycle, Phase 1.1 RED.
    /// Verifies that after `register → unregister` the catalog invariants
    /// are clean (no dangling path_index / role_index entries).
    #[test]
    fn register_then_unregister_roundtrip() {
        let mut catalog = SceneAssetCatalog::new();
        let e = entry("id_x", "actors/player", SceneAssetRole::Actor);
        catalog
            .register(e.clone())
            .expect("register should succeed");

        assert_eq!(catalog.get("id_x"), Some(&e));
        assert_eq!(catalog.resolve_path("actors/player"), Some("id_x"));
        assert_eq!(catalog.list_by_role(SceneAssetRole::Actor).len(), 1);

        // Rollback path used by WASM create/rename helpers on metadata failure
        let removed = catalog
            .unregister("id_x")
            .expect("unregister should succeed");
        assert_eq!(removed.asset_id, "id_x");

        assert_eq!(catalog.get("id_x"), None);
        assert_eq!(catalog.resolve_path("actors/player"), None);
        assert!(catalog.list_all().is_empty());
        assert!(catalog.list_by_role(SceneAssetRole::Actor).is_empty());
        assert!(catalog.validate_invariants().is_empty());
    }

    /// ADR-0026 S1.2 / D3.3: a catalog JSON literal that omits the new
    /// `preview_resource` field (older catalog format) deserialises to
    /// an entry with `preview_resource = None`, and re-serialising omits
    /// the field. This is the back-compat contract for pre-v0.83.0
    /// on-disk catalogs.
    #[test]
    fn catalog_without_preview_resource_round_trips() {
        let json = r#"{"entries":{"id_x":{"asset_id":"id_x","logical_path":"actors/player","role":"actor","current_version":1,"tags":[],"created_at":1,"updated_at":1}}}"#;
        let catalog: SceneAssetCatalog =
            serde_json::from_str(json).expect("back-compat deserialize");
        let entry = catalog.get("id_x").expect("entry should be present");
        assert_eq!(entry.preview_resource, None);

        let reserialized = serde_json::to_string(&catalog).expect("re-serialize");
        assert!(
            !reserialized.contains("preview_resource"),
            "preview_resource must be skipped when None: {}",
            reserialized
        );
    }

    /// ADR-0026 S1.2: when `preview_resource = Some("x.png")` is set,
    /// it round-trips through serialise/deserialise losslessly.
    #[test]
    fn catalog_with_preview_resource_round_trips() {
        let mut catalog = SceneAssetCatalog::new();
        let mut e = entry("id_x", "actors/player", SceneAssetRole::Actor);
        e.preview_resource = Some("textures/player.png".to_string());
        catalog
            .register(e.clone())
            .expect("register should succeed");

        let json = serde_json::to_string(&catalog).expect("serialize");
        assert!(json.contains("preview_resource"));

        let reparsed: SceneAssetCatalog = serde_json::from_str(&json).expect("deserialize");
        let entry = reparsed.get("id_x").expect("entry present");
        assert_eq!(
            entry.preview_resource.as_deref(),
            Some("textures/player.png")
        );
    }

    /// ADR-0026 S1.2: an entry registered without setting
    /// `preview_resource` (via the public Rust surface) ends up with
    /// `None`. This is the structural test of `#[serde(default)]`.
    #[test]
    fn register_assigns_default_none() {
        let mut catalog = SceneAssetCatalog::new();
        let e = entry("id_x", "actors/player", SceneAssetRole::Actor);
        assert_eq!(e.preview_resource, None);
        catalog.register(e).expect("register should succeed");
        let stored = catalog.get("id_x").expect("entry should be present");
        assert_eq!(stored.preview_resource, None);
    }
}
