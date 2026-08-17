//! Re-exports from editor_model for types that cross the WASM boundary.
//! These types are defined in editor-model and used by editor-protocol
//! without creating a dependency on editor-bevy.

pub use editor_model::ChangeSet;
pub use editor_model::PendingChangeSet;
pub use editor_model::PendingChangeSetSummary;
pub use editor_model::ExtensionManifest;
pub use editor_model::StableId;
pub use editor_model::LocalId;

pub use editor_model::importer::{
    Importer, ImporterDescriptor, ImporterError, ImporterInput, ParseOutput, BuildChangeSetOutput,
};

pub use editor_model::external_source::{
    ExternalSource, ExternalSourceKind, ConflictPolicy, ProvenanceDiff,
};

pub use editor_model::ports::{ProjectStore, StoreEntry, StoreError, ExtensionRegistryPort, ImporterRegistryPort};

pub use editor_model::session_port::EditorSessionPort;
