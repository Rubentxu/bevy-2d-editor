//! Operation Log for the Bevy 2D Editor.
//!
//! The canonical reversible history of typed editor commands, used for undo/redo
//! and future agent auditing (Hito 0 §6.4, CONTEXT.md).
//!
//! As of H2.3 the implementation lives in `editor_model::operation_log` so that
//! both the editor-application layer and the Bevy layer can share the same types
//! without `editor-bevy` having to expose its `processor::apply` callback. Bevy
//! supplies an `ApplyCommandFn` impl (see `ProcessorApply` below) that the log
//! uses to apply/inverse commands during undo/redo.
//!
//! Re-exports keep `crate::operation_log::*` working unchanged for downstream
//! call sites (scene_session.rs, scenes.rs, processor.rs, transaction_bridge.rs).

pub use editor_model::operation_log::{
    ApplyCommandFn, DEFAULT_MAX_LOG_SIZE, LogEntry, OperationLog, OperationLogError,
};

use crate::document::SceneDocument;
use editor_model::command::{Command, CommandError};

/// Bevy-side adapter that delegates to `crate::processor::apply` so the
/// canonical OperationLog (in editor_model) can apply commands without taking
/// a hard dependency on the Bevy processor module.
///
/// Used by Bevy code paths that need to perform undo/redo while keeping the
/// processor as the single authority on command application semantics.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessorApply;

impl ApplyCommandFn for ProcessorApply {
    fn apply(&self, doc: &mut SceneDocument, cmd: &Command) -> Result<Command, CommandError> {
        crate::processor::apply(doc, cmd)
    }
}
