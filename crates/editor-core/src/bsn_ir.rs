//! BSN IR — lossy semantic projection of SceneAssetDocument aligned with Bevy 0.19 BSN.
//! See ADR-0005 §BSN IR. One-way adapter only; write-back is future.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::scene_asset::{LocalId, RelationshipKind, SceneAssetDocument};

/// A relationship edge in the BSN IR graph: identifies a target node
/// and the kind of edge (e.g., "child", "reference").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BsnIrRelationship {
    pub kind: String,
    pub target_identifier: String,
}

/// One node in the BSN IR tree: a stable identifier, a flat map of
/// Hito 4 Order 7 (`scene-component-authoring`): discriminator for a BSN node.
///
/// `Plain` is the default (a regular entity with components). `SceneComponent`
/// indicates the node is the materialized instance of a `SceneComponent`
/// schema (Bevy 0.19 `#[derive(SceneComponent)]`); `type_id` is the
/// schema's `type_id` (e.g. `game.Enemy`).
///
/// When emitting, the codegen produces `@type_id { ... }` for `SceneComponent`
/// and plain `identifier { ... }` for `Plain`. When importing, the
/// `bsn!`-style `@` prefix is parsed into this variant.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    #[default]
    Plain,
    SceneComponent(String),
}

/// component type_id → component values, child nodes (nested), and
/// outgoing relationship edges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BsnIrNode {
    pub identifier: String,
    pub components: BTreeMap<String, serde_json::Value>,
    pub children: Vec<BsnIrNode>,
    pub relationships: Vec<BsnIrRelationship>,
    /// Hito 4 Order 7: discriminator. `Plain` (default) for regular entities,
    /// `SceneComponent(type_id)` for materialized SceneComponent instances.
    #[serde(default)]
    pub kind: NodeKind,
}

impl Default for BsnIrNode {
    fn default() -> Self {
        Self {
            identifier: String::new(),
            components: BTreeMap::new(),
            children: Vec::new(),
            relationships: Vec::new(),
            kind: NodeKind::default(),
        }
    }
}

/// The kind of mutation a `BsnPatch` represents on the target node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BsnPatchOp {
    Replace,
    AddChild,
    RemoveChild,
    #[serde(rename = "custom")]
    Custom(String),
}

/// A pending mutation against a BSN IR node. Identified by the target
/// node's identifier; carries the operation and the new value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BsnPatch {
    pub target_identifier: String,
    pub op: BsnPatchOp,
    pub value: serde_json::Value,
}

/// Lossy semantic projection of a SceneAssetDocument (ADR-0005 §7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BsnIr {
    pub scene_root: BsnIrNode,
    pub asset_refs: Vec<String>,
    pub patches: Vec<BsnPatch>,
}

/// Editor source-of-truth → BSN semantic projection (one-way, lossy).
/// Drops: metadata, exposed_properties, logical_path, asset_id, version.
/// Root = first entity; children built from RelationshipKind::Child.
/// asset_refs/patches are empty for a pure document (they come from instances).
pub fn bsn_ir_from_scene_asset(doc: &SceneAssetDocument) -> BsnIr {
    let root_identifier = doc
        .entities
        .first()
        .map(|e| e.local_id.0.clone())
        .unwrap_or_else(|| "empty".to_string());

    let scene_root = if let Some(root_entity) = doc.entities.first() {
        let components: BTreeMap<String, serde_json::Value> = root_entity
            .components
            .iter()
            .map(|c| (c.type_id.clone(), c.values.clone()))
            .collect();

        let relationships: Vec<BsnIrRelationship> = doc
            .relationships
            .iter()
            .filter(|r| {
                r.from_local_id == root_entity.local_id && matches!(r.kind, RelationshipKind::Child)
            })
            .map(|r| BsnIrRelationship {
                kind: "child".to_string(),
                target_identifier: r.to_local_id.0.clone(),
            })
            .collect();

        let child_ids: std::collections::HashSet<&LocalId> = doc
            .relationships
            .iter()
            .filter(|r| {
                r.from_local_id == root_entity.local_id && matches!(r.kind, RelationshipKind::Child)
            })
            .map(|r| &r.to_local_id)
            .collect();

        let children: Vec<BsnIrNode> = doc
            .entities
            .iter()
            .skip(1)
            .filter(|e| child_ids.contains(&e.local_id))
            .map(|e| {
                let comps: BTreeMap<String, serde_json::Value> = e
                    .components
                    .iter()
                    .map(|c| (c.type_id.clone(), c.values.clone()))
                    .collect();
                BsnIrNode {
                    identifier: e.local_id.0.clone(),
                    components: comps,
                    children: Vec::new(),
                    relationships: Vec::new(),
                ..Default::default()
        }
            })
            .collect();

        BsnIrNode {
            identifier: root_identifier,
            components,
            children,
            relationships,
        ..Default::default()
        }
    } else {
        BsnIrNode {
            identifier: "empty".to_string(),
            components: BTreeMap::new(),
            children: Vec::new(),
            relationships: Vec::new(),
        ..Default::default()
        }
    };

    let asset_refs = if doc.logical_path.is_empty() {
        Vec::new()
    } else {
        vec![doc.logical_path.clone()]
    };

    BsnIr {
        scene_root,
        asset_refs,
        patches: Vec::new(),
    }
}
