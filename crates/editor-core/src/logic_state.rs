//! HIGH-1 phase 2: logic-graph state sub-module.
//!
//! Owns the LOGIC_GRAPH_DOC (active graph being edited) and the
//! LOGIC_OPERATION_LOG (per-graph undo/redo).

use std::cell::RefCell;

use crate::logic_command::LogicOperationLog;
use crate::logic_graph::LogicGraphAsset;

thread_local! {
    /// Logic Graph document: the active logic graph being edited.
    pub static LOGIC_GRAPH_DOC: RefCell<Option<LogicGraphAsset>> = const { RefCell::new(None) };
    /// Logic operation log: per-graph undo/redo history.
    pub static LOGIC_OPERATION_LOG: RefCell<LogicOperationLog> = const { RefCell::new(LogicOperationLog::new_const()) };
}

/// Get an immutable borrowed reference to the LogicGraphAsset.
pub fn with_logic_graph<F, R>(f: F) -> R
where
    F: FnOnce(&Option<LogicGraphAsset>) -> R,
{
    LOGIC_GRAPH_DOC.with(|cell| f(&*cell.borrow()))
}

/// Get a mutable borrowed reference to the LogicGraphAsset.
pub fn with_logic_graph_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Option<LogicGraphAsset>) -> R,
{
    LOGIC_GRAPH_DOC.with(|cell| f(&mut *cell.borrow_mut()))
}

/// Get an immutable borrowed reference to the LogicOperationLog.
pub fn with_logic_log<F, R>(f: F) -> R
where
    F: FnOnce(&LogicOperationLog) -> R,
{
    LOGIC_OPERATION_LOG.with(|cell| f(&*cell.borrow()))
}

/// Get a mutable borrowed reference to the LogicOperationLog.
pub fn with_logic_log_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut LogicOperationLog) -> R,
{
    LOGIC_OPERATION_LOG.with(|cell| f(&mut *cell.borrow_mut()))
}
