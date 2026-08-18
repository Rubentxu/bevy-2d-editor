//! Round-trip tests for external source provenance types (ADR-0041 — v0.93).

use editor_model::external_source::{
    ExternalSource, ExternalSourceKind, OwnershipRule, ProvenanceDiff, SourceMapping,
};
use editor_model::importer::ImporterVersion;
use editor_model::time::Timestamp;

// ─── ExternalSourceKind round-trip ────────────────────────────────────────────
//
// ExternalSourceKind uses `#[serde(rename_all = "lowercase")]` (no tag), so
// unit variants serialize as bare strings (e.g., "aseprite") and Custom(s) as
// {"custom": "s"}.

#[test]
fn external_source_kind_round_trip_aseprite() {
    let json = serde_json::to_string(&ExternalSourceKind::Aseprite).unwrap();
    assert_eq!(json, r#""aseprite""#);
    let parsed: ExternalSourceKind = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, ExternalSourceKind::Aseprite);
}

#[test]
fn external_source_kind_round_trip_ldtk() {
    let json = serde_json::to_string(&ExternalSourceKind::Ldtk).unwrap();
    assert_eq!(json, r#""ldtk""#);
    let parsed: ExternalSourceKind = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, ExternalSourceKind::Ldtk);
}

#[test]
fn external_source_kind_round_trip_tiled() {
    let json = serde_json::to_string(&ExternalSourceKind::Tiled).unwrap();
    assert_eq!(json, r#""tiled""#);
    let parsed: ExternalSourceKind = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, ExternalSourceKind::Tiled);
}

#[test]
fn external_source_kind_custom_round_trip() {
    let custom = ExternalSourceKind::Custom("foo".to_string());
    let json = serde_json::to_string(&custom).unwrap();
    // With rename_all = "lowercase" on a newtype variant, Custom("foo") → {"custom": "foo"}
    let parsed: ExternalSourceKind = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, ExternalSourceKind::Custom(s) if s == "foo"));
}

// ─── OwnershipRule round-trip ─────────────────────────────────────────────────

#[test]
fn ownership_rule_round_trip_all_variants() {
    for rule in [
        OwnershipRule::SourceOwned,
        OwnershipRule::EditorOwned,
        OwnershipRule::Mergeable,
        OwnershipRule::Derived,
    ] {
        let json = serde_json::to_string(&rule).unwrap();
        let parsed: OwnershipRule = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, rule, "Round-trip failed for {:?}", rule);
    }
}

// ─── SourceMapping round-trip ─────────────────────────────────────────────────

#[test]
fn source_mapping_round_trip() {
    let mapping = SourceMapping::new(
        "entity_42",
        "actors/goblin.json",
        OwnershipRule::SourceOwned,
    );
    let json = serde_json::to_string(&mapping).unwrap();
    let parsed: SourceMapping = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.source_object_id, "entity_42");
    assert_eq!(parsed.target_resource_ref, "actors/goblin.json");
    assert_eq!(parsed.ownership, OwnershipRule::SourceOwned);
}

#[test]
fn source_mapping_with_local_id_round_trip() {
    use editor_model::ids::LocalId;
    let mut mapping = SourceMapping::new("tile_1", "levels/world1.json", OwnershipRule::Mergeable);
    mapping.target_local_id = Some(LocalId::new("local_abc"));

    let json = serde_json::to_string(&mapping).unwrap();
    let parsed: SourceMapping = serde_json::from_str(&json).unwrap();
    assert!(parsed.target_local_id.is_some());
}

// ─── ProvenanceDiff round-trip ────────────────────────────────────────────────

#[test]
fn provenance_diff_round_trip() {
    let diff = ProvenanceDiff {
        added: vec![SourceMapping::new(
            "e1",
            "actors/a.json",
            OwnershipRule::SourceOwned,
        )],
        removed: vec![],
        modified_source: vec![],
        modified_editor: vec![],
        ownership_conflicts: vec![],
    };

    let json = serde_json::to_string(&diff).unwrap();
    let parsed: ProvenanceDiff = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.added.len(), 1);
    assert_eq!(parsed.added[0].source_object_id, "e1");
    assert!(diff.is_empty() == false);
}

#[test]
fn provenance_diff_is_empty_when_no_changes() {
    let diff = ProvenanceDiff::default();
    assert!(diff.is_empty());
}

// ─── ExternalSource round-trip ────────────────────────────────────────────────

#[test]
fn external_source_round_trip() {
    use editor_model::importer::ImporterVersionRange;

    let source = ExternalSource::new(
        ExternalSourceKind::Ldtk,
        "imports/world.ldtk",
        "abc123def456",
        "builtin.ldtk",
        ImporterVersion::new(1, 3, 0),
        Timestamp::from(1_700_000_000_000u64),
    );

    let json = serde_json::to_string(&source).unwrap();
    let parsed: ExternalSource = serde_json::from_str(&json).unwrap();

    assert!(matches!(parsed.kind, ExternalSourceKind::Ldtk));
    assert_eq!(parsed.source_uri, "imports/world.ldtk");
    assert_eq!(parsed.fingerprint, "abc123def456");
    assert_eq!(parsed.importer_id, "builtin.ldtk");
    assert_eq!(parsed.schema_version, 1);
}

#[test]
fn external_source_with_mappings_round_trip() {
    let mut source = ExternalSource::new(
        ExternalSourceKind::Aseprite,
        "imports/player.aseprite.json",
        "fingerprint123",
        "builtin.aseprite",
        ImporterVersion::new(1, 0, 0),
        Timestamp::from(1_700_000_000_000u64),
    );

    source.mappings.push(SourceMapping::new(
        "frame_0",
        "resources/player.png",
        OwnershipRule::SourceOwned,
    ));
    source.mappings.push(SourceMapping::new(
        "layer_animations",
        "actors/player.json",
        OwnershipRule::SourceOwned,
    ));

    let json = serde_json::to_string(&source).unwrap();
    let parsed: ExternalSource = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.mappings.len(), 2);
    assert_eq!(parsed.mappings[0].source_object_id, "frame_0");
    assert_eq!(parsed.mappings[1].source_object_id, "layer_animations");
}

// The external_source_unknown_kind_is_custom test was removed — unknown kinds
// during deserialization of ExternalSourceKind are handled by the #[non_exhaustive]
// annotation, but serde will reject unknown variants by default. The Custom variant
// handles intentional custom formats in the enum itself.

// ─── ImporterVersion tests ────────────────────────────────────────────────────

#[test]
fn importer_version_comparison() {
    let v1 = ImporterVersion::new(1, 2, 3);
    let v2 = ImporterVersion::new(1, 2, 4);
    assert!(v1 < v2);
}

#[test]
fn importer_version_display() {
    let v = ImporterVersion::new(1, 2, 3);
    assert_eq!(format!("{}", v), "1.2.3");
}

#[test]
fn importer_version_parse() {
    let v = ImporterVersion::parse("1.2.3").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);
}

#[test]
fn importer_version_range_contains() {
    use editor_model::importer::ImporterVersionRange;
    let range =
        ImporterVersionRange::new(ImporterVersion::new(1, 0, 0), ImporterVersion::new(2, 0, 0));
    assert!(range.contains(ImporterVersion::new(1, 3, 0)));
    assert!(range.contains(ImporterVersion::new(2, 0, 0)));
    assert!(!range.contains(ImporterVersion::new(3, 0, 0)));
}
