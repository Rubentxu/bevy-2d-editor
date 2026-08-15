//! `editor_model` — pure data model crate for the Bevy 2D Editor.
//!
//! Zero Bevy / zero WASM dependencies. Contains only serde-serializable types
//! that represent editor-owned domain state.

pub mod ids;
pub mod component;
pub mod document;
pub mod scene_instance;
pub mod scene_asset;
pub mod logic_graph;
pub mod schema;
pub mod tile_layer;
pub mod tileset;
pub mod auto_layer;

// Re-export all public types at the root for ergonomic use.
pub use ids::{
    AssetId, DocumentId, EntityId, LayerId, LocalId, SceneAssetLocalId, StableId,
};
pub use component::ComponentInstance;
pub use document::{Anchor, Color, Entity, SceneDocument, Vec2};
pub use scene_instance::{
    component_override_status_after_field_rename, ComponentOverride, ComponentOverrideStatus,
    SceneInstance,
};
pub use scene_asset::{
    AssetReference, ExposedProperty, LevelLayer,
    RelationshipKind, RoleWarning, SceneAssetDocument, SceneAssetEntity,
    SceneAssetRelationship, SceneAssetRole, SceneInstanceLayer, SceneInstanceLayerKind,
};
pub use logic_graph::{
    count_logic_bindings, editor_logic_binding_component, find_dangling_edge_nodes,
    find_duplicate_node_id, LogicEdge, LogicGraphAsset, LogicInstance, LogicNode,
    LogicNodeRole, NodeId, NodeTypeId, PortId,
};
pub use schema::{
    ComponentSchema, ComponentTypeId, Constraint, FieldDef, FieldType, SchemaKind,
    SourceLocation,
};
pub use tile_layer::TileLayer;
pub use tileset::{
    AsepriteFrame, AsepriteMetadata, AsepriteSlice, AsepriteTag, TileCoord,
    TileGrid, TileRef, TilesetAsset, TilesetId, TilesetManager, TilesetMetadata,
};
pub use auto_layer::{
    is_auto_layer_stale, regenerate, AutoLayer, AutoLayerId, Pattern3x3,
    PatternCell,
};
