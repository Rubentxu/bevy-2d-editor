//! Tests for `transaction_kernel_check_importer_permission` (ADR-0040 step 3 — v0.93).

use editor_application::importer_registry::ImporterRegistry;
use editor_application::transaction::{
    ChangeOrigin, ChangeSet, KernelError, transaction_kernel_check_importer_permission,
};
use editor_model::importer::{
    Importer, ImporterDescriptor, ImporterInput, ImporterVersion, ImporterVersionRange, ParseOutput,
};
use editor_model::ports::{ImporterRegistryPort, register_importer_registry};
use std::sync::Arc;

struct DummyImporter {
    descriptor: ImporterDescriptor,
}

impl Importer for DummyImporter {
    fn descriptor(&self) -> ImporterDescriptor {
        self.descriptor.clone()
    }
    fn parse(
        &self,
        _: ImporterInput<'_>,
    ) -> Result<ParseOutput, editor_model::importer::ImporterError> {
        Ok(ParseOutput::default())
    }
    fn build_change_set(
        &self,
        draft: ParseOutput,
        _: editor_model::session::EditorSnapshot,
    ) -> Result<editor_model::importer::BuildChangeSetOutput, editor_model::importer::ImporterError>
    {
        Ok(editor_model::importer::BuildChangeSetOutput {
            provenance_diff: None,
            change_set_json: serde_json::to_string(&draft).unwrap(),
        })
    }
}

fn register_test_registry() {
    let registry: Arc<Mutex<dyn ImporterRegistryPort>> =
        Arc::new(Mutex::new(ImporterRegistry::with_builtins()));
    let _ = register_importer_registry(registry);
}

// We need Mutex in scope for the Arc type alias.
use std::sync::Mutex;

#[test]
fn importer_permission_check_no_op_for_human_origin() {
    let mut cs: ChangeSet<String> = ChangeSet::new(
        "cs1".into(),
        ChangeOrigin::Human,
        "user".into(),
        "test".into(),
    );
    cs.add_resource("scene", "test.json");
    assert!(transaction_kernel_check_importer_permission(&cs).is_ok());
}

#[test]
fn importer_permission_check_no_op_for_agent_origin() {
    let mut cs: ChangeSet<String> = ChangeSet::new(
        "cs2".into(),
        ChangeOrigin::Agent,
        "agent:code-writer".into(),
        "test".into(),
    );
    cs.add_resource("scene", "test.json");
    assert!(transaction_kernel_check_importer_permission(&cs).is_ok());
}

#[test]
fn importer_permission_check_no_op_for_plugin_origin() {
    let mut cs: ChangeSet<String> = ChangeSet::new(
        "cs3".into(),
        ChangeOrigin::Plugin,
        "extension:builtin.test".into(),
        "test".into(),
    );
    cs.add_resource("scene", "test.json");
    // Plugin origin should pass through without checking the importer registry
    assert!(transaction_kernel_check_importer_permission(&cs).is_ok());
}

#[test]
fn importer_permission_denied_for_unknown_importer() {
    register_test_registry();

    let mut cs: ChangeSet<String> = ChangeSet::new(
        "cs4".into(),
        ChangeOrigin::Importer,
        "importer:builtin.unknown".into(),
        "test".into(),
    );
    cs.add_resource("scene", "test.json");

    let err = transaction_kernel_check_importer_permission(&cs).unwrap_err();
    assert!(matches!(err, KernelError::PermissionDenied { .. }));
}

#[test]
fn importer_permission_denied_for_missing_prefix() {
    let mut cs: ChangeSet<String> = ChangeSet::new(
        "cs5".into(),
        ChangeOrigin::Importer,
        "builtin.aseprite".into(), // missing "importer:" prefix
        "test".into(),
    );
    cs.add_resource("scene", "test.json");

    let err = transaction_kernel_check_importer_permission(&cs).unwrap_err();
    assert!(matches!(err, KernelError::PermissionDenied { .. }));
}

#[test]
fn importer_permission_allowed_for_builtin_aseprite() {
    register_test_registry();

    let mut cs: ChangeSet<String> = ChangeSet::new(
        "cs6".into(),
        ChangeOrigin::Importer,
        "importer:builtin.aseprite".into(),
        "test".into(),
    );
    cs.add_resource("scene", "test.json");

    assert!(transaction_kernel_check_importer_permission(&cs).is_ok());
}

#[test]
fn importer_permission_allowed_for_builtin_ldtk() {
    register_test_registry();

    let mut cs: ChangeSet<String> = ChangeSet::new(
        "cs7".into(),
        ChangeOrigin::Importer,
        "importer:builtin.ldtk".into(),
        "test".into(),
    );
    cs.add_resource("scene", "test.json");

    assert!(transaction_kernel_check_importer_permission(&cs).is_ok());
}

#[test]
fn importer_permission_allowed_for_builtin_tiled() {
    register_test_registry();

    let mut cs: ChangeSet<String> = ChangeSet::new(
        "cs8".into(),
        ChangeOrigin::Importer,
        "importer:builtin.tiled".into(),
        "test".into(),
    );
    cs.add_resource("scene", "test.json");

    assert!(transaction_kernel_check_importer_permission(&cs).is_ok());
}
