//! Integration tests for the Aseprite importer.
//!
//! Exercises the full parse → build_change_set pipeline and verifies the
//! Level SceneAsset shape that results from the import.

use editor_bevy::importer::AsepriteImporter;
use editor_model::importer::{Importer, ImporterInput};
use editor_model::session::EditorSnapshot;

/// Sample Aseprite JSON matching the test fixture at `tests/fixtures/aseprite/sample.json`.
fn sample_aseprite_json() -> &'static [u8] {
    r#"{
      "frames": {
        "player_idle_0.png": { "frame": {"x":0,"y":0,"w":32,"h":32}, "duration": 100 },
        "player_idle_1.png": { "frame": {"x":32,"y":0,"w":32,"h":32}, "duration": 100 },
        "player_idle_2.png": { "frame": {"x":64,"y":0,"w":32,"h":32}, "duration": 100 },
        "player_idle_3.png": { "frame": {"x":96,"y":0,"w":32,"h":32}, "duration": 100 }
      },
      "meta": {
        "version": "1.3-rc12",
        "size": {"w": 128, "h": 32 },
        "frameTags": [{"name": "idle", "from": 0, "to": 3, "direction": "forward"}],
        "layers": [{"name": "Background", "opacity": 255}],
        "slices": [{"name": "Body", "bounds": {"x": 4, "y": 4, "w": 24, "h": 24}}]
      }
    }"#
    .as_bytes()
}

#[test]
fn parse_produces_three_resource_drafts() {
    let importer = AsepriteImporter::new();
    let input = ImporterInput {
        bytes: sample_aseprite_json(),
        source_uri: "player_idle.json",
        fingerprint_hint: None,
    };
    let output = importer.parse(input).expect("parse should succeed");

    // Three drafts: PNG AssetFile, Aseprite JSON AssetFile, Level SceneAsset
    assert_eq!(
        output.resource_drafts.len(),
        3,
        "expected 3 resource drafts"
    );

    // Check each draft type is present
    let has_png = output.resource_drafts.iter().any(|r| {
        matches!(r, editor_model::importer::ResourceDraft::AssetFile { logical_path, .. }
            if logical_path.contains(".png"))
    });
    let has_json = output.resource_drafts.iter().any(|r| {
        matches!(r, editor_model::importer::ResourceDraft::AssetFile { logical_path, .. }
            if logical_path.contains(".aseprite.json"))
    });
    let has_level = output
        .resource_drafts
        .iter()
        .any(|r| matches!(r, editor_model::importer::ResourceDraft::Level { .. }));

    assert!(has_png, "should have PNG AssetFile draft");
    assert!(has_json, "should have Aseprite JSON AssetFile draft");
    assert!(has_level, "should have Level SceneAsset draft");
}

#[test]
fn parse_produces_four_source_mappings() {
    let importer = AsepriteImporter::new();
    let input = ImporterInput {
        bytes: sample_aseprite_json(),
        source_uri: "player_idle.json",
        fingerprint_hint: None,
    };
    let output = importer.parse(input).expect("parse should succeed");

    assert_eq!(output.mappings.len(), 4);
    // All mappings should be SourceOwned
    for m in &output.mappings {
        assert_eq!(
            m.ownership,
            editor_model::external_source::OwnershipRule::SourceOwned
        );
    }
}

#[test]
fn build_change_set_emits_level_document_with_tile_layer() {
    let importer = AsepriteImporter::new();
    let input = ImporterInput {
        bytes: sample_aseprite_json(),
        source_uri: "player_idle.json",
        fingerprint_hint: None,
    };
    let parse_output = importer.parse(input).expect("parse should succeed");

    let build_output = importer
        .build_change_set(parse_output.clone(), EditorSnapshot::new())
        .expect("build_change_set should succeed");

    // Parse the emitted change_set_json as AssetCommands
    let commands: Vec<editor_bevy::asset_command::AssetCommand> =
        serde_json::from_str(&build_output.change_set_json)
            .expect("change_set_json should be valid AssetCommand JSON");

    // Should have a Batch command containing AddComponent commands
    assert!(!commands.is_empty(), "should emit at least one command");
    let batch = commands.iter().find_map(|c| {
        if matches!(c, editor_bevy::asset_command::AssetCommand::Batch { .. }) {
            Some(c)
        } else {
            None
        }
    });
    assert!(batch.is_some(), "should have a Batch command");
}

#[test]
fn level_document_contains_four_tiles() {
    let importer = AsepriteImporter::new();
    let input = ImporterInput {
        bytes: sample_aseprite_json(),
        source_uri: "player_idle.json",
        fingerprint_hint: None,
    };
    let parse_output = importer.parse(input).expect("parse should succeed");

    let build_output = importer
        .build_change_set(parse_output.clone(), EditorSnapshot::new())
        .expect("build_change_set should succeed");

    // Extract the LevelDocument component from the Batch command
    let commands: Vec<editor_bevy::asset_command::AssetCommand> =
        serde_json::from_str(&build_output.change_set_json).unwrap();

    // Find the LevelDocument component value
    let level_doc_json = commands.iter().find_map(|c| {
        if let editor_bevy::asset_command::AssetCommand::Batch {
            commands: batch_cmds,
            ..
        } = c
        {
            batch_cmds.iter().find_map(|cmd| {
                if let editor_bevy::asset_command::AssetCommand::AddComponent {
                    type_id,
                    values,
                    ..
                } = cmd
                {
                    if type_id == "editor.LevelDocument" {
                        Some(values)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        } else {
            None
        }
    });

    let level_doc_json = level_doc_json.expect("should have a LevelDocument component");
    let doc_str = level_doc_json
        .get("document")
        .and_then(|v| v.as_str())
        .expect("document field should be a string");

    let doc: editor_model::scene_asset::SceneAssetDocument =
        serde_json::from_str(doc_str).expect("level document should be valid JSON");

    // Verify it's a Level role
    assert_eq!(doc.role, editor_model::scene_asset::SceneAssetRole::Level);

    // Verify it has one Tile layer with 4 tiles
    assert_eq!(doc.layers.len(), 1, "level should have exactly one layer");
    let tile_layer = &doc.layers[0];
    assert!(
        matches!(tile_layer, editor_model::scene_asset::LevelLayer::Tile(_)),
        "layer should be Tile variant"
    );

    if let editor_model::scene_asset::LevelLayer::Tile(tl) = tile_layer {
        assert_eq!(tl.tile_count(), 4, "tile layer should have 4 tiles");
        assert_eq!(tl.grid_width, 4, "grid width should be 4 (one per frame)");
        assert_eq!(tl.grid_height, 1, "grid height should be 1");
    }
}

#[test]
fn parse_rejects_unsupported_version() {
    let importer = AsepriteImporter::new();
    let old_json = r#"{
      "frames": { "a.png": { "frame": {"x":0,"y":0,"w":16,"h":16}, "duration": 100 } },
      "meta": { "version": "99.0.0", "size": {"w": 16, "h": 16 } }
    }"#
    .as_bytes();
    let input = ImporterInput {
        bytes: old_json,
        source_uri: "old.json",
        fingerprint_hint: None,
    };
    let err = importer.parse(input).unwrap_err();
    assert!(
        matches!(
            err,
            editor_model::importer::ImporterError::UnsupportedVersion { .. }
        ),
        "should reject unsupported version"
    );
}

#[test]
fn parse_accepts_version_1_and_2() {
    let importer = AsepriteImporter::new();

    for version in ["1.0.0", "1.3-rc12", "2.0.0"] {
        let json = format!(
            r#"{{
              "frames": {{ "a.png": {{ "frame": {{"x":0,"y":0,"w":16,"h":16}}, "duration": 100 }} }},
              "meta": {{ "version": "{}", "size": {{"w": 16, "h": 16 }} }}
            }}"#,
            version
        );
        let input = ImporterInput {
            bytes: json.as_bytes(),
            source_uri: "test.json",
            fingerprint_hint: None,
        };
        importer
            .parse(input)
            .expect(&format!("version {} should be accepted", version));
    }
}

#[test]
fn texture_ref_points_to_png() {
    let importer = AsepriteImporter::new();
    let input = ImporterInput {
        bytes: sample_aseprite_json(),
        source_uri: "player_idle.json",
        fingerprint_hint: None,
    };
    let parse_output = importer.parse(input).expect("parse should succeed");

    let build_output = importer
        .build_change_set(parse_output.clone(), EditorSnapshot::new())
        .expect("build_change_set should succeed");

    let commands: Vec<editor_bevy::asset_command::AssetCommand> =
        serde_json::from_str(&build_output.change_set_json).unwrap();

    let texture_ref = commands.iter().find_map(|c| {
        if let editor_bevy::asset_command::AssetCommand::Batch {
            commands: batch_cmds,
            ..
        } = c
        {
            batch_cmds.iter().find_map(|cmd| {
                if let editor_bevy::asset_command::AssetCommand::AddComponent {
                    type_id,
                    values,
                    ..
                } = cmd
                {
                    if type_id == "editor.TextureRef" {
                        Some(values)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        } else {
            None
        }
    });

    let texture_ref = texture_ref.expect("should have a TextureRef component");
    let path = texture_ref.get("path").and_then(|v| v.as_str()).unwrap();
    assert!(
        path.contains(".png"),
        "texture ref should point to PNG file, got: {}",
        path
    );
}
