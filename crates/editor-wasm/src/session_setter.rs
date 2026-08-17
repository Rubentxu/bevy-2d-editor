//! Session setter — called once at startup to hand off EditorSession ownership.

use std::sync::{Arc, Mutex};
use editor_application::EditorSession;

/// Set the global EditorSession. Called once from init_project_store().
/// Panics if called more than once.
pub fn set_session(session: Arc<Mutex<EditorSession>>) {
    // The static is in lib.rs
    crate::set_session_impl(session);
}
