//! `editor_model` — pure data model crate for the Bevy 2D Editor.
//!
//! Zero Bevy / zero WASM dependencies. Contains only serde-serializable types
//! that represent editor-owned domain state.

#![deny(missing_docs)]

pub mod auto_layer;
pub mod component;
pub mod document;
pub mod ids;
pub mod logic_graph;
pub mod pending_change_set;
pub mod ports;
pub mod scene_asset;
pub mod scene_instance;
pub mod schema;
pub mod session;
pub mod tile_layer;
pub mod tileset;
pub mod time;
pub mod transaction;

// Re-export all public types at the root for ergonomic use.
pub use auto_layer::{
    AutoLayer, AutoLayerId, Pattern3x3, PatternCell, is_auto_layer_stale, regenerate,
};
pub use component::ComponentInstance;
pub use document::{Anchor, Color, Entity, SceneDocument, Vec2};
pub use ids::{AssetId, DocumentId, EntityId, LayerId, LocalId, SceneAssetLocalId, StableId};
pub use logic_graph::{
    LogicEdge, LogicGraphAsset, LogicInstance, LogicNode, LogicNodeRole, NodeId, NodeTypeId,
    PortId, count_logic_bindings, editor_logic_binding_component, find_dangling_edge_nodes,
    find_duplicate_node_id,
};
pub use pending_change_set::{PendingChangeSet, PendingChangeSetSummary};
pub use ports::{ProjectStore, StoreEntry, StoreError};
pub use scene_asset::{
    AssetReference, ExposedProperty, LevelLayer, RelationshipKind, RoleWarning, SceneAssetDocument,
    SceneAssetEntity, SceneAssetRelationship, SceneAssetRole, SceneInstanceLayer,
    SceneInstanceLayerKind,
};
pub use scene_instance::{
    ComponentOverride, ComponentOverrideStatus, SceneInstance,
    component_override_status_after_field_rename,
};
pub use schema::{
    ComponentSchema, ComponentTypeId, Constraint, FieldDef, FieldType, SchemaKind, SourceLocation,
};
pub use session::{AppliedChangeMeta, HistoryScope};
pub use tile_layer::TileLayer;
pub use tileset::{
    AsepriteFrame, AsepriteMetadata, AsepriteSlice, AsepriteTag, TileCoord, TileGrid, TileRef,
    TilesetAsset, TilesetId, TilesetManager, TilesetMetadata,
};
pub use time::{Clock, Timestamp};
pub use transaction::{
    Applier, ApprovalPolicy, ChangeOrigin, ChangeSet, DiffSummary, EffectsSummary, ResourceRef,
    TransactionKernel, ValidationReport,
};
pub use transaction::{ApplyReceipt, KernelError};
