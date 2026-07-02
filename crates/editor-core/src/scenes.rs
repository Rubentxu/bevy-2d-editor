//! SceneRegistry — in-memory multi-scene management with value-swap thread_locals.
//!
//! Design: Value-swap over accessor refactor (ADR-001).
//! Scene name = registry key = OPFS filename (Decision: Scene name as identifier).

use std::collections::HashMap;
use std::sync::Mutex;

use crate::document::SceneDocument;
use crate::operation_log::OperationLog;

/// Maximum number of scenes allowed in a single project.
pub const MAX_SCENES: usize = 16;

/// An entry in the registry: holds the scene document, its operation log,
/// and the dirty flag for unsaved-changes indicator.
#[derive(Clone)]
pub struct SceneEntry {
    pub scene: SceneDocument,
    pub log: OperationLog,
    pub is_dirty: bool,
}

/// Public metadata for a scene, returned by `list_scenes_extended`.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneInfo {
    pub id: String,
    pub name: String,
    pub is_current: bool,
    pub is_dirty: bool,
}

/// Result of a switch attempt.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchResult {
    pub switched: bool,
    pub dirty_prompt_required: bool,
    pub source_name: String,
}

/// Errors that can occur during registry operations.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SceneRegistryError {
    pub code: String,
    pub message: String,
}

impl SceneRegistryError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }

    pub fn to_js_value(&self) -> wasm_bindgen::JsValue {
        wasm_bindgen::JsValue::from_str(&serde_json::to_string(self).unwrap_or_default())
    }
}

/// Thread-local scene registry holding all scenes in memory.
pub struct SceneRegistry {
    entries: Mutex<HashMap<String, SceneEntry>>,
    current_scene_id: Mutex<Option<String>>,
}

impl SceneRegistry {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            current_scene_id: Mutex::new(None),
        }
    }

    /// Create a new scene with the given name.
    /// If the name already exists, appends "-2", "-3", etc.
    /// Returns the actual name used (may differ from input due to deduplication).
    /// Returns Err if MAX_SCENES is reached.
    pub fn create(&self, name: &str) -> Result<String, SceneRegistryError> {
        let mut entries = self.entries.lock().unwrap();

        if entries.len() >= MAX_SCENES {
            return Err(SceneRegistryError::new(
                "MAX_SCENES",
                &format!("Cannot create more than {} scenes", MAX_SCENES),
            ));
        }

        let actual_name = Self::make_unique_name(name, &entries);
        let entry = SceneEntry {
            scene: Self::default_scene(&actual_name),
            log: OperationLog::new_const(),
            is_dirty: true, // new scenes are "dirty" until first save
        };
        entries.insert(actual_name.clone(), entry);

        // Set as current if first scene
        let mut current = self.current_scene_id.lock().unwrap();
        if current.is_none() {
            *current = Some(actual_name.clone());
        }

        Ok(actual_name)
    }

    /// Switch the current scene. Returns `SwitchResult` indicating whether
    /// the switch happened and whether a dirty prompt is required.
    ///
    /// The caller is responsible for actually performing the value-swap in
    /// SCENE_DOC / OPERATION_LOG thread_locals after checking `dirty_prompt_required`.
    pub fn switch(&self, id: &str) -> Result<SwitchResult, SceneRegistryError> {
        let mut current = self.current_scene_id.lock().unwrap();

        let source_name = current
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "".to_string());

        if current.as_deref() == Some(id) {
            return Ok(SwitchResult {
                switched: false,
                dirty_prompt_required: false,
                source_name,
            });
        }

        // Check if source scene is dirty — if so, caller must prompt
        let dirty_prompt_required = if let Some(cur) = current.as_ref() {
            let entries = self.entries.lock().unwrap();
            entries.get(cur).map(|e| e.is_dirty).unwrap_or(false)
        } else {
            false
        };

        if !dirty_prompt_required {
            // Safe to switch immediately
            let entries = self.entries.lock().unwrap();
            if !entries.contains_key(id) {
                return Err(SceneRegistryError::new("NOT_FOUND", &format!("Scene '{}' not found", id)));
            }
            *current = Some(id.to_string());
            return Ok(SwitchResult {
                switched: true,
                dirty_prompt_required: false,
                source_name,
            });
        }

        // Dirty source — frontend must prompt; switch not yet performed
        Ok(SwitchResult {
            switched: false,
            dirty_prompt_required: true,
            source_name,
        })
    }

    /// Commit the switch after user resolves the dirty prompt.
    /// Call this only after confirming the source scene is saved or discarded.
    pub fn commit_switch(&self, target_id: &str) -> Result<(), SceneRegistryError> {
        let mut current = self.current_scene_id.lock().unwrap();
        let entries = self.entries.lock().unwrap();
        if !entries.contains_key(target_id) {
            return Err(SceneRegistryError::new("NOT_FOUND", &format!("Scene '{}' not found", target_id)));
        }
        *current = Some(target_id.to_string());
        Ok(())
    }

    /// Delete a scene. Returns Err if it's the last remaining scene.
    pub fn delete(&self, id: &str) -> Result<(), SceneRegistryError> {
        let mut entries = self.entries.lock().unwrap();

        if entries.len() <= 1 {
            return Err(SceneRegistryError::new(
                "LAST_SCENE",
                "Cannot delete the last remaining scene",
            ));
        }

        if !entries.remove(id).is_some() {
            return Err(SceneRegistryError::new("NOT_FOUND", &format!("Scene '{}' not found", id)));
        }

        // If we deleted the current scene, switch to the first remaining
        let mut current = self.current_scene_id.lock().unwrap();
        if current.as_deref() == Some(id) {
            *current = entries.keys().next().cloned();
        }

        Ok(())
    }

    /// Rename a scene. Returns the new name (may differ if the new name was taken).
    pub fn rename(&self, id: &str, new_name: &str) -> Result<String, SceneRegistryError> {
        if new_name == id {
            return Ok(id.to_string());
        }

        let actual_new_name = {
            let entries = self.entries.lock().unwrap();
            // Check existence first (release lock before re-acquiring for mut ops)
            if !entries.contains_key(id) {
                return Err(SceneRegistryError::new("NOT_FOUND", &format!("Scene '{}' not found", id)));
            }
            Self::make_unique_name(new_name, &entries)
        };

        // Now acquire mutable access to perform the rename
        let mut entries = self.entries.lock().unwrap();
        let mut entry = entries
            .get_mut(id)
            .ok_or_else(|| SceneRegistryError::new("NOT_FOUND", &format!("Scene '{}' not found", id)))?;

        // Update the scene's internal name
        entry.scene.name = actual_new_name.clone();

        // Re-key the HashMap
        let entry = entries.remove(id).unwrap();
        entries.insert(actual_new_name.clone(), entry);

        // Update current pointer if needed
        let mut current = self.current_scene_id.lock().unwrap();
        if current.as_deref() == Some(id) {
            *current = Some(actual_new_name.clone());
        }

        Ok(actual_new_name)
    }

    /// List all scenes with metadata.
    pub fn list(&self) -> Vec<SceneInfo> {
        let entries = self.entries.lock().unwrap();
        let current = self.current_scene_id.lock().unwrap();

        entries
            .iter()
            .map(|(id, entry)| SceneInfo {
                id: id.clone(),
                name: entry.scene.name.clone(),
                is_current: current.as_deref() == Some(id),
                is_dirty: entry.is_dirty,
            })
            .collect()
    }

    /// Get the current scene entry (cloned).
    pub fn current(&self) -> Option<SceneEntry> {
        let entries = self.entries.lock().unwrap();
        let current = self.current_scene_id.lock().unwrap();
        current.as_ref().and_then(|id| entries.get(id).cloned())
    }

    /// Mark the current scene as dirty (unsaved changes).
    pub fn mark_current_dirty(&self) {
        if let Some(current) = self.current_scene_id.lock().unwrap().clone() {
            if let Some(entry) = self.entries.lock().unwrap().get_mut(&current) {
                entry.is_dirty = true;
            }
        }
    }

    /// Clear the dirty flag on the current scene (called after save).
    pub fn clear_current_dirty(&self) {
        if let Some(current) = self.current_scene_id.lock().unwrap().clone() {
            if let Some(entry) = self.entries.lock().unwrap().get_mut(&current) {
                entry.is_dirty = false;
            }
        }
    }

    /// Get the current scene ID.
    pub fn current_id(&self) -> Option<String> {
        self.current_scene_id.lock().unwrap().clone()
    }

    /// Get the scene entry by id.
    pub fn get(&self, id: &str) -> Option<SceneEntry> {
        self.entries.lock().unwrap().get(id).cloned()
    }

    /// Check if a scene with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.entries.lock().unwrap().contains_key(name)
    }

    /// Number of scenes currently in the registry.
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// Load an existing scene into the registry (used during project load).
    pub fn load_scene(&self, name: String, scene: SceneDocument, log: OperationLog) {
        let mut entries = self.entries.lock().unwrap();
        let entry = SceneEntry {
            scene,
            log,
            is_dirty: false, // loaded scenes are clean until edited
        };
        entries.insert(name.clone(), entry);
    }

    /// Set the current scene ID (used after loading project).
    pub fn set_current(&self, id: Option<String>) {
        let mut current = self.current_scene_id.lock().unwrap();
        *current = id;
    }

    /// Take a snapshot of the current scene's doc+log for value-swap.
    /// Returns (scene, log) that should be stored back into registry[old_id].
    pub fn swap_out_current(&self) -> Option<(SceneDocument, OperationLog)> {
        let current = self.current_scene_id.lock().unwrap().clone()?;
        let mut entries = self.entries.lock().unwrap();
        let entry = entries.get_mut(&current)?;
        Some((entry.scene.clone(), entry.log.clone()))
    }

    /// Load a scene's doc+log into the active view (value-swap target).
    /// Takes (scene, log) from registry[new_id] and makes them active.
    pub fn swap_in(&self, id: &str) -> Option<(SceneDocument, OperationLog)> {
        let mut entries = self.entries.lock().unwrap();
        let entry = entries.get_mut(id)?;
        Some((entry.scene.clone(), entry.log.clone()))
    }

    /// Store the active doc+log back into a specific scene entry (after editing).
    pub fn store_to(&self, id: &str, scene: SceneDocument, log: OperationLog) {
        if let Some(entry) = self.entries.lock().unwrap().get_mut(id) {
            entry.scene = scene;
            entry.log = log;
        }
    }

    /// Generate a unique name by appending "-2", "-3", etc. if needed.
    fn make_unique_name(base: &str, entries: &HashMap<String, SceneEntry>) -> String {
        if !entries.contains_key(base) {
            return base.to_string();
        }
        let mut counter = 2;
        loop {
            let candidate = format!("{}-{}", base, counter);
            if !entries.contains_key(&candidate) {
                return candidate;
            }
            counter += 1;
            if counter > 1000 {
                // Safety valve
                return candidate;
            }
        }
    }

    /// Create a default empty scene with the given name.
    fn default_scene(name: &str) -> SceneDocument {
        use std::collections::BTreeMap;
        SceneDocument {
            version: "0.1".to_string(),
            scene_id: format!(
                "scene-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ),
            name: name.to_string(),
            entities: Vec::new(),
            instances: BTreeMap::new(),
        }
    }
}

impl Default for SceneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn make_doc(name: &str) -> SceneDocument {
        SceneDocument {
            version: "0.1".to_string(),
            scene_id: "test-id".to_string(),
            name: name.to_string(),
            entities: Vec::new(),
            instances: BTreeMap::new(),
        }
    }

    #[test]
    fn test_create_first_scene() {
        let registry = SceneRegistry::new();
        let result = registry.create("Level 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Level 1");
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.current_id(), Some("Level 1".to_string()));
    }

    #[test]
    fn test_create_duplicate_name_gets_suffix() {
        let registry = SceneRegistry::new();
        registry.create("Level").unwrap();
        let name2 = registry.create("Level").unwrap();
        assert_eq!(name2, "Level-2");
    }

    #[test]
    fn test_max_scenes_limit() {
        let registry = SceneRegistry::new();
        for i in 0..MAX_SCENES {
            registry.create(&format!("Scene {}", i)).unwrap();
        }
        let result = registry.create("Too Many");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "MAX_SCENES");
    }

    #[test]
    fn test_delete_last_scene_fails() {
        let registry = SceneRegistry::new();
        registry.create("Solo").unwrap();
        let result = registry.delete("Solo");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "LAST_SCENE");
    }

    #[test]
    fn test_delete_non_current_switches_remaining() {
        let registry = SceneRegistry::new();
        registry.create("A").unwrap();
        registry.create("B").unwrap();
        // Current is "A"
        registry.delete("B").unwrap();
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.current_id(), Some("A".to_string()));
    }

    #[test]
    fn test_switch_to_dirty_source_requires_prompt() {
        let registry = SceneRegistry::new();
        registry.create("Clean").unwrap();
        registry.create("Dirty").unwrap();
        // After two creates, current is still "Clean" (first scene).
        // Mark it dirty so the next switch will be blocked.
        registry.mark_current_dirty();

        // Switch is blocked because current ("Clean") is dirty
        let switch_result = registry.switch("Dirty").unwrap();
        assert!(!switch_result.switched);
        assert!(switch_result.dirty_prompt_required);
        assert_eq!(switch_result.source_name, "Clean");
    }

    #[test]
    fn test_switch_clean_to_clean_succeeds() {
        let registry = SceneRegistry::new();
        registry.create("A").unwrap();
        registry.create("B").unwrap();
        // After two creates, current is still "A" (first scene).
        // Clear dirty flag so switch is allowed.
        registry.clear_current_dirty();

        let result = registry.switch("B").unwrap();
        assert!(result.switched);
        assert!(!result.dirty_prompt_required);
        assert_eq!(registry.current_id(), Some("B".to_string()));
    }

    #[test]
    fn test_commit_switch_after_dirty_resolution() {
        let registry = SceneRegistry::new();
        registry.create("A").unwrap();
        registry.create("B").unwrap();
        registry.mark_current_dirty();

        // Probe switch
        let probe = registry.switch("B").unwrap();
        assert!(!probe.switched);

        // Commit switch (after save/discard)
        registry.commit_switch("B").unwrap();
        assert_eq!(registry.current_id(), Some("B".to_string()));
    }

    #[test]
    fn test_rename() {
        let registry = SceneRegistry::new();
        registry.create("Old Name").unwrap();
        let new_name = registry.rename("Old Name", "New Name").unwrap();
        assert_eq!(new_name, "New Name");
        assert!(registry.contains("New Name"));
        assert!(!registry.contains("Old Name"));
        assert_eq!(registry.current_id(), Some("New Name".to_string()));
    }

    #[test]
    fn test_rename_duplicate_suffix() {
        let registry = SceneRegistry::new();
        registry.create("Level").unwrap();
        registry.create("Level-2").unwrap();
        let new_name = registry.rename("Level", "Level-2").unwrap();
        assert_eq!(new_name, "Level-2-2");
    }

    #[test]
    fn test_list_returns_metadata() {
        let registry = SceneRegistry::new();
        registry.create("A").unwrap();
        registry.create("B").unwrap();
        // After two creates, current is still "A" (first scene).
        // Switch to B so the current-is_dirty test makes sense for B.
        registry.clear_current_dirty(); // clear A's dirty flag
        registry.switch("B").unwrap();
        registry.mark_current_dirty(); // mark B dirty

        let list = registry.list();
        assert_eq!(list.len(), 2);
        let current = list.iter().find(|s| s.is_current).unwrap();
        assert_eq!(current.name, "B"); // B is current after switch
        assert!(current.is_dirty);
    }

    #[test]
    fn test_mark_and_clear_dirty() {
        let registry = SceneRegistry::new();
        registry.create("Test").unwrap();
        // New scenes are dirty by default (need first save to become clean)
        assert!(registry.current().unwrap().is_dirty);

        registry.mark_current_dirty();
        assert!(registry.current().unwrap().is_dirty);

        registry.clear_current_dirty();
        assert!(!registry.current().unwrap().is_dirty);
    }

    #[test]
    fn test_load_scene_populates_registry() {
        let registry = SceneRegistry::new();
        let doc = make_doc("Loaded Scene");
        registry.load_scene("Loaded Scene".to_string(), doc.clone(), OperationLog::new_const());

        assert!(registry.contains("Loaded Scene"));
        assert_eq!(registry.get("Loaded Scene").unwrap().scene.name, "Loaded Scene");
    }

    #[test]
    fn test_swap_out_and_store() {
        let registry = SceneRegistry::new();
        registry.create("Test").unwrap();

        let (scene, log) = registry.swap_out_current().unwrap();
        assert_eq!(scene.name, "Test");

        // Simulate edits and store back
        let mut edited_doc = scene.clone();
        edited_doc.name = "Edited".to_string();
        registry.store_to("Test", edited_doc, log);

        assert_eq!(registry.get("Test").unwrap().scene.name, "Edited");
    }
}
