//! Integration tests for the LDtk world-bridge feature (ADR-0037 v0.95).
//!
//! Tests the importer's ability to produce `WorldDocument` commands when
//! an LDtkWorld contains ≥2 levels.

use editor_model::importer::{Importer, ImporterInput};
use editor_model::session::EditorSnapshot;

/// Load a fixture file from the fixtures directory.
fn load_fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/crates/editor-bevy/tests/fixtures/ldtk/{}",
        env!("CARGO_MANIFEST_DIR").replace("/crates/editor-bevy", ""),
        name
    );
    std::fs::read(&path).unwrap_or_else(|_| panic!("fixture not found: {}", path))
}

fn make_importer() -> impl Importer {
    // Use fully-qualified path to construct the importer
    editor_bevy::importer::ldtk::LdtkImporter::new()
}

/// Parse the change_set_json as a Vec of mixed JSON values (AssetCommand + WorldCommand).
fn parse_change_set(change_set_json: &str) -> Vec<serde_json::Value> {
    serde_json::from_str(change_set_json).expect("change_set_json should be valid JSON")
}

/// Count commands of a specific type in the change set.
fn count_commands(change_set_json: &str, type_name: &str) -> usize {
    let commands = parse_change_set(change_set_json);
    commands
        .iter()
        .filter(|c| c.pointer("/type").and_then(|t| t.as_str()) == Some(type_name))
        .count()
}

/// Returns all commands of a specific type.
fn get_commands(change_set_json: &str, type_name: &str) -> Vec<serde_json::Value> {
    let commands = parse_change_set(change_set_json);
    commands
        .into_iter()
        .filter(|c| c.pointer("/type").and_then(|t| t.as_str()) == Some(type_name))
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: 3-level world emits WorldDocument commands
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_ldtk_world_with_3_levels_emits_world_document() {
    let importer = make_importer();
    let bytes = load_fixture("sample_world.ldtk");
    let input = ImporterInput {
        bytes: &bytes,
        source_uri: "sample_world.ldtk",
        fingerprint_hint: None,
    };

    let parse_output = importer.parse(input).expect("parse should succeed");
    let build_output = importer
        .build_change_set(parse_output, EditorSnapshot::new())
        .expect("build_change_set should succeed");

    let json = &build_output.change_set_json;

    // WorldCreate should be emitted once
    assert_eq!(
        count_commands(json, "WorldCreate"),
        1,
        "expected 1 WorldCreate command for 3-level world"
    );

    // WorldPlaceLevel should be emitted for each of the 3 levels
    assert_eq!(
        count_commands(json, "WorldPlaceLevel"),
        3,
        "expected 3 WorldPlaceLevel commands (one per level)"
    );

    // WorldConnectLevels: 6 links total
    // - 4 from LDtk __neighbours (Room_0→1 east, Room_1→0 west, Room_1→2 south, Room_2→1 north)
    // - 2 from create_room_chain recipe (lvl_0→lvl_1 east, lvl_1→lvl_2 east)
    assert_eq!(
        count_commands(json, "WorldConnectLevels"),
        6,
        "expected 6 WorldConnectLevels commands (4 from neighbours + 2 from chain recipe)"
    );

    // WorldSave should be emitted once
    assert_eq!(
        count_commands(json, "WorldSave"),
        1,
        "expected 1 WorldSave command"
    );

    // The WorldCreate should have the correct name
    let create_cmds = get_commands(json, "WorldCreate");
    assert_eq!(create_cmds.len(), 1);
    assert_eq!(
        create_cmds[0].pointer("/name").and_then(|v| v.as_str()),
        Some("SampleWorld"),
        "WorldCreate.name should be SampleWorld"
    );
    assert_eq!(
        create_cmds[0]
            .pointer("/layout_policy/kind")
            .and_then(|v| v.as_str()),
        Some("free"),
        "WorldCreate.layout_policy should be free"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: 1-level world emits NO WorldDocument commands
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_ldtk_world_with_1_level_emits_no_world() {
    let importer = make_importer();
    let bytes = load_fixture("single_level.ldtk");
    let input = ImporterInput {
        bytes: &bytes,
        source_uri: "single_level.ldtk",
        fingerprint_hint: None,
    };

    let parse_output = importer.parse(input).expect("parse should succeed");
    let build_output = importer
        .build_change_set(parse_output, EditorSnapshot::new())
        .expect("build_change_set should succeed");

    let json = &build_output.change_set_json;

    // No world commands should be emitted for a 1-level world
    assert_eq!(
        count_commands(json, "WorldCreate"),
        0,
        "expected NO WorldCreate for single-level world"
    );
    assert_eq!(
        count_commands(json, "WorldPlaceLevel"),
        0,
        "expected NO WorldPlaceLevel for single-level world"
    );
    assert_eq!(
        count_commands(json, "WorldConnectLevels"),
        0,
        "expected NO WorldConnectLevels for single-level world"
    );
    assert_eq!(
        count_commands(json, "WorldSave"),
        0,
        "expected NO WorldSave for single-level world"
    );

    // But per-level AssetCommands should still be present
    assert!(
        count_commands(json, "AddComponent") >= 1,
        "expected at least 1 AddComponent for the single level"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: change_set is replayable (parses back deterministically)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_ldtk_world_change_set_is_replayable() {
    let importer = make_importer();
    let bytes = load_fixture("sample_world.ldtk");
    let input = ImporterInput {
        bytes: &bytes,
        source_uri: "sample_world.ldtk",
        fingerprint_hint: None,
    };

    let parse_output = importer.parse(input).expect("parse should succeed");
    let build_output = importer
        .build_change_set(parse_output, EditorSnapshot::new())
        .expect("build_change_set should succeed");

    let json1 = &build_output.change_set_json;

    // Should parse back to a valid JSON array
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(json1).expect("change_set_json should be valid JSON");

    // Should have non-zero commands
    assert!(!parsed.is_empty(), "change_set should not be empty");

    // Re-serialize and parse again — should be identical
    let json2 = serde_json::to_string(&parsed).expect("should serialize back");
    let parsed2: Vec<serde_json::Value> =
        serde_json::from_str(&json2).expect("re-parsed JSON should be valid");

    assert_eq!(
        parsed.len(),
        parsed2.len(),
        "command count should be stable across re-serialization"
    );

    // All commands should have a "type" field
    for cmd in &parsed2 {
        assert!(
            cmd.get("type").is_some(),
            "every command should have a 'type' field: {:?}",
            cmd
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: golden file match
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_ldtk_world_with_3_levels_golden_match() {
    let importer = make_importer();
    let bytes = load_fixture("sample_world.ldtk");
    let input = ImporterInput {
        bytes: &bytes,
        source_uri: "sample_world.ldtk",
        fingerprint_hint: None,
    };

    let parse_output = importer.parse(input).expect("parse should succeed");
    let build_output = importer
        .build_change_set(parse_output, EditorSnapshot::new())
        .expect("build_change_set should succeed");

    let golden_path = format!(
        "{}/crates/editor-bevy/tests/fixtures/ldtk/expected_world.json",
        env!("CARGO_MANIFEST_DIR").replace("/crates/editor-bevy", ""),
    );
    let golden = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|_| panic!("golden file not found: {}", golden_path));

    // Parse both as JSON values for whitespace-insensitive comparison
    let actual: serde_json::Value = serde_json::from_str(&build_output.change_set_json)
        .expect("change_set_json should be valid JSON");
    let expected: serde_json::Value =
        serde_json::from_str(&golden).expect("golden file should be valid JSON");

    assert_eq!(
        actual, expected,
        "change_set_json should match expected_world.json"
    );
}
