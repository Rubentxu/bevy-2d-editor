//! Aseprite JSON importer for the Bevy 2D Editor.
//!
//! Parses Aseprite JSON export format and produces a Level `SceneAssetDocument`
//! with `LevelLayer::Tile` entries (one tile per frame), plus an `AssetFile`
//! for the PNG texture and provenance sidecar.
//!
//! ## Aseprite JSON format
//!
//! The Aseprite JSON has the structure:
//! ```json
//! {
//!   "frames": { "<name>.png": { "frame": {...}, "duration": ms } },
//!   "meta": {
//!     "size": { "w": width, "h": height },
//!     "frameTags": [{ "name": "idle", "from": 0, "to": 3 }],
//!     "slices": [{ "name": "Body", "bounds": {...} }]
//!   }
//! }
//! ```
//!
//! ## Design decisions (v0.93 PR2)
//!
//! - Output is a Level `SceneAssetDocument` (per spec §3 decision #4).
//! - One tile per Aseprite frame (frame index = tile index).
//! - PNG texture goes to `resources/<basename>.png` with `AssetFile { kind: Texture }`.
//! - Aseprite JSON is persisted to `resources/<basename>.aseprite.json`.
//! - Sidecar `.meta.json` records `ExternalSource` provenance for both.

use base64::Engine;
use editor_model::TilesetId;
use editor_model::external_source::{ExternalSourceKind, OwnershipRule, SourceMapping};
use editor_model::importer::{
    BuildChangeSetOutput, Importer, ImporterDescriptor, ImporterError, ImporterInput,
    ImporterVersion, ImporterVersionRange, ParseOutput, ResourceDraft,
};
use editor_model::scene_asset::{LevelLayer, SceneAssetRole};
use editor_model::session::EditorSnapshot;
use editor_model::tile_layer::{TileLayer, TileLayerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Aseprite IR (intermediate representation)
// ─────────────────────────────────────────────────────────────────────────────

/// Parsed Aseprite JSON structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AsepriteIr {
    /// Parsed frames in source order.
    frames: Vec<AsepriteFrameIr>,
    /// Animation tags.
    #[serde(default)]
    tags: Vec<AsepriteTagIr>,
    /// Named slices.
    #[serde(default)]
    slices: Vec<AsepriteSliceIr>,
    /// Canvas size in pixels.
    size_w: u32,
    size_h: u32,
    /// Detected Aseprite format version string.
    version: String,
    /// Image basename (without extension) derived from the first frame name.
    image_basename: String,
    /// Number of frames.
    frame_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AsepriteFrameIr {
    /// Frame name as exported (e.g. "player_idle_0.png").
    name: String,
    /// Frame bounds in the spritesheet.
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    /// Duration in milliseconds.
    duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AsepriteTagIr {
    name: String,
    from: u32,
    to: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AsepriteSliceIr {
    name: String,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

/// Raw JSON frame entry from Aseprite JSON.
#[derive(Debug, Deserialize)]
struct JsonFrame {
    frame: JsonRect,
    #[serde(default)]
    duration: u32,
}

#[derive(Debug, Deserialize)]
struct JsonRect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

/// Raw JSON meta envelope.
#[derive(Debug, Deserialize)]
struct JsonMeta {
    version: Option<String>,
    size: JsonSize,
    #[serde(default)]
    frame_tags: Vec<JsonFrameTag>,
    #[serde(default)]
    slices: Vec<JsonSlice>,
}

#[derive(Debug, Deserialize)]
struct JsonSize {
    w: u32,
    h: u32,
}

#[derive(Debug, Deserialize)]
struct JsonFrameTag {
    name: String,
    from: u32,
    to: u32,
    #[serde(default)]
    direction: String,
}

#[derive(Debug, Deserialize)]
struct JsonSlice {
    name: String,
    #[serde(default)]
    bounds: Option<JsonRect>,
    #[serde(default)]
    color: Option<String>,
}

/// Raw top-level Aseprite JSON.
#[derive(Debug, Deserialize)]
struct AsepriteJson {
    frames: HashMap<String, JsonFrame>,
    meta: JsonMeta,
}

// ─────────────────────────────────────────────────────────────────────────────
// AsepriteImporter
// ─────────────────────────────────────────────────────────────────────────────

/// Built-in Aseprite JSON importer.
///
/// Handles Aseprite `.json` + `.png` pairs exported from Aseprite.
/// Version range: 1.0.0 – 2.0.0 (covers all known Aseprite JSON export formats).
#[derive(Debug)]
pub struct AsepriteImporter {
    descriptor: ImporterDescriptor,
}

impl AsepriteImporter {
    /// Construct a new AsepriteImporter.
    pub fn new() -> Self {
        Self {
            descriptor: ImporterDescriptor::new(
                "builtin.aseprite",
                ExternalSourceKind::Aseprite,
                ImporterVersionRange::new(
                    ImporterVersion::new(1, 0, 0),
                    ImporterVersion::new(2, 0, 0),
                ),
                "Aseprite",
            ),
        }
    }

    /// Parse Aseprite JSON bytes into an intermediate representation.
    ///
    /// Returns `Err(ImporterError::ParseError)` if the JSON is malformed.
    /// Returns `Err(ImporterError::UnsupportedVersion)` if the version is out of range.
    fn parse_json(&self, bytes: &[u8]) -> Result<AsepriteIr, ImporterError> {
        let json: AsepriteJson = serde_json::from_slice(bytes)
            .map_err(|e| ImporterError::ParseError(format!("invalid Aseprite JSON: {}", e)))?;

        // meta envelope is required
        let meta = json.meta;
        if meta.size.w == 0 || meta.size.h == 0 {
            return Err(ImporterError::ParseError(
                "missing or invalid 'meta.size' in Aseprite JSON".to_string(),
            ));
        }

        // Version check (parse the detected version)
        let detected_version = meta.version.as_deref().unwrap_or("1.0.0").to_string();
        let detected =
            ImporterVersion::parse(&detected_version).unwrap_or(ImporterVersion::new(1, 0, 0));

        if !self.descriptor.supported_versions.contains(detected) {
            return Err(ImporterError::UnsupportedVersion {
                detected,
                supported_min: self.descriptor.supported_versions.min,
                supported_max: self.descriptor.supported_versions.max,
            });
        }

        // Collect frames in sorted order (sorted by name, which is the convention)
        let mut frame_names: Vec<_> = json.frames.keys().collect();
        frame_names.sort();

        let frames: Vec<AsepriteFrameIr> = frame_names
            .into_iter()
            .filter_map(|name| {
                let jf = json.frames.get(name)?;
                Some(AsepriteFrameIr {
                    name: name.clone(),
                    x: jf.frame.x,
                    y: jf.frame.y,
                    w: jf.frame.w,
                    h: jf.frame.h,
                    duration_ms: if jf.duration == 0 { 100 } else { jf.duration },
                })
            })
            .collect();

        let frame_count = frames.len();

        // Derive image_basename from first frame name (e.g. "player_idle_0.png" -> "player_idle")
        let image_basename = frames
            .first()
            .and_then(|f| {
                std::path::Path::new(&f.name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| {
                        // Strip trailing frame index underscore pattern
                        let stem = s.to_string();
                        // e.g. "player_idle_0" -> "player_idle"
                        stem.trim_end_matches(|c: char| c.is_ascii_digit())
                            .trim_end_matches('_')
                            .to_string()
                    })
            })
            .unwrap_or_else(|| "untitled".to_string());

        let tags: Vec<AsepriteTagIr> = meta
            .frame_tags
            .into_iter()
            .map(|t| AsepriteTagIr {
                name: t.name,
                from: t.from,
                to: t.to,
            })
            .collect();

        let slices: Vec<AsepriteSliceIr> = meta
            .slices
            .into_iter()
            .filter_map(|s| {
                let bounds = s.bounds?;
                Some(AsepriteSliceIr {
                    name: s.name,
                    x: bounds.x,
                    y: bounds.y,
                    w: bounds.w,
                    h: bounds.h,
                })
            })
            .collect();

        Ok(AsepriteIr {
            frames,
            tags,
            slices,
            size_w: meta.size.w,
            size_h: meta.size.h,
            version: detected_version,
            image_basename,
            frame_count,
        })
    }
}

impl Importer for AsepriteImporter {
    fn descriptor(&self) -> ImporterDescriptor {
        self.descriptor.clone()
    }

    fn parse(&self, source: ImporterInput<'_>) -> Result<ParseOutput, ImporterError> {
        let ir = self.parse_json(source.bytes)?;
        let image_basename = &ir.image_basename;
        let frame_count = ir.frame_count;
        let ownership = OwnershipRule::SourceOwned;

        let mut resource_drafts = Vec::new();
        let mut mappings = Vec::new();

        // ── Resource draft: PNG texture (AssetFile) ─────────────────────────
        let png_path = format!("resources/{}.png", image_basename);
        resource_drafts.push(ResourceDraft::AssetFile {
            logical_path: png_path.clone(),
            bytes_b64: None, // PNG bytes come separately in import_external_source_wasm
        });
        mappings.push(SourceMapping::new(
            format!("{}.png", image_basename),
            png_path.clone(),
            ownership.clone(),
        ));

        // ── Resource draft: Aseprite JSON (stored alongside for round-trip) ──
        let aseprite_json_path = format!("resources/{}.aseprite.json", image_basename);
        resource_drafts.push(ResourceDraft::AssetFile {
            logical_path: aseprite_json_path.clone(),
            bytes_b64: Some(base64::engine::general_purpose::STANDARD.encode(source.bytes)),
        });
        mappings.push(SourceMapping::new(
            format!("{}.aseprite.json", image_basename),
            aseprite_json_path.clone(),
            ownership.clone(),
        ));

        // ── Resource draft: Level SceneAssetDocument ─────────────────────────
        let level_path = format!("levels/{}", image_basename);
        // Encode frame count in display_name so build_change_set can recover it.
        // Format: "Basename (N frames)"
        let display_name = format!("{} ({} frames)", image_basename, frame_count);
        resource_drafts.push(ResourceDraft::Level {
            logical_path: level_path.clone(),
            display_name: Some(display_name),
        });

        // Mapping for the level document
        mappings.push(SourceMapping::new(
            format!("{}/level", image_basename),
            level_path.clone(),
            ownership.clone(),
        ));

        // Mapping for the world tile data
        mappings.push(SourceMapping::new(
            format!("{}/tile_data", image_basename),
            level_path.clone(),
            ownership.clone(),
        ));

        Ok(ParseOutput {
            resource_drafts,
            mappings,
            ownership_rules: vec![ownership],
            detected_version: Some(ir.version.clone()),
            detected_version_parsed: ImporterVersion::parse(&ir.version),
        })
    }

    fn build_change_set(
        &self,
        draft: ParseOutput,
        _snapshot: EditorSnapshot,
    ) -> Result<BuildChangeSetOutput, ImporterError> {
        use crate::asset_command::AssetCommand;

        // Find the level draft
        let level_draft = draft
            .resource_drafts
            .iter()
            .find(|r| matches!(r, ResourceDraft::Level { .. }))
            .ok_or_else(|| {
                ImporterError::ParseError("no Level draft found in ParseOutput".to_string())
            })?;

        let (level_path, display_name) = match level_draft {
            ResourceDraft::Level {
                logical_path,
                display_name,
            } => (logical_path.clone(), display_name.clone()),
            _ => unreachable!(),
        };

        // Extract image_basename and frame_count from display_name
        // Format: "Basename (N frames)"
        let (image_basename, frame_count) = display_name
            .as_ref()
            .ok_or_else(|| {
                ImporterError::ParseError("Level draft missing display_name".to_string())
            })?
            .strip_suffix(" frames)")
            .and_then(|s| s.rsplit_once(" ("))
            .map(|(basename, count_str)| {
                (
                    basename.to_string(),
                    count_str.parse::<usize>().unwrap_or(1),
                )
            })
            .unwrap_or_else(|| (level_path.clone(), 1));

        let png_path = format!("resources/{}.png", image_basename);

        // Build the Level SceneAssetDocument with one TileLayer
        let tile_layer_id = TileLayerId::new(format!("{}_tiles", image_basename));
        let tileset_id = TilesetId::new(format!("ts_{}", image_basename));

        let mut tile_layer = TileLayer::with_dimensions(
            tile_layer_id,
            format!("{} Frames", image_basename),
            tileset_id.clone(),
            frame_count as u32,
            1,
        );

        // Paint tiles in order (one per frame)
        for i in 0..frame_count {
            use editor_model::tileset::{TileCoord, TileRef};
            let coord = TileCoord::new(i as i32, 0);
            let tile_ref = TileRef {
                tileset_id: tileset_id.0.clone(),
                local_index: i as u32,
            };
            tile_layer.paint_tile(coord, tile_ref);
        }

        let level_layer = LevelLayer::Tile(tile_layer);

        // Create the SceneAssetDocument for the level
        let doc = editor_model::scene_asset::SceneAssetDocument {
            asset_id: format!("lvl_{}", image_basename),
            logical_path: level_path.clone(),
            role: SceneAssetRole::Level,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: Default::default(),
            layers: vec![level_layer],
        };

        // Serialize the document
        let doc_json = serde_json::to_string(&doc)
            .map_err(|e| ImporterError::ParseError(format!("serialization error: {}", e)))?;

        // Build AssetCommands
        let commands = vec![AssetCommand::Batch {
            label: format!("import {}", image_basename),
            commands: vec![
                AssetCommand::AddComponent {
                    local_id: format!("lvl_{}_root", image_basename),
                    type_id: "editor.LevelDocument".to_string(),
                    values: serde_json::json!({
                        "logical_path": level_path,
                        "document": doc_json
                    }),
                },
                AssetCommand::AddComponent {
                    local_id: format!("lvl_{}_root", image_basename),
                    type_id: "editor.TextureRef".to_string(),
                    values: serde_json::json!({ "path": png_path }),
                },
            ],
        }];

        let change_set_json = serde_json::to_string(&commands)
            .map_err(|e| ImporterError::ParseError(e.to_string()))?;

        Ok(BuildChangeSetOutput {
            provenance_diff: None,
            change_set_json,
        })
    }
}

impl Default for AsepriteImporter {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parse_aseprite_json_happy_path() {
        let importer = AsepriteImporter::new();
        let input = ImporterInput {
            bytes: sample_aseprite_json(),
            source_uri: "player_idle.json",
            fingerprint_hint: None,
        };

        let output = importer.parse(input).expect("parse should succeed");
        assert!(!output.resource_drafts.is_empty());

        // Has level draft with frame count encoded in display_name
        let level = output
            .resource_drafts
            .iter()
            .find(|r| matches!(r, ResourceDraft::Level { .. }));
        assert!(level.is_some(), "should have a Level draft");
        if let ResourceDraft::Level { display_name, .. } = level.unwrap() {
            assert!(
                display_name.as_ref().unwrap().contains("4 frames"),
                "display_name should encode frame count"
            );
        }

        // Has asset file (PNG)
        let png = output.resource_drafts.iter().find(|r| {
            if let ResourceDraft::AssetFile { logical_path, .. } = r {
                logical_path.contains(".png")
            } else {
                false
            }
        });
        assert!(png.is_some(), "should have a PNG AssetFile draft");

        // Has Aseprite JSON draft
        let aseprite_json = output.resource_drafts.iter().find(|r| {
            if let ResourceDraft::AssetFile { logical_path, .. } = r {
                logical_path.contains(".aseprite.json")
            } else {
                false
            }
        });
        assert!(
            aseprite_json.is_some(),
            "should have an .aseprite.json AssetFile draft"
        );

        // Has 4 source mappings
        assert_eq!(output.mappings.len(), 4, "should have 4 source mappings");

        // Has detected version
        assert!(output.detected_version.is_some());
        assert_eq!(output.detected_version.as_deref(), Some("1.3-rc12"));
    }

    #[test]
    fn parse_rejects_missing_meta() {
        let importer = AsepriteImporter::new();
        let bad_json = r#"{"frames": {}}"#.as_bytes();
        let input = ImporterInput {
            bytes: bad_json,
            source_uri: "bad.json",
            fingerprint_hint: None,
        };
        let err = importer.parse(input).unwrap_err();
        // Either missing 'meta' or missing/invalid 'meta.size' are both acceptable
        let msg = err.to_string();
        assert!(
            msg.contains("missing") || msg.contains("invalid"),
            "error should mention missing or invalid, got: {}",
            msg
        );
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
        assert!(matches!(err, ImporterError::UnsupportedVersion { .. }));
    }

    #[test]
    fn parse_accepts_supported_version_1_3() {
        let importer = AsepriteImporter::new();
        let json = r#"{
          "frames": { "a.png": { "frame": {"x":0,"y":0,"w":16,"h":16}, "duration": 100 } },
          "meta": { "version": "1.3-rc12", "size": {"w": 16, "h": 16 } }
        }"#
        .as_bytes();
        let input = ImporterInput {
            bytes: json,
            source_uri: "v13.json",
            fingerprint_hint: None,
        };
        importer
            .parse(input)
            .expect("1.3-rc12 should be in supported range 1.0.0-2.0.0");
    }

    #[test]
    fn build_change_set_produces_level_with_tile_layer() {
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

        // Verify change_set_json is valid JSON containing AssetCommands
        let commands: Vec<crate::asset_command::AssetCommand> =
            serde_json::from_str(&build_output.change_set_json)
                .expect("change_set_json should be valid AssetCommand JSON");

        assert!(!commands.is_empty());
        // Should have a Batch command
        assert!(
            commands
                .iter()
                .any(|c| matches!(c, crate::asset_command::AssetCommand::Batch { .. }))
        );
    }

    #[test]
    fn importer_descriptor_has_correct_kind() {
        let importer = AsepriteImporter::new();
        let desc = importer.descriptor();
        assert_eq!(desc.kind, ExternalSourceKind::Aseprite);
        assert_eq!(desc.id, "builtin.aseprite");
    }

    #[test]
    fn version_range_contains() {
        let range =
            ImporterVersionRange::new(ImporterVersion::new(1, 0, 0), ImporterVersion::new(2, 0, 0));
        assert!(range.contains(ImporterVersion::new(1, 0, 0)));
        assert!(range.contains(ImporterVersion::new(1, 3, 0)));
        assert!(range.contains(ImporterVersion::new(2, 0, 0)));
        assert!(!range.contains(ImporterVersion::new(0, 9, 0)));
        assert!(!range.contains(ImporterVersion::new(2, 1, 0)));
    }

    #[test]
    fn parse_image_basename_extraction() {
        let importer = AsepriteImporter::new();
        // Test with frame index suffix stripped
        let json = r#"{
          "frames": { "hero_run_05.png": { "frame": {"x":0,"y":0,"w":16,"h":16}, "duration": 50 } },
          "meta": { "version": "1.2", "size": {"w": 16, "h": 16 } }
        }"#
        .as_bytes();
        let input = ImporterInput {
            bytes: json,
            source_uri: "hero_run.json",
            fingerprint_hint: None,
        };
        let output = importer.parse(input).expect("parse should succeed");
        // Display name should contain "hero_run (1 frames)"
        let display_name = output
            .resource_drafts
            .iter()
            .find_map(|r| {
                if let ResourceDraft::Level { display_name, .. } = r {
                    display_name.clone()
                } else {
                    None
                }
            })
            .unwrap();
        assert!(
            display_name.contains("hero_run"),
            "should extract basename as 'hero_run'"
        );
    }
}
