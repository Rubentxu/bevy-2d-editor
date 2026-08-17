//! Importer registry — concrete implementation of [`editor_model::ports::ImporterRegistryPort`].
//!
//! Lives in `editor_application` (not `editor_model`) because it holds runtime state
//! and is composed into `EditorSession`. Mirrors the `ExtensionRegistry` pattern.

use std::collections::HashMap;
use std::sync::Arc;

use editor_model::external_source::ExternalSourceKind;
use editor_model::importer::{Importer, ImporterDescriptor, ImporterError, ImporterHandle};
use editor_model::ports::ImporterRegistryPort;
use editor_model::importer::{ImporterInput, ParseOutput};

/// In-memory importer registry.
///
/// Stores descriptors by ID and holds `Arc<dyn Importer>` trait objects.
/// Thread-safe: all operations go through `self.0` (the `Arc<Mutex<...>>` wrapping this struct).
///
/// Note: no `Debug` impl because `dyn Importer` is not `Debug`.
pub struct ImporterRegistry {
    /// Maps importer ID → descriptor.
    descriptors: HashMap<String, ImporterDescriptor>,
    /// Maps importer ID → trait object.
    importers: HashMap<String, Arc<dyn Importer>>,
    /// Maps importer ID → handle.
    handles: HashMap<String, ImporterHandle>,
    /// Maps kind → list of importer IDs that handle this kind.
    by_kind: HashMap<String, Vec<String>>,
    /// Next handle value to assign.
    next_handle: u64,
}

impl ImporterRegistry {
    /// Construct an empty registry.
    pub fn empty() -> Self {
        Self {
            descriptors: HashMap::new(),
            importers: HashMap::new(),
            handles: HashMap::new(),
            by_kind: HashMap::new(),
            next_handle: 1,
        }
    }

    /// Construct a registry pre-populated with the three built-in importers.
    ///
    /// Built-in importers (Aseprite, LDtk, Tiled) are registered here so that
    /// `EditorSession::with_builtins` seeds them at session creation.
    ///
    /// The concrete `AsepriteImporter` implementation lives in `editor_core::importer::aseprite`
    /// (only available on wasm32 targets). LDtk and Tiled implementations are added
    /// in PR3 and PR4 respectively.
    #[cfg(target_arch = "wasm32")]
    pub fn with_builtins() -> Self {
        let mut registry = Self::empty();

        // Built-in Aseprite importer — real implementation from editor-core (v0.93 PR2)
        let aseprite_importer = editor_core::importer::AsepriteImporter::new();
        let aseprite_desc = aseprite_importer.descriptor();
        registry
            .register(aseprite_desc, std::sync::Arc::new(aseprite_importer))
            .expect("builtin.aseprite must not duplicate");

        // Built-in LDtk importer — real implementation from editor-core (v0.93 PR3)
        let ldtk_importer = editor_core::importer::LdtkImporter::new();
        let ldtk_desc = ldtk_importer.descriptor();
        registry
            .register(ldtk_desc, std::sync::Arc::new(ldtk_importer))
            .expect("builtin.ldtk must not duplicate");

        // Built-in Tiled importer — placeholder (implemented in PR4)
        let tiled_desc = ImporterDescriptor::new(
            "builtin.tiled",
            ExternalSourceKind::Tiled,
            editor_model::importer::ImporterVersionRange::new(
                editor_model::importer::ImporterVersion::new(1, 0, 0),
                editor_model::importer::ImporterVersion::new(1, 10, 0),
            ),
            "Tiled",
        );
        Self::register_single(&mut registry, tiled_desc)
            .expect("builtin.tiled must not duplicate");

        registry
    }

    /// Construct a registry pre-populated with the three built-in importers (non-wasm32 stub).
    ///
    /// On non-wasm32 targets, editor-core is not available so this creates
    /// a registry with only descriptors (no implementations).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_builtins() -> Self {
        Self::with_builtins_descriptors_only()
    }

    /// Non-wasm32 version that only registers descriptors without implementations.
    fn with_builtins_descriptors_only() -> Self {
        let mut registry = Self::empty();

        // Built-in Aseprite importer descriptor only (implementation requires wasm32)
        let aseprite_desc = ImporterDescriptor::new(
            "builtin.aseprite",
            ExternalSourceKind::Aseprite,
            editor_model::importer::ImporterVersionRange::new(
                editor_model::importer::ImporterVersion::new(1, 0, 0),
                editor_model::importer::ImporterVersion::new(2, 0, 0),
            ),
            "Aseprite",
        );
        Self::register_single(&mut registry, aseprite_desc)
            .expect("builtin.aseprite must not duplicate");

        // Built-in LDtk importer descriptor
        let ldtk_desc = ImporterDescriptor::new(
            "builtin.ldtk",
            ExternalSourceKind::Ldtk,
            editor_model::importer::ImporterVersionRange::new(
                editor_model::importer::ImporterVersion::new(1, 0, 0),
                editor_model::importer::ImporterVersion::new(1, 5, 0),
            ),
            "LDtk",
        );
        Self::register_single(&mut registry, ldtk_desc)
            .expect("builtin.ldtk must not duplicate");

        // Built-in Tiled importer descriptor
        let tiled_desc = ImporterDescriptor::new(
            "builtin.tiled",
            ExternalSourceKind::Tiled,
            editor_model::importer::ImporterVersionRange::new(
                editor_model::importer::ImporterVersion::new(1, 0, 0),
                editor_model::importer::ImporterVersion::new(1, 10, 0),
            ),
            "Tiled",
        );
        Self::register_single(&mut registry, tiled_desc)
            .expect("builtin.tiled must not duplicate");

        registry
    }

    /// Register a single descriptor into a registry (used by with_builtins).
    ///
    /// Does NOT store an importer — call `register_with_importer` to do both.
    fn register_single(
        registry: &mut ImporterRegistry,
        descriptor: ImporterDescriptor,
    ) -> Result<ImporterHandle, ImporterError> {
        let id_str = descriptor.id.clone();
        if registry.descriptors.contains_key(&id_str) {
            return Err(ImporterError::DuplicateId(id_str));
        }
        let handle = ImporterHandle::new(registry.next_handle);
        registry.next_handle += 1;
        registry.handles.insert(id_str.clone(), handle);

        // Index by kind
        let kind_key = kind_key(&descriptor.kind);
        registry
            .by_kind
            .entry(kind_key)
            .or_default()
            .push(id_str.clone());

        registry.descriptors.insert(id_str, descriptor);
        Ok(handle)
    }

    /// Register a descriptor together with its importer implementation.
    pub fn register(
        &mut self,
        descriptor: ImporterDescriptor,
        importer: Arc<dyn Importer>,
    ) -> Result<ImporterHandle, ImporterError> {
        let id_str = descriptor.id.clone();
        if self.descriptors.contains_key(&id_str) {
            return Err(ImporterError::DuplicateId(id_str));
        }
        let handle = ImporterHandle::new(self.next_handle);
        self.next_handle += 1;
        self.handles.insert(id_str.clone(), handle);
        self.importers.insert(id_str.clone(), importer);

        // Index by kind
        let kind_key = kind_key(&descriptor.kind);
        self.by_kind
            .entry(kind_key)
            .or_default()
            .push(id_str.clone());

        self.descriptors.insert(id_str, descriptor);
        Ok(handle)
    }

    /// Returns the number of registered importers.
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }
}

/// Return a stable string key for a kind (used for BTreeMap indexing).
fn kind_key(kind: &ExternalSourceKind) -> String {
    match kind {
        ExternalSourceKind::Aseprite => "aseprite".to_string(),
        ExternalSourceKind::Ldtk => "ldtk".to_string(),
        ExternalSourceKind::Tiled => "tiled".to_string(),
        ExternalSourceKind::Custom(s) => format!("custom:{}", s),
        _ => format!("custom:unknown"),
    }
}

impl ImporterRegistryPort for ImporterRegistry {
    fn register(
        &mut self,
        descriptor: ImporterDescriptor,
        importer: Arc<dyn Importer>,
    ) -> Result<ImporterHandle, ImporterError> {
        ImporterRegistry::register(self, descriptor, importer)
    }

    fn unregister(&mut self, id: &str) -> Result<(), ImporterError> {
        if self.descriptors.remove(id).is_none() {
            return Err(ImporterError::NotFound(id.to_string()));
        }
        self.importers.remove(id);
        self.handles.remove(id);
        // Remove from by_kind index
        for ids in self.by_kind.values_mut() {
            ids.retain(|i| i != id);
        }
        Ok(())
    }

    fn list_by_kind(&self, kind: &ExternalSourceKind) -> Vec<ImporterDescriptor> {
        let key = kind_key(kind);
        self.by_kind
            .get(&key)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.descriptors.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn dispatch(
        &self,
        kind: &ExternalSourceKind,
        source: ImporterInput<'_>,
    ) -> Result<ParseOutput, ImporterError> {
        let candidates = self.list_by_kind(kind);
        if candidates.is_empty() {
            return Err(ImporterError::NoImporterForKind(format!("{:?}", kind)));
        }
        // Use the first registered importer for this kind
        let importer_id = candidates[0].id.clone();
        let importer = self
            .importers
            .get(&importer_id)
            .ok_or_else(|| ImporterError::NotFound(importer_id))?;
        importer.parse(source)
    }

    fn get(&self, id: &str) -> Option<Arc<dyn Importer>> {
        self.importers.get(id).cloned()
    }

    /// Check whether a given importer ID is registered (descriptor only).
    ///
    /// This is used by the transaction kernel's permission check, which only
    /// needs to verify the importer is known — not that it has an active
    /// implementation handle.
    fn is_registered(&self, id: &str) -> bool {
        self.descriptors.contains_key(id)
    }
}

impl Default for ImporterRegistry {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::fmt::Debug for ImporterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImporterRegistry")
            .field("descriptors", &self.descriptors.keys().collect::<Vec<_>>())
            .field("next_handle", &self.next_handle)
            .finish()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::importer::{ImporterVersion, ImporterVersionRange, ParseOutput};
    use editor_model::ports::ImporterRegistryPort;

    /// A trivial importer used only in tests.
    struct DummyImporter {
        descriptor: ImporterDescriptor,
    }

    impl Importer for DummyImporter {
        fn descriptor(&self) -> ImporterDescriptor {
            self.descriptor.clone()
        }

        fn parse(
            &self,
            _source: ImporterInput<'_>,
        ) -> Result<ParseOutput, ImporterError> {
            Ok(ParseOutput::default())
        }

        fn build_change_set(
            &self,
            draft: ParseOutput,
            _snapshot: editor_model::session::EditorSnapshot,
        ) -> Result<editor_model::importer::BuildChangeSetOutput, ImporterError> {
            Ok(editor_model::importer::BuildChangeSetOutput {
                provenance_diff: None,
                change_set_json: serde_json::to_string(&draft).unwrap(),
            })
        }
    }

    fn make_descriptor(id: &str, kind: ExternalSourceKind) -> ImporterDescriptor {
        ImporterDescriptor::new(
            id,
            kind,
            ImporterVersionRange::new(
                ImporterVersion::new(1, 0, 0),
                ImporterVersion::new(2, 0, 0),
            ),
            id,
        )
    }

    #[test]
    fn register_list_unregister_round_trip() {
        let mut registry = ImporterRegistry::empty();
        let desc = make_descriptor("test.importer", ExternalSourceKind::Aseprite);
        let importer: Arc<dyn Importer> =
            Arc::new(DummyImporter { descriptor: desc.clone() });

        let handle = registry.register(desc.clone(), importer).unwrap();
        assert_eq!(registry.list_by_kind(&ExternalSourceKind::Aseprite).len(), 1);
        assert!(registry.get("test.importer").is_some());

        registry.unregister("test.importer").unwrap();
        assert!(registry.list_by_kind(&ExternalSourceKind::Aseprite).is_empty());
    }

    #[test]
    fn duplicate_id_rejected() {
        let mut registry = ImporterRegistry::empty();
        let desc = make_descriptor("dup", ExternalSourceKind::Aseprite);
        let importer: Arc<dyn Importer> =
            Arc::new(DummyImporter { descriptor: desc.clone() });
        registry.register(desc.clone(), importer).unwrap();
        let err = registry
            .register(desc.clone(), Arc::new(DummyImporter { descriptor: make_descriptor("dup", ExternalSourceKind::Ldtk) }))
            .unwrap_err();
        assert!(matches!(err, ImporterError::DuplicateId(_)));
    }

    #[test]
    fn list_by_kind_filter() {
        let mut registry = ImporterRegistry::empty();
        let aseprite: Arc<dyn Importer> = Arc::new(DummyImporter {
            descriptor: make_descriptor("a", ExternalSourceKind::Aseprite),
        });
        let ldtk: Arc<dyn Importer> = Arc::new(DummyImporter {
            descriptor: make_descriptor("b", ExternalSourceKind::Ldtk),
        });
        registry
            .register(make_descriptor("a", ExternalSourceKind::Aseprite), aseprite)
            .unwrap();
        registry
            .register(make_descriptor("b", ExternalSourceKind::Ldtk), ldtk)
            .unwrap();

        assert!(registry
            .list_by_kind(&ExternalSourceKind::Tiled)
            .is_empty());
        assert_eq!(
            registry.list_by_kind(&ExternalSourceKind::Aseprite).len(),
            1
        );
    }

    #[test]
    fn with_builtins_has_three_entries() {
        let registry = ImporterRegistry::with_builtins();
        let ids: Vec<_> = ["builtin.aseprite", "builtin.ldtk", "builtin.tiled"]
            .iter()
            .filter(|id| registry.descriptors.contains_key(*id as &str))
            .collect();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn dispatch_unknown_kind_returns_error() {
        let registry = ImporterRegistry::with_builtins();
        let err = registry
            .dispatch(
                &ExternalSourceKind::Custom("unknown".to_string()),
                ImporterInput {
                    bytes: &[],
                    source_uri: "test://unknown",
                    fingerprint_hint: None,
                },
            )
            .unwrap_err();
        assert!(matches!(err, ImporterError::NoImporterForKind(_)));
    }
}
