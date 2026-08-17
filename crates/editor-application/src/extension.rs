//! Extension registry — concrete implementation of [`editor_model::ports::ExtensionRegistryPort`].
//!
//! Lives in `editor_application` (not `editor_model`) because it holds runtime state
//! and is composed into `EditorSession`. Mirrors the `InMemoryProjectStore` pattern.

use std::collections::HashMap;

use editor_model::extension::{
    Capability, CapabilityDescriptor, ExtensionError, ExtensionHandle, ExtensionId,
    ExtensionManifest, ExtensionSummary, Permission, PermissionArea, PermissionScope, SemVer,
};
use editor_model::ports::ExtensionRegistryPort;

/// In-memory extension registry.
///
/// Stores manifests by ID and assigns sequential `ExtensionHandle` values on registration.
/// Thread-safe: all operations go through `self.0` (the `Arc<Mutex<...>>` wrapping this struct).
#[derive(Debug)]
pub struct ExtensionRegistry {
    /// Maps `ExtensionId` → `ExtensionManifest`.
    manifests: HashMap<String, ExtensionManifest>,
    /// Maps `ExtensionId` → `ExtensionHandle`.
    handles: HashMap<String, ExtensionHandle>,
    /// Next handle value to assign.
    next_handle: u64,
}

impl ExtensionRegistry {
    /// Construct an empty registry.
    pub fn empty() -> Self {
        Self {
            manifests: HashMap::new(),
            handles: HashMap::new(),
            next_handle: 1,
        }
    }

    /// Construct a registry pre-populated with built-in extensions.
    ///
    /// Registers the three built-in manifests defined inline below.
    /// This is the canonical constructor for production use
    /// (called by `EditorSession::with_builtins`).
    pub fn with_builtins() -> Self {
        let mut registry = Self::empty();

        // Built-in 1: Logic Bricks RustController extension
        let manifest1 = ExtensionManifest::new(
            ExtensionId::new("builtin.logic-bricks.controllers"),
            SemVer::new(0, 92, 0),
            vec![CapabilityDescriptor {
                kind: Capability::Commands,
                description: Some("Built-in RustController evaluators".to_string()),
            }],
            vec![Permission::new(PermissionArea::Commands, PermissionScope::Propose)],
        );
        Self::register_single(&mut registry, manifest1)
            .expect("builtin.logic-bricks.controllers must not duplicate");

        // Built-in 2: Logic Bricks recipes extension
        let manifest2 = ExtensionManifest::new(
            ExtensionId::new("builtin.logic-recipes"),
            SemVer::new(0, 92, 0),
            vec![CapabilityDescriptor {
                kind: Capability::Recipes,
                description: Some(
                    "Built-in recipes: platformer_jump, health_damage, proximity_trigger".to_string(),
                ),
            }],
            vec![Permission::new(PermissionArea::Recipes, PermissionScope::Write)],
        );
        Self::register_single(&mut registry, manifest2)
            .expect("builtin.logic-recipes must not duplicate");

        // Built-in 3: Scene validator extension
        let manifest3 = ExtensionManifest::new(
            ExtensionId::new("builtin.scene-validator"),
            SemVer::new(0, 92, 0),
            vec![CapabilityDescriptor {
                kind: Capability::Validators,
                description: Some("Built-in scene document validator".to_string()),
            }],
            vec![Permission::new(PermissionArea::Project, PermissionScope::Read)],
        );
        Self::register_single(&mut registry, manifest3)
            .expect("builtin.scene-validator must not duplicate");

        registry
    }

    /// Register a single manifest into a registry (used by with_builtins).
    fn register_single(
        registry: &mut ExtensionRegistry,
        manifest: ExtensionManifest,
    ) -> Result<ExtensionHandle, ExtensionError> {
        let id_str = manifest.id.0.clone();
        if registry.manifests.contains_key(&id_str) {
            return Err(ExtensionError::DuplicateId(ExtensionId::new(id_str)));
        }
        let handle = ExtensionHandle::new(registry.next_handle);
        registry.next_handle += 1;
        registry.handles.insert(id_str.clone(), handle);
        registry.manifests.insert(id_str, manifest);
        Ok(handle)
    }

    /// Returns the number of registered extensions.
    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }
}

impl ExtensionRegistryPort for ExtensionRegistry {
    fn register(&mut self, manifest: ExtensionManifest) -> Result<ExtensionHandle, ExtensionError> {
        let id_str = manifest.id.0.clone();
        if self.manifests.contains_key(&id_str) {
            return Err(ExtensionError::DuplicateId(ExtensionId::new(id_str)));
        }
        let handle = ExtensionHandle::new(self.next_handle);
        self.next_handle += 1;
        self.handles.insert(id_str.clone(), handle);
        self.manifests.insert(id_str, manifest);
        Ok(handle)
    }

    fn unregister(&mut self, id: &str) -> Result<(), ExtensionError> {
        if self.manifests.remove(id).is_none() {
            return Err(ExtensionError::NotFound(ExtensionId::new(id.to_string())));
        }
        self.handles.remove(id);
        Ok(())
    }

    fn list(&self) -> Vec<ExtensionSummary> {
        self.manifests.values().map(ExtensionSummary::from).collect()
    }

    fn get(&self, id: &str) -> Option<ExtensionManifest> {
        self.manifests.get(id).cloned()
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::empty()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::extension::{Permission, PermissionArea, PermissionScope, SemVer};
    use editor_model::ports::ExtensionRegistryPort;

    fn make_manifest(id: &str) -> ExtensionManifest {
        ExtensionManifest::new(
            ExtensionId::new(id),
            SemVer::new(0, 92, 0),
            vec![CapabilityDescriptor {
                kind: Capability::Validators,
                description: None,
            }],
            vec![Permission::new(PermissionArea::Validators, PermissionScope::Read)],
        )
    }

    #[test]
    fn register_list_unregister_round_trip() {
        let mut registry = ExtensionRegistry::empty();
        let manifest = make_manifest("com.example.test");
        let handle = registry.register(manifest.clone()).unwrap();

        let summaries = registry.list();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, "com.example.test");

        let fetched = registry.get("com.example.test").unwrap();
        assert_eq!(fetched.id.0, "com.example.test");

        registry.unregister("com.example.test").unwrap();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn duplicate_id_rejected() {
        let mut registry = ExtensionRegistry::empty();
        let manifest = make_manifest("com.example.dup");
        registry.register(manifest).unwrap();
        let err = registry.register(make_manifest("com.example.dup")).unwrap_err();
        assert!(matches!(err, ExtensionError::DuplicateId(_)));
    }

    #[test]
    fn unregister_unknown_id_error() {
        let mut registry = ExtensionRegistry::empty();
        let err = registry.unregister("does-not-exist").unwrap_err();
        assert!(matches!(err, ExtensionError::NotFound(_)));
    }

    #[test]
    fn with_builtins_has_three_entries() {
        let registry = ExtensionRegistry::with_builtins();
        let summaries = registry.list();
        assert_eq!(summaries.len(), 3);

        let ids: Vec<_> = summaries.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"builtin.logic-bricks.controllers"));
        assert!(ids.contains(&"builtin.logic-recipes"));
        assert!(ids.contains(&"builtin.scene-validator"));
    }

    #[test]
    fn with_builtins_preserves_existing_session() {
        // Regression: with_builtins() must not break when called multiple times
        // (each call creates a fresh registry).
        let r1 = ExtensionRegistry::with_builtins();
        let r2 = ExtensionRegistry::with_builtins();
        assert_eq!(r1.list().len(), r2.list().len());
    }

    #[test]
    fn handle_uniqueness() {
        let mut registry = ExtensionRegistry::empty();
        let h1 = registry.register(make_manifest("a")).unwrap();
        let h2 = registry.register(make_manifest("b")).unwrap();
        let h3 = registry.register(make_manifest("c")).unwrap();
        assert_ne!(h1.to_u64(), h2.to_u64());
        assert_ne!(h2.to_u64(), h3.to_u64());
    }
}
