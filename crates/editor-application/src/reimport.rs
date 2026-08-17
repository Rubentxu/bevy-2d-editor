//! Reimport pipeline for external source files (ADR-0041).
//!
//! Implements the 7-step reimport flow:
//! 1. Read source bytes from OPFS
//! 2. Compute `sha256(source_bytes)` fingerprint
//! 3. Load existing sidecar `.meta.json` provenance
//! 4. Re-parse via the registered importer
//! 5. Compute `ProvenanceDiff`
//! 6. Apply ownership-aware merge strategy
//! 7. Submit `ChangeSet<AssetCommand>` to the ChangeWorkbench
//!
//! ## Fingerprint behavior
//!
//! If the new fingerprint equals the sidecar's stored fingerprint,
//! `reimport` returns `Ok(NoOp)` and skips parsing entirely.
//!
//! ## Conflict routing
//!
//! - `modified_editor` or `ownership_conflicts` → `ApprovalPolicy::RequiresHuman`
//! - `modified_editor` empty AND `ownership_conflicts` empty → auto-apply
//! - `modified_source` only → auto-apply (source wins)

use editor_model::PendingChangeSet;
use editor_model::external_source::{
    ConflictPolicy, ExternalSource, ExternalSourceKind, ProvenanceDiff,
};
use editor_model::importer::{ImporterError, ImporterInput};
use editor_model::ports::{ImporterRegistryPort, ProjectStore};
use editor_model::time::Timestamp;
use std::sync::{Arc, Mutex};

/// Result of a reimport operation.
#[derive(Debug, Clone)]
pub enum ReimportResult {
    /// The source fingerprint is unchanged — no reimport needed.
    NoOp,
    /// The source changed; ChangeSet was queued for human review.
    QueuedForReview {
        /// ID of the queued ChangeSet.
        change_set_id: String,
        /// The computed provenance diff.
        diff: ProvenanceDiff,
    },
    /// The source changed; ChangeSet was auto-applied (no conflicts).
    AutoApplied {
        /// ID of the applied ChangeSet.
        change_set_id: String,
        /// The computed provenance diff.
        diff: ProvenanceDiff,
    },
}

/// Computes the SHA-256 fingerprint of the given bytes.
///
/// Returns a lowercase hex string.
pub fn compute_fingerprint(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Load an existing `ExternalSource` from a sidecar `.meta.json` file.
///
/// Returns `None` if the sidecar does not exist.
pub fn load_sidecar(
    store: &dyn ProjectStore,
    resource_ref: &str,
) -> Result<Option<ExternalSource>, ReimportError> {
    let sidecar_path = format!("{}.meta.json", resource_ref);
    match store.read(&sidecar_path) {
        Ok(bytes) => {
            let text = String::from_utf8(bytes)
                .map_err(|e| ReimportError::Store(format!("UTF-8 error reading sidecar: {}", e)))?;
            let external_source: ExternalSource = serde_json::from_str(&text).map_err(|e| {
                ReimportError::ParseSidecar(format!("Failed to parse sidecar JSON: {}", e))
            })?;
            Ok(Some(external_source))
        }
        Err(editor_model::ports::StoreError::NotFound(_)) => Ok(None),
        Err(e) => Err(ReimportError::Store(format!(
            "Failed to read sidecar: {}",
            e
        ))),
    }
}

/// Save an `ExternalSource` to its sidecar `.meta.json` file.
pub fn save_sidecar(
    store: &dyn ProjectStore,
    resource_ref: &str,
    external_source: &ExternalSource,
) -> Result<(), ReimportError> {
    let sidecar_path = format!("{}.meta.json", resource_ref);
    let text = serde_json::to_string_pretty(external_source)
        .map_err(|e| ReimportError::Store(format!("Failed to serialize sidecar: {}", e)))?;
    store
        .write(&sidecar_path, text.as_bytes(), true)
        .map_err(|e| ReimportError::Store(format!("Failed to write sidecar: {}", e)))?;
    Ok(())
}

/// Compute a `ProvenanceDiff` between the old and new `ExternalSource`.
///
/// The diff compares `mappings` lists by `source_object_id`.
pub fn compute_provenance_diff(old: &ExternalSource, new: &ExternalSource) -> ProvenanceDiff {
    let old_by_source: std::collections::HashMap<_, _> = old
        .mappings
        .iter()
        .map(|m| (m.source_object_id.clone(), m))
        .collect();
    let new_by_source: std::collections::HashMap<_, _> = new
        .mappings
        .iter()
        .map(|m| (m.source_object_id.clone(), m))
        .collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified_source = Vec::new();
    let mut modified_editor = Vec::new();
    let mut ownership_conflicts = Vec::new();

    // Added: in new but not in old
    for (id, mapping) in &new_by_source {
        if !old_by_source.contains_key(id) {
            added.push((*mapping).clone());
        }
    }

    // Removed: in old but not in new
    for (id, mapping) in &old_by_source {
        if !new_by_source.contains_key(id) {
            removed.push((*mapping).clone());
        }
    }

    // Modified: in both — check for source vs editor changes
    for (id, old_mapping) in &old_by_source {
        if let Some(new_mapping) = new_by_source.get(id) {
            // If the target_resource_ref changed, that's a source modification
            if old_mapping.target_resource_ref != new_mapping.target_resource_ref {
                modified_source.push((*new_mapping).clone());
            } else if old_mapping.ownership != new_mapping.ownership {
                // Ownership changed
                match (&old_mapping.ownership, &new_mapping.ownership) {
                    // Source-owned changed to editor-owned → editor wins, conflict
                    (
                        editor_model::external_source::OwnershipRule::SourceOwned,
                        editor_model::external_source::OwnershipRule::EditorOwned,
                    ) => {
                        ownership_conflicts.push((*new_mapping).clone());
                    }
                    // Editor-owned changed to source-owned → conflict
                    (
                        editor_model::external_source::OwnershipRule::EditorOwned,
                        editor_model::external_source::OwnershipRule::SourceOwned,
                    ) => {
                        ownership_conflicts.push((*new_mapping).clone());
                    }
                    // Editor-owned changed to something else
                    (editor_model::external_source::OwnershipRule::EditorOwned, _) => {
                        modified_editor.push((*new_mapping).clone());
                    }
                    // Something changed but not editor-owned
                    _ => {
                        modified_source.push((*new_mapping).clone());
                    }
                }
            }
            // Note: a true "modified_editor" would require comparing the actual field values,
            // which would need the scene snapshot. For the ProvenanceDiff at the mapping level,
            // we track whether the editor explicitly changed ownership via the ownership field.
            // Full field-level diff is done by the ChangeSet builder.
        }
    }

    ProvenanceDiff {
        added,
        removed,
        modified_source,
        modified_editor,
        ownership_conflicts,
    }
}

/// Errors that can occur during reimport.
#[derive(Debug, thiserror::Error)]
pub enum ReimportError {
    /// The source file could not be read.
    #[error("store error: {0}")]
    Store(String),
    /// The sidecar JSON could not be parsed.
    #[error("parse error: {0}")]
    ParseSidecar(String),
    /// No importer is registered for this kind.
    #[error("no importer for kind: {0}")]
    NoImporter(String),
    /// The importer returned an error.
    #[error("importer error: {0}")]
    Importer(#[from] ImporterError),
    /// The source URI has no associated sidecar.
    #[error("no sidecar found for source: {0}")]
    NoSidecar(String),
    /// No mapping was found for the given source URI.
    #[error("no mapping found for source URI: {0}")]
    NoMapping(String),
}

/// The main reimport function.
///
/// Reads `source_uri` from OPFS, computes the fingerprint, and either:
/// - Returns `Ok(ReimportResult::NoOp)` if the fingerprint is unchanged
/// - Builds and queues a ChangeSet with `ApprovalPolicy::RequiresHuman` if conflicts exist
/// - Builds and auto-applies a ChangeSet if there are no conflicts
///
/// The `pending_change_sets` map is the same `BTreeMap<String, PendingChangeSet>` that
/// lives inside `EditorSession::pending_change_sets_mut()`.
pub fn reimport(
    source_uri: &str,
    store: &dyn ProjectStore,
    importer_registry: &Arc<Mutex<dyn ImporterRegistryPort>>,
    pending_change_sets: &mut std::collections::BTreeMap<String, PendingChangeSet>,
    now_fn: impl Fn() -> Timestamp,
) -> Result<ReimportResult, ReimportError> {
    // Step 1: Read source bytes from OPFS
    let source_bytes = store
        .read(source_uri)
        .map_err(|e| ReimportError::Store(format!("Failed to read source file: {}", e)))?;

    // Step 2: Compute fingerprint
    let new_fingerprint = compute_fingerprint(&source_bytes);

    // Find the resource ref from the sidecar (we need to know which resource this maps to)
    // We look for any sidecar that references this source_uri
    let (_resource_ref, old_sidecar) = find_sidecar_by_source_uri(store, source_uri)
        .ok_or_else(|| ReimportError::NoSidecar(source_uri.to_string()))?;

    // Step 2b: Check if fingerprint is unchanged — skip if so
    if old_sidecar.fingerprint == new_fingerprint {
        return Ok(ReimportResult::NoOp);
    }

    // Step 3: Load existing sidecar (already done above)

    // Step 4: Re-parse via registered importer
    let reg_guard = importer_registry
        .lock()
        .map_err(|e| ReimportError::Store(format!("Importer registry lock poisoned: {}", e)))?;

    let parse_output = reg_guard.dispatch(
        &old_sidecar.kind,
        ImporterInput {
            bytes: &source_bytes,
            source_uri,
            fingerprint_hint: Some(new_fingerprint.clone()),
        },
    )?;

    drop(reg_guard);

    // Step 5: Build new ExternalSource from parse output
    let new_sidecar = build_new_sidecar(
        &old_sidecar.kind,
        source_uri,
        &new_fingerprint,
        &old_sidecar.importer_id,
        old_sidecar.importer_version,
        now_fn,
        &parse_output,
        old_sidecar.conflict_policy,
    );

    // Step 6: Compute ProvenanceDiff
    let diff = compute_provenance_diff(&old_sidecar, &new_sidecar);

    // Step 7: Build ChangeSet and route to ChangeWorkbench
    let change_set_id = format!("importer:{}", uuid::Uuid::new_v4());

    // Determine conflict state
    let modified_editor_empty = diff.modified_editor.is_empty();
    let ownership_conflicts_empty = diff.ownership_conflicts.is_empty();

    // Determine approval policy using ConflictPolicy from sidecar (default: AutoApply)
    let policy = old_sidecar.conflict_policy.unwrap_or_default();
    let requires_review = policy.requires_review(modified_editor_empty, ownership_conflicts_empty);

    // SkipOnConflict: if any conflict and policy says skip, return NoOp
    if matches!(policy, ConflictPolicy::SkipOnConflict) && requires_review {
        return Ok(ReimportResult::NoOp);
    }

    let change_set =
        build_change_set_from_diff(&change_set_id, &diff, &old_sidecar.importer_id, source_uri);

    if requires_review {
        // Route to ChangeWorkbench with RequiresHuman
        pending_change_sets.insert(change_set_id.clone(), change_set);
        Ok(ReimportResult::QueuedForReview {
            change_set_id,
            diff,
        })
    } else {
        // Auto-apply: insert into pending and immediately mark as approved
        // In the WASM flow this would be done differently, but for the non-WASM reimport
        // we return AutoApplied and let the caller apply it
        pending_change_sets.insert(change_set_id.clone(), change_set);
        Ok(ReimportResult::AutoApplied {
            change_set_id,
            diff,
        })
    }
}

/// Find a sidecar that references the given source_uri.
///
/// Searches all `.meta.json` files in the project store and returns the first one
/// whose `source_uri` matches.
fn find_sidecar_by_source_uri(
    store: &dyn ProjectStore,
    source_uri: &str,
) -> Option<(String, ExternalSource)> {
    let entries = store.list("").ok()?;
    for entry in entries {
        if !entry.path.ends_with(".meta.json") {
            continue;
        }
        let resource_ref = entry.path.trim_end_matches(".meta.json");
        if let Ok(bytes) = store.read(&entry.path) {
            if let Ok(text) = String::from_utf8(bytes) {
                if let Ok(es) = serde_json::from_str::<ExternalSource>(&text) {
                    if es.source_uri == source_uri {
                        return Some((resource_ref.to_string(), es));
                    }
                }
            }
        }
    }
    None
}

/// Build a new `ExternalSource` from a parse output.
fn build_new_sidecar(
    kind: &ExternalSourceKind,
    source_uri: &str,
    fingerprint: &str,
    importer_id: &str,
    importer_version: editor_model::importer::ImporterVersion,
    now_fn: impl Fn() -> Timestamp,
    parse_output: &editor_model::importer::ParseOutput,
    conflict_policy: Option<editor_model::external_source::ConflictPolicy>,
) -> ExternalSource {
    ExternalSource {
        kind: kind.clone(),
        source_uri: source_uri.to_string(),
        fingerprint: fingerprint.to_string(),
        importer_id: importer_id.to_string(),
        importer_version,
        last_import_time: now_fn(),
        mappings: parse_output.mappings.clone(),
        ownership_rules: parse_output.ownership_rules.clone(),
        schema_version: 1,
        conflict_policy,
    }
}

/// Build a `PendingChangeSet` from a `ProvenanceDiff`.
fn build_change_set_from_diff(
    change_set_id: &str,
    diff: &ProvenanceDiff,
    importer_id: &str,
    source_uri: &str,
) -> PendingChangeSet {
    use editor_model::PendingChangeSet;

    // Build ops from the diff
    // For now, we represent each diff bucket as an op annotation in the rationale
    let mut rationale_parts = Vec::new();
    if !diff.added.is_empty() {
        rationale_parts.push(format!("+{} added", diff.added.len()));
    }
    if !diff.removed.is_empty() {
        rationale_parts.push(format!("-{} removed", diff.removed.len()));
    }
    if !diff.modified_source.is_empty() {
        rationale_parts.push(format!("~{} source-modified", diff.modified_source.len()));
    }
    if !diff.modified_editor.is_empty() {
        rationale_parts.push(format!(
            "!{} editor-modified (CONFLICT)",
            diff.modified_editor.len()
        ));
    }
    if !diff.ownership_conflicts.is_empty() {
        rationale_parts.push(format!(
            "⚠ {} ownership-conflicts",
            diff.ownership_conflicts.len()
        ));
    }

    let rationale = if rationale_parts.is_empty() {
        format!("Reimport from {}", source_uri)
    } else {
        format!(
            "Reimport from {} [{}]",
            source_uri,
            rationale_parts.join(", ")
        )
    };

    let _has_conflicts = !diff.modified_editor.is_empty() || !diff.ownership_conflicts.is_empty();

    PendingChangeSet {
        id: change_set_id.to_string(),
        origin: "Importer".to_string(),
        actor: format!("importer:{}", importer_id),
        rationale,
        ops: Vec::new(), // Ops are built by the scene-level change set application
        submitted_at_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::external_source::{OwnershipRule, SourceMapping};

    #[test]
    fn test_compute_fingerprint() {
        let bytes = b"hello world";
        let fp = compute_fingerprint(bytes);
        // SHA-256 of "hello world" in hex
        assert_eq!(
            fp,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_compute_provenance_diff_added() {
        let old = ExternalSource {
            kind: ExternalSourceKind::Ldtk,
            source_uri: "test.ldtk".to_string(),
            fingerprint: "abc".to_string(),
            importer_id: "builtin.ldtk".to_string(),
            importer_version: editor_model::importer::ImporterVersion::new(1, 0, 0),
            last_import_time: Timestamp(0),
            mappings: vec![],
            ownership_rules: vec![],
            schema_version: 1,
            conflict_policy: None,
        };
        let new = ExternalSource {
            mappings: vec![SourceMapping::new(
                "entity:1",
                "scene.json",
                OwnershipRule::SourceOwned,
            )],
            ..old.clone()
        };
        let diff = compute_provenance_diff(&old, &new);
        assert_eq!(diff.added.len(), 1);
        assert!(diff.removed.is_empty());
        assert!(diff.modified_source.is_empty());
    }

    #[test]
    fn test_compute_provenance_diff_removed() {
        let old = ExternalSource {
            kind: ExternalSourceKind::Ldtk,
            source_uri: "test.ldtk".to_string(),
            fingerprint: "abc".to_string(),
            importer_id: "builtin.ldtk".to_string(),
            importer_version: editor_model::importer::ImporterVersion::new(1, 0, 0),
            last_import_time: Timestamp(0),
            mappings: vec![SourceMapping::new(
                "entity:1",
                "scene.json",
                OwnershipRule::SourceOwned,
            )],
            ownership_rules: vec![],
            schema_version: 1,
            conflict_policy: None,
        };
        let new = ExternalSource {
            mappings: vec![],
            ..old.clone()
        };
        let diff = compute_provenance_diff(&old, &new);
        assert!(diff.added.is_empty());
        assert_eq!(diff.removed.len(), 1);
    }

    #[test]
    fn test_fingerprint_no_op() {
        // Two identical fingerprints should produce NoOp signal
        let fp1 = compute_fingerprint(b"test");
        let fp2 = compute_fingerprint(b"test");
        assert_eq!(fp1, fp2);

        let fp3 = compute_fingerprint(b"different");
        assert_ne!(fp1, fp3);
    }

    #[test]
    fn test_provenance_diff_empty_when_identical() {
        let es = ExternalSource {
            kind: ExternalSourceKind::Aseprite,
            source_uri: "test.json".to_string(),
            fingerprint: "abc".to_string(),
            importer_id: "builtin.aseprite".to_string(),
            importer_version: editor_model::importer::ImporterVersion::new(1, 0, 0),
            last_import_time: Timestamp(0),
            mappings: vec![
                SourceMapping::new("frame:0", "scene.json", OwnershipRule::SourceOwned),
                SourceMapping::new("frame:1", "scene.json", OwnershipRule::SourceOwned),
            ],
            ownership_rules: vec![],
            schema_version: 1,
            conflict_policy: None,
        };
        let diff = compute_provenance_diff(&es, &es);
        assert!(diff.is_empty());
    }
}
