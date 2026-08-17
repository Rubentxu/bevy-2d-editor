//! External source provenance — sidecar `.meta.json` structure (ADR-0041).
//!
//! All fields carry `#[serde(default)]` so legacy sidecars load gracefully.

use crate::ids::LocalId;
use crate::time::Timestamp;
use serde::{Deserialize, Serialize};

/// The kind of external source that produced an import.
///
/// `#[non_exhaustive]` so new variants (Custom file formats, future editors)
/// can be added without a breaking change.
///
/// Serialization: bare snake_case string (e.g., `"aseprite"`, `"ldtk"`) matching
/// `ValidationCategory` format.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExternalSourceKind {
    /// An Aseprite `.json` + `.png` pair.
    #[default]
    Aseprite,
    /// An LDtk `.ldtk` project file.
    Ldtk,
    /// A Tiled `.json` map file.
    Tiled,
    /// An unknown format — preserved so sidecars remain loadable.
    Custom(String),
}

impl ExternalSourceKind {
    /// Returns `true` if this is `ExternalSourceKind::Aseprite`.
    pub fn is_aseprite(&self) -> bool {
        matches!(self, ExternalSourceKind::Aseprite)
    }

    /// Returns `true` if this is `ExternalSourceKind::Ldtk`.
    pub fn is_ldtk(&self) -> bool {
        matches!(self, ExternalSourceKind::Ldtk)
    }

    /// Returns `true` if this is `ExternalSourceKind::Tiled`.
    pub fn is_tiled(&self) -> bool {
        matches!(self, ExternalSourceKind::Tiled)
    }
}

/// How an imported object is owned and whether it may be overwritten on reimport.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipRule {
    /// The object is fully owned by the external source — reimport overwrites.
    ///
    /// Merge strategy: **overwrite** — source wins.
    #[default]
    SourceOwned,
    /// The object was created or modified inside the editor — reimport preserves.
    ///
    /// Merge strategy: **preserve** — editor wins; source diff is dropped.
    EditorOwned,
    /// Both source and editor may have modified the object — three-way merge.
    ///
    /// Merge strategy: **three-way** — both changes combined; conflicts flagged.
    Mergeable,
    /// The object is derived from another (e.g. auto-layer output) — recompute.
    ///
    /// Merge strategy: **recompute** — discard and regenerate from source rules.
    Derived,
}

/// A single link from an external object to an editor resource.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SourceMapping {
    /// The identifier of the object in the external format (e.g. LDtk entity UID).
    pub source_object_id: String,
    /// The editor resource this maps to.
    pub target_resource_ref: String,
    /// The optional editor-local ID for the specific entity/instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_local_id: Option<LocalId>,
    /// How the mapped object is owned.
    pub ownership: OwnershipRule,
}

impl SourceMapping {
    /// Construct a new source mapping.
    pub fn new(
        source_object_id: impl Into<String>,
        target_resource_ref: impl Into<String>,
        ownership: OwnershipRule,
    ) -> Self {
        Self {
            source_object_id: source_object_id.into(),
            target_resource_ref: target_resource_ref.into(),
            target_local_id: None,
            ownership,
        }
    }
}

/// The result of comparing two provenance sidecars during reimport.
///
/// Carries all four diff buckets so the reimport pipeline can route each
/// through the appropriate `OwnershipRule` merge strategy.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProvenanceDiff {
    /// Objects that exist in the new source but not in the old sidecar.
    pub added: Vec<SourceMapping>,
    /// Objects that exist in the old sidecar but not in the new source.
    pub removed: Vec<SourceMapping>,
    /// Objects present in both but whose source-side data changed.
    pub modified_source: Vec<SourceMapping>,
    /// Objects present in both but whose editor-side data changed.
    pub modified_editor: Vec<SourceMapping>,
    /// Objects where source and editor both changed and the rules conflict.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ownership_conflicts: Vec<SourceMapping>,
}

impl ProvenanceDiff {
    /// Returns `true` if there are no differences.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.modified_source.is_empty()
            && self.modified_editor.is_empty()
            && self.ownership_conflicts.is_empty()
    }
}

/// Full provenance record persisted as `<resource>.meta.json` alongside each
/// imported editor resource.
///
/// Stored by the reimport pipeline and read at reimport time to compute
/// `ProvenanceDiff`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ExternalSource {
    /// Which external tool produced this import.
    pub kind: ExternalSourceKind,
    /// The URI/path of the external source file (e.g. `imports/player.ldtk`).
    pub source_uri: String,
    /// `sha256(source_bytes)` at import time — unchanged value means no-op reimport.
    pub fingerprint: String,
    /// The importer that processed this source (e.g. `builtin.ldtk`).
    pub importer_id: String,
    /// The version of the importer that was used.
    pub importer_version: crate::importer::ImporterVersion,
    /// When this source was last imported.
    pub last_import_time: Timestamp,
    /// All object-level links from the source to editor resources.
    pub mappings: Vec<SourceMapping>,
    /// Per-object ownership rules (mirrors the rules embedded in each mapping).
    pub ownership_rules: Vec<OwnershipRule>,
    /// Schema version of this sidecar format (currently `1`).
    pub schema_version: u32,
}

impl ExternalSource {
    /// Construct a new `ExternalSource`.
    pub fn new(
        kind: ExternalSourceKind,
        source_uri: impl Into<String>,
        fingerprint: impl Into<String>,
        importer_id: impl Into<String>,
        importer_version: crate::importer::ImporterVersion,
        last_import_time: Timestamp,
    ) -> Self {
        Self {
            kind,
            source_uri: source_uri.into(),
            fingerprint: fingerprint.into(),
            importer_id: importer_id.into(),
            importer_version,
            last_import_time,
            mappings: Vec::new(),
            ownership_rules: Vec::new(),
            schema_version: 1,
        }
    }
}
