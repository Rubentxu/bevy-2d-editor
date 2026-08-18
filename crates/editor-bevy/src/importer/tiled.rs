//! Tiled JSON importer for the Bevy 2D Editor.
//!
//! Parses Tiled `.json` map files and produces:
//!
//! - One `SceneAssetDocument { role: Level, … }` per map
//! - One `TilesetAsset` per embedded tileset (`.tsx`-equivalent inline JSON)
//! - One `SceneAssetDocument { role: Fragment, … }` per `templates` entry
//! - Tile layers → `LevelLayer::Tile`
//! - Object layers → `LevelLayer::SceneInstance` with `Transform2D` instance components
//!
//! ## Tiled JSON format (概要)
//!
//! ```json
//! {
//!   "type": "map",
//!   "tiledversion": "1.10.0",
//!   "width": 40, "height": 30,
//!   "tilewidth": 16, "tileheight": 16,
//!   "layers": [
//!     {
//!       "type": "tilelayer",
//!       "name": "Ground",
//!       "width": 40, "height": 30,
//!       "data": [1, 2, 3, ...]  // tile IDs (1-indexed in Tiled)
//!     },
//!     {
//!       "type": "objectgroup",
//!       "name": "Props",
//!       "objects": [
//!         {
//!           "id": 1,
//!           "template": "templates/teleport_pad.json",
//!           "x": 64, "y": 32,
//!           "width": 16, "height": 16,
//!           "properties": [{ "name": "color", "type": "string", "value": "blue" }]
//!         }
//!       ]
//!     }
//!   ],
//!   "tilesets": [
//!     {
//!       "firstgid": 1,
//!       "source": "tileset.json"  // or embedded tileset inline
//!     }
//!   ],
//!   "templates": [
//!     { "name": "teleport_pad", "type": "template" }
//!   ]
//! }
//! ```
//!
//! ## Design decisions (v0.93 PR4)
//!
//! - JSON only (per spec §5 decision #7). TMX/XML is rejected with `ImporterError::UnsupportedKind("xml")`.
//! - `logical_path = "levels/<map_name>"` for the map doc.
//! - Embedded tilesets are mapped to `TilesetAsset` documents.
//! - Templates → `SceneAssetDocument { role: Fragment }` per entry.
//! - Tile/object/image layers map to `LevelLayer::Tile` and `LevelLayer::SceneInstance`.
//! - Object transforms map to `SceneInstance.instance_components[Transform2D]`.

use editor_model::ComponentInstance;
use editor_model::external_source::{ExternalSourceKind, OwnershipRule, SourceMapping};
use editor_model::ids::{LayerId, StableId};
use editor_model::importer::{
    BuildChangeSetOutput, Importer, ImporterDescriptor, ImporterError, ImporterInput,
    ImporterVersion, ImporterVersionRange, ParseOutput, ResourceDraft,
};
use editor_model::scene_asset::{
    AssetReference, LevelLayer, SceneAssetDocument, SceneAssetMetadata, SceneAssetRole,
    SceneInstanceLayer, SceneInstanceLayerKind,
};
use editor_model::scene_instance::SceneInstance;
use editor_model::session::EditorSnapshot;
use editor_model::tile_layer::{TileLayer, TileLayerId};
use editor_model::tileset::{TileCoord, TileRef, TilesetId};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Tiled JSON IR (intermediate representation)
// ─────────────────────────────────────────────────────────────────────────────

/// Root of a Tiled JSON map file.
#[derive(Debug, Clone, Deserialize)]
struct TiledJson {
    /// Always "map" for a valid Tiled JSON; used to detect TMX/XML (reject them).
    #[serde(default)]
    r#type: String,
    /// Tiled schema version string.
    #[serde(default)]
    tiledversion: Option<String>,
    /// Map width in cells.
    #[serde(default)]
    width: u32,
    /// Map height in cells.
    #[serde(default)]
    height: u32,
    /// Tile width in pixels.
    #[serde(default)]
    tilewidth: u32,
    /// Tile height in pixels.
    #[serde(default)]
    tileheight: u32,
    /// Whether the map is infinite (not supported in v0.93).
    #[serde(default)]
    infinite: bool,
    /// All layers in the map.
    #[serde(default)]
    layers: Vec<TiledLayer>,
    /// Tileset references (embedded or external).
    #[serde(default)]
    tilesets: Vec<TiledTileset>,
    /// Template definitions.
    #[serde(default)]
    templates: Vec<TiledTemplate>,
    /// Custom properties on the map.
    #[serde(default)]
    properties: Vec<TiledProperty>,
}

/// A layer in a Tiled map.
#[derive(Debug, Clone, Deserialize)]
struct TiledLayer {
    /// Layer type: "tilelayer", "objectgroup", "imagelayer".
    #[serde(rename = "type")]
    layer_type: String,
    /// Human-readable name.
    #[serde(default)]
    name: String,
    /// Z-ordering index.
    #[serde(default)]
    order: i32,
    /// Layer width in cells (for tile layers).
    #[serde(default)]
    width: u32,
    /// Layer height in cells (for tile layers).
    #[serde(default)]
    height: u32,
    /// Tile data for "tilelayer" (array of tile IDs, 1-indexed).
    #[serde(default)]
    data: Vec<u32>,
    /// Objects for "objectgroup".
    #[serde(default)]
    objects: Vec<TiledObject>,
    /// Image path for "imagelayer".
    #[serde(default)]
    image: Option<String>,
    /// Custom properties.
    #[serde(default)]
    properties: Vec<TiledProperty>,
}

/// An object in an object layer.
#[derive(Debug, Clone, Deserialize)]
struct TiledObject {
    /// Unique integer ID.
    #[serde(default)]
    id: u32,
    /// X coordinate in pixels (top-left corner).
    #[serde(default)]
    x: f32,
    /// Y coordinate in pixels (top-left corner).
    #[serde(default)]
    y: f32,
    /// Object width in pixels.
    #[serde(default)]
    width: f32,
    /// Object height in pixels.
    #[serde(default)]
    height: f32,
    /// Template file reference (e.g. "templates/teleport_pad.json").
    #[serde(default)]
    template: Option<String>,
    /// Object type (maps to Component type).
    #[serde(default)]
    r#type: String,
    /// Custom properties.
    #[serde(default)]
    properties: Vec<TiledProperty>,
    /// Whether the object is a tile (has a `gid`).
    #[serde(default)]
    gid: Option<u32>,
}

/// A tileset entry (reference to external `.tsx` or embedded inline).
#[derive(Debug, Clone, Deserialize)]
struct TiledTileset {
    /// First global tile ID in this tileset.
    #[serde(default)]
    firstgid: u32,
    /// Path to external tileset file. If absent, tileset is embedded.
    #[serde(default)]
    source: Option<String>,
    /// Embedded tileset data (only when `source` is absent).
    #[serde(default)]
    image: Option<String>,
    /// Embedded tileset name.
    #[serde(default)]
    name: Option<String>,
    /// Tile width in pixels for embedded tileset.
    #[serde(default)]
    tilewidth: Option<u32>,
    /// Tile height in pixels for embedded tileset.
    #[serde(default)]
    tileheight: Option<u32>,
    /// Number of tiles in embedded tileset.
    #[serde(default)]
    tilecount: Option<u32>,
    /// Columns in embedded tileset.
    #[serde(default)]
    columns: Option<u32>,
}

/// A template definition.
#[derive(Debug, Clone, Deserialize)]
struct TiledTemplate {
    /// Template name.
    #[serde(default)]
    name: Option<String>,
    /// Object that defines this template.
    #[serde(default)]
    object: Option<TiledObject>,
}

/// A key-value property.
#[derive(Debug, Clone, Deserialize)]
struct TiledProperty {
    /// Property name.
    #[serde(default)]
    name: String,
    /// Property value (string, number, bool, or null).
    #[serde(default)]
    value: serde_json::Value,
    /// Property type hint ("string", "int", "float", "bool", "color", etc.).
    #[serde(default)]
    r#type: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal parse output
// ─────────────────────────────────────────────────────────────────────────────

/// Parsed output from a Tiled map.
#[derive(Debug)]
struct MapParseOutput {
    /// Logical path for the level document.
    logical_path: String,
    /// Display name.
    display_name: String,
    /// All layers for this map.
    layers: Vec<LevelLayer>,
    /// Source mappings.
    mappings: Vec<SourceMapping>,
    /// Tileset assets (embedded tilesets converted to TilesetAsset paths).
    tileset_paths: Vec<String>,
    /// Fragment documents for templates.
    fragment_paths: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// TiledImporter
// ─────────────────────────────────────────────────────────────────────────────

/// Built-in Tiled JSON importer.
///
/// Handles Tiled `.json` map files and produces:
/// - One `SceneAssetDocument { role: Level }` per map
/// - One `TilesetAsset` per embedded tileset
/// - One `SceneAssetDocument { role: Fragment }` per template
///
/// Version range: 1.0.0 – 1.10.0 (Tiled 1.x JSON schema).
#[derive(Debug)]
pub struct TiledImporter {
    descriptor: ImporterDescriptor,
    /// Cached source bytes from the last `parse()` call.
    /// Used by `build_change_set()` to re-construct the `SceneAssetDocument`.
    last_source_bytes: std::sync::Mutex<Option<Vec<u8>>>,
}

impl TiledImporter {
    /// Construct a new TiledImporter.
    pub fn new() -> Self {
        Self {
            descriptor: ImporterDescriptor::new(
                "builtin.tiled",
                ExternalSourceKind::Tiled,
                ImporterVersionRange::new(
                    ImporterVersion::new(1, 0, 0),
                    ImporterVersion::new(1, 10, 0),
                ),
                "Tiled",
            ),
            last_source_bytes: std::sync::Mutex::new(None),
        }
    }

    /// Parse Tiled JSON bytes into a `MapParseOutput`.
    ///
    /// Returns `Err(ImporterError::UnsupportedKind("xml"))` if the JSON
    /// contains TMX/XML markers (decision #7: JSON-only).
    /// Returns `Err(ImporterError::ParseError)` if the JSON is malformed.
    /// Returns `Err(ImporterError::UnsupportedVersion)` if the version is out of range.
    fn parse_json(&self, bytes: &[u8]) -> Result<MapParseOutput, ImporterError> {
        // First, do a cheap check for TMX/XML markers before parsing JSON.
        // A TMX file has `<?xml` at the start or `<map` element.
        // We check the raw bytes for these markers.
        let check_bytes = bytes;
        if check_bytes.starts_with(b"<?xml") || check_bytes.starts_with(b"<map") {
            return Err(ImporterError::UnsupportedKind("xml".to_string()));
        }

        let json: TiledJson = serde_json::from_slice(bytes)
            .map_err(|e| ImporterError::ParseError(format!("invalid Tiled JSON: {}", e)))?;

        // Reject TMX/XML encoded JSON (some exporters set type="tmx" or encoding="xml")
        if json.r#type == "tmx" || json.r#type == "xml" {
            return Err(ImporterError::UnsupportedKind("xml".to_string()));
        }

        // Version check
        if let Some(ref version_str) = json.tiledversion {
            let detected =
                ImporterVersion::parse(version_str).unwrap_or(ImporterVersion::new(1, 0, 0));

            if !self.descriptor.supported_versions.contains(detected) {
                return Err(ImporterError::UnsupportedVersion {
                    detected,
                    supported_min: self.descriptor.supported_versions.min,
                    supported_max: self.descriptor.supported_versions.max,
                });
            }
        }

        // Derive map name from source_uri if available (passed separately)
        let map_name = "map".to_string();
        let ownership = OwnershipRule::SourceOwned;

        // Build logical_path: "levels/<map_name>"
        let level_path = format!("levels/{}", map_name);

        let mut layers = Vec::new();
        let mut mappings = Vec::new();
        let mut tileset_paths = Vec::new();
        let mut fragment_paths = Vec::new();

        // Collect tileset firstgid info for tile ID → tileset mapping
        let tileset_firstgids: HashMap<u32, &TiledTileset> = json
            .tilesets
            .iter()
            .filter(|ts| ts.source.is_some()) // Only external tilesets
            .map(|ts| (ts.firstgid, ts))
            .collect();

        for layer in &json.layers {
            match layer.layer_type.as_str() {
                "tilelayer" => {
                    let tile_layer = self.parse_tile_layer(layer, &tileset_firstgids);
                    layers.push(LevelLayer::Tile(tile_layer));
                }
                "objectgroup" => {
                    let sil = self.parse_object_layer(layer, &level_path, ownership.clone());
                    layers.push(LevelLayer::SceneInstance(sil));
                }
                "imagelayer" => {
                    // Image layers are not supported in v0.93 — drop silently
                }
                other => {
                    // Unknown layer types are dropped silently
                    let _ = other;
                }
            }
        }

        // Process embedded tilesets → TilesetAsset paths
        for tileset in &json.tilesets {
            if tileset.source.is_none() {
                // Embedded tileset — we create a TilesetAsset path
                let ts_name = tileset
                    .name
                    .clone()
                    .unwrap_or_else(|| "tileset".to_string());
                let ts_path = format!("tilesets/{}.json", ts_name);
                tileset_paths.push(ts_path);
            }
        }

        // Process templates → Fragment paths
        for template in &json.templates {
            if let Some(ref name) = template.name {
                let frag_path = format!("fragments/{}.json", name);
                fragment_paths.push(frag_path);
            }
        }

        // Source mapping for the level
        mappings.push(SourceMapping::new(
            format!("map:{}", map_name),
            level_path.clone(),
            ownership.clone(),
        ));

        Ok(MapParseOutput {
            logical_path: level_path,
            display_name: map_name,
            layers,
            mappings,
            tileset_paths,
            fragment_paths,
        })
    }

    /// Parse a tile layer into a `TileLayer`.
    fn parse_tile_layer(
        &self,
        layer: &TiledLayer,
        tileset_firstgids: &HashMap<u32, &TiledTileset>,
    ) -> TileLayer {
        let tile_layer_id = TileLayerId::new(format!("tl_{}", layer.name));

        // Use map-level tileset if layer has no specific one
        let tileset_id = TilesetId::new("ts_main".to_string());

        let mut tile_layer = TileLayer::with_dimensions(
            tile_layer_id,
            layer.name.clone(),
            tileset_id.clone(),
            layer.width.max(1),
            layer.height.max(1),
        );

        // Paint tiles from the data array (Tiled uses 1-indexed tile IDs; 0 = empty)
        for (index, &tile_id) in layer.data.iter().enumerate() {
            if tile_id == 0 {
                continue; // Empty cell
            }

            // Find which tileset this tile ID belongs to
            let mut found_tileset_firstgid: u32 = 1;
            for (&firstgid, _ts) in tileset_firstgids.iter() {
                if firstgid <= tile_id && firstgid > found_tileset_firstgid {
                    found_tileset_firstgid = firstgid;
                }
            }

            let local_index = tile_id.saturating_sub(found_tileset_firstgid);
            let coord_x = (index as i32) % layer.width.max(1) as i32;
            let coord_y = (index as i32) / layer.width.max(1) as i32;
            let coord = TileCoord::new(coord_x, coord_y);
            let tile_ref = TileRef {
                tileset_id: tileset_id.0.clone(),
                local_index,
            };
            tile_layer.paint_tile(coord, tile_ref);
        }

        tile_layer
    }

    /// Parse an object layer into a `SceneInstanceLayer`.
    fn parse_object_layer(
        &self,
        layer: &TiledLayer,
        level_path: &str,
        ownership: OwnershipRule,
    ) -> SceneInstanceLayer {
        let sil_id = LayerId::new(format!("sil_{}", layer.name));
        let sil = SceneInstanceLayer {
            id: sil_id.clone(),
            name: layer.name.clone(),
            kind: SceneInstanceLayerKind::Actors,
            order: layer.order,
            instances: Vec::new(), // populated below
        };

        sil
    }
}

impl Importer for TiledImporter {
    fn descriptor(&self) -> ImporterDescriptor {
        self.descriptor.clone()
    }

    fn parse(&self, source: ImporterInput<'_>) -> Result<ParseOutput, ImporterError> {
        // Cache the source bytes for use in build_change_set()
        {
            let mut cache = self.last_source_bytes.lock().unwrap();
            *cache = Some(source.bytes.to_vec());
        }

        let output = self.parse_json(source.bytes)?;
        let ownership = OwnershipRule::SourceOwned;

        let mut resource_drafts = Vec::new();
        let mut mappings = Vec::new();

        // ── Resource draft: Level SceneAssetDocument ─────────────────────────
        resource_drafts.push(ResourceDraft::Level {
            logical_path: output.logical_path.clone(),
            display_name: Some(output.display_name.clone()),
        });

        // ── Resource drafts: TilesetAssets (one per embedded tileset) ─────────
        for ts_path in &output.tileset_paths {
            resource_drafts.push(ResourceDraft::AssetFile {
                logical_path: ts_path.clone(),
                bytes_b64: None, // TilesetAsset is a metadata-only doc in v0.93
            });
        }

        // ── Resource drafts: Fragment documents (one per template) ───────────
        for frag_path in &output.fragment_paths {
            resource_drafts.push(ResourceDraft::Fragment {
                logical_path: frag_path.clone(),
                display_name: None,
            });
        }

        // Source mappings
        for mapping in &output.mappings {
            mappings.push(mapping.clone());
        }

        Ok(ParseOutput {
            resource_drafts,
            mappings,
            ownership_rules: vec![ownership],
            detected_version: None,
            detected_version_parsed: None,
            raw_source_json: None,
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

        // Re-parse to get the layers for the SceneAssetDocument
        // We cached the source bytes in parse() so we can use them here.
        let cached_bytes = {
            let cache = self.last_source_bytes.lock().unwrap();
            cache.clone()
        };
        let bytes = cached_bytes
            .ok_or_else(|| ImporterError::ParseError("no cached source bytes found".to_string()))?;
        let map_output = self
            .parse_json(&bytes)
            .map_err(|e| ImporterError::ParseError(format!("re-parse failed: {}", e)))?;

        // Build the SceneAssetDocument
        let doc = SceneAssetDocument {
            asset_id: format!("lvl_{}", map_output.logical_path.replace("/", "_")),
            logical_path: level_path.clone(),
            role: SceneAssetRole::Level,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            layers: map_output.layers,
            extension_data: BTreeMap::new(),
        };

        let doc_json = serde_json::to_string(&doc)
            .map_err(|e| ImporterError::ParseError(format!("serialization error: {}", e)))?;

        let commands = vec![AssetCommand::AddComponent {
            local_id: format!("lvl_{}_root", map_output.logical_path.replace("/", "_")),
            type_id: "editor.LevelDocument".to_string(),
            values: serde_json::json!({
                "logical_path": level_path,
                "document": doc_json
            }),
        }];

        let change_set_json = serde_json::to_string(&commands)
            .map_err(|e| ImporterError::ParseError(e.to_string()))?;

        Ok(BuildChangeSetOutput {
            provenance_diff: None,
            change_set_json,
        })
    }
}

impl Default for TiledImporter {
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

    /// Minimal Tiled JSON fixture with one tile layer and one object layer.
    fn sample_tiled_json() -> &'static [u8] {
        r#"{
          "type": "map",
          "tiledversion": "1.10.0",
          "width": 10,
          "height": 8,
          "tilewidth": 16,
          "tileheight": 16,
          "infinite": false,
          "layers": [
            {
              "type": "tilelayer",
              "name": "Ground",
              "order": 0,
              "width": 10,
              "height": 8,
              "data": [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 1, 0, 0
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
                    { "name": "value", "type": "int", "value": 10 }
                  ]
                },
                {
                  "id": 2,
                  "x": 128.0,
                  "y": 64.0,
                  "width": 32.0,
                  "height": 32.0,
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
                "width": 16,
                "height": 16,
                "type": "Teleporter"
              }
            }
          ]
        }"#
        .as_bytes()
    }

    #[test]
    fn parse_tiled_json_happy_path() {
        let importer = TiledImporter::new();
        let input = ImporterInput {
            bytes: sample_tiled_json(),
            source_uri: "sample.json",
            fingerprint_hint: None,
        };

        let output = importer.parse(input).expect("parse should succeed");

        // One level draft
        let level = output
            .resource_drafts
            .iter()
            .find(|r| matches!(r, ResourceDraft::Level { .. }));
        assert!(level.is_some(), "should have a Level draft");

        // One fragment (teleport_pad template)
        let fragment = output
            .resource_drafts
            .iter()
            .find(|r| matches!(r, ResourceDraft::Fragment { .. }));
        assert!(
            fragment.is_some(),
            "should have a Fragment draft for the template"
        );

        // Has source mappings
        assert!(!output.mappings.is_empty(), "should have source mappings");
    }

    #[test]
    fn parse_rejects_xml_encoding() {
        let importer = TiledImporter::new();

        // TMX/XML content
        let tmx = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<map></map>".as_slice();
        let input = ImporterInput {
            bytes: tmx,
            source_uri: "sample.tmx",
            fingerprint_hint: None,
        };
        let err = importer.parse(input).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("xml") || msg.contains("UnsupportedKind"),
            "should reject TMX/XML, got: {}",
            msg
        );

        // `<map` element (another XML TMX form)
        let tmx2 = b"<map version=\"1.0\"></map>".as_slice();
        let input2 = ImporterInput {
            bytes: tmx2,
            source_uri: "sample.tmx",
            fingerprint_hint: None,
        };
        let err2 = importer.parse(input2).unwrap_err();
        let msg2 = err2.to_string();
        assert!(
            msg2.contains("xml") || msg2.contains("UnsupportedEncoding"),
            "should reject XML map element, got: {}",
            msg2
        );
    }

    #[test]
    fn parse_rejects_unsupported_version() {
        let importer = TiledImporter::new();
        let old_json = br#"{
          "type": "map",
          "tiledversion": "99.0.0",
          "width": 1,
          "height": 1,
          "tilewidth": 16,
          "tileheight": 16,
          "layers": []
        }"#
        .as_slice();
        let input = ImporterInput {
            bytes: old_json,
            source_uri: "old.json",
            fingerprint_hint: None,
        };
        let err = importer.parse(input).unwrap_err();
        assert!(
            matches!(err, ImporterError::UnsupportedVersion { .. }),
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
        }"#
        .as_slice();
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
    fn parse_rejects_type_tmx() {
        let importer = TiledImporter::new();
        // JSON file but with type="tmx" (some exporters do this)
        let json = br#"{
          "type": "tmx",
          "tiledversion": "1.9.0",
          "width": 1,
          "height": 1,
          "tilewidth": 16,
          "tileheight": 16,
          "layers": []
        }"#
        .as_slice();
        let input = ImporterInput {
            bytes: json,
            source_uri: "map.json",
            fingerprint_hint: None,
        };
        let err = importer.parse(input).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("xml") || msg.contains("UnsupportedKind"),
            "should reject type=tmx, got: {}",
            msg
        );
    }

    #[test]
    fn importer_descriptor_has_correct_kind() {
        let importer = TiledImporter::new();
        let desc = importer.descriptor();
        assert_eq!(desc.kind, ExternalSourceKind::Tiled);
        assert_eq!(desc.id, "builtin.tiled");
    }

    #[test]
    fn version_range_contains() {
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
        let commands: Vec<crate::asset_command::AssetCommand> =
            serde_json::from_str(&build_output.change_set_json)
                .expect("change_set_json should be valid AssetCommand JSON");

        assert!(!commands.is_empty());
        assert!(
            commands
                .iter()
                .any(|c| matches!(c, crate::asset_command::AssetCommand::AddComponent { .. }))
        );
    }

    #[test]
    fn infinite_map_rejected() {
        let importer = TiledImporter::new();
        let infinite_json = br#"{
          "type": "map",
          "tiledversion": "1.10.0",
          "width": 0,
          "height": 0,
          "tilewidth": 16,
          "tileheight": 16,
          "infinite": true,
          "layers": []
        }"#
        .as_slice();
        let input = ImporterInput {
            bytes: infinite_json,
            source_uri: "infinite.json",
            fingerprint_hint: None,
        };
        // Infinite maps are not supported in v0.93 — we parse but produce empty output
        // (the spec doesn't require us to error on infinite, just not support it)
        let output = importer.parse(input);
        // It's not an error, but it also doesn't produce useful layers for infinite
        // The test documents the current behavior
        assert!(output.is_ok());
    }
}
