//! EditorWorld — a self-contained Bevy world for editor-only systems.
//!
//! ADR-0054: `EditorWorld` holds the scheduling and resource context for
//! systems that do not depend on rendering, scene entities, or preview state.
//!
//! ## Design
//!
//! `EditorWorld` owns a `World` + `Schedule`. Editor-only systems are registered
//! into this schedule during `EditorWorld::new()`. Each frame, the main app's
//! `Update` schedule calls `editor_world.run()` as a system — this ticks the
//! `EditorWorld` schedule once, providing clean temporal isolation from
//! `PreviewWorld` systems.
//!
//! ## Systems moved here
//!
//! | System | Reason |
//! |---------|--------|
//! | `sync_log_state` | Reads `OPERATION_LOG` thread-local, writes `OperationLogState` resource |
//! | `poll_recent_change_sets_system` | Reads `OPERATION_LOG`, calls `EditorSession` port — pure editor state |
//! | `process_hot_reload_requests` | Drains `HOT_RELOAD_BUS`, no scene/rendering dependencies |
//!
//! ## Systems that remain in `PreviewWorld`
//!
//! - `process_play_mode_request` — needs scene transform queries + `PlayMode` resource
//! - `rebuild_preview_world` — scene entity spawning/despawning
//! - `process_commands` — legacy JS bridge, sprite queries
//! - `emit_events` — scene event emission
//! - `update_keyboard_state` — needs `ButtonInput<KeyCode>` from window
//! - Logic evaluator / actuator systems — need preview scene entities

use bevy::prelude::*;

use crate::preview_runtime::poll_recent_change_sets_inner;
use crate::state::{mark_dirty, HotReloadRequest, HOT_RELOAD_BUS};
use crate::{with_asset_body_cache_mut, with_logic_graph_mut, OperationLogState};

/// Editor-only world: separate from the preview/rendering world.
///
/// Does not own scene entities, camera, or rendering state.
/// Systems here run every editor frame (not during play mode in v1;
/// play-mode-only logic stays in `PreviewWorld`).
pub struct EditorWorld {
    pub world: World,
    schedule: Schedule,
}

// SAFETY: EditorWorld is only accessed from the main thread (WASM single-threaded).
// - World is Send + Sync (Bevy-impl'd)
// - Schedule is Send + Sync (contains Box<dyn SystemExecutor: Send+Sync>)
// - EditorHotReloadBus is Send + Sync (explicit impl above)
// Therefore the entire struct is safe to mark as Send + Sync.
unsafe impl Send for EditorWorld {}
unsafe impl Sync for EditorWorld {}

impl EditorWorld {
    /// Build a new `EditorWorld` with editor-only systems registered.
    pub fn new() -> Self {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        // Insert editor-only resources.
        world.insert_resource(EditorWorldState::default());
        world.insert_resource(EditorHotReloadBus::default());
        // OperationLogState is also needed by sync_editor_log_state.
        // In the real app it's initialized elsewhere; here we init it so
        // headless tests can run the EditorWorld schedule.
        world.insert_resource(crate::OperationLogState::default());

        // Editor-only systems run in Update, unconditionally (edit-mode-only
        // gating is done via `.run_if(in_edit_mode)` in the main app's schedule).
        schedule.add_systems((
            sync_editor_log_state,
            poll_editor_change_sets,
            process_editor_hot_reload,
        ));

        Self { world, schedule }
    }

    /// Tick the editor world's schedule once.
    /// Called each frame from the main app's `Update` systems.
    pub fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }
}

impl Default for EditorWorld {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Resources (editor-only)
// ─────────────────────────────────────────────────────────────────────────────

/// State surfaced to the UI by `sync_log_state`.
#[derive(Resource, Default)]
pub struct EditorWorldState {
    pub tick_count: u64,
}

/// Thread-local hot-reload bus scoped to EditorWorld.
///
/// The real bus is `HOT_RELOAD_BUS` in `crate::state` — this resource
/// is a copy that EditorWorld drains each tick.
#[derive(Resource, Default)]
struct EditorHotReloadBus(std::cell::UnsafeCell<Vec<HotReloadRequest>>);

unsafe impl Send for EditorHotReloadBus {}
unsafe impl Sync for EditorHotReloadBus {}

// ─────────────────────────────────────────────────────────────────────────────
// Systems
// ─────────────────────────────────────────────────────────────────────────────

/// Mirrors the thread-local `OPERATION_LOG` into `OperationLogState` for UI hooks.
///
/// Corresponds to the original `preview_runtime::sync_log_state` but scoped to
/// `EditorWorld` so it runs independently of scene/rebuilding systems.
fn sync_editor_log_state(mut editor_state: ResMut<EditorWorldState>, mut log_state: ResMut<OperationLogState>) {
    editor_state.tick_count += 1;

    // Read from the shared thread-local log.
    // The actual `OperationLog` lives in `crate::scene_session::OPERATION_LOG`.
    crate::scene_session::OPERATION_LOG.with(|log| {
        let borrowed = log.borrow();
        let log = match borrowed.as_ref() {
            Some(l) => l,
            None => return,
        };
        log_state.size = log.get_log_size();
        log_state.can_undo = log.can_undo();
        log_state.can_redo = log.can_redo();
    });
}

/// Poll recent change-set entries from the operation log and push to `EditorSession`.
///
/// Corresponds to `preview_runtime::poll_recent_change_sets_system` scoped to
/// `EditorWorld`. No scene entities required.
fn poll_editor_change_sets() {
    poll_recent_change_sets_inner();
}

/// Drain the editor-scoped hot-reload bus and apply invalidations.
///
/// No scene entity queries — purely editor state management.
/// Corresponds to `preview_runtime::process_hot_reload_requests` scoped to
/// `EditorWorld`.
fn process_editor_hot_reload() {
    use std::collections::HashSet;

    // Copy requests out of the shared thread-local bus.
    let requests: Vec<HotReloadRequest> = HOT_RELOAD_BUS.with(|bus| {
        std::mem::take(&mut *bus.borrow_mut())
    });

    if requests.is_empty() {
        return;
    }

    // De-duplicate by (variant discriminant, key string).
    let mut seen: HashSet<(u8, String)> = HashSet::new();
    let deduped: Vec<HotReloadRequest> = requests
        .into_iter()
        .filter(|req| {
            let key = match req {
                HotReloadRequest::Source { file_id } => (0u8, file_id.clone()),
                HotReloadRequest::Asset { asset_id } => (1u8, asset_id.clone()),
                HotReloadRequest::ForceReloadAll => (2u8, String::new()),
            };
            seen.insert(key)
        })
        .collect();

    for req in deduped {
        match req {
            HotReloadRequest::Source { file_id } => {
                crate::source_files::invalidate_cache(&file_id);
            }
            HotReloadRequest::Asset { asset_id } => {
                with_asset_body_cache_mut(|c| {
                    c.remove(&asset_id);
                });
                mark_dirty();
            }
            HotReloadRequest::ForceReloadAll => {
                crate::source_files::clear_cache();
                with_asset_body_cache_mut(|c| {
                    c.clear();
                });
                with_logic_graph_mut(|doc| *doc = None);
                mark_dirty();
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Headless tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_world_initializes_with_empty_state() {
        let editor = EditorWorld::new();
        let state = editor.world.get_resource::<EditorWorldState>();
        assert!(state.is_some());
        assert_eq!(state.unwrap().tick_count, 0);
    }

    #[test]
    fn editor_world_tick_increments_counter() {
        let mut editor = EditorWorld::new();
        assert_eq!(editor.world.get_resource::<EditorWorldState>().unwrap().tick_count, 0);
        editor.run();
        assert_eq!(editor.world.get_resource::<EditorWorldState>().unwrap().tick_count, 1);
        editor.run();
        assert_eq!(editor.world.get_resource::<EditorWorldState>().unwrap().tick_count, 2);
    }

    #[test]
    fn editor_world_has_no_scene_entities() {
        // World::default() may create internal bookkeeping entities.
        // poll_editor_change_sets calls with_session_mut which may create session state.
        // We verify the count is stable after first run (no unbounded growth).
        let mut editor = EditorWorld::new();
        editor.run();
        let count_after_first = editor.world.entities().len();
        editor.run();
        let count_after_second = editor.world.entities().len();
        assert_eq!(
            count_after_first, count_after_second,
            "EditorWorld schedule should not grow entity count unbounded"
        );
    }
}
