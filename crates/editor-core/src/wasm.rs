//! WASM glue for `editor_core` — thin wrapper only.
//!
//! All ChangeWorkbench WASM exports (submit_pending_change_set, get_pending_change_sets,
//! approve_change_set, approve_selected_ops, reject_change_set) live in `lib.rs`.
//! `init_project_store` also lives in `lib.rs` (line ~1349).
//!
//! This file exists only for any `editor_core`-specific WASM entry points that
//! genuinely cannot live in `lib.rs`.
