//! Built-in World Workspace recipe catalog.
//!
//! Ships the `create_room_chain` recipe which emits `WorldConnectLevels` commands
//! for each consecutive pair of levels in a chain.
//!
//! ## Recipe: create_room_chain
//!
//! Takes a sequence of level IDs and a direction, and produces `WorldConnectLevels`
//! commands for each consecutive pair: `A→B`, `B→C`, etc.
//!
//! Example: `create_room_chain(world, ["lvl-a", "lvl-b", "lvl-c"], East)`
//! produces: `A→B east`, `B→C east`

use editor_model::world::{LinkDirection, WorldDocument, WorldLinkKind};

use crate::world_command::WorldCommand;
use std::collections::BTreeMap;

/// Produce `WorldConnectLevels` commands for each consecutive pair of levels in a chain.
///
/// # Arguments
///
/// * `world` - The `WorldDocument` being authored (used to validate level IDs exist).
/// * `level_ids` - Ordered list of level IDs forming the chain.
/// * `direction` - The `LinkDirection` to use for each link in the chain.
///
/// # Returns
///
/// A `Vec<WorldCommand>` containing `WorldConnectLevels` for each consecutive pair.
/// If `level_ids.len() < 2`, returns an empty `Vec`.
///
/// # Example
///
/// ```
/// // 3 levels ["lvl-a", "lvl-b", "lvl-c"] with East direction produces:
/// // - Link: lvl-a → lvl-b (east)
/// // - Link: lvl-b → lvl-c (east)
/// ```
pub fn create_room_chain(
    world: &WorldDocument,
    level_ids: &[String],
    direction: LinkDirection,
) -> Vec<WorldCommand> {
    if level_ids.len() < 2 {
        return Vec::new();
    }

    let mut commands = Vec::new();

    for window in level_ids.windows(2) {
        let from = &window[0];
        let to = &window[1];

        // Validate both levels exist in the world
        if !world.levels.iter().any(|l| l.level_id == *from) {
            continue;
        }
        if !world.levels.iter().any(|l| l.level_id == *to) {
            continue;
        }

        let link_id = format!("chain_{}_{}", from, to);

        commands.push(WorldCommand::WorldConnectLevels {
            world_path: world.id.as_str().to_string(),
            link_id,
            from: from.clone(),
            to: to.clone(),
            direction,
            kind: WorldLinkKind::OneWay,
            entrance: None,
            exit: None,
        });
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::world::{LayoutPolicy, StreamingPolicy, WorldId, WorldLevelRef};

    fn make_test_world() -> WorldDocument {
        WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test World".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![
                WorldLevelRef {
                    level_id: "lvl-a".to_string(),
                    asset_ref: "levels/a".to_string(),
                    position: [0.0, 0.0],
                    dimensions: None,
                    tags: vec![],
                    streaming: StreamingPolicy::AlwaysResident,
                },
                WorldLevelRef {
                    level_id: "lvl-b".to_string(),
                    asset_ref: "levels/b".to_string(),
                    position: [100.0, 0.0],
                    dimensions: None,
                    tags: vec![],
                    streaming: StreamingPolicy::AlwaysResident,
                },
                WorldLevelRef {
                    level_id: "lvl-c".to_string(),
                    asset_ref: "levels/c".to_string(),
                    position: [200.0, 0.0],
                    dimensions: None,
                    tags: vec![],
                    streaming: StreamingPolicy::AlwaysResident,
                },
            ],
            links: Vec::new(),
            updated_at: 0,
            extension_data: BTreeMap::new(),
        }
    }

    #[test]
    fn test_create_room_chain_three_levels_produces_two_links() {
        let world = make_test_world();
        let level_ids = vec![
            "lvl-a".to_string(),
            "lvl-b".to_string(),
            "lvl-c".to_string(),
        ];

        let commands = create_room_chain(&world, &level_ids, LinkDirection::East);

        assert_eq!(commands.len(), 2);

        // First link: lvl-a → lvl-b
        match &commands[0] {
            WorldCommand::WorldConnectLevels {
                from,
                to,
                direction,
                ..
            } => {
                assert_eq!(from, "lvl-a");
                assert_eq!(to, "lvl-b");
                assert_eq!(*direction, LinkDirection::East);
            }
            _ => panic!("Expected WorldConnectLevels"),
        }

        // Second link: lvl-b → lvl-c
        match &commands[1] {
            WorldCommand::WorldConnectLevels {
                from,
                to,
                direction,
                ..
            } => {
                assert_eq!(from, "lvl-b");
                assert_eq!(to, "lvl-c");
                assert_eq!(*direction, LinkDirection::East);
            }
            _ => panic!("Expected WorldConnectLevels"),
        }
    }

    #[test]
    fn test_create_room_chain_two_levels_produces_one_link() {
        let world = make_test_world();
        let level_ids = vec!["lvl-a".to_string(), "lvl-b".to_string()];

        let commands = create_room_chain(&world, &level_ids, LinkDirection::North);

        assert_eq!(commands.len(), 1);

        match &commands[0] {
            WorldCommand::WorldConnectLevels {
                from,
                to,
                direction,
                ..
            } => {
                assert_eq!(from, "lvl-a");
                assert_eq!(to, "lvl-b");
                assert_eq!(*direction, LinkDirection::North);
            }
            _ => panic!("Expected WorldConnectLevels"),
        }
    }

    #[test]
    fn test_create_room_chain_single_level_returns_empty() {
        let world = make_test_world();
        let level_ids = vec!["lvl-a".to_string()];

        let commands = create_room_chain(&world, &level_ids, LinkDirection::East);

        assert!(commands.is_empty());
    }

    #[test]
    fn test_create_room_chain_empty_level_ids_returns_empty() {
        let world = make_test_world();
        let level_ids: Vec<String> = vec![];

        let commands = create_room_chain(&world, &level_ids, LinkDirection::East);

        assert!(commands.is_empty());
    }

    #[test]
    fn test_create_room_chain_skips_missing_levels() {
        let world = make_test_world();
        // lvl-a exists, missing-level does NOT exist, lvl-c exists
        let level_ids = vec![
            "lvl-a".to_string(),
            "missing-level".to_string(),
            "lvl-c".to_string(),
        ];

        let commands = create_room_chain(&world, &level_ids, LinkDirection::East);

        // Both pairs contain a missing level, so no links are produced
        // (lvl-a, missing-level) → missing-level doesn't exist → skip
        // (missing-level, lvl-c) → missing-level doesn't exist → skip
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_create_room_chain_all_missing_levels_returns_empty() {
        let world = make_test_world();
        let level_ids = vec!["missing-a".to_string(), "missing-b".to_string()];

        let commands = create_room_chain(&world, &level_ids, LinkDirection::East);

        assert!(commands.is_empty());
    }

    #[test]
    fn test_create_room_chain_west_direction() {
        let world = make_test_world();
        let level_ids = vec!["lvl-a".to_string(), "lvl-b".to_string()];

        let commands = create_room_chain(&world, &level_ids, LinkDirection::West);

        assert_eq!(commands.len(), 1);
        match &commands[0] {
            WorldCommand::WorldConnectLevels { direction, .. } => {
                assert_eq!(*direction, LinkDirection::West);
            }
            _ => panic!("Expected WorldConnectLevels"),
        }
    }

    #[test]
    fn test_create_room_chain_undirected() {
        let world = make_test_world();
        let level_ids = vec!["lvl-a".to_string(), "lvl-b".to_string()];

        let commands = create_room_chain(&world, &level_ids, LinkDirection::Undirected);

        assert_eq!(commands.len(), 1);
        match &commands[0] {
            WorldCommand::WorldConnectLevels { direction, .. } => {
                assert_eq!(*direction, LinkDirection::Undirected);
            }
            _ => panic!("Expected WorldConnectLevels"),
        }
    }
}
