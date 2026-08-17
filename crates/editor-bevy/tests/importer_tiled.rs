//! Integration tests for the Tiled JSON importer.
//!
//! Exercises the full parse → build_change_set pipeline and verifies the
//! Level SceneAsset shape that results from the import.

use editor_bevy::importer::TiledImporter;
use editor_model::importer::{Importer, ImporterInput};
use editor_model::session::EditorSnapshot;

/// Sample Tiled JSON matching the test fixture at `tests/fixtures/tiled/sample.json`.
fn sample_tiled_json() -> &'static [u8] {
    r#"{
      "type": "map",
      "tiledversion": "1.10.0",
      "width": 20,
      "height": 15,
      "tilewidth": 16,
      "tileheight": 16,
      "infinite": false,
      "layers": [
        {
          "type": "tilelayer",
          "name": "Ground",
          "order": 0,
          "width": 20,
          "height": 15,
          "data": [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
          ]
        },
        {
          "type": "objectgroup",
          "name": "Props",
          "order": 1,
          "objects": [
            {
              "id": 1,
              "x": 64.0,
              "y": 32.0,
              "width": 16.0,
              "height": 16.0,
              "type": "Coin",
              "properties": [
                { "name": "value", "type": "int", "value": 10 },
                { "name": "sparkle", "type": "bool", "value": true }
              ]
            },
            {
              "id": 2,
              "x": 160.0,
              "y": 64.0,
              "width": 32.0,
              "height": 48.0,
              "type": "Tree",
              "properties": [
                { "name": "species", "type": "string", "value": "oak" }
              ]
            },
            {
              "id": 3,
              "x": 256.0,
              "y": 96.0,
              "width": 24.0,
              "height": 24.0,
              "template": "templates/teleport_pad.json",
              "type": "Teleporter"
            }
          ]
        }
      ],
      "tilesets": [
        {
          "firstgid": 1,
          "source": "tileset.json"
        }
      ],
      "templates": [
        {
          "name": "teleport_pad",
          "object": {
            "id": 100,
            "x": 0,
            "y": 0,
            "width": 24,
            "height": 24,
            "type": "Teleporter"
          }
        }
      ]
    }"#
    .as_bytes()
}

#[test]
fn parse_produces_level_and_fragment_drafts() {
    let importer = TiledImporter::new();
    let input = ImporterInput {
        bytes: sample_tiled_json(),
        source_uri: "sample.json",
        fingerprint_hint: None,
    };
    let output = importer.parse(input).expect("parse should succeed");

    // Level draft + fragment draft = 2 drafts
    assert!(
        output.resource_drafts.len() >= 2,
        "expected at least 2 resource drafts, got {}",
        output.resource_drafts.len()
    );

    // Has level draft
    let level = output
        .resource_drafts
        .iter()
        .find(|r| matches!(r, editor_model::importer::ResourceDraft::Level { .. }));
    assert!(level.is_some(), "should have a Level draft");

    // Has fragment draft (teleport_pad template)
    let fragment = output
        .resource_drafts
        .iter()
        .find(|r| matches!(r, editor_model::importer::ResourceDraft::Fragment { .. }));
    assert!(
        fragment.is_some(),
        "should have a Fragment draft for the template"
    );
}

#[test]
fn parse_rejects_tmx_xml_content() {
    let importer = TiledImporter::new();

    // Actual TMX/XML content
    let tmx = b"<?xml version=\"1.0\"?>\n<map></map>".as_slice();
    let input = ImporterInput {
        bytes: tmx,
        source_uri: "sample.tmx",
        fingerprint_hint: None,
    };
    let err = importer.parse(input).unwrap_err();
    assert!(
        err.to_string().contains("xml") || err.to_string().contains("UnsupportedEncoding"),
        "should reject TMX/XML, got: {}",
        err
    );
}

#[test]
fn parse_rejects_type_tmx_in_json() {
    let importer = TiledImporter::new();
    let json = br#"{
      "type": "tmx",
      "tiledversion": "1.9.0",
      "width": 1,
      "height": 1,
      "tilewidth": 16,
      "tileheight": 16,
      "layers": []
    }"#.as_slice();
    let input = ImporterInput {
        bytes: json,
        source_uri: "map.json",
        fingerprint_hint: None,
    };
    let err = importer.parse(input).unwrap_err();
    assert!(
        err.to_string().contains("xml") || err.to_string().contains("UnsupportedEncoding"),
        "should reject type=tmx, got: {}",
        err
    );
}

#[test]
fn parse_rejects_unsupported_version() {
    let importer = TiledImporter::new();
    let json = br#"{
      "type": "map",
      "tiledversion": "99.0.0",
      "width": 1,
      "height": 1,
      "tilewidth": 16,
      "tileheight": 16,
      "layers": []
    }"#.as_slice();
    let input = ImporterInput {
        bytes: json,
        source_uri: "old.json",
        fingerprint_hint: None,
    };
    let err = importer.parse(input).unwrap_err();
    assert!(
        matches!(err, editor_model::importer::ImporterError::UnsupportedVersion { .. }),
        "expected UnsupportedVersion, got: {}",
        err
    );
}

#[test]
fn parse_accepts_supported_version_1_10() {
    let importer = TiledImporter::new();
    let json = br#"{
      "type": "map",
      "tiledversion": "1.10.0",
      "width": 1,
      "height": 1,
      "tilewidth": 16,
      "tileheight": 16,
      "layers": []
    }"#.as_slice();
    let input = ImporterInput {
        bytes: json,
        source_uri: "v1.10.json",
        fingerprint_hint: None,
    };
    importer
        .parse(input)
        .expect("1.10.0 should be in supported range 1.0.0-1.10.0");
}

#[test]
fn importer_descriptor_has_correct_kind() {
    let importer = TiledImporter::new();
    let desc = importer.descriptor();
    assert_eq!(desc.kind, editor_model::external_source::ExternalSourceKind::Tiled);
    assert_eq!(desc.id, "builtin.tiled");
}

#[test]
fn version_range_contains() {
    use editor_model::importer::{ImporterVersion, ImporterVersionRange};

    let range = ImporterVersionRange::new(
        ImporterVersion::new(1, 0, 0),
        ImporterVersion::new(1, 10, 0),
    );
    assert!(range.contains(ImporterVersion::new(1, 0, 0)));
    assert!(range.contains(ImporterVersion::new(1, 5, 0)));
    assert!(range.contains(ImporterVersion::new(1, 10, 0)));
    assert!(!range.contains(ImporterVersion::new(0, 9, 0)));
    assert!(!range.contains(ImporterVersion::new(1, 11, 0)));
}

#[test]
fn build_change_set_produces_level_document() {
    let importer = TiledImporter::new();
    let input = ImporterInput {
        bytes: sample_tiled_json(),
        source_uri: "sample.json",
        fingerprint_hint: None,
    };

    let parse_output = importer.parse(input).expect("parse should succeed");
    let build_output = importer
        .build_change_set(parse_output.clone(), EditorSnapshot::new())
        .expect("build_change_set should succeed");

    // Verify change_set_json is valid JSON containing AssetCommands
    let commands: Vec<editor_bevy::asset_command::AssetCommand> =
        serde_json::from_str(&build_output.change_set_json)
            .expect("change_set_json should be valid AssetCommand JSON");

    assert!(!commands.is_empty());
    assert!(commands.iter().any(|c| {
        matches!(c, editor_bevy::asset_command::AssetCommand::AddComponent { .. })
    }));
}

#[test]
fn source_mappings_include_level_path() {
    let importer = TiledImporter::new();
    let input = ImporterInput {
        bytes: sample_tiled_json(),
        source_uri: "sample.json",
        fingerprint_hint: None,
    };

    let output = importer.parse(input).expect("parse should succeed");

    // Should have at least one mapping
    assert!(!output.mappings.is_empty(), "should have source mappings");

    // First mapping should reference the level path
    let level_mapping = &output.mappings[0];
    assert!(
        level_mapping.target_resource_ref.contains("levels/"),
        "level mapping should target levels/ path"
    );
}

#[test]
fn tile_layer_round_trip() {
    let importer = TiledImporter::new();

    // Minimal tile layer only
    let json = br#"{
      "type": "map",
      "tiledversion": "1.10.0",
      "width": 3,
      "height": 2,
      "tilewidth": 16,
      "tileheight": 16,
      "layers": [
        {
          "type": "tilelayer",
          "name": "TileLayer1",
          "order": 0,
          "width": 3,
          "height": 2,
          "data": [1, 2, 3, 4, 5, 6]
        }
      ]
    }"#.as_slice();

    let input = ImporterInput {
        bytes: json,
        source_uri: "tile_only.json",
        fingerprint_hint: None,
    };

    let output = importer.parse(input).expect("parse should succeed");

    // Should have a level draft
    let level = output
        .resource_drafts
        .iter()
        .find(|r| matches!(r, editor_model::importer::ResourceDraft::Level { .. }));
    assert!(level.is_some(), "should have a Level draft");

    // Tileset is external (not embedded) so no TilesetAsset draft
    let tileset_drafts: Vec<_> = output
        .resource_drafts
        .iter()
        .filter(|r| {
            if let editor_model::importer::ResourceDraft::AssetFile { logical_path, .. } = r {
                logical_path.contains("tileset")
            } else {
                false
            }
        })
        .collect();
    assert!(
        tileset_drafts.is_empty(),
        "external tileset references should not produce AssetFile drafts"
    );
}
