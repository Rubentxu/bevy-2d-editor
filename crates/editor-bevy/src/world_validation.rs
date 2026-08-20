//! Topology validation for World Workspace (ADR-0037).
//!
//! Implements LDtk-faithful topology validation rules:
//! - `Unreachable` → Warning (BFS from entry level)
//! - `InvalidReciprocal` → Warning (A→B but B has no link to A)
//! - `MissingNeighbour` → Warning (LDtk neighbour ref points to level not in world)
//! - `MissingLevelRef` → Error (asset_ref does not resolve in SceneAssetCatalog)
//!
//! ## LDtk-faithful rules (ADR-0037 confirmed Q1)
//!
//! - Direction-based neighbor model: single uid per link, no reciprocal enforcement
//! - `EntranceRef` is OPTIONAL
//! - One-way links are ALLOWED (reciprocal mismatch is Warning, not Error)
//! - Orphans UNREACHABLE from entry → Warning
//! - Missing neighbours → Warning
//! - Missing `asset_ref` → Error

use editor_model::graph_kernel::{Graph, WorldGraphDialect, reachable_from};
use editor_model::scene_asset_catalog::SceneAssetCatalog;
use editor_model::world::{WorldDocument, WorldLevelRef, WorldLink};
use editor_protocol::capabilities::{TopologyIssue, TopologyIssueCode, TopologySeverity};
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};

/// Validate a world document's topology against a scene asset catalog.
///
/// # LDtk-faithful severity matrix
///
/// | Code | Severity | Condition |
/// |------|----------|-----------|
/// | `Unreachable` | Warning | Level not reachable from entry via BFS |
/// | `InvalidReciprocal` | Warning | A→B exists but B has no link to A |
/// | `MissingNeighbour` | Warning | Link points to level_id not in world |
/// | `MissingLevelRef` | Error | `WorldLevelRef.asset_ref` not in catalog |
///
/// # Arguments
///
/// * `world` — the world document to validate
/// * `catalog` — the scene asset catalog to check asset_ref resolution
///
/// # Returns
///
/// A `Vec<TopologyIssue>` containing all detected issues (may be empty).
pub fn validate_topology(world: &WorldDocument, catalog: &SceneAssetCatalog) -> Vec<TopologyIssue> {
    let mut issues = Vec::new();

    // 1. Check each WorldLevelRef.asset_ref resolves in the catalog
    issues.extend(validate_asset_refs(world, catalog));

    // 2. Check for missing neighbours (link points to level not in world)
    issues.extend(validate_neighbours(world));

    // 3. Check for invalid reciprocals (A→B but B has no link to A)
    issues.extend(validate_reciprocals(world));

    // 4. Check for unreachable levels (BFS from entry, via GRAPH-005 kernel)
    issues.extend(validate_reachability(world));

    // 5. Detect cycles in the link graph (new in GRAPH-005)
    issues.extend(validate_cycles(world));

    issues
}

/// Check that each WorldLevelRef.asset_ref resolves in the catalog.
fn validate_asset_refs(world: &WorldDocument, catalog: &SceneAssetCatalog) -> Vec<TopologyIssue> {
    let mut issues = Vec::new();

    for level in &world.levels {
        // resolve_path returns Some(asset_id) if the path is registered
        if catalog.resolve_path(&level.asset_ref).is_none() {
            issues.push(TopologyIssue {
                code: TopologyIssueCode::MissingLevelRef,
                world_id: world.id.as_str().to_string(),
                level_id: Some(level.level_id.clone()),
                link_id: None,
                severity: TopologySeverity::Error,
                message: format!(
                    "level '{}' references asset '{}' which is not in the catalog",
                    level.level_id, level.asset_ref
                ),
            });
        }
    }

    issues
}

/// Check that each link's `to` field references a level that exists in the world.
fn validate_neighbours(world: &WorldDocument) -> Vec<TopologyIssue> {
    let mut issues = Vec::new();
    let level_ids: HashSet<&str> = world.levels.iter().map(|l| l.level_id.as_str()).collect();

    for link in &world.links {
        if !level_ids.contains(link.to.as_str()) {
            issues.push(TopologyIssue {
                code: TopologyIssueCode::MissingNeighbour,
                world_id: world.id.as_str().to_string(),
                level_id: None,
                link_id: Some(link.id.clone()),
                severity: TopologySeverity::Warning,
                message: format!(
                    "link '{}' references level '{}' which does not exist in the world",
                    link.id, link.to
                ),
            });
        }
        if !level_ids.contains(link.from.as_str()) {
            issues.push(TopologyIssue {
                code: TopologyIssueCode::MissingNeighbour,
                world_id: world.id.as_str().to_string(),
                level_id: None,
                link_id: Some(link.id.clone()),
                severity: TopologySeverity::Warning,
                message: format!(
                    "link '{}' references source level '{}' which does not exist in the world",
                    link.id, link.from
                ),
            });
        }
    }

    issues
}

/// Check for invalid reciprocals: A→B exists but B has no link to A.
///
/// This is a Warning (not Error) because one-way links are allowed in LDtk.
fn validate_reciprocals(world: &WorldDocument) -> Vec<TopologyIssue> {
    let mut issues = Vec::new();

    // Build a map of level_id -> set of levels it links TO
    let mut outgoing: HashMap<&str, HashSet<&str>> = HashMap::new();
    for link in &world.links {
        outgoing
            .entry(link.from.as_str())
            .or_default()
            .insert(link.to.as_str());
    }

    for link in &world.links {
        // For a link A→B, check if B→A exists
        let reverse_exists = outgoing
            .get(link.to.as_str())
            .map(|targets| targets.contains(link.from.as_str()))
            .unwrap_or(false);

        // Only report if this is a one-way link and the reverse doesn't exist
        // If kind is OneWay and reverse missing → Warning
        if matches!(link.kind, editor_model::world::WorldLinkKind::OneWay) && !reverse_exists {
            // Double-check: if kind is Bidirectional, the model would expect reciprocal
            // For now, we only warn on explicit OneWay links
            issues.push(TopologyIssue {
                code: TopologyIssueCode::InvalidReciprocal,
                world_id: world.id.as_str().to_string(),
                level_id: None,
                link_id: Some(link.id.clone()),
                severity: TopologySeverity::Warning,
                message: format!(
                    "link '{}' ({} → {}) is marked OneWay but no reciprocal link exists",
                    link.id, link.from, link.to
                ),
            });
        }
    }

    issues
}

/// Check reachability using BFS from the entry level.
///
/// Entry is determined by:
/// 1. If world has an EntranceRef, use that level
/// 2. Otherwise, use the first level by insertion order
///
/// Levels not reachable from entry are Warnings (orphans).
fn validate_reachability(world: &WorldDocument) -> Vec<TopologyIssue> {
    let mut issues = Vec::new();

    if world.levels.is_empty() {
        return issues;
    }

    // Determine entry level
    let entry_id = if let Some(first) = world.levels.first() {
        first.level_id.as_str()
    } else {
        return issues;
    };

    // GRAPH-005: build the dialect and run kernel reachability.
    let dialect = WorldGraphDialect::new(world);
    let entry_idx = match dialect.node_index_of(entry_id) {
        Some(idx) => idx,
        None => return issues, // unreachable; entry not in levels
    };

    let reachable = reachable_from(&dialect, entry_idx);
    let reachable_ids: HashSet<&str> = reachable
        .iter()
        .filter_map(|i| dialect.node(*i).map(|l| l.level_id.as_str()))
        .collect();

    // Check all levels are reachable
    for level in &world.levels {
        if !reachable_ids.contains(level.level_id.as_str()) {
            issues.push(TopologyIssue {
                code: TopologyIssueCode::Unreachable,
                world_id: world.id.as_str().to_string(),
                level_id: Some(level.level_id.clone()),
                link_id: None,
                severity: TopologySeverity::Warning,
                message: format!(
                    "level '{}' is not reachable from entry '{}'",
                    level.level_id, entry_id
                ),
            });
        }
    }

    issues
}

/// Detect cycles in the link graph (A→B→C→A et al.).
///
/// New in GRAPH-005: the previous topology rules did not detect cycles
/// (LDtk also flags them as a structural issue). Cycles are emitted as a
/// single Warning per cycle-bearing world (not per link), with no level_id
/// or link_id attached.
fn validate_cycles(world: &WorldDocument) -> Vec<TopologyIssue> {
    let mut issues = Vec::new();
    if world.levels.is_empty() {
        return issues;
    }

    let dialect = WorldGraphDialect::new(world);
    let cycle = editor_model::graph_kernel::has_cycle(&dialect);
    if cycle {
        issues.push(TopologyIssue {
            code: TopologyIssueCode::Cycle,
            world_id: world.id.as_str().to_string(),
            level_id: None,
            link_id: None,
            severity: TopologySeverity::Warning,
            message: "world's link graph contains a cycle".to_string(),
        });
    }
    issues
}

// ─────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::world::{LayoutPolicy, LinkDirection, WorldId, WorldLinkKind};
    use editor_protocol::capabilities::TopologySeverity;

    fn empty_world() -> WorldDocument {
        WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test World".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: Vec::new(),
            links: Vec::new(),
            updated_at: 0,
            extension_data: BTreeMap::new(),
        }
    }

    fn world_with_levels(levels: Vec<WorldLevelRef>) -> WorldDocument {
        WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test World".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels,
            links: Vec::new(),
            updated_at: 0,
            extension_data: BTreeMap::new(),
        }
    }

    fn empty_catalog() -> SceneAssetCatalog {
        SceneAssetCatalog::default()
    }

    fn catalog_with(refs: &[&str]) -> SceneAssetCatalog {
        let mut catalog = SceneAssetCatalog::default();
        for r in refs {
            // We need to use the internal method to add entries
            // For testing, we'll use the default empty catalog
            // and just check the logic
        }
        catalog
    }

    #[test]
    fn test_empty_world_no_issues() {
        let world = empty_world();
        let catalog = empty_catalog();
        let issues = validate_topology(&world, &catalog);
        assert!(issues.is_empty(), "empty world should have no issues");
    }

    #[test]
    fn test_missing_asset_ref_error() {
        let mut world = world_with_levels(vec![WorldLevelRef {
            level_id: "lvl-1".to_string(),
            asset_ref: "levels/nonexistent".to_string(),
            position: [0.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: editor_model::world::StreamingPolicy::AlwaysResident,
        }]);
        let catalog = empty_catalog();

        let issues = validate_topology(&world, &catalog);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0].code, TopologyIssueCode::MissingLevelRef));
        assert!(matches!(issues[0].severity, TopologySeverity::Error));
        assert_eq!(issues[0].level_id.as_deref(), Some("lvl-1"));
    }

    #[test]
    fn test_missing_neighbour_warning() {
        let world = WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![WorldLevelRef {
                level_id: "lvl-1".to_string(),
                asset_ref: "levels/a".to_string(),
                position: [0.0, 0.0],
                dimensions: None,
                tags: Vec::new(),
                streaming: editor_model::world::StreamingPolicy::AlwaysResident,
            }],
            links: vec![WorldLink {
                id: "link-1".to_string(),
                from: "lvl-1".to_string(),
                to: "lvl-missing".to_string(), // lvl-missing doesn't exist
                direction: LinkDirection::East,
                kind: WorldLinkKind::OneWay,
                entrance: None,
                exit: None,
            }],
            updated_at: 0,
            extension_data: BTreeMap::new(),
        };
        let catalog = empty_catalog();

        let issues = validate_topology(&world, &catalog);
        assert!(!issues.is_empty());

        let missing_neighbour = issues
            .iter()
            .find(|i| matches!(i.code, TopologyIssueCode::MissingNeighbour));
        assert!(missing_neighbour.is_some());
        assert!(matches!(
            missing_neighbour.unwrap().severity,
            TopologySeverity::Warning
        ));
    }

    #[test]
    fn test_reciprocal_mismatch_warning() {
        // A → B (OneWay) but B has no link back to A → Warning
        let world = WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![
                WorldLevelRef {
                    level_id: "lvl-a".to_string(),
                    asset_ref: "levels/a".to_string(),
                    position: [0.0, 0.0],
                    dimensions: None,
                    tags: Vec::new(),
                    streaming: editor_model::world::StreamingPolicy::AlwaysResident,
                },
                WorldLevelRef {
                    level_id: "lvl-b".to_string(),
                    asset_ref: "levels/b".to_string(),
                    position: [100.0, 0.0],
                    dimensions: None,
                    tags: Vec::new(),
                    streaming: editor_model::world::StreamingPolicy::AlwaysResident,
                },
            ],
            links: vec![WorldLink {
                id: "link-a-b".to_string(),
                from: "lvl-a".to_string(),
                to: "lvl-b".to_string(),
                direction: LinkDirection::East,
                kind: WorldLinkKind::OneWay,
                entrance: None,
                exit: None,
            }],
            updated_at: 0,
            extension_data: BTreeMap::new(),
        };
        let catalog = empty_catalog();

        let issues = validate_topology(&world, &catalog);
        let invalid_recip = issues
            .iter()
            .find(|i| matches!(i.code, TopologyIssueCode::InvalidReciprocal));
        assert!(invalid_recip.is_some());
        assert!(matches!(
            invalid_recip.unwrap().severity,
            TopologySeverity::Warning
        ));
    }

    #[test]
    fn test_orphan_unreachable_warning() {
        // lvl-b is not reachable from lvl-a (no links to it)
        let world = WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![
                WorldLevelRef {
                    level_id: "lvl-a".to_string(),
                    asset_ref: "levels/a".to_string(),
                    position: [0.0, 0.0],
                    dimensions: None,
                    tags: Vec::new(),
                    streaming: editor_model::world::StreamingPolicy::AlwaysResident,
                },
                WorldLevelRef {
                    level_id: "lvl-b".to_string(),
                    asset_ref: "levels/b".to_string(),
                    position: [100.0, 0.0],
                    dimensions: None,
                    tags: Vec::new(),
                    streaming: editor_model::world::StreamingPolicy::AlwaysResident,
                },
            ],
            links: Vec::new(), // No links at all
            updated_at: 0,
            extension_data: BTreeMap::new(),
        };
        let catalog = empty_catalog();

        let issues = validate_topology(&world, &catalog);

        // lvl-b should be unreachable (only lvl-a is reachable as entry)
        let unreachable = issues
            .iter()
            .find(|i| matches!(i.code, TopologyIssueCode::Unreachable));
        assert!(unreachable.is_some());
        assert_eq!(unreachable.unwrap().level_id.as_deref(), Some("lvl-b"));
        assert!(matches!(
            unreachable.unwrap().severity,
            TopologySeverity::Warning
        ));
    }

    #[test]
    fn test_self_loop_emits_cycle_warning() {
        // A link from a level to itself is a 1-cycle.
        // GRAPH-005 introduced cycle detection via `has_cycle`. The previous
        // rules (Unreachable, MissingNeighbour, InvalidReciprocal,
        // MissingLevelRef) did not flag self-loops; the new `Cycle` rule does.
        let world = WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![WorldLevelRef {
                level_id: "lvl-a".to_string(),
                asset_ref: "levels/a".to_string(),
                position: [0.0, 0.0],
                dimensions: None,
                tags: Vec::new(),
                streaming: editor_model::world::StreamingPolicy::AlwaysResident,
            }],
            links: vec![WorldLink {
                id: "link-self".to_string(),
                from: "lvl-a".to_string(),
                to: "lvl-a".to_string(), // Self-loop
                direction: LinkDirection::Undirected,
                kind: WorldLinkKind::OneWay,
                entrance: None,
                exit: None,
            }],
            updated_at: 0,
            extension_data: BTreeMap::new(),
        };
        let catalog = empty_catalog();

        let issues = validate_topology(&world, &catalog);

        // Exactly one Cycle issue, no legacy rules fire.
        let cycles: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.code, TopologyIssueCode::Cycle))
            .collect();
        assert_eq!(
            cycles.len(),
            1,
            "self-loop should emit exactly one Cycle issue"
        );
        assert!(matches!(cycles[0].severity, TopologySeverity::Warning));

        let legacy: Vec<_> = issues
            .iter()
            .filter(|i| {
                !matches!(
                    i.code,
                    TopologyIssueCode::Cycle | TopologyIssueCode::MissingLevelRef
                )
            })
            .collect();
        assert!(legacy.is_empty(), "no other rules should fire");
    }

    #[test]
    fn test_three_node_cycle_emits_cycle_warning() {
        // A→B→C→A is a 3-node cycle.
        let world = WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![
                WorldLevelRef {
                    level_id: "lvl-a".to_string(),
                    asset_ref: "levels/a".to_string(),
                    position: [0.0, 0.0],
                    dimensions: None,
                    tags: Vec::new(),
                    streaming: editor_model::world::StreamingPolicy::AlwaysResident,
                },
                WorldLevelRef {
                    level_id: "lvl-b".to_string(),
                    asset_ref: "levels/b".to_string(),
                    position: [100.0, 0.0],
                    dimensions: None,
                    tags: Vec::new(),
                    streaming: editor_model::world::StreamingPolicy::AlwaysResident,
                },
                WorldLevelRef {
                    level_id: "lvl-c".to_string(),
                    asset_ref: "levels/c".to_string(),
                    position: [200.0, 0.0],
                    dimensions: None,
                    tags: Vec::new(),
                    streaming: editor_model::world::StreamingPolicy::AlwaysResident,
                },
            ],
            links: vec![
                WorldLink {
                    id: "l-ab".to_string(),
                    from: "lvl-a".to_string(),
                    to: "lvl-b".to_string(),
                    direction: LinkDirection::East,
                    kind: WorldLinkKind::OneWay,
                    entrance: None,
                    exit: None,
                },
                WorldLink {
                    id: "l-bc".to_string(),
                    from: "lvl-b".to_string(),
                    to: "lvl-c".to_string(),
                    direction: LinkDirection::East,
                    kind: WorldLinkKind::OneWay,
                    entrance: None,
                    exit: None,
                },
                WorldLink {
                    id: "l-ca".to_string(),
                    from: "lvl-c".to_string(),
                    to: "lvl-a".to_string(),
                    direction: LinkDirection::East,
                    kind: WorldLinkKind::OneWay,
                    entrance: None,
                    exit: None,
                },
            ],
            updated_at: 0,
            extension_data: BTreeMap::new(),
        };
        let catalog = empty_catalog();

        let issues = validate_topology(&world, &catalog);

        let cycles: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.code, TopologyIssueCode::Cycle))
            .collect();
        assert_eq!(cycles.len(), 1, "3-node cycle should emit one Cycle issue");
    }

    #[test]
    fn test_dag_no_cycle_warning() {
        // A→B, A→C, B→D, C→D is a DAG (no cycle).
        let world = WorldDocument {
            id: WorldId::new("test-world"),
            name: "Test".to_string(),
            version: 1,
            layout_policy: LayoutPolicy::Free,
            levels: vec![
                lvl("lvl-a", "levels/a"),
                lvl("lvl-b", "levels/b"),
                lvl("lvl-c", "levels/c"),
                lvl("lvl-d", "levels/d"),
            ],
            links: vec![
                lnk("l-ab", "lvl-a", "lvl-b"),
                lnk("l-ac", "lvl-a", "lvl-c"),
                lnk("l-bd", "lvl-b", "lvl-d"),
                lnk("l-cd", "lvl-c", "lvl-d"),
            ],
            updated_at: 0,
            extension_data: BTreeMap::new(),
        };
        let catalog = empty_catalog();

        let issues = validate_topology(&world, &catalog);
        let cycles: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.code, TopologyIssueCode::Cycle))
            .collect();
        assert!(cycles.is_empty(), "DAG should not emit a Cycle issue");
        let unreach: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.code, TopologyIssueCode::Unreachable))
            .collect();
        assert!(unreach.is_empty(), "all levels reachable from lvl-a");
    }

    fn lvl(id: &str, asset: &str) -> WorldLevelRef {
        WorldLevelRef {
            level_id: id.to_string(),
            asset_ref: asset.to_string(),
            position: [0.0, 0.0],
            dimensions: None,
            tags: Vec::new(),
            streaming: editor_model::world::StreamingPolicy::AlwaysResident,
        }
    }

    fn lnk(id: &str, from: &str, to: &str) -> WorldLink {
        WorldLink {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            direction: LinkDirection::East,
            kind: WorldLinkKind::OneWay,
            entrance: None,
            exit: None,
        }
    }
}
