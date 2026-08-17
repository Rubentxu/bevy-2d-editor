//! WASM glue for `editor_core` — thin wrapper only.
//!
//! NOTE: `editor_core` is NO LONGER the WASM cdylib. The cdylib is now
//! `editor_application`. This file is kept for any future editor_core-specific
//! WASM entry points that genuinely cannot live in `editor_application`.
//!
//! All ChangeWorkbench WASM exports have been moved to
//! `crates/editor-application/src/wasm.rs`.
