//! World Workspace types — ADR-0037 v0.95.0.
//!
//! INVARIANT: a `WorldDocument` REFERS-TO Level Scene Assets; it does NOT
//! reproduce level content. Do not add fields shaped like entities,
//! components, layers, tile_grids, IntGrid, or scene-instance lists.
//!
//! ## ADR-0037 line 14 invariant
//!
//! `WorldDocument` owns: `position`, `dimensions`, `tags`, `streaming`,
//! links to other levels. `WorldDocument` MUST NOT store: `entities`,
//! `components`, `layers`, `tile_grids`, `IntGrid`, scene-instance lists,
//! or any field that duplicates `SceneAssetDocument`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Opaque stable identifier for a WorldDocument (parallel to `AssetId`).
///
/// The string is opaque — callers should treat it as an opaque token.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldId(pub String);

impl WorldId {
    /// Construct a new `WorldId`.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// How a world lays out its levels on the canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LayoutPolicy {
    /// Free-form placement; positions are user-driven.
    Free,
    /// Snap-to-grid placement with `cell_size` pixels.
    Grid {
        /// Cell size in pixels.
        cell_size: u32,
    },
    /// Levels laid out left-to-right at `cell_size` intervals.
    Horizontal,
    /// Levels laid out top-to-bottom at `cell_size` intervals.
    Vertical,
    /// Reserved for future policies (the v0.95 set is closed by spec §5).
    Custom {
        /// The custom layout name.
        value: String,
    },
}

/// Optional entrance anchor on the source level (e.g. "door_east").
/// Always optional per the LDtk-faithful answer to Q1 — the absence of
/// an `EntranceRef` is a soft warning, never an error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntranceRef {
    /// The level id this entrance belongs to.
    pub level_id: String,
    /// Named anchor on the level (e.g. "door_east").
    pub anchor: String,
}

/// Per-level streaming policy. Defaults to `AlwaysResident` when unset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamingPolicy {
    /// Always resident in memory.
    AlwaysResident,
    /// Loaded on demand.
    OnDemand,
    /// Manually loaded/unloaded.
    Manual,
}

impl Default for StreamingPolicy {
    fn default() -> Self {
        Self::AlwaysResident
    }
}

impl StreamingPolicy {
    /// Returns true if this policy is the default (AlwaysResident).
    fn is_default(&self) -> bool {
        matches!(self, StreamingPolicy::AlwaysResident)
    }
}

/// A single placed Level Scene Asset inside a WorldDocument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldLevelRef {
    /// Editor-stable id (different from `LocalId` — survives renames).
    pub level_id: String,
    /// Logical path to the referenced Scene Asset (e.g. "levels/act1/cave").
    pub asset_ref: String,
    /// World-space position in pixels.
    pub position: [f32; 2],
    /// Optional cached dimensions; renderer falls back to the
    /// SceneAssetDocument when `None` (mirror of `SceneAssetCatalogEntry::preview_resource`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<[u32; 2]>,
    /// User-defined tags (e.g. "boss", "shop", "secret").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Streaming policy; defaults to `AlwaysResident` via `#[serde(default)]`.
    #[serde(default, skip_serializing_if = "StreamingPolicy::is_default")]
    pub streaming: StreamingPolicy,
}

/// Direction of a WorldLink. Encodes LDtk's neighbour model:
/// a single `from → to` with a direction, no reciprocal enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkDirection {
    /// Points toward increasing Y (up on screen).
    North,
    /// Points toward decreasing Y (down on screen).
    South,
    /// Points toward increasing X (right on screen).
    East,
    /// Points toward decreasing X (left on screen).
    West,
    /// No direction (backwards/forwards through a portal, etc.).
    Undirected,
}

/// One-way / bidirectional discriminator. `Custom(String)` keeps the
/// door open for future `teleport` / `conditional` link kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorldLinkKind {
    /// Traversal is only allowed in the from → to direction.
    OneWay,
    /// Traversal is allowed in both directions.
    Bidirectional,
    /// Custom traversal logic encoded as a string.
    Custom {
        /// The custom link kind identifier.
        value: String,
    },
}

/// A directed connection between two levels. Direction is encoded on
/// the link itself, not derived from a reciprocal pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldLink {
    /// Editor-stable id for this link.
    pub id: String,
    /// Source level's `level_id`.
    pub from: String,
    /// Target level's `level_id`.
    pub to: String,
    /// Direction (LDtk-faithful: north / south / east / west / undirected).
    pub direction: LinkDirection,
    /// One-way / bidirectional / custom.
    pub kind: WorldLinkKind,
    /// Optional entrance anchor on the source level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrance: Option<EntranceRef>,
    /// Optional exit anchor on the target level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit: Option<EntranceRef>,
}

/// Top-level WorldDocument — references Level Scene Assets, never
/// duplicates them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldDocument {
    /// Unique identifier for this world.
    pub id: WorldId,
    /// Human-readable name.
    pub name: String,
    /// Schema version; bumped on any breaking change.
    pub version: u32,
    /// How levels are laid out on the canvas.
    pub layout_policy: LayoutPolicy,
    /// All level references in this world.
    pub levels: Vec<WorldLevelRef>,
    /// All links between levels in this world.
    pub links: Vec<WorldLink>,
    /// Unix millis of the last write (set by `save_world_wasm`).
    pub updated_at: u64,
    /// Unknown JSON fields preserved for forward compatibility (ADR-0046 rule 2).
    #[serde(default, flatten)]
    pub extension_data: BTreeMap<String, serde_json::Value>,
}

/// Catalog entry — shadow of `WorldDocument` kept on `ProjectMetadata`.
/// Mirrors `SceneAssetCatalogEntry` so `ProjectMetadata` migration is
/// mechanical.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldCatalogEntry {
    /// Unique identifier for this world.
    pub world_id: WorldId,
    /// Logical OPFS path to the world document body file.
    pub logical_path: String,
    /// Monotonically increasing version number.
    pub current_version: u32,
    /// Unix millis of the last write.
    pub updated_at: u64,
    /// Unix millis of creation.
    pub created_at: u64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── WorldId ────────────────────────────────────────────────────────────────

    #[test]
    fn test_world_id_new() {
        let id = WorldId::new("test-world");
        assert_eq!(id.as_str(), "test-world");
    }

    #[test]
    fn test_world_id_transparent_serde() {
        let id = WorldId::new("my-world");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"my-world\"");
        let rt: WorldId = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, id);
    }

    // ── LayoutPolicy ───────────────────────────────────────────────────────────

    #[test]
    fn test_layout_policy_free_serde() {
        let policy = LayoutPolicy::Free;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, r#"{"kind":"free"}"#);
        let rt: LayoutPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, policy);
    }

    #[test]
    fn test_layout_policy_grid_serde() {
        let policy = LayoutPolicy::Grid { cell_size: 64 };
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, r#"{"kind":"grid","cell_size":64}"#);
        let rt: LayoutPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, policy);
    }

    #[test]
    fn test_layout_policy_horizontal_serde() {
        let policy = LayoutPolicy::Horizontal;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, r#"{"kind":"horizontal"}"#);
        let rt: LayoutPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, policy);
    }

    #[test]
    fn test_layout_policy_vertical_serde() {
        let policy = LayoutPolicy::Vertical;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, r#"{"kind":"vertical"}"#);
        let rt: LayoutPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, policy);
    }

    #[test]
    fn test_layout_policy_custom_serde() {
        let policy = LayoutPolicy::Custom {
            value: "hexagonal".to_string(),
        };
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, r#"{"kind":"custom","value":"hexagonal"}"#);
        let rt: LayoutPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, policy);
    }

    // ── StreamingPolicy ────────────────────────────────────────────────────────

    #[test]
    fn test_streaming_policy_default() {
        assert_eq!(StreamingPolicy::default(), StreamingPolicy::AlwaysResident);
    }

    #[test]
    fn test_streaming_policy_serde() {
        let sp = StreamingPolicy::OnDemand;
        let json = serde_json::to_string(&sp).unwrap();
        assert_eq!(json, r#""on_demand""#);
        let rt: StreamingPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, sp);
    }

    #[test]
    fn test_streaming_policy_all_variants() {
        for sp in &[
            StreamingPolicy::AlwaysResident,
            StreamingPolicy::OnDemand,
            StreamingPolicy::Manual,
        ] {
            let json = serde_json::to_string(sp).unwrap();
            let rt: StreamingPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(rt, *sp);
        }
    }

    // ── EntranceRef ───────────────────────────────────────────────────────────

    #[test]
    fn test_entrance_ref_serde() {
        let er = EntranceRef {
            level_id: "level-a".to_string(),
            anchor: "door_east".to_string(),
        };
        let json = serde_json::to_string(&er).unwrap();
        let rt: EntranceRef = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, er);
    }

    #[test]
    fn test_entrance_ref_optional_roundtrip() {
        // EntranceRef is itself optional on WorldLink
        let er: Option<EntranceRef> = Some(EntranceRef {
            level_id: "level-b".to_string(),
            anchor: "portal_north".to_string(),
        });
        let json = serde_json::to_string(&er).unwrap();
        let rt: Option<EntranceRef> = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, er);

        let none: Option<EntranceRef> = None;
        let json = serde_json::to_string(&none).unwrap();
        let rt: Option<EntranceRef> = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, none);
    }

    // ── WorldLevelRef ─────────────────────────────────────────────────────────

    #[test]
    fn test_world_level_ref_minimal() {
        let lvl = WorldLevelRef {
            level_id: "lvl-1".to_string(),
            asset_ref: "levels/cave".to_string(),
            position: [100.0, 200.0],
            dimensions: None,
            tags: vec![],
            streaming: StreamingPolicy::default(),
        };
        let json = serde_json::to_string(&lvl).unwrap();
        // dimensions: None → skipped; tags: [] → skipped; streaming: default → omitted
        assert!(!json.contains("dimensions"));
        assert!(!json.contains("tags"));
        assert!(!json.contains("streaming"));
        let rt: WorldLevelRef = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.level_id, "lvl-1");
    }

    #[test]
    fn test_world_level_ref_full() {
        let lvl = WorldLevelRef {
            level_id: "lvl-2".to_string(),
            asset_ref: "levels/boss_room".to_string(),
            position: [320.0, 480.0],
            dimensions: Some([1024, 768]),
            tags: vec!["boss".to_string(), "secret".to_string()],
            streaming: StreamingPolicy::OnDemand,
        };
        let json = serde_json::to_string(&lvl).unwrap();
        let rt: WorldLevelRef = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, lvl);
    }

    #[test]
    fn test_world_level_ref_dimensions_none_roundtrip() {
        // dimensions: None must deserialize back to None (not error)
        let json = r#"{"level_id":"lvl-x","asset_ref":"a/b","position":[0,0]}"#;
        let rt: WorldLevelRef = serde_json::from_str(json).unwrap();
        assert!(rt.dimensions.is_none());
    }

    // ── LinkDirection ─────────────────────────────────────────────────────────

    #[test]
    fn test_link_direction_all_variants() {
        for dir in &[
            LinkDirection::North,
            LinkDirection::South,
            LinkDirection::East,
            LinkDirection::West,
            LinkDirection::Undirected,
        ] {
            let json = serde_json::to_string(dir).unwrap();
            let rt: LinkDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(rt, *dir);
        }
    }

    // ── WorldLinkKind ─────────────────────────────────────────────────────────

    #[test]
    fn test_world_link_kind_one_way() {
        let kind = WorldLinkKind::OneWay;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, r#"{"kind":"one_way"}"#);
        let rt: WorldLinkKind = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, kind);
    }

    #[test]
    fn test_world_link_kind_bidirectional() {
        let kind = WorldLinkKind::Bidirectional;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, r#"{"kind":"bidirectional"}"#);
        let rt: WorldLinkKind = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, kind);
    }

    #[test]
    fn test_world_link_kind_custom() {
        let kind = WorldLinkKind::Custom {
            value: "teleport".to_string(),
        };
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, r#"{"kind":"custom","value":"teleport"}"#);
        let rt: WorldLinkKind = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, kind);
    }

    // ── WorldLink ─────────────────────────────────────────────────────────────

    #[test]
    fn test_world_link_minimal() {
        let link = WorldLink {
            id: "link-1".to_string(),
            from: "lvl-a".to_string(),
            to: "lvl-b".to_string(),
            direction: LinkDirection::East,
            kind: WorldLinkKind::OneWay,
            entrance: None,
            exit: None,
        };
        let json = serde_json::to_string(&link).unwrap();
        let rt: WorldLink = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, link);
    }

    #[test]
    fn test_world_link_with_anchors() {
        let link = WorldLink {
            id: "link-2".to_string(),
            from: "lvl-a".to_string(),
            to: "lvl-c".to_string(),
            direction: LinkDirection::North,
            kind: WorldLinkKind::Bidirectional,
            entrance: Some(EntranceRef {
                level_id: "lvl-a".to_string(),
                anchor: "door_north".to_string(),
            }),
            exit: Some(EntranceRef {
                level_id: "lvl-c".to_string(),
                anchor: "door_south".to_string(),
            }),
        };
        let json = serde_json::to_string(&link).unwrap();
        let rt: WorldLink = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, link);
    }

    // ── WorldDocument ──────────────────────────────────────────────────────────

    #[test]
    fn test_world_document_empty() {
        let doc = WorldDocument {
            id: WorldId::new("test-world"),
            name: "demo".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: Vec::new(),
            links: Vec::new(),
            updated_at: 0,
            extension_data: BTreeMap::new(),
        };
        assert_eq!(doc.name, "demo");
        assert_eq!(doc.version, 1);
        assert!(matches!(doc.layout_policy, LayoutPolicy::Free));
        assert!(doc.levels.is_empty());
        assert!(doc.links.is_empty());
        assert_eq!(doc.updated_at, 0);
    }

    #[test]
    fn test_world_document_roundtrip() {
        let doc = WorldDocument {
            id: WorldId::new("world-1"),
            name: "Test World".to_string(),
            version: 3,
            layout_policy: LayoutPolicy::Grid { cell_size: 128 },
            levels: vec![
                WorldLevelRef {
                    level_id: "lvl-1".to_string(),
                    asset_ref: "levels/start".to_string(),
                    position: [0.0, 0.0],
                    dimensions: None,
                    tags: vec![],
                    streaming: StreamingPolicy::AlwaysResident,
                },
                WorldLevelRef {
                    level_id: "lvl-2".to_string(),
                    asset_ref: "levels/end".to_string(),
                    position: [128.0, 0.0],
                    dimensions: Some([256, 256]),
                    tags: vec!["final".to_string()],
                    streaming: StreamingPolicy::Manual,
                },
            ],
            links: vec![WorldLink {
                id: "link-a".to_string(),
                from: "lvl-1".to_string(),
                to: "lvl-2".to_string(),
                direction: LinkDirection::East,
                kind: WorldLinkKind::OneWay,
                entrance: None,
                exit: None,
            }],
            updated_at: 1_700_000_000_000_u64,
            extension_data: BTreeMap::new(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        let rt: WorldDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, doc);
    }

    // ── WorldCatalogEntry ─────────────────────────────────────────────────────

    #[test]
    fn test_world_catalog_entry_roundtrip() {
        let entry = WorldCatalogEntry {
            world_id: WorldId::new("world-2"),
            logical_path: "worlds/demo".to_string(),
            current_version: 2,
            updated_at: 1_700_000_001_000_u64,
            created_at: 1_600_999_000_000_u64,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let rt: WorldCatalogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, entry);
    }
}
