//! Dispatch error types for the WASM boundary.
//! Replaces JsValue return type from dispatch_command_via_kernel.

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DispatchError {
    #[error("command validation failed: {0}")]
    ValidationFailed(String),
    #[error("command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("kernel error: {0}")]
    KernelError(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// WorldDocument has reached the documented 100-level cap.
    #[error("workspace too large: {0} levels (max 100)")]
    WorkspaceTooLarge(usize),
    /// `WorldLevelRef.asset_ref` does not resolve in `SceneAssetCatalog`.
    #[error("missing level reference: asset_ref '{0}' not in catalog")]
    MissingLevelRef(String),
}
