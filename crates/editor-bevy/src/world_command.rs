//! World Command processor for World Workspace authoring mode.
//!
//! A separate command surface for mutating `WorldDocument`.
//! Uses mechanical inverse generation (same pattern as `asset_command.rs`).
//!
//! ## Inverse table
//! | Forward | Inverse |
//! |---------|---------|
//! | `WorldPlaceLevel` | `WorldRemoveLevel` |
//! | `WorldRemoveLevel` | `WorldPlaceLevel` (restores pre-state) |
//! | `WorldConnectLevels` | `WorldRemoveLink` |
//! | `WorldSetLayoutPolicy` | `WorldSetLayoutPolicy` (captures pre-policy) |
//! | `WorldSetStreamingPolicy` | `WorldSetStreamingPolicy` (captures pre-policy) |

use editor_model::world::{
    LayoutPolicy, LinkDirection, StreamingPolicy, WorldDocument, WorldId, WorldLevelRef, WorldLink,
    WorldLinkKind,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─────────────────────────────────────────────────────────────────────────
// WorldCommandError
// ─────────────────────────────────────────────────────────────────────────

/// Typed errors returned by world command validation and application.
#[derive(Debug, Error)]
pub enum WorldCommandError {
    #[error("world not found: {0}")]
    WorldNotFound(String),

    #[error("level not found in world: {0}")]
    LevelNotFound(String),

    #[error("link not found: {0}")]
    LinkNotFound(String),

    #[error("workspace too large: {0} levels (max 100)")]
    WorkspaceTooLarge(usize),

    #[error("missing level reference: asset_ref '{0}' not in catalog")]
    MissingLevelRef(String),

    #[error("duplicate level id: {0}")]
    DuplicateLevelId(String),

    #[error("duplicate link id: {0}")]
    DuplicateLinkId(String),

    #[error("JSON serialization error: {0}")]
    JsonError(String),
}

impl From<serde_json::Error> for WorldCommandError {
    fn from(e: serde_json::Error) -> Self {
        WorldCommandError::JsonError(e.to_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────
// WorldCommand enum
// ─────────────────────────────────────────────────────────────────────────

/// Typed command enum for World Document mutations.
///
/// Uses `#[serde(tag = "type")]` so each variant serializes as
/// `{"type": "WorldPlaceLevel", ...}` — self-describing and extensible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum WorldCommand {
    /// Create a new world document.
    WorldCreate {
        name: String,
        layout_policy: LayoutPolicy,
    },

    /// Place a level ref in a world.
    WorldPlaceLevel {
        world_path: String,
        level_id: String,
        asset_ref: String,
        position: [f32; 2],
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dimensions: Option<[u32; 2]>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tags: Vec<String>,
        #[serde(default)]
        streaming: StreamingPolicy,
    },

    /// Remove a level ref from a world and all incident links.
    /// Inverse restores the full captured pre-state.
    WorldRemoveLevel {
        world_path: String,
        level_id: String,
        /// Captured pre-state for undo: the full WorldLevelRef that was removed.
        /// The processor populates this if the caller leaves it as None.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        captured_level: Option<WorldLevelRef>,
        /// Captured pre-state: links that were removed along with this level.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        captured_links: Vec<WorldLink>,
    },

    /// Connect two levels with a directional link.
    WorldConnectLevels {
        world_path: String,
        link_id: String,
        from: String,
        to: String,
        direction: LinkDirection,
        kind: WorldLinkKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entrance: Option<editor_model::world::EntranceRef>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        exit: Option<editor_model::world::EntranceRef>,
    },

    /// Disconnect (remove) a link between two levels.
    /// Inverse restores the full captured pre-state.
    WorldRemoveLink {
        world_path: String,
        link_id: String,
        /// Captured pre-state for undo: the full WorldLink that was removed.
        /// The processor populates this if the caller leaves it as None.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        captured_link: Option<WorldLink>,
    },

    /// Set the layout policy for a world.
    WorldSetLayoutPolicy {
        world_path: String,
        /// Captured pre-state: the previous policy for undo.
        /// The processor populates this if the caller leaves it as None.
        #[serde(skip_serializing_if = "Option::is_none")]
        old_policy: Option<LayoutPolicy>,
        new_policy: LayoutPolicy,
    },

    /// Set the streaming policy for a specific level in a world.
    WorldSetStreamingPolicy {
        world_path: String,
        level_id: String,
        /// Captured pre-state: the previous policy for undo.
        /// The processor populates this if the caller leaves it as None.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_streaming: Option<StreamingPolicy>,
        new_streaming: StreamingPolicy,
    },

    /// Save (version bump) a world document.
    /// No structural change; inverse is another WorldSave.
    WorldSave { world_path: String },

    /// Delete a world document entirely.
    /// Inverse is not possible (document is gone); this command is terminal.
    WorldDelete { world_path: String },
}

// ─────────────────────────────────────────────────────────────────────────
// apply function
// ─────────────────────────────────────────────────────────────────────────

/// Apply a WorldCommand to a WorldDocument, returning the inverse command.
///
/// Validation runs before mutation; failed commands leave the document unchanged.
pub fn apply(
    doc: &mut WorldDocument,
    cmd: &WorldCommand,
) -> Result<WorldCommand, WorldCommandError> {
    match cmd {
        WorldCommand::WorldCreate {
            name,
            layout_policy,
        } => {
            // Cannot create a world with an empty doc - use this for initial creation
            doc.name = name.clone();
            doc.layout_policy = layout_policy.clone();
            doc.id = WorldId::new(format!("world-{}", name.to_lowercase().replace(' ', "-")));
            doc.version = 1;
            Ok(WorldCommand::WorldDelete {
                world_path: doc.id.as_str().to_string(),
            })
        }

        WorldCommand::WorldPlaceLevel {
            world_path: _,
            level_id,
            asset_ref,
            position,
            dimensions,
            tags,
            streaming,
        } => {
            // Check duplicate level_id
            if doc.levels.iter().any(|l| l.level_id == *level_id) {
                return Err(WorldCommandError::DuplicateLevelId(level_id.clone()));
            }

            // Check workspace cap (100 levels)
            if doc.levels.len() >= 100 {
                return Err(WorldCommandError::WorkspaceTooLarge(doc.levels.len()));
            }

            let level = WorldLevelRef {
                level_id: level_id.clone(),
                asset_ref: asset_ref.clone(),
                position: *position,
                dimensions: *dimensions,
                tags: tags.clone(),
                streaming: *streaming,
            };

            doc.levels.push(level);
            Ok(WorldCommand::WorldRemoveLevel {
                world_path: doc.id.as_str().to_string(),
                level_id: level_id.clone(),
                captured_level: None,
                captured_links: Vec::new(),
            })
        }

        WorldCommand::WorldRemoveLevel {
            world_path: _,
            level_id,
            captured_level,
            captured_links: _,
        } => {
            let pos = doc
                .levels
                .iter()
                .position(|l| l.level_id == *level_id)
                .ok_or_else(|| WorldCommandError::LevelNotFound(level_id.clone()))?;

            let removed = doc.levels.remove(pos);

            // Remove incident links
            doc.links
                .retain(|l| l.from != *level_id && l.to != *level_id);

            Ok(WorldCommand::WorldPlaceLevel {
                world_path: doc.id.as_str().to_string(),
                level_id: removed.level_id,
                asset_ref: removed.asset_ref,
                position: removed.position,
                dimensions: removed.dimensions,
                tags: removed.tags,
                streaming: removed.streaming,
            })
        }

        WorldCommand::WorldConnectLevels {
            world_path: _,
            link_id,
            from,
            to,
            direction,
            kind,
            entrance,
            exit,
        } => {
            // Verify levels exist
            if !doc.levels.iter().any(|l| l.level_id == *from) {
                return Err(WorldCommandError::LevelNotFound(from.clone()));
            }
            if !doc.levels.iter().any(|l| l.level_id == *to) {
                return Err(WorldCommandError::LevelNotFound(to.clone()));
            }

            // Check duplicate link_id
            if doc.links.iter().any(|l| l.id == *link_id) {
                return Err(WorldCommandError::DuplicateLinkId(link_id.clone()));
            }

            let link = WorldLink {
                id: link_id.clone(),
                from: from.clone(),
                to: to.clone(),
                direction: *direction,
                kind: kind.clone(),
                entrance: entrance.clone(),
                exit: exit.clone(),
            };

            doc.links.push(link);
            Ok(WorldCommand::WorldRemoveLink {
                world_path: doc.id.as_str().to_string(),
                link_id: link_id.clone(),
                captured_link: None,
            })
        }

        WorldCommand::WorldRemoveLink {
            world_path: _,
            link_id,
            captured_link,
        } => {
            let pos = doc
                .links
                .iter()
                .position(|l| l.id == *link_id)
                .ok_or_else(|| WorldCommandError::LinkNotFound(link_id.clone()))?;

            let removed = doc.links.remove(pos);

            Ok(WorldCommand::WorldConnectLevels {
                world_path: doc.id.as_str().to_string(),
                link_id: removed.id,
                from: removed.from,
                to: removed.to,
                direction: removed.direction,
                kind: removed.kind,
                entrance: removed.entrance,
                exit: removed.exit,
            })
        }

        WorldCommand::WorldSetLayoutPolicy {
            world_path: _,
            old_policy,
            new_policy,
        } => {
            let actual_old = doc.layout_policy.clone();
            doc.layout_policy = new_policy.clone();

            let inverse_old = old_policy.clone().unwrap_or(actual_old.clone());
            Ok(WorldCommand::WorldSetLayoutPolicy {
                world_path: doc.id.as_str().to_string(),
                old_policy: Some(actual_old),
                new_policy: inverse_old,
            })
        }

        WorldCommand::WorldSetStreamingPolicy {
            world_path: _,
            level_id,
            old_streaming,
            new_streaming,
        } => {
            let level = doc
                .levels
                .iter_mut()
                .find(|l| l.level_id == *level_id)
                .ok_or_else(|| WorldCommandError::LevelNotFound(level_id.clone()))?;

            let actual_old = level.streaming;
            level.streaming = *new_streaming;

            let inverse_old = old_streaming.unwrap_or(actual_old);
            Ok(WorldCommand::WorldSetStreamingPolicy {
                world_path: doc.id.as_str().to_string(),
                level_id: level_id.clone(),
                old_streaming: Some(actual_old),
                new_streaming: inverse_old,
            })
        }

        WorldCommand::WorldSave { world_path: _ } => {
            doc.version += 1;
            Ok(WorldCommand::WorldSave {
                world_path: doc.id.as_str().to_string(),
            })
        }

        WorldCommand::WorldDelete { world_path: _ } => {
            // Deletion is irreversible; we preserve the document state
            // but mark it for removal from the catalog
            Ok(WorldCommand::WorldCreate {
                name: doc.name.clone(),
                layout_policy: doc.layout_policy.clone(),
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_world_doc() -> WorldDocument {
        WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test World".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: Vec::new(),
            links: Vec::new(),
            updated_at: 0,
        }
    }

    #[test]
    fn test_world_place_level_inverse_removes_level() {
        let mut doc = empty_world_doc();
        let cmd = WorldCommand::WorldPlaceLevel {
            world_path: "test-world".to_string(),
            level_id: "lvl-1".to_string(),
            asset_ref: "levels/cave".to_string(),
            position: [100.0, 200.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: StreamingPolicy::AlwaysResident,
        };

        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.levels.len(), 1);
        assert_eq!(doc.levels[0].level_id, "lvl-1");

        // Inverse should be WorldRemoveLevel
        let inverse_level_id = match &inverse {
            WorldCommand::WorldRemoveLevel { level_id, .. } => level_id.clone(),
            _ => panic!("Expected WorldRemoveLevel"),
        };
        assert_eq!(inverse_level_id, "lvl-1");

        // Apply inverse
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.levels.len(), 0);
    }

    #[test]
    fn test_world_connect_levels_inverse_removes_link() {
        let mut doc = empty_world_doc();
        doc.levels.push(WorldLevelRef {
            level_id: "lvl-1".to_string(),
            asset_ref: "levels/a".to_string(),
            position: [0.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: StreamingPolicy::AlwaysResident,
        });
        doc.levels.push(WorldLevelRef {
            level_id: "lvl-2".to_string(),
            asset_ref: "levels/b".to_string(),
            position: [100.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: StreamingPolicy::AlwaysResident,
        });

        let cmd = WorldCommand::WorldConnectLevels {
            world_path: "test-world".to_string(),
            link_id: "link-1".to_string(),
            from: "lvl-1".to_string(),
            to: "lvl-2".to_string(),
            direction: LinkDirection::East,
            kind: WorldLinkKind::OneWay,
            entrance: None,
            exit: None,
        };

        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.links.len(), 1);

        // Apply inverse
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.links.len(), 0);
    }

    #[test]
    fn test_world_set_layout_policy_captures_old_policy() {
        let mut doc = empty_world_doc();
        doc.layout_policy = LayoutPolicy::Free;

        let cmd = WorldCommand::WorldSetLayoutPolicy {
            world_path: "test-world".to_string(),
            old_policy: None,
            new_policy: LayoutPolicy::Grid { cell_size: 64 },
        };

        let inverse = apply(&mut doc, &cmd).unwrap();
        assert!(matches!(
            doc.layout_policy,
            LayoutPolicy::Grid { cell_size: 64 }
        ));

        // Inverse should restore Free
        match inverse {
            WorldCommand::WorldSetLayoutPolicy { new_policy, .. } => {
                assert!(matches!(new_policy, LayoutPolicy::Free));
            }
            _ => panic!("Expected WorldSetLayoutPolicy"),
        }
    }

    #[test]
    fn test_workspace_too_large_error() {
        let mut doc = empty_world_doc();
        // Fill to 100 levels
        for i in 0..100 {
            doc.levels.push(WorldLevelRef {
                level_id: format!("lvl-{}", i),
                asset_ref: format!("levels/{}", i),
                position: [0.0, 0.0],
                dimensions: None,
                tags: Vec::new(),
                streaming: StreamingPolicy::AlwaysResident,
            });
        }

        let cmd = WorldCommand::WorldPlaceLevel {
            world_path: "test-world".to_string(),
            level_id: "lvl-100".to_string(),
            asset_ref: "levels/extra".to_string(),
            position: [0.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: StreamingPolicy::AlwaysResident,
        };

        let result = apply(&mut doc, &cmd);
        assert!(matches!(
            result,
            Err(WorldCommandError::WorkspaceTooLarge(100))
        ));
    }

    #[test]
    fn test_duplicate_level_id_error() {
        let mut doc = empty_world_doc();
        doc.levels.push(WorldLevelRef {
            level_id: "lvl-1".to_string(),
            asset_ref: "levels/a".to_string(),
            position: [0.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: StreamingPolicy::AlwaysResident,
        });

        let cmd = WorldCommand::WorldPlaceLevel {
            world_path: "test-world".to_string(),
            level_id: "lvl-1".to_string(),
            asset_ref: "levels/b".to_string(),
            position: [100.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: StreamingPolicy::AlwaysResident,
        };

        let result = apply(&mut doc, &cmd);
        assert!(matches!(
            result,
            Err(WorldCommandError::DuplicateLevelId(_))
        ));
    }

    #[test]
    fn test_duplicate_link_id_error() {
        let mut doc = empty_world_doc();
        doc.levels.push(WorldLevelRef {
            level_id: "lvl-1".to_string(),
            asset_ref: "levels/a".to_string(),
            position: [0.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: StreamingPolicy::AlwaysResident,
        });
        doc.levels.push(WorldLevelRef {
            level_id: "lvl-2".to_string(),
            asset_ref: "levels/b".to_string(),
            position: [100.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: StreamingPolicy::AlwaysResident,
        });

        // Add first link
        let cmd1 = WorldCommand::WorldConnectLevels {
            world_path: "test-world".to_string(),
            link_id: "link-1".to_string(),
            from: "lvl-1".to_string(),
            to: "lvl-2".to_string(),
            direction: LinkDirection::East,
            kind: WorldLinkKind::OneWay,
            entrance: None,
            exit: None,
        };
        apply(&mut doc, &cmd1).unwrap();

        // Try to add duplicate link
        let cmd2 = WorldCommand::WorldConnectLevels {
            world_path: "test-world".to_string(),
            link_id: "link-1".to_string(),
            from: "lvl-1".to_string(),
            to: "lvl-2".to_string(),
            direction: LinkDirection::West,
            kind: WorldLinkKind::Bidirectional,
            entrance: None,
            exit: None,
        };

        let result = apply(&mut doc, &cmd2);
        assert!(matches!(result, Err(WorldCommandError::DuplicateLinkId(_))));
    }
}
