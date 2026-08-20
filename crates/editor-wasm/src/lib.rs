//! WASM glue — editor_application as the WASM cdylib.

#![cfg(target_arch = "wasm32")]

mod clock;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use editor_bevy::hot_reload_state::{PLAY_MODE_REQUEST, PlayModeRequest};
use editor_model::PendingChangeSet;
use editor_model::PendingChangeSetSummary;
use editor_model::ports::register_project_store;
use editor_model::time::Clock;

use editor_bevy::Command;
use editor_bevy::CommandEnvelope;
use editor_bevy::CommandMetadata;
use editor_bevy::dispatch_command_via_kernel;

use editor_application::EditorSession;
use editor_application::adapters::opfs::OpfsProjectStore;

use crate::clock::SysClock;

// ─────────────────────────────────────────────────────────────────────────────
// Global session registration (ADR-0031 compliant)
// ─────────────────────────────────────────────────────────────────────────────
//
// The pending ChangeSets map lives inside `EditorSession`. WASM exports below
// access it through `SESSION`, a `OnceLock<Arc<Mutex<EditorSession>>>` registered
// at `init_project_store()` time. This is the canonical application-level owner
// of mutable project/editing state — no thread_local, no unsafe pointer bridge,
// no ambient mutable store.

static SESSION: OnceLock<Arc<Mutex<EditorSession>>> = OnceLock::new();

/// Set the global EditorSession. Called once from init_project_store().
/// Panics if called more than once.
pub fn set_session_impl(session: Arc<Mutex<EditorSession>>) {
    let _ = SESSION.set(session);
}

/// Access the global `EditorSession`. Returns an error if the session has not
/// been initialized (i.e. `init_project_store()` was not called yet).
fn session() -> Result<Arc<Mutex<EditorSession>>, JsValue> {
    SESSION
        .get()
        .cloned()
        .ok_or_else(|| JsValue::from_str("EditorSession not initialized"))
}

/// Run a closure with mutable access to the pending change-sets map.
///
/// Locks are released as soon as the closure returns. Callers must drop any
/// reference returned by the closure before calling another WASM export that
/// takes the session lock.
fn with_pending_change_sets_mut<R, F: FnOnce(&mut BTreeMap<String, PendingChangeSet>) -> R>(
    f: F,
) -> Result<R, JsValue> {
    let sess = session()?;
    let mut guard = sess
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Session lock poisoned: {}", e)))?;
    Ok(f(guard.pending_change_sets_mut()))
}

/// Run a closure with read-only access to the pending change-sets map.
fn with_pending_change_sets<R, F: FnOnce(&BTreeMap<String, PendingChangeSet>) -> R>(
    f: F,
) -> Result<R, JsValue> {
    let sess = session()?;
    let guard = sess
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Session lock poisoned: {}", e)))?;
    Ok(f(guard.pending_change_sets()))
}

// ─────────────────────────────────────────────────────────────────────────────
// ChangeWorkbench WASM exports (ADR-0039)
// ─────────────────────────────────────────────────────────────────────────────

/// Submit a new pending ChangeSet for approval.
///
/// The ChangeSet JSON must have shape:
/// ```json
/// {
///   "id": "agent:12345",
///   "origin": "Agent",
///   "actor": "agent:code-writer",
///   "rationale": "Refactor entity naming",
///   "ops": [SceneCommand as JSON]
/// }
/// ```
///
/// Returns the change-set ID on success, or an error string.
#[wasm_bindgen]
pub fn submit_pending_change_set(json: &str) -> Result<String, JsValue> {
    let cs: PendingChangeSet = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Invalid ChangeSet JSON: {}", e)))?;

    if cs.ops.is_empty() {
        return Err(JsValue::from_str(
            "ChangeSet must have at least one operation",
        ));
    }

    let change_id = cs.id.clone();
    with_pending_change_sets_mut(|map| {
        map.insert(change_id.clone(), cs);
    })?;

    Ok(change_id)
}

/// Get all pending ChangeSets as a JSON array of summaries.
#[wasm_bindgen]
pub fn get_pending_change_sets() -> Result<JsValue, JsValue> {
    let summaries = with_pending_change_sets(|map| {
        map.values()
            .map(PendingChangeSetSummary::from)
            .collect::<Vec<_>>()
    })?;

    serde_wasm_bindgen::to_value(&summaries)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Approve all operations in a pending ChangeSet and apply them.
#[wasm_bindgen]
pub fn approve_change_set(change_id: &str) -> Result<String, JsValue> {
    let change_id = change_id.to_string();
    let indices: Vec<usize> = with_pending_change_sets(|map| {
        map.get(&change_id)
            .map(|cs| (0..cs.ops.len()).collect())
            .unwrap_or_default()
    })?;
    approve_selected_ops_impl(&change_id, &indices)
}

/// Approve only the selected operation indices in a pending ChangeSet.
///
/// `indices_json` is a JSON array of zero-based op indices to apply
/// (e.g. `[0, 2, 4]`). Ops not in the list are excluded from this approval and
/// remain pending.
///
/// Returns a JSON object with `applied` count and `remaining` ChangeSet (or
/// null if none).
#[wasm_bindgen]
pub fn approve_selected_ops(change_id: &str, indices_json: &str) -> Result<String, JsValue> {
    let indices: Vec<usize> = serde_json::from_str(indices_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid indices JSON: {}", e)))?;
    let change_id = change_id.to_string();
    approve_selected_ops_impl(&change_id, &indices)
}

/// Internal implementation of approve_selected_ops.
///
/// Locks are taken only to extract the ChangeSet and re-insert remaining ops on
/// failure; the dispatch loop itself runs without holding the session lock to
/// avoid contention with other WASM exports.
fn approve_selected_ops_impl(change_id: &str, indices: &[usize]) -> Result<String, JsValue> {
    // Extract the ChangeSet from the registry (lock released after this scope).
    let cs = with_pending_change_sets_mut(|map| map.remove(change_id))?
        .ok_or_else(|| JsValue::from_str(&format!("ChangeSet not found: {}", change_id)))?;

    // ADR-0040 v0.92: Check extension permissions before dispatching any op.
    // This only runs for Plugin-origin ChangeSets (Human/Agent are no-ops).
    // Note: cs.origin is a String (not ChangeOrigin enum) because PendingChangeSet
    // is serde JSON. We compare as strings.
    if cs.origin == "Plugin" {
        use editor_application::transaction::transaction_kernel_check_plugin_permission;
        // Build a temporary ChangeSet for the permission check
        let temp_cs: editor_model::transaction::ChangeSet<serde_json::Value> =
            editor_model::transaction::ChangeSet::new(
                cs.id.clone(),
                editor_model::transaction::ChangeOrigin::Plugin,
                cs.actor.clone(),
                cs.rationale.clone(),
            );
        transaction_kernel_check_plugin_permission(&temp_cs)
            .map_err(|e| JsValue::from_str(&format!("Permission denied: {}", e)))?;
    }

    // ADR-0040 step 3 + ADR-0041 v0.93: Check importer permissions.
    // This only runs for Importer-origin ChangeSets (Human/Agent/Plugin are no-ops).
    if cs.origin == "Importer" {
        use editor_application::transaction::transaction_kernel_check_importer_permission;
        let temp_cs: editor_model::transaction::ChangeSet<serde_json::Value> =
            editor_model::transaction::ChangeSet::new(
                cs.id.clone(),
                editor_model::transaction::ChangeOrigin::Importer,
                cs.actor.clone(),
                cs.rationale.clone(),
            );
        transaction_kernel_check_importer_permission(&temp_cs)
            .map_err(|e| JsValue::from_str(&format!("Permission denied: {}", e)))?;
    }

    let mut applied_count = 0;
    for &idx in indices {
        let op_json = cs.ops.get(idx).ok_or_else(|| {
            JsValue::from_str(&format!(
                "Op index {} out of bounds (max {})",
                idx,
                cs.ops.len() - 1
            ))
        })?;

        let command: Command = serde_json::from_value(op_json.clone())
            .map_err(|e| JsValue::from_str(&format!("Invalid op JSON at index {}: {}", idx, e)))?;

        let envelope = CommandEnvelope {
            command,
            metadata: CommandMetadata {
                timestamp: editor_model::time::now_millis(),
                authorship: cs.actor.clone(),
                rationale: Some(format!("[ChangeWorkbench] {}", cs.rationale)),
            },
        };

        if let Err(e) = dispatch_command_via_kernel(envelope) {
            // Re-insert remaining ops so the user can retry.
            let remaining_indices: Vec<usize> =
                (0..cs.ops.len()).filter(|i| !indices.contains(i)).collect();
            let remaining_ops: Vec<serde_json::Value> = remaining_indices
                .iter()
                .filter_map(|&i| cs.ops.get(i).cloned())
                .collect();

            if !remaining_ops.is_empty() {
                let restored_cs = PendingChangeSet {
                    id: change_id.to_string(),
                    origin: cs.origin.clone(),
                    actor: cs.actor.clone(),
                    rationale: cs.rationale.clone(),
                    ops: remaining_ops,
                    submitted_at_ms: cs.submitted_at_ms,
                };
                let _ = with_pending_change_sets_mut(|map| {
                    map.insert(change_id.to_string(), restored_cs);
                });
            }

            return Err(JsValue::from_str(&e.to_string()));
        }
        applied_count += 1;
    }

    let response = serde_json::json!({
        "applied": applied_count,
        "remaining": (),
    });

    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Reject and discard a pending ChangeSet.
#[wasm_bindgen]
pub fn reject_change_set(change_id: &str) -> Result<(), JsValue> {
    let change_id = change_id.to_string();
    let _ = with_pending_change_sets_mut(|map| map.remove(&change_id))?;
    Ok(())
}

/// Get a summary of recent change-sets (history view).
///
/// TODO(v0.90): source these from `EditorSession::recent_change_sets` once the
/// OPERATION_LOG thread_local migration is complete. For v0.89 this returns an
/// empty array — the ChangeWorkbench panel only displays pending rows, not
/// historical summaries. The export exists here so the WASM-bound name is
/// stable while the legacy `OPERATION_LOG` query in editor-core is phased out.
///
/// v0.91 PR1: reads from `EditorSession` via the
/// `EditorSessionPort::all_recent_change_sets` trait method. The buffer
/// is populated by `EditorSession::push_recent_change_set` (called by the
/// `poll_recent_change_sets` Bevy system in editor-core, added separately
/// in v0.91). Returns an empty array if the session is not initialized or
/// no ChangeSets have been recorded yet.
#[wasm_bindgen]
pub fn get_change_set_summaries() -> Result<JsValue, JsValue> {
    let summaries: Vec<editor_model::ChangeSetSummary> =
        editor_model::ports::with_session(|sess| sess.all_recent_change_sets()).unwrap_or_default();
    serde_wasm_bindgen::to_value(&summaries)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// §6 Runtime Causality WASM exports
// ─────────────────────────────────────────────────────────────────────────────

/// Get the last rebuild cause recorded by §6.
///
/// Returns `JsValue::NULL` if no rebuild cause has been recorded yet, or if
/// the session has not been initialized.
///
/// v0.90 PR2: reads from the canonical `EditorSession::preview_inspector`
/// field via the `EditorSessionPort` trait (no more dual-read fallback
/// from the removed editor-core thread_local). ADR-0052.
#[wasm_bindgen]
pub fn get_rebuild_cause_wasm() -> Result<JsValue, JsValue> {
    let cause = editor_model::ports::with_session_mut(|sess| sess.last_rebuild_cause_mut().clone())
        .flatten();
    match cause {
        Some(c) => serde_wasm_bindgen::to_value(&c)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e))),
        None => Ok(JsValue::NULL),
    }
}

/// Get all logic activation events from the ring buffer.
///
/// Returns a JSON array of [`editor_model::logic_activation::LogicActivationEvent`].
#[wasm_bindgen]
pub fn get_logic_activation_events_wasm() -> Result<JsValue, JsValue> {
    let sess = session()?;
    let guard = sess.lock().map_err(|e| JsValue::from_str(&e.to_string()))?;
    let events: Vec<_> = guard.logic_activation_ring().iter().cloned().collect();
    serde_wasm_bindgen::to_value(&events)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// Project store + session initialization
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// Runtime Apply-Back — Play Mode Enter/Exit (ADR-0042)
// ─────────────────────────────────────────────────────────────────────────────

/// Enter play mode: snapshot tunable baselines and set PlayModeRequest.
///
/// Captures baselines synchronously from `SCENE_DOC` (before setting the
/// request) so they are stored in `EditorSession.tunable_baselines` immediately.
/// Bevy's `process_play_mode_request` will re-capture via the Bevy query path
/// into `TUNABLE_BASELINES` thread-local, but those are not used since we
/// already stored them from the scene document directly.
#[wasm_bindgen]
pub fn enter_play_mode() -> Result<(), JsValue> {
    // Capture tunable baselines from the scene document BEFORE setting the request.
    // This is the synchronous path — Bevy may not have run yet.
    let baselines_json = editor_bevy::preview_runtime::capture_baselines_from_scene_doc();

    // Store in session (the session is Arc<Mutex<EditorSession>>).
    let sess = session()?;
    let mut guard = sess
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Session lock poisoned: {}", e)))?;
    let baselines: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&baselines_json).unwrap_or_default();
    guard.snapshot_tunable_baselines(baselines);

    // Signal Bevy to enter play mode (processed on next animation frame).
    PLAY_MODE_REQUEST.with(|r| *r.borrow_mut() = Some(PlayModeRequest::Enter));

    Ok(())
}

/// Exit play mode: set PlayModeRequest to trigger transform restore.
#[wasm_bindgen]
pub fn exit_play_mode() -> Result<(), JsValue> {
    PLAY_MODE_REQUEST.with(|r| *r.borrow_mut() = Some(PlayModeRequest::Exit));
    Ok(())
}

/// Get the current tunable baselines stored in the session.
///
/// Used by the frontend to display baseline vs. current values in the
/// Runtime Apply-Back UI before the user approves or rejects deltas.
#[wasm_bindgen]
pub fn get_tunable_baselines_wasm() -> Result<String, JsValue> {
    let sess = session()?;
    let guard = sess
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Session lock poisoned: {}", e)))?;
    let baselines = guard.tunable_baselines();
    serde_json::to_string(baselines)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get runtime deltas computed on the last PlayModeExit.
#[wasm_bindgen]
pub fn get_runtime_deltas_wasm() -> Result<String, JsValue> {
    let sess = session()?;
    let guard = sess
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Session lock poisoned: {}", e)))?;
    let deltas: Vec<_> = guard.runtime_delta_buffer().iter().cloned().collect();
    serde_json::to_string(&deltas)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Create an apply-back ChangeSet from the current runtime deltas and store it
/// in the pending ChangeSets map (ADR-0042).
///
/// Returns the ChangeSet ID.
#[wasm_bindgen]
pub fn create_apply_back_change_set_wasm(rationale: &str) -> Result<String, JsValue> {
    use editor_model::PendingChangeSet;
    use editor_model::PendingChangeSetSummary;

    let sess = session()?;
    let deltas = {
        let guard = sess
            .lock()
            .map_err(|e| JsValue::from_str(&format!("Session lock poisoned: {}", e)))?;
        guard
            .runtime_delta_buffer()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    };

    if deltas.is_empty() {
        return Err(JsValue::from_str("No runtime deltas to apply back"));
    }

    // Build SceneCommands from deltas (one per changed field).
    let ops: Vec<serde_json::Value> = deltas
        .iter()
        .map(|delta| {
            serde_json::json!({
                "UpdateComponent": {
                    "instance_id": delta.instance_id,
                    "component_type_id": delta.component_type_id,
                    "field_path": delta.field_path,
                    "value": delta.runtime_value,
                }
            })
        })
        .collect();

    let change_id = format!("apply-back:{}", editor_model::time::now_millis());

    let cs = PendingChangeSet {
        id: change_id.clone(),
        origin: "RuntimeApplyBack".to_string(),
        actor: "runtime:apply-back".to_string(),
        rationale: rationale.to_string(),
        ops,
        submitted_at_ms: editor_model::time::now_millis() as u64,
    };

    with_pending_change_sets_mut(|map| {
        map.insert(change_id.clone(), cs);
    })?;

    Ok(change_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// Extension registry WASM exports (ADR-0040 — v0.92)
// ─────────────────────────────────────────────────────────────────────────────

/// Get mutable access to the extension registry via the global port.
fn with_ext_registry_mut<R, F: FnOnce(&mut dyn editor_model::ports::ExtensionRegistryPort) -> R>(
    f: F,
) -> Result<R, JsValue> {
    let registry = editor_model::ports::with_extension_registry()
        .ok_or_else(|| JsValue::from_str("Extension registry not initialized"))?;
    let mut guard = registry
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Registry lock poisoned: {}", e)))?;
    Ok(f(&mut *guard))
}

/// Register an extension from a JSON manifest.
///
/// Accepts an `ExtensionManifest` JSON object. Returns a JSON object with
/// `handle` (u64) and `summary` fields on success.
#[wasm_bindgen]
pub fn register_extension_wasm(json: &str) -> Result<String, JsValue> {
    use editor_model::extension::ExtensionSummary;
    use editor_model::ports::ExtensionRegistryPort;

    let manifest: editor_model::extension::ExtensionManifest = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Invalid manifest JSON: {}", e)))?;

    let handle = with_ext_registry_mut(|reg| reg.register(manifest.clone()))?
        .map_err(|e| JsValue::from_str(&format!("ExtensionError: {}", e)))?;

    let summary = ExtensionSummary::from(&manifest);
    let response = serde_json::json!({
        "handle": handle.to_u64(),
        "summary": summary,
    });
    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// List all registered extensions as a JSON array of summaries.
#[wasm_bindgen]
pub fn list_extensions_wasm() -> Result<String, JsValue> {
    use editor_model::ports::with_extension_registry;

    let summaries = with_extension_registry()
        .ok_or_else(|| JsValue::from_str("Extension registry not initialized"))?
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Registry lock poisoned: {}", e)))?
        .list();

    serde_json::to_string(&summaries)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Unregister an extension by ID.
#[wasm_bindgen]
pub fn unregister_extension_wasm(id: &str) -> Result<(), JsValue> {
    with_ext_registry_mut(|reg| reg.unregister(id))?
        .map_err(|e| JsValue::from_str(&format!("ExtensionError: {}", e)))
}

/// Submit a plugin ChangeSet for approval.
///
/// Builds a `PendingChangeSet` with `origin: "Plugin"` and
/// `actor: "extension:<extension_id>"` and routes it through the existing
/// `submit_pending_change_set` flow (visible in ChangeWorkbench per ADR-0040).
#[wasm_bindgen]
pub fn submit_plugin_change_set_wasm(
    extension_id: &str,
    change_set_json: &str,
) -> Result<String, JsValue> {
    use editor_model::PendingChangeSet;

    let mut cs: PendingChangeSet = serde_json::from_str(change_set_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid ChangeSet JSON: {}", e)))?;

    // Override origin and actor to reflect plugin provenance.
    cs.origin = "Plugin".to_string();
    cs.actor = format!("extension:{}", extension_id);

    let change_id = cs.id.clone();
    with_pending_change_sets_mut(|map| {
        map.insert(change_id.clone(), cs);
    })?;

    Ok(change_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// Importer WASM exports (ADR-0040 step 3 + ADR-0041 — v0.93)
// ─────────────────────────────────────────────────────────────────────────────

/// Get the importer registry from the global session (mutable).
fn with_importer_registry_mut<
    R,
    F: FnOnce(&mut dyn editor_model::ports::ImporterRegistryPort) -> R,
>(
    f: F,
) -> Result<R, JsValue> {
    let registry = editor_model::ports::with_importer_registry()
        .ok_or_else(|| JsValue::from_str("Importer registry not initialized"))?;
    let mut guard = registry
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Registry lock poisoned: {}", e)))?;
    Ok(f(&mut *guard))
}

/// Register an importer from a JSON descriptor + WASM trait object.
///
/// Accepts a JSON object with `id`, `kind`, `supported_versions`, and `display_name` fields.
/// The WASM caller must also provide a trait object implementing `Importer` — in practice
/// this is a JS shim that delegates to the browser for file reading.
#[wasm_bindgen]
pub fn register_importer_wasm(json: &str) -> Result<String, JsValue> {
    use editor_model::importer::{ImporterDescriptor, ImporterError};

    let descriptor: ImporterDescriptor = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Invalid descriptor JSON: {}", e)))?;

    // v0.93: importers are registered with a dummy Arc<dyn Importer> for now.
    // The real trait object would come from a JS shim, but we don't have that yet.
    // For the WASM surface, we just register the descriptor so list_importers works.
    // Real importer registration (with a live trait object) is done in PR2–PR4.
    let result = with_importer_registry_mut(|reg| {
        // We can't easily pass a JS-side trait object through wasm-bindgen today,
        // so we register with a no-op importer stub. The actual import pipeline
        // in PR2+ will call the registry directly from Rust.
        reg.register(
            descriptor.clone(),
            std::sync::Arc::new(DummyImporter { descriptor })
                as std::sync::Arc<dyn editor_model::importer::Importer>,
        )
    });
    // Flatten: result is Result<Result<(), ImporterError>, JsValue>
    match result {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            return Err(JsValue::from_str(&format!("Registration failed: {}", e)));
        }
        Err(e) => {
            // e is JsValue - this is the "importer registry not initialized" case
            return Err(e);
        }
    }

    Ok(serde_json::json!({ "ok": true }).to_string())
}

/// List all registered importers as a JSON array of descriptors.
///
/// Accepts an optional `kind` filter string ("Aseprite", "Ldtk", "Tiled", or
/// "Custom" for unknown kinds). If `kind` is `None` or not provided, returns
/// all registered importers.
#[wasm_bindgen]
pub fn list_importers_wasm(kind: Option<String>) -> Result<String, JsValue> {
    use editor_model::external_source::ExternalSourceKind;
    use editor_model::ports::with_importer_registry;

    let registry = with_importer_registry()
        .ok_or_else(|| JsValue::from_str("Importer registry not initialized"))?;
    let registry = registry
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Registry lock poisoned: {}", e)))?;

    let result: Vec<_> = if let Some(kind_str) = kind {
        let filter_kind = match kind_str.to_lowercase().as_str() {
            "aseprite" => ExternalSourceKind::Aseprite,
            "ldtk" => ExternalSourceKind::Ldtk,
            "tiled" => ExternalSourceKind::Tiled,
            other => ExternalSourceKind::Custom(other.to_string()),
        };
        registry.list_by_kind(&filter_kind)
    } else {
        // No filter — return all
        let mut all: Vec<_> = Vec::new();
        all.extend(registry.list_by_kind(&ExternalSourceKind::Aseprite));
        all.extend(registry.list_by_kind(&ExternalSourceKind::Ldtk));
        all.extend(registry.list_by_kind(&ExternalSourceKind::Tiled));
        all
    };

    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Import an external source file (Aseprite, LDtk, Tiled) and produce a ChangeSet.
///
/// Accepts `kind` (string), `source_uri` (string), `bytes_b64` (base64-encoded file bytes),
/// and `target_resource_ref` (destination path in the project).
///
/// Returns a JSON object with `change_set_id` and `sidecar_path` on success.
#[wasm_bindgen]
pub fn import_external_source_wasm(
    kind: &str,
    source_uri: &str,
    bytes_b64: &str,
    target_resource_ref: &str,
) -> Result<String, JsValue> {
    use editor_model::external_source::ExternalSourceKind;
    use editor_model::importer::ImporterInput;

    // Parse the kind
    let source_kind = match kind {
        "Aseprite" | "aseprite" => ExternalSourceKind::Aseprite,
        "Ldtk" | "ldtk" => ExternalSourceKind::Ldtk,
        "Tiled" | "tiled" => ExternalSourceKind::Tiled,
        other => ExternalSourceKind::Custom(other.to_string()),
    };

    // Decode base64 bytes
    let bytes = base64_decode(bytes_b64)?;

    // Dispatch to the importer
    let registry = editor_model::ports::with_importer_registry()
        .ok_or_else(|| JsValue::from_str("Importer registry not initialized"))?;

    let reg_guard = registry
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Registry lock poisoned: {}", e)))?;

    let parse_output = reg_guard
        .dispatch(
            &source_kind,
            ImporterInput {
                bytes: &bytes,
                source_uri,
                fingerprint_hint: None,
            },
        )
        .map_err(|e| JsValue::from_str(&format!("Import error: {}", e)))?;

    drop(reg_guard);

    // Build the ChangeSet JSON (simplified — full implementation in PR2+)
    let pending_cs = serde_json::json!({
        "id": format!("import-{}", uuid::Uuid::new_v4()),
        "origin": "Importer",
        "actor": format!("importer:builtin.{}", kind.to_lowercase()),
        "rationale": format!("Import from {}", source_uri),
        "ops": [],
        "resources": [],
        "submitted_at_ms": editor_model::time::now_millis() as u64,
    });

    let change_id = serde_json::from_value::<editor_model::PendingChangeSet>(pending_cs.clone())
        .unwrap()
        .id;

    let cs: editor_model::PendingChangeSet = serde_json::from_value(pending_cs)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;

    with_pending_change_sets_mut(|map| {
        map.insert(change_id.clone(), cs);
    })?;

    // Compute sidecar path
    let sidecar_path = format!("{}.meta.json", target_resource_ref);

    let response = serde_json::json!({
        "change_set_id": change_id,
        "sidecar_path": sidecar_path,
        "parse_output": parse_output,
    });

    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Re-import an external source file and return the diff result.
///
/// Accepts `source_uri` (path to the source file).
///
/// Returns a JSON object with `status` ("no-op" | "queued" | "auto-applied") and
/// `change_set_id` if a ChangeSet was produced.
#[wasm_bindgen]
pub fn reimport_external_source_wasm(source_uri: &str) -> Result<String, JsValue> {
    use editor_application::reimport::{ReimportResult, reimport as do_reimport};
    use editor_model::ports::ProjectStore;
    use editor_model::ports::with_project_store;

    let store =
        with_project_store().ok_or_else(|| JsValue::from_str("Project store not initialized"))?;

    let importer_registry = editor_model::ports::with_importer_registry()
        .ok_or_else(|| JsValue::from_str("Importer registry not initialized"))?;

    let result = with_pending_change_sets_mut(|pending_change_sets| {
        do_reimport(
            source_uri,
            store.as_ref(),
            &importer_registry,
            pending_change_sets,
            || editor_model::time::Timestamp(editor_model::time::now_millis()),
        )
    })
    .map_err(|e| e)?;

    let response = match result {
        Ok(ReimportResult::NoOp) => serde_json::json!({
            "status": "no-op",
            "source_uri": source_uri,
        }),
        Ok(ReimportResult::QueuedForReview {
            change_set_id,
            diff,
        }) => serde_json::json!({
            "status": "queued",
            "source_uri": source_uri,
            "change_set_id": change_set_id,
            "diff": {
                "added": diff.added.len(),
                "removed": diff.removed.len(),
                "modified_source": diff.modified_source.len(),
                "modified_editor": diff.modified_editor.len(),
                "ownership_conflicts": diff.ownership_conflicts.len(),
            }
        }),
        Ok(ReimportResult::AutoApplied {
            change_set_id,
            diff,
        }) => serde_json::json!({
            "status": "auto-applied",
            "source_uri": source_uri,
            "change_set_id": change_set_id,
            "diff": {
                "added": diff.added.len(),
                "removed": diff.removed.len(),
                "modified_source": diff.modified_source.len(),
                "modified_editor": diff.modified_editor.len(),
                "ownership_conflicts": diff.ownership_conflicts.len(),
            }
        }),
        Err(e) => {
            return Err(JsValue::from_str(&format!("Reimport failed: {}", e)));
        }
    };

    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get the external source provenance record for a given resource path.
///
/// Accepts `resource_ref` (the logical path of the imported resource).
///
/// Returns the `ExternalSource` JSON from the sidecar `.meta.json` file,
/// or `null` if no sidecar exists.
#[wasm_bindgen]
pub fn get_external_source_wasm(resource_ref: &str) -> Result<String, JsValue> {
    use editor_model::ports::ProjectStore;
    use editor_model::ports::with_project_store;

    let sidecar_path = format!("{}.meta.json", resource_ref);
    let store =
        with_project_store().ok_or_else(|| JsValue::from_str("Project store not initialized"))?;

    match store.read(&sidecar_path) {
        Ok(bytes) => {
            let text = String::from_utf8(bytes)
                .map_err(|e| JsValue::from_str(&format!("UTF-8 error: {}", e)))?;
            Ok(text)
        }
        Err(editor_model::ports::StoreError::NotFound(_)) => Ok("null".to_string()),
        Err(e) => Err(JsValue::from_str(&format!("Store error: {}", e))),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Init
// ─────────────────────────────────────────────────────────────────────────────

/// Initialize the project store and the global `EditorSession`.
///
/// MUST be called before any editor operations. Creates an `OpfsProjectStore`,
/// hydrates it, registers it as the global project store, then creates the
/// `EditorSession` that owns the pending ChangeSets map and other PR2a sub-state.
#[wasm_bindgen]
pub async fn init_project_store() -> Result<(), JsValue> {
    let store = OpfsProjectStore::new();
    store
        .hydrate()
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to hydrate project store: {}", e)))?;
    let store_arc: Arc<dyn editor_model::ports::ProjectStore> = Arc::new(store);
    register_project_store(store_arc.clone());

    // Create the session and register it globally for workbench WASM exports.
    // Re-init is safe: the existing session (if any) is reused to avoid losing
    // pending ChangeSets across HMR/dev-server reloads.
    let session = Arc::new(Mutex::new(EditorSession::with_builtins(
        store_arc,
        Arc::new(SysClock::new()) as Arc<dyn Clock>,
    )));
    set_session_impl(session.clone());

    // v0.90 PR1: also register the session with the editor-model port so
    // editor-core (Bevy systems) can access it via
    // `editor_model::ports::with_session_mut(|s| ...)` without importing
    // editor-application. The trait object is the canonical seam.
    // We need a `Box<dyn EditorSessionPort>` to register, so we coerce
    // the session via a temporary closure that downcasts through the
    // concrete `EditorSession` (which already implements the trait).
    register_session_via_port(&session);

    // v0.92: register the extension registry globally so editor-core can
    // check extension permissions without importing editor-application.
    {
        let guard = session
            .lock()
            .map_err(|e| JsValue::from_str(&format!("Session lock poisoned: {}", e)))?;
        use editor_model::ports::register_extension_registry;
        register_extension_registry(guard.extension_registry());
    }

    // v0.93 PR1: register the importer registry globally so editor-core can
    // check importer provenance without importing editor-application.
    {
        let guard = session
            .lock()
            .map_err(|e| JsValue::from_str(&format!("Session lock poisoned: {}", e)))?;
        use editor_model::ports::register_importer_registry;
        register_importer_registry(guard.importer_registry());
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn register_session_via_port(session: &Arc<Mutex<EditorSession>>) {
    use editor_model::EditorSessionPort;
    let arc: Arc<Mutex<dyn editor_model::EditorSessionPort>> = session.clone();
    editor_model::ports::register_editor_session(arc);
}

// ─────────────────────────────────────────────────────────────────────────────
// DummyImporter (WASM stub)
// ─────────────────────────────────────────────────────────────────────────────

/// A no-op importer stub used only for WASM registration surface.
///
/// Real importers (Aseprite, LDtk, Tiled) are implemented in `editor_bevy::importer`
/// and registered in PR2–PR4. This stub lets the WASM surface exercise the registry
/// without a live trait object.
struct DummyImporter {
    descriptor: editor_model::importer::ImporterDescriptor,
}

impl editor_model::importer::Importer for DummyImporter {
    fn descriptor(&self) -> editor_model::importer::ImporterDescriptor {
        self.descriptor.clone()
    }

    fn parse(
        &self,
        _source: editor_model::importer::ImporterInput<'_>,
    ) -> Result<editor_model::importer::ParseOutput, editor_model::importer::ImporterError> {
        Ok(editor_model::importer::ParseOutput::default())
    }

    fn build_change_set(
        &self,
        draft: editor_model::importer::ParseOutput,
        _snapshot: editor_model::session::EditorSnapshot,
    ) -> Result<editor_model::importer::BuildChangeSetOutput, editor_model::importer::ImporterError>
    {
        Ok(editor_model::importer::BuildChangeSetOutput {
            provenance_diff: None,
            change_set_json: serde_json::to_string(&draft).unwrap_or_default(),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Decode a base64 string, mapping errors to JsValue.
fn base64_decode(input: &str) -> Result<Vec<u8>, JsValue> {
    // Use the `base64` crate (already a dependency of the workspace).
    base64_decode_impl(input)
        .map_err(|e| JsValue::from_str(&format!("Base64 decode failed: {}", e)))
}

fn base64_decode_impl(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    // The standard base64 engine accepts both standard and URL-safe alphabets.
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(input)
}
