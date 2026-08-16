//! WASM glue — editor_application as the WASM cdylib.
//!
//! `editor_application` IS the WASM cdylib (wasm-pack builds this crate).
//! The ChangeWorkbench session lives in `EditorSession` (editor_application::session),
//! accessed here via a thread_local raw pointer set at WASM init time.
//!
//! ## Architecture (ADR-0031)
//!
//! `EditorSession` owns the pending ChangeSets map directly — no unsafe pointer
//! bridge needed since this code IS compiled into the WASM cdylib alongside
//! `EditorSession`. The thread_local pointer here is an internal implementation
//! detail of the WASM composition root, NOT an ADR-0031 violation (unlike the
//! old `WORKBENCH_SESSION` in editor-core which was accessed from a DIFFERENT
//! crate's WASM boundary).

#![cfg(target_arch = "wasm32")]

use std::collections::BTreeMap;
use std::sync::Arc;

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use editor_model::PendingChangeSet;
use editor_model::PendingChangeSetSummary;

use editor_core::CommandEnvelope;
use editor_core::CommandMetadata;
use editor_core::dispatch_command_via_kernel;

use crate::adapters::opfs::OpfsProjectStore;
use editor_model::ports::register_project_store;

// ─────────────────────────────────────────────────────────────────────────────
// Session pointer (internal implementation — lives in composition root)
// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    /// Raw u32 pointer to the active `EditorSession`'s pending_change_sets map.
    /// Set by `set_workbench_session_ptr` during WASM initialization.
    static WORKBENCH_SESSION_PTR: std::cell::RefCell<Option<u32>> =
        const { std::cell::RefCell::new(None) };
}

/// Set the workbench session pointer (called by JS glue at init time).
///
/// `ptr` is a raw u32 address of the session's `pending_change_sets` map.
#[wasm_bindgen]
pub fn set_workbench_session_ptr(ptr: u32) {
    let ptr = if ptr == 0 { None } else { Some(ptr) };
    WORKBENCH_SESSION_PTR.with(|cell| {
        *cell.borrow_mut() = ptr;
    });
}

/// Access the pending_change_sets map via the raw pointer.
/// Internal helper — the unsafe is confined here.
fn with_pending_map<R, F: FnOnce(&mut BTreeMap<String, PendingChangeSet>) -> R>(
    f: F,
) -> Result<R, JsValue> {
    WORKBENCH_SESSION_PTR
        .try_with(|cell| {
            let mut borrow = cell.borrow_mut();
            if let Some(ptr) = *borrow {
                // Safety: ptr was set by set_workbench_session_ptr from the same
                // JS glue layer that owns the EditorSession. The map lives in
                // WASM memory and is valid for the lifetime of the session.
                let map = unsafe { &mut *(ptr as *mut BTreeMap<String, PendingChangeSet>) };
                Ok(f(map))
            } else {
                Err(JsValue::from_str("Workbench session not initialized"))
            }
        })
        .map_err(|_| JsValue::from_str("Workbench session not initialized"))?
}

// ─────────────────────────────────────────────────────────────────────────────
// ChangeWorkbench WASM exports (ADR-0039)
// ─────────────────────────────────────────────────────────────────────────────

/// Submit a new pending ChangeSet for approval.
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
    with_pending_map(|map| {
        map.insert(change_id.clone(), cs);
    })?;

    Ok(change_id)
}

/// Get all pending ChangeSets as a JSON array of summaries.
#[wasm_bindgen]
pub fn get_pending_change_sets() -> Result<JsValue, JsValue> {
    let summaries = with_pending_map(|map| {
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
    let indices: Vec<usize> = with_pending_map(|map| {
        map.get(&change_id)
            .map(|cs| (0..cs.ops.len()).collect())
            .unwrap_or_default()
    })?;
    approve_selected_ops_impl(&change_id, &indices)
}

/// Approve only the selected operation indices in a pending ChangeSet.
#[wasm_bindgen]
pub fn approve_selected_ops(change_id: &str, indices_json: &str) -> Result<String, JsValue> {
    let indices: Vec<usize> = serde_json::from_str(indices_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid indices JSON: {}", e)))?;
    let change_id = change_id.to_string();
    approve_selected_ops_impl(&change_id, &indices)
}

/// Internal implementation of approve_selected_ops.
fn approve_selected_ops_impl(change_id: &str, indices: &[usize]) -> Result<String, JsValue> {
    // Get and remove the pending ChangeSet.
    let cs = with_pending_map(|map| map.remove(change_id))?
        .ok_or_else(|| JsValue::from_str(&format!("ChangeSet not found: {}", change_id)))?;

    // Dispatch each selected op.
    let mut applied_count = 0;
    for &idx in indices {
        let op_json = cs.ops.get(idx).ok_or_else(|| {
            JsValue::from_str(&format!(
                "Op index {} out of bounds (max {})",
                idx,
                cs.ops.len() - 1
            ))
        })?;

        let command: editor_core::Command = serde_json::from_value(op_json.clone())
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

        let result = dispatch_command_via_kernel(envelope);
        match result {
            Ok(_) => applied_count += 1,
            Err(e) => {
                // Restore remaining ops to the registry for retry.
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
                    let _ = with_pending_map(|map| {
                        map.insert(change_id.to_string(), restored_cs);
                    });
                }

                return Err(e);
            }
        }
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
    let _ = with_pending_map(|map| map.remove(&change_id))?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Project store initialization (moved from editor-core stub)
// ─────────────────────────────────────────────────────────────────────────────

/// Initialize the project store — MUST be called before any editor operations.
///
/// Creates an `OpfsProjectStore`, hydrates it, and registers it as the global
/// project store accessible via `editor_model::ports::with_project_store()`.
#[wasm_bindgen]
pub async fn init_project_store() -> Result<(), JsValue> {
    let store = OpfsProjectStore::new();
    store
        .hydrate()
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to hydrate project store: {}", e)))?;
    register_project_store(Arc::new(store));
    Ok(())
}
