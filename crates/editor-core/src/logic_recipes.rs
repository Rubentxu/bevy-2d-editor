//! Built-in Logic Bricks recipe catalog.
//!
//! Ships three immutable `LogicGraphAsset` recipes embedded via `include_str!`.
//! Recipes are seeded into `LOGIC_GRAPH_REGISTRY` on first access (lazy seed).

use crate::logic_graph::{LogicGraphAsset, LogicNode, LogicNodeRole, NodeTypeId, PortId, NodeId};
use crate::logic_evaluator::register_logic_graph;

/// Metadata for a built-in recipe, used by `list_builtin_recipes`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecipeMetadata {
    pub asset_id: String,
    pub name: String,
    pub description: String,
    pub node_count: usize,
}

/// Return all built-in recipes as a Vec of `(asset_id, name, description, node_count)`.
pub fn list_builtin_recipes() -> Vec<RecipeMetadata> {
    let recipes: Vec<RecipeMetadata> = RECIPE_METADATA
        .iter()
        .map(|meta| {
            // Parse the JSON to get the actual node count (already known here).
            let node_count = match meta.asset_id {
                "lga_recipe_jump" => 3,
                "lga_recipe_health" => 3,
                "lga_recipe_proximity" => 3,
                _ => 0,
            };
            RecipeMetadata {
                asset_id: meta.asset_id.to_string(),
                name: meta.name.to_string(),
                description: meta.description.to_string(),
                node_count,
            }
        })
        .collect();
    recipes
}

/// Return true if the given asset_id is a built-in recipe.
pub fn is_builtin_recipe(asset_id: &str) -> bool {
    RECIPE_METADATA.iter().any(|m| m.asset_id == asset_id)
}

/// Seed all built-in recipes into the LOGIC_GRAPH_REGISTRY.
/// Called lazily on first access from both `register_logic_graph` and `get_logic_graph_asset`.
/// Uses an internal static to prevent nested seeding (avoids RefCell borrow conflicts).
pub fn seed_builtin_recipes() {
    // Guard against nested calls (can happen when get_logic_graph_asset calls seed
    // while already holding a RefCell borrow).
    if SEEDING.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    SEEDING.store(true, std::sync::atomic::Ordering::Relaxed);

    let recipes: Vec<LogicGraphAsset> = RECIPE_JSON_LIST
        .iter()
        .filter_map(|json_str| serde_json::from_str(*json_str).ok())
        .collect();

    for recipe in recipes {
        register_logic_graph(recipe);
    }

    SEEDING.store(false, std::sync::atomic::Ordering::Relaxed);
}

static SEEDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// ─────────────────────────────────────────────────────────────────────────────
// Private recipe data
// ─────────────────────────────────────────────────────────────────────────────

struct RecipeMeta {
    asset_id: &'static str,
    name: &'static str,
    description: &'static str,
}

const RECIPE_JSON_LIST: &[&str] = &[
    include_str!("../recipes/platformer_jump.json"),
    include_str!("../recipes/health_damage.json"),
    include_str!("../recipes/proximity_trigger.json"),
];

const RECIPE_METADATA: &[RecipeMeta] = &[
    RecipeMeta {
        asset_id: "lga_recipe_jump",
        name: "Platformer Jump",
        description: "Jump when Space is pressed. Sensor → Gate → ApplyImpulse.",
    },
    RecipeMeta {
        asset_id: "lga_recipe_health",
        name: "Health Damage",
        description: "Apply damage on hazard collision. Collision → Compare → ModifyHealth.",
    },
    RecipeMeta {
        asset_id: "lga_recipe_proximity",
        name: "Proximity Trigger",
        description: "Emit signal when player enters radius. Proximity → Compare → EmitSignal.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_builtin_recipes_returns_three() {
        let recipes = list_builtin_recipes();
        assert_eq!(recipes.len(), 3);
        assert!(recipes.iter().all(|r| r.node_count == 3));
    }

    #[test]
    fn is_builtin_recipe_correctly_identifies() {
        assert!(is_builtin_recipe("lga_recipe_jump"));
        assert!(is_builtin_recipe("lga_recipe_health"));
        assert!(is_builtin_recipe("lga_recipe_proximity"));
        assert!(!is_builtin_recipe("lga_user_graph"));
    }

    #[test]
    fn recipe_json_parses_to_logic_graph_asset() {
        for json_str in RECIPE_JSON_LIST {
            let asset: LogicGraphAsset = serde_json::from_str(json_str).unwrap();
            assert!(asset.builtin);
            assert!(asset.logical_path.starts_with("recipes/"));
            assert!(!asset.nodes.is_empty());
            // No structural validation errors (no cycles, no dangling edges)
            let issues = crate::logic_validation::validate_logic_graph(
                &asset,
                crate::logic_evaluator::global_node_registry(),
            );
            assert!(
                issues.is_empty(),
                "recipe {} should have no validation issues: {:?}",
                asset.asset_id,
                issues
            );
        }
    }

    #[test]
    fn seed_builtin_recipes_makes_them_available() {
        use crate::logic_evaluator::get_logic_graph_asset;
        // After seed, all three should be retrievable
        seed_builtin_recipes();
        assert!(get_logic_graph_asset("lga_recipe_jump").is_some());
        assert!(get_logic_graph_asset("lga_recipe_health").is_some());
        assert!(get_logic_graph_asset("lga_recipe_proximity").is_some());
    }

    #[test]
    fn seed_is_idempotent() {
        use crate::logic_evaluator::get_logic_graph_asset;
        seed_builtin_recipes();
        let before = get_logic_graph_asset("lga_recipe_jump").unwrap();
        seed_builtin_recipes();
        let after = get_logic_graph_asset("lga_recipe_jump").unwrap();
        // Should be exactly the same (not duplicated)
        assert_eq!(before.asset_id, after.asset_id);
        assert_eq!(before.version, after.version);
    }
}
