//! Importer protocol — trait, descriptors, errors, and versioning (ADR-0040 step 3).
//!
//! Lives in `editor-model` so both `editor-core` (Bevy systems doing import) and
//! `editor-application` (WASM boundary, session state) can use the types without
//! a circular dependency.

use serde::{Deserialize, Serialize};

use crate::external_source::{ExternalSourceKind, ProvenanceDiff, SourceMapping};

/// An importer semantic version.
///
/// Used both in `ImporterDescriptor` (supported range) and in `ExternalSource`
/// (the version that was used to produce a given sidecar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct ImporterVersion {
    /// Major version — incremented on breaking changes.
    pub major: u32,
    /// Minor version — incremented on new features (backwards compatible).
    pub minor: u32,
    /// Patch version — incremented on bug fixes.
    pub patch: u32,
}

impl ImporterVersion {
    /// Construct a version from major.minor.patch.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Parse from a string like `"1.2.3"`.
    pub fn parse(s: &str) -> Option<Self> {
        let mut parts = s.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        Some(Self { major, minor, patch })
    }
}

impl std::fmt::Display for ImporterVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// An inclusive version range for importer compatibility negotiation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImporterVersionRange {
    /// Inclusive lower bound.
    pub min: ImporterVersion,
    /// Inclusive upper bound.
    pub max: ImporterVersion,
}

impl ImporterVersionRange {
    /// Construct an inclusive range `[min, max]`.
    pub fn new(min: ImporterVersion, max: ImporterVersion) -> Self {
        Self { min, max }
    }

    /// Returns `true` if `v` falls within `[min, max]`.
    pub fn contains(&self, v: ImporterVersion) -> bool {
        self.min <= v && v <= self.max
    }

    /// Returns `true` if the range is empty (min > max).
    pub fn is_empty(&self) -> bool {
        self.min > self.max
    }

    /// Construct an empty range — useful for sentinel values.
    pub fn empty() -> Self {
        Self {
            min: ImporterVersion::new(u32::MAX, 0, 0),
            max: ImporterVersion::new(0, 0, 0),
        }
    }
}

/// Errors that can occur during import operations.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ImporterError {
    /// The source bytes could not be parsed as the expected format.
    #[error("parse error: {0}")]
    ParseError(String),
    /// The detected schema version is outside the importer's supported range.
    #[error("unsupported version {detected} (supported: {supported_min}–{supported_max})")]
    UnsupportedVersion {
        /// The version detected in the source file.
        detected: ImporterVersion,
        /// The minimum supported version.
        supported_min: ImporterVersion,
        /// The maximum supported version.
        supported_max: ImporterVersion,
    },
    /// The source kind (e.g. XML TMX) is not supported by this importer.
    #[error("unsupported kind: {0}")]
    UnsupportedKind(String),
    /// A source object maps to an editor resource that already has a different mapping.
    #[error("mapping conflict for source object {source_id}: {message}")]
    MappingConflict {
        /// The source object ID that conflicted.
        source_id: String,
        /// Human-readable message describing the conflict.
        message: String,
    },
    /// An IO error occurred while reading the source bytes.
    #[error("io error: {0}")]
    IoError(String),
    /// An importer with the same ID is already registered.
    #[error("duplicate importer id: {0}")]
    DuplicateId(String),
    /// The importer was not found.
    #[error("importer not found: {0}")]
    NotFound(String),
    /// The version range is empty (min > max).
    #[error("invalid version range: min > max")]
    InvalidVersionRange,
    /// No importer is registered for the requested kind.
    #[error("no importer for kind: {0}")]
    NoImporterForKind(String),
}

impl ImporterError {
    /// Returns `true` if this is `UnsupportedKind`.
    pub fn is_unsupported_kind(&self) -> bool {
        matches!(self, ImporterError::UnsupportedKind(_))
    }

    /// Returns `true` if this is `UnsupportedVersion`.
    pub fn is_unsupported_version(&self) -> bool {
        matches!(self, ImporterError::UnsupportedVersion { .. })
    }
}

/// Input to an importer's `parse` method.
#[derive(Debug, Clone)]
pub struct ImporterInput<'a> {
    /// Raw source bytes.
    pub bytes: &'a [u8],
    /// The source URI (for error messages and provenance).
    pub source_uri: &'a str,
    /// Optional fingerprint of the source bytes (if already computed externally).
    pub fingerprint_hint: Option<String>,
}

/// A resource draft produced by an importer during `parse`.
///
/// Carries enough information for `build_change_set` to emit `AssetCommand`s.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResourceDraft {
    /// A new `SceneAssetDocument`.
    SceneAsset {
        /// Logical path in the project (e.g. `actors/player.json`).
        logical_path: String,
        /// Optional human-facing display name.
        display_name: Option<String>,
    },
    /// A new `AssetFile` (e.g. PNG texture).
    AssetFile {
        /// Logical path in the project (e.g. `resources/player.png`).
        logical_path: String,
        /// Base64-encoded bytes if embedded; `None` means the file is referenced externally.
        bytes_b64: Option<String>,
    },
    /// A new `SceneAssetDocument` with role `Level`.
    Level {
        /// Logical path in the project (e.g. `levels/world_1/level_1.json`).
        logical_path: String,
        /// Optional human-facing display name.
        display_name: Option<String>,
    },
    /// A new `SceneAssetDocument` with role `Fragment`.
    Fragment {
        /// Logical path in the project (e.g. `fragments/teleport_pad.json`).
        logical_path: String,
        /// Optional human-facing display name.
        display_name: Option<String>,
    },
}

/// The output of `Importer::parse` — resource drafts + provenance metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ParseOutput {
    /// All resource drafts produced from the source.
    pub resource_drafts: Vec<ResourceDraft>,
    /// Object-level source→editor mappings.
    pub mappings: Vec<SourceMapping>,
    /// Ownership rules for the imported objects.
    pub ownership_rules: Vec<crate::external_source::OwnershipRule>,
    /// Detected schema version string (for version negotiation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_version: Option<String>,
    /// Detected schema version as structured `ImporterVersion` (for range check).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_version_parsed: Option<ImporterVersion>,
}

impl ParseOutput {
    /// Returns `true` if no resource drafts were produced.
    pub fn is_empty(&self) -> bool {
        self.resource_drafts.is_empty()
    }
}

/// Output of `Importer::build_change_set`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildChangeSetOutput {
    /// The provenance diff between the previous import (if any) and this one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_diff: Option<ProvenanceDiff>,
    /// Serialized `ChangeSet<AssetCommand>` JSON ready for the transaction kernel.
    pub change_set_json: String,
}

/// A lightweight descriptor registered for each importer.
///
/// Used for `list_by_kind` queries and WASM UI surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImporterDescriptor {
    /// Unique identifier (e.g. `builtin.aseprite`, `builtin.ldtk`).
    pub id: String,
    /// The external source kind this importer handles.
    pub kind: ExternalSourceKind,
    /// The range of external schema versions this importer supports.
    pub supported_versions: ImporterVersionRange,
    /// Human-readable name shown in the UI.
    pub display_name: String,
}

impl ImporterDescriptor {
    /// Construct a descriptor.
    pub fn new(
        id: impl Into<String>,
        kind: ExternalSourceKind,
        supported_versions: ImporterVersionRange,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            supported_versions,
            display_name: display_name.into(),
        }
    }
}

/// Handle returned on successful importer registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImporterHandle(pub u64);

impl ImporterHandle {
    /// Construct a new handle.
    pub fn new(n: u64) -> Self {
        Self(n)
    }

    /// Unwrap to raw u64.
    pub fn to_u64(self) -> u64 {
        self.0
    }
}

/// The importer trait — implemented by built-in and extension-provided importers.
///
/// Object-safe (`dyn Importer`) so the registry can hold `Arc<dyn Importer>`.
pub trait Importer: Send + Sync {
    /// Return this importer's descriptor.
    fn descriptor(&self) -> ImporterDescriptor;

    /// Parse the input bytes into resource drafts + provenance metadata.
    ///
    /// Returns `Err(ImporterError::UnsupportedVersion)` if the detected schema
    /// version falls outside `supported_versions`.
    fn parse(&self, source: ImporterInput<'_>) -> Result<ParseOutput, ImporterError>;

    /// Build a `ChangeSet<AssetCommand>` from the parse output against the
    /// current editor snapshot.
    ///
    /// The returned `BuildChangeSetOutput.change_set_json` is a serialized
    /// `PendingChangeSet` ready for the transaction kernel.
    fn build_change_set(
        &self,
        draft: ParseOutput,
        snapshot: crate::session::EditorSnapshot,
    ) -> Result<BuildChangeSetOutput, ImporterError>;
}
