//! Re-exports from editor_model for types that cross the WASM boundary.
//! These types are defined in editor-model and used by editor-protocol
//! without creating a dependency on editor-bevy.

pub use editor_model::ChangeSet;
pub use editor_model::ExtensionManifest;
pub use editor_model::LocalId;
pub use editor_model::PendingChangeSet;
pub use editor_model::PendingChangeSetSummary;
pub use editor_model::StableId;

pub use editor_model::importer::{
    BuildChangeSetOutput, Importer, ImporterDescriptor, ImporterError, ImporterInput, ParseOutput,
};

pub use editor_model::external_source::{
    ConflictPolicy, ExternalSource, ExternalSourceKind, ProvenanceDiff,
};

pub use editor_model::ports::{
    ExtensionRegistryPort, ImporterRegistryPort, ProjectStore, StoreEntry, StoreError,
};

pub use editor_model::session_port::EditorSessionPort;

// World Workspace types (ADR-0037)
pub use editor_model::world::{
    EntranceRef, LayoutPolicy, LinkDirection, StreamingPolicy, WorldCatalogEntry, WorldDocument,
    WorldId, WorldLevelRef, WorldLink, WorldLinkKind,
};
