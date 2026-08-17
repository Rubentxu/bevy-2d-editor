//! Editor Extension SDK — pure data types for the extension registry and capability permissions.
//!
//! All types here are `serde`-derivable and JSON-stable. Zero Bevy/WASM dependencies.
//!
//! ## Architecture
//!
//! - `ExtensionManifest` — the declaration signed by an extension author
//! - `Capability` — what the extension is allowed to do (non-exhaustive)
//! - `Permission` / `PermissionArea` / `PermissionScope` — the security model
//! - `ExtensionId` / `SemVer` — opaque identifiers
//! - `ExtensionHandle` — opaque u64 returned by the registry on registration
//! - `ExtensionSummary` — lightweight info returned by `list_extensions`

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::transaction::KernelError;

// ─────────────────────────────────────────────────────────────────────────────
// ExtensionId / SemVer — opaque identifiers
// ─────────────────────────────────────────────────────────────────────────────

/// Opaque extension identifier.
///
/// Must be unique within a session. Convention: reverse-domain notation for
/// third-party (`com.example.my-extension`) and `builtin.` prefix for
/// built-in extensions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExtensionId(pub String);

impl ExtensionId {
    /// Construct from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl AsRef<str> for ExtensionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ExtensionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Semantic version for an extension.
///
/// Stored as three components; the fourth pre-release / build metadata components
/// are not used in v0.92.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    /// Major version — breaking changes.
    pub major: u32,
    /// Minor version — new capabilities, backward-compatible.
    pub minor: u32,
    /// Patch version — bug fixes, backward-compatible.
    pub patch: u32,
}

impl SemVer {
    /// Construct a version from major.minor.patch.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Construct from a string like "0.92.0".
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Capability — what an extension declares it will do (non-exhaustive)
// ─────────────────────────────────────────────────────────────────────────────

/// What an extension is capable of doing (v0.92-P1 set).
///
/// Marked `#[non_exhaustive]` so future SDK versions can add variants without
/// a breaking change. Unknown variants deserialize without error (they are
/// skipped by `list_extensions`).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    /// Emits commands into the editor command stream.
    Commands,
    /// Validates documents and emits `ValidationIssue` reports.
    Validators,
    /// Provides reusable `LogicGraphAsset` recipes.
    Recipes,
    /// Imports external data sources (Aseprite, LDtk, Tiled, etc.).
    Importers,
    /// Contributes custom inspector panels or property editors.
    Inspectors,
    /// Processes assets (texture compression, atlas generation, etc.).
    AssetProcessors,
    /// Contributes new dock panels or UI surfaces.
    Panels,
    /// Provides diagnostic data to the Validation Center.
    DiagnosticProviders,
}

impl Capability {
    /// Returns the number of built-in capability categories.
    pub const fn builtin_count() -> usize {
        8
    }
}

/// Descriptor for a single declared capability — pairs a `Capability` with
/// optional metadata about how it is used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    /// The capability kind.
    pub kind: Capability,
    /// Human-readable description of how this capability is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Permission model
// ─────────────────────────────────────────────────────────────────────────────

/// The functional area a permission grants access to.
///
/// Marked `#[non_exhaustive]` to allow future permission areas without a
/// breaking change to the SDK.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionArea {
    /// Commands — dispatching editor commands.
    Commands,
    /// Validators — reading/scanning documents.
    Validators,
    /// Recipes — reading/writing `LogicGraphAsset` documents.
    Recipes,
    /// Importers — reading external files.
    Importers,
    /// Inspectors — reading document structure.
    Inspectors,
    /// AssetProcessors — reading/writing assets.
    AssetProcessors,
    /// Panels — UI contributions (read-only).
    Panels,
    /// DiagnosticProviders — diagnostic data (read-only).
    DiagnosticProviders,
    /// Project — project-level resources (settings, metadata).
    Project,
}

/// The scope of access within a permission area.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionScope {
    /// Read-only access to the area's resources.
    Read,
    /// Read and write access.
    Write,
    /// Propose changes (for approval workflows — ChangeWorkbench).
    Propose,
    /// Subscribe to events / live updates.
    Subscribe,
}

/// A single declared permission — pairs an area with a scope and optional
/// resource filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    /// The functional area this permission covers.
    pub area: PermissionArea,
    /// The access scope within that area.
    pub scope: PermissionScope,
    /// Optional resource filter (e.g. `"scenes/players/**"`).
    /// `None` means "all resources in this area".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
}

impl Permission {
    /// Construct a permission for all resources in an area.
    pub fn new(area: PermissionArea, scope: PermissionScope) -> Self {
        Self {
            area,
            scope,
            resource: None,
        }
    }

    /// Construct a permission for a specific resource glob.
    pub fn for_resource(
        area: PermissionArea,
        scope: PermissionScope,
        resource: impl Into<String>,
    ) -> Self {
        Self {
            area,
            scope,
            resource: Some(resource.into()),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ExtensionManifest / ExtensionError / ExtensionHandle / ExtensionSummary
// ─────────────────────────────────────────────────────────────────────────────

/// The manifest signed by an extension author at registration time.
///
/// This is the canonical registration document for an extension. It declares
/// the extension's identity, its capabilities, and the permissions it requires.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionManifest {
    /// Unique identifier (reverse-domain notation for third-party,
    /// `builtin.` prefix for built-ins).
    pub id: ExtensionId,
    /// Semantic version.
    pub version: SemVer,
    /// Declared capabilities (at least one required).
    pub capabilities: Vec<CapabilityDescriptor>,
    /// Declared permissions (may be empty for read-only extensions).
    pub permissions: Vec<Permission>,
}

impl ExtensionManifest {
    /// Construct a new manifest.
    pub fn new(
        id: ExtensionId,
        version: SemVer,
        capabilities: Vec<CapabilityDescriptor>,
        permissions: Vec<Permission>,
    ) -> Self {
        Self {
            id,
            version,
            capabilities,
            permissions,
        }
    }

    /// Returns true if this manifest declares the given capability.
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.iter().any(|c| &c.kind == capability)
    }

    /// Returns true if this manifest grants the given permission.
    pub fn has_permission(&self, area: &PermissionArea, scope: &PermissionScope) -> bool {
        self.permissions
            .iter()
            .any(|p| &p.area == area && &p.scope == scope)
    }
}

/// Errors that can occur when registering or unregistering an extension.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ExtensionError {
    /// An extension with this ID is already registered.
    #[error("extension already registered: {0}")]
    DuplicateId(ExtensionId),
    /// No extension with this ID is registered.
    #[error("extension not found: {0}")]
    NotFound(ExtensionId),
    /// The manifest is malformed or fails validation.
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
    /// Permission denied — extension lacks required permission for an operation.
    #[error(
        "permission denied for extension '{extension}': {area} requires {scope_needed} but manifest grants {scope_granted}"
    )]
    PermissionDenied {
        /// The extension ID.
        extension: String,
        /// The permission area that was denied.
        area: String,
        /// The scope that was required.
        scope_needed: String,
        /// The scope that was actually granted.
        scope_granted: String,
    },
    /// Permission denied — extension not registered.
    #[error("extension not registered: {0}")]
    ExtensionNotRegistered(String),
}

impl From<ExtensionError> for KernelError<std::convert::Infallible> {
    fn from(err: ExtensionError) -> Self {
        match err {
            ExtensionError::PermissionDenied {
                extension,
                area,
                scope_needed,
                scope_granted,
            } => KernelError::PermissionDenied {
                extension,
                area,
                scope_needed,
                scope_granted,
            },
            other => KernelError::Preflight(format!("extension error: {}", other)),
        }
    }
}

/// Opaque handle returned by `ExtensionRegistryPort::register`.
///
/// In v0.92 this is a simple `u64` counter. Future SDK versions may add
/// re-binding, versioning, or capability queries to the handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtensionHandle(u64);

impl ExtensionHandle {
    /// Construct from a u64 value.
    pub fn new(n: u64) -> Self {
        Self(n)
    }

    /// Returns the underlying u64 value.
    pub fn to_u64(self) -> u64 {
        self.0
    }
}

/// Lightweight summary returned by `list_extensions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionSummary {
    /// Extension identifier.
    pub id: String,
    /// Version string (e.g. "0.92.0").
    pub version: String,
    /// Number of declared capabilities.
    pub capability_count: usize,
}

impl From<&ExtensionManifest> for ExtensionSummary {
    fn from(manifest: &ExtensionManifest) -> Self {
        Self {
            id: manifest.id.0.clone(),
            version: manifest.version.to_string(),
            capability_count: manifest.capabilities.len(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_manifest_round_trip() {
        let manifest = ExtensionManifest::new(
            ExtensionId::new("builtin.logic-bricks.controllers"),
            SemVer::new(0, 92, 0),
            vec![CapabilityDescriptor {
                kind: Capability::Commands,
                description: Some("Rust controller evaluators".to_string()),
            }],
            vec![Permission::new(
                PermissionArea::Commands,
                PermissionScope::Propose,
            )],
        );

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: ExtensionManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id.0, "builtin.logic-bricks.controllers");
        assert_eq!(parsed.capabilities.len(), 1);
        assert!(parsed.has_capability(&Capability::Commands));
    }

    #[test]
    fn semver_parse_and_display() {
        let v = SemVer::parse("0.92.0").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.to_string(), "0.92.0");
    }

    #[test]
    fn permission_new_and_for_resource() {
        let p = Permission::new(PermissionArea::Commands, PermissionScope::Propose);
        assert!(p.resource.is_none());

        let p2 = Permission::for_resource(
            PermissionArea::AssetProcessors,
            PermissionScope::Write,
            "assets/players/**",
        );
        assert_eq!(p2.resource.as_ref().unwrap(), "assets/players/**");
    }

    #[test]
    fn extension_summary_from_manifest() {
        let manifest = ExtensionManifest::new(
            ExtensionId::new("com.example.my-ext"),
            SemVer::new(1, 0, 0),
            vec![
                CapabilityDescriptor {
                    kind: Capability::Validators,
                    description: None,
                },
                CapabilityDescriptor {
                    kind: Capability::Inspectors,
                    description: None,
                },
            ],
            vec![Permission::new(
                PermissionArea::Validators,
                PermissionScope::Read,
            )],
        );

        let summary = ExtensionSummary::from(&manifest);
        assert_eq!(summary.id, "com.example.my-ext");
        assert_eq!(summary.version, "1.0.0");
        assert_eq!(summary.capability_count, 2);
    }

    #[test]
    fn permission_denied_error_display() {
        let err = ExtensionError::PermissionDenied {
            extension: "my.ext".to_string(),
            area: "AssetProcessors".to_string(),
            scope_needed: "Write".to_string(),
            scope_granted: "Read".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("my.ext"));
        assert!(msg.contains("AssetProcessors"));
    }
}
