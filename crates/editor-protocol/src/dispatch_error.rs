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
}
