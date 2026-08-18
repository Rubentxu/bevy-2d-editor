//! World Workspace recipe registry.
//!
//! Ships metadata for built-in world recipes. The actual recipe functions
//! live in `world_recipes.rs`.
//!
//! ## Recipe: create_room_chain
//!
//! - ID: `world.room_chain.v1`
//! - Produces `WorldConnectLevels` commands for each consecutive pair of levels.
//! - Permission: `Capability::Commands` (produces `WorldCommand`s)

use editor_model::extension::{Capability, CapabilityDescriptor};

/// Metadata for a built-in world recipe.
#[derive(Debug, Clone)]
pub struct WorldRecipeMetadata {
    /// Unique recipe identifier.
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Description of what the recipe does.
    pub description: &'static str,
    /// Required capability to invoke this recipe.
    pub capability: Capability,
}

/// Return all built-in world recipes as a slice.
pub fn list_world_recipes() -> &'static [WorldRecipeMetadata] {
    &[WorldRecipeMetadata {
        id: "world.room_chain.v1",
        name: "Create Room Chain",
        description: "Connect consecutive levels in a chain with directional links. Takes a list of level IDs and produces WorldConnectLevels commands for each adjacent pair.",
        capability: Capability::Commands,
    }]
}

/// Return true if the given recipe ID is a built-in world recipe.
pub fn is_world_recipe(id: &str) -> bool {
    list_world_recipes().iter().any(|r| r.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_world_recipes_returns_one() {
        let recipes = list_world_recipes();
        assert_eq!(recipes.len(), 1);
        assert_eq!(recipes[0].id, "world.room_chain.v1");
    }

    #[test]
    fn test_is_world_recipe_correctly_identifies() {
        assert!(is_world_recipe("world.room_chain.v1"));
        assert!(!is_world_recipe("world.nonexistent"));
        assert!(!is_world_recipe("logic.jump"));
    }

    #[test]
    fn test_world_recipe_capability_is_commands() {
        let recipes = list_world_recipes();
        assert!(matches!(recipes[0].capability, Capability::Commands));
    }
}
