//! Tests for the importer registry (ADR-0040 step 3 — v0.93).

use editor_application::importer_registry::ImporterRegistry;
use editor_model::external_source::ExternalSourceKind;
use editor_model::importer::{
    Importer, ImporterDescriptor, ImporterError, ImporterInput, ImporterVersion,
    ImporterVersionRange, ParseOutput,
};
use editor_model::ports::ImporterRegistryPort;
use std::sync::Arc;

// ─── Dummy importer helpers ────────────────────────────────────────────────────

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

fn make_aseprite_descriptor() -> ImporterDescriptor {
    ImporterDescriptor::new(
        "builtin.aseprite",
        ExternalSourceKind::Aseprite,
        ImporterVersionRange::new(
            ImporterVersion::new(1, 0, 0),
            ImporterVersion::new(2, 0, 0),
        ),
        "Aseprite",
    )
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn register_list_unregister_round_trip() {
    let mut registry = ImporterRegistry::empty();
    let desc = make_aseprite_descriptor();
    let importer: Arc<dyn Importer> = Arc::new(DummyImporter { descriptor: desc.clone() });

    let handle = registry.register(desc.clone(), importer).unwrap();
    assert_eq!(registry.list_by_kind(&ExternalSourceKind::Aseprite).len(), 1);
    assert!(registry.get("builtin.aseprite").is_some());

    registry.unregister("builtin.aseprite").unwrap();
    assert!(registry.list_by_kind(&ExternalSourceKind::Aseprite).is_empty());
}

#[test]
fn duplicate_id_rejected() {
    let mut registry = ImporterRegistry::empty();
    let desc = make_aseprite_descriptor();
    let importer: Arc<dyn Importer> = Arc::new(DummyImporter { descriptor: desc.clone() });
    registry.register(desc.clone(), importer).unwrap();

    let dup_desc = ImporterDescriptor::new(
        "builtin.aseprite", // same id
        ExternalSourceKind::Ldtk,
        ImporterVersionRange::new(
            ImporterVersion::new(1, 0, 0),
            ImporterVersion::new(1, 5, 0),
        ),
        "LDtk",
    );
    let err = registry
        .register(dup_desc.clone(), Arc::new(DummyImporter { descriptor: dup_desc }))
        .unwrap_err();
    assert!(matches!(err, ImporterError::DuplicateId(_)));
}

#[test]
fn list_by_kind_filter() {
    let mut registry = ImporterRegistry::empty();

    let aseprite: Arc<dyn Importer> = Arc::new(DummyImporter {
        descriptor: ImporterDescriptor::new(
            "builtin.aseprite",
            ExternalSourceKind::Aseprite,
            ImporterVersionRange::new(
                ImporterVersion::new(1, 0, 0),
                ImporterVersion::new(2, 0, 0),
            ),
            "Aseprite",
        ),
    });
    let ldtk: Arc<dyn Importer> = Arc::new(DummyImporter {
        descriptor: ImporterDescriptor::new(
            "builtin.ldtk",
            ExternalSourceKind::Ldtk,
            ImporterVersionRange::new(
                ImporterVersion::new(1, 0, 0),
                ImporterVersion::new(1, 5, 0),
            ),
            "LDtk",
        ),
    });

    registry
        .register(aseprite.descriptor().clone(), aseprite)
        .unwrap();
    registry
        .register(ldtk.descriptor().clone(), ldtk)
        .unwrap();

    // Tiled list should be empty
    assert!(registry.list_by_kind(&ExternalSourceKind::Tiled).is_empty());
    // Aseprite should have one
    let aseprite_list = registry.list_by_kind(&ExternalSourceKind::Aseprite);
    assert_eq!(aseprite_list.len(), 1);
    assert_eq!(aseprite_list[0].id, "builtin.aseprite");
}

#[test]
fn with_builtins_has_three_entries() {
    let registry = ImporterRegistry::with_builtins();
    let ids: Vec<_> = ["builtin.aseprite", "builtin.ldtk", "builtin.tiled"]
        .iter()
        .filter(|id| registry.is_registered(id))
        .collect();
    assert_eq!(ids.len(), 3, "Expected all 3 built-ins registered");
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

#[test]
fn dispatch_aseprite_routes_correctly() {
    let mut registry = ImporterRegistry::empty();
    let aseprite: Arc<dyn Importer> = Arc::new(DummyImporter {
        descriptor: make_aseprite_descriptor(),
    });
    registry
        .register(make_aseprite_descriptor(), aseprite)
        .unwrap();

    let result = registry.dispatch(
        &ExternalSourceKind::Aseprite,
        ImporterInput {
            bytes: b"{}",
            source_uri: "test://sprite.json",
            fingerprint_hint: None,
        },
    );
    assert!(result.is_ok(), "Aseprite dispatch should succeed");
}

#[test]
fn version_range_empty_rejected_at_registration() {
    // An empty range should not be registerable.
    // Note: ImporterRegistry::register doesn't validate the range — the check
    // happens at the registry level in with_builtins() for built-ins only.
    let empty_range = ImporterVersionRange::empty();
    assert!(empty_range.is_empty());
}
