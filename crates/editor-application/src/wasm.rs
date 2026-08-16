//! WASM glue — editor_application's WASM-specific initialization.
//!
//! `editor_application` is NOT directly compiled to WASM (wasm-pack builds `editor_core`).
//! This module is compiled for wasm32 when `editor_application` is built as part of the
//! workspace, but is NOT part of the wasm-pack WASM output.
//!
//! The ChangeWorkbench session bridge is set up in the JavaScript glue layer
//! (frontend/src/engine-bridge.ts) by calling `set_workbench_session_ptr`
//! exported from `editor_core::wasm`.
//
//! NOTE: `editor_core::wasm::init_project_store()` handles project store initialization
//! for the WASM binary. `editor_application::wasm::init_project_store` is NOT called
//! by the frontend (the WASM binary is `editor_core`, not `editor_application`).
