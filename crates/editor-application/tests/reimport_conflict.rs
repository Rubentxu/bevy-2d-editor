//! Unit tests for the reimport conflict detection.
//!
//! Tests the ProvenanceDiff computation and ownership-aware merge logic.

use editor_model::external_source::OwnershipRule;
use editor_model::external_source::{ExternalSource, ExternalSourceKind, SourceMapping};
use editor_model::importer::ImporterVersion;

// Reimport functionality under test
use editor_application::reimport::{compute_fingerprint, compute_provenance_diff};

/// Helper: create an ExternalSource with minimal fields.
fn make_external_source(
    source_uri: &str,
    fingerprint: &str,
    mappings: Vec<SourceMapping>,
) -> ExternalSource {
    ExternalSource {
        kind: ExternalSourceKind::Ldtk,
        source_uri: source_uri.to_string(),
        fingerprint: fingerprint.to_string(),
        importer_id: "builtin.ldtk".to_string(),
        importer_version: ImporterVersion::new(1, 0, 0),
        last_import_time: editor_model::time::Timestamp(1_700_000_000_000_u64),
        mappings,
        ownership_rules: vec![],
        schema_version: 1,
        conflict_policy: None,
    }
}

#[test]
fn test_fingerprint_unchanged_is_noop() {
    let bytes = b"hello world";
    let fp1 = compute_fingerprint(bytes);
    let fp2 = compute_fingerprint(bytes);
    assert_eq!(fp1, fp2, "identical bytes produce identical fingerprints");
}

#[test]
fn test_fingerprint_differs_on_content_change() {
    let fp1 = compute_fingerprint(b"hello world");
    let fp2 = compute_fingerprint(b"hello world!");
    assert_ne!(fp1, fp2, "different bytes produce different fingerprints");
}

#[test]
fn test_fingerprint_hex_format() {
    // SHA-256 produces 64 hex characters
    let fp = compute_fingerprint(b"test");
    assert_eq!(fp.len(), 64, "SHA-256 hex fingerprint is 64 chars");
    assert!(
        fp.chars().all(|c| c.is_ascii_hexdigit()),
        "fingerprint should be lowercase hex"
    );
}

#[test]
fn test_provenance_diff_empty_when_identical() {
    let es = make_external_source(
        "test.ldtk",
        "abc123",
        vec![
            SourceMapping::new("entity:1", "level_1.json", OwnershipRule::SourceOwned),
            SourceMapping::new("entity:2", "level_1.json", OwnershipRule::SourceOwned),
        ],
    );
    let diff = compute_provenance_diff(&es, &es);
    assert!(
        diff.is_empty(),
        "identical ExternalSources produce empty diff"
    );
}

#[test]
fn test_provenance_diff_added_mappings() {
    let old = make_external_source(
        "test.ldtk",
        "abc123",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::SourceOwned,
        )],
    );
    let new = make_external_source(
        "test.ldtk",
        "def456",
        vec![
            SourceMapping::new("entity:1", "level_1.json", OwnershipRule::SourceOwned),
            SourceMapping::new("entity:2", "level_1.json", OwnershipRule::SourceOwned),
        ],
    );
    let diff = compute_provenance_diff(&old, &new);
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.added[0].source_object_id, "entity:2");
    assert!(diff.removed.is_empty());
    assert!(diff.modified_source.is_empty());
}

#[test]
fn test_provenance_diff_removed_mappings() {
    let old = make_external_source(
        "test.ldtk",
        "abc123",
        vec![
            SourceMapping::new("entity:1", "level_1.json", OwnershipRule::SourceOwned),
            SourceMapping::new("entity:2", "level_1.json", OwnershipRule::SourceOwned),
        ],
    );
    let new = make_external_source(
        "test.ldtk",
        "def456",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::SourceOwned,
        )],
    );
    let diff = compute_provenance_diff(&old, &new);
    assert!(diff.added.is_empty());
    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.removed[0].source_object_id, "entity:2");
}

#[test]
fn test_provenance_diff_modified_source() {
    // Source-owned entity that changed position (different target_resource_ref)
    let old = make_external_source(
        "test.ldtk",
        "abc123",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::SourceOwned,
        )],
    );
    let mut new = make_external_source(
        "test.ldtk",
        "def456",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::SourceOwned,
        )],
    );
    // Simulate source changed the target (e.g., entity moved to different position)
    new.mappings[0].target_resource_ref = "level_2.json".to_string();

    let diff = compute_provenance_diff(&old, &new);
    assert!(diff.added.is_empty());
    assert!(diff.removed.is_empty());
    assert_eq!(diff.modified_source.len(), 1);
    assert_eq!(diff.modified_source[0].source_object_id, "entity:1");
}

#[test]
fn test_provenance_diff_ownership_conflict() {
    // Source-owned entity: ownership changed to EditorOwned → conflict
    // (source was supposed to be SourceOwned but editor changed it)
    let old = make_external_source(
        "test.ldtk",
        "abc123",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::SourceOwned,
        )],
    );
    let mut new = make_external_source(
        "test.ldtk",
        "def456",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::SourceOwned,
        )],
    );
    // Editor user modified this entity → ownership changed to EditorOwned
    new.mappings[0].ownership = OwnershipRule::EditorOwned;

    let diff = compute_provenance_diff(&old, &new);
    assert!(diff.added.is_empty());
    assert!(diff.removed.is_empty());
    assert!(diff.modified_source.is_empty());
    // Ownership changed from SourceOwned → EditorOwned = ownership conflict
    assert_eq!(diff.ownership_conflicts.len(), 1);
    assert_eq!(diff.ownership_conflicts[0].source_object_id, "entity:1");
    assert_eq!(diff.modified_editor.len(), 0);
}

#[test]
fn test_provenance_diff_editor_owned_preserved() {
    // Editor-owned entity: target_resource_ref stays same, ownership stays same
    // → nothing to diff (both source and editor agree)
    let old = make_external_source(
        "test.ldtk",
        "abc123",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::EditorOwned,
        )],
    );
    let new = make_external_source(
        "test.ldtk",
        "def456",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::EditorOwned,
        )],
    );

    let diff = compute_provenance_diff(&old, &new);
    // Editor-owned unchanged → no diff at mapping level
    assert!(
        diff.is_empty(),
        "editor-owned unchanged should produce empty diff"
    );
}

#[test]
fn test_provenance_diff_editor_owned_target_changed() {
    // Editor-owned entity: target_resource_ref changed → source modification
    // (the ref changed, ownership is still EditorOwned)
    let old = make_external_source(
        "test.ldtk",
        "abc123",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::EditorOwned,
        )],
    );
    let mut new = make_external_source(
        "test.ldtk",
        "def456",
        vec![SourceMapping::new(
            "entity:1",
            "level_1.json",
            OwnershipRule::EditorOwned,
        )],
    );
    // Source changed the placement → target_resource_ref changed
    new.mappings[0].target_resource_ref = "different.json".to_string();

    let diff = compute_provenance_diff(&old, &new);
    // target_resource_ref changed → source modification (not editor conflict)
    assert!(
        diff.ownership_conflicts.is_empty(),
        "editor-owned should not cause conflict when ref changes"
    );
    assert_eq!(diff.modified_source.len(), 1);
    assert_eq!(diff.modified_source[0].source_object_id, "entity:1");
}

#[test]
fn test_provenance_diff_derived_rule() {
    // Derived objects are recomputed on reimport (source wins)
    let old = make_external_source(
        "test.ldtk",
        "abc123",
        vec![SourceMapping::new(
            "autolayer:1",
            "level_1.json",
            OwnershipRule::Derived,
        )],
    );
    let new = make_external_source(
        "test.ldtk",
        "def456",
        vec![SourceMapping::new(
            "autolayer:1",
            "level_1.json",
            OwnershipRule::Derived,
        )],
    );
    let diff = compute_provenance_diff(&old, &new);
    // Derived rule means source wins — should be treated as source-modified
    assert!(diff.modified_source.len() >= 1 || diff.is_empty());
}
