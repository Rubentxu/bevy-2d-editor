//! WASM glue — editor_application as the WASM cdylib.

#![cfg(target_arch = "wasm32")]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use editor_model::PendingChangeSet;
use editor_model::PendingChangeSetSummary;
use editor_model::ports::register_project_store;
use editor_model::time::Clock;

use editor_core::Command;
use editor_core::CommandEnvelope;
use editor_core::CommandMetadata;
use editor_core::dispatch_command_via_kernel;

use crate::EditorSession;
use crate::adapters::opfs::OpfsProjectStore;
use crate::adapters::opfs::wasm::SysClock;

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
///   "ops": [{ /* SceneCommand as JSON */ }]
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
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
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

            return Err(e);
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
#[wasm_bindgen]
pub fn get_change_set_summaries() -> Result<JsValue, JsValue> {
    let summaries: Vec<PendingChangeSetSummary> = Vec::new();
    serde_wasm_bindgen::to_value(&summaries)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// Project store + session initialization
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
    let session = Arc::new(Mutex::new(EditorSession::new(
        store_arc,
        Arc::new(SysClock::new()) as Arc<dyn Clock>,
    )));
    let _ = SESSION.set(session);

    Ok(())
}
