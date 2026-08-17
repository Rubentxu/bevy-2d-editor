//! Characterization tests for `scene_session` — they document the
//! contract that the four invariants (active document, operation log,
//! dirty flag, registry) move together. The tests are written before
//! any refactor so they lock down the current behaviour.

use std::collections::BTreeMap;

use editor_bevy::StableId;
use editor_bevy::command::{Command, CommandEnvelope, CommandMetadata};
use editor_bevy::document::SceneDocument;
use editor_bevy::scene_session::{
    apply_command, clear_active_doc, clear_dirty, is_dirty, log_state_snapshot, mark_dirty, redo,
    replace_active_doc, replace_with_empty, snapshot_active_doc, swap_scene, undo, with_active_doc,
    with_log,
};

fn empty_doc(id: &str, name: &str) -> SceneDocument {
    SceneDocument {
        version: "0.1".to_string(),
        scene_id: id.to_string(),
        name: name.to_string(),
        entities: Vec::new(),
        instances: BTreeMap::new(),
    }
}

fn load_default_scene() {
    replace_active_doc(empty_doc("scene-1", "Default"));
    clear_dirty();
}

#[test]
fn replace_active_doc_marks_scene_dirty() {
    load_default_scene();
    clear_dirty();
    assert!(!is_dirty());
    replace_active_doc(empty_doc("scene-2", "Other"));
    assert!(is_dirty(), "replace_active_doc must mark the scene dirty");
}

#[test]
fn apply_command_records_inverse_and_marks_dirty() {
    load_default_scene();
    clear_dirty();

    let envelope = CommandEnvelope {
        command: Command::CreateEntity {
            id: StableId::new("ent-1"),
            name: "Alpha".to_string(),

            components: vec![],
        },
        metadata: CommandMetadata::now("test"),
    };

    let snap_before = snapshot_active_doc().expect("doc loaded");
    let size_before = log_state_snapshot().size;
    let result = apply_command(&envelope).expect("apply must succeed");
    assert_eq!(result.snapshot.entities.len(), 1);
    assert_eq!(result.snapshot.entities[0].id.as_str(), "ent-1");

    // The previous snapshot must NOT yet contain the new entity —
    // confirms that the closure ran with the mut borrow and we did
    // not commit the change after taking the snapshot.
    assert_eq!(snap_before.entities.len(), 0);
    assert_eq!(log_state_snapshot().size, size_before + 1);
    assert!(is_dirty());
}

#[test]
fn undo_restores_previous_state() {
    load_default_scene();

    let envelope = CommandEnvelope {
        command: Command::CreateEntity {
            id: StableId::new("ent-1"),
            name: "Alpha".to_string(),

            components: vec![],
        },
        metadata: CommandMetadata::now("test"),
    };
    apply_command(&envelope).expect("apply");

    let snap_after = snapshot_active_doc().expect("doc");
    assert_eq!(snap_after.entities.len(), 1);

    let undone = undo().expect("undo");
    assert_eq!(undone.entities.len(), 0);
    // log cursor moves back, can_redo is true
    let state = log_state_snapshot();
    assert!(!state.can_undo);
    assert!(state.can_redo);
}

#[test]
fn redo_replays_command() {
    load_default_scene();
    let envelope = CommandEnvelope {
        command: Command::CreateEntity {
            id: StableId::new("ent-1"),
            name: "Alpha".to_string(),

            components: vec![],
        },
        metadata: CommandMetadata::now("test"),
    };
    apply_command(&envelope).expect("apply");
    undo().expect("undo");

    let replayed = redo().expect("redo");
    assert_eq!(replayed.entities.len(), 1);
    let state = log_state_snapshot();
    assert!(state.can_undo);
    assert!(!state.can_redo);
}

#[test]
fn undo_without_active_doc_returns_none() {
    clear_active_doc();
    let state = log_state_snapshot();
    assert_eq!(state.size, 0);
    assert!(!state.can_undo);
    assert!(!state.can_redo);
    assert!(undo().is_none());
}

#[test]
fn swap_scene_moves_active_doc_between_scenes() {
    load_default_scene();
    let first = with_active_doc(|d| d.scene_id.clone()).expect("doc");
    assert_eq!(first, "scene-1");

    // Pre-populate the registry with scene-2 so the swap has a target.
    // swap_scene moves scene-1 into the registry, then loads scene-2.
    let mut doc2 = empty_doc("scene-2", "Other");
    doc2.entities.push(editor_bevy::document::Entity {
        id: editor_bevy::StableId::new("ent-other"),
        local_id: editor_bevy::document::LocalId::new("ent-other"),
        name: "Other".to_string(),
        parent: None,
        components: vec![],
    });
    {
        use editor_bevy::scene_state::with_registry_mut;
        with_registry_mut(|r| {
            r.store_to(
                "scene-2",
                doc2.clone(),
                editor_bevy::operation_log::OperationLog::new_const(),
            );
        });
    }

    // Move scene-1 to the registry slot and load scene-2.
    swap_scene("scene-1", "scene-2");
    let second = with_active_doc(|d| d.scene_id.clone()).expect("doc");
    assert_eq!(second, "scene-2");
    // The dirty flag must be set after a swap so the next frame rebuilds.
    assert!(is_dirty());
}

#[test]
fn replace_with_empty_clears_log_and_marks_dirty() {
    load_default_scene();
    let envelope = CommandEnvelope {
        command: Command::CreateEntity {
            id: StableId::new("ent-1"),
            name: "Alpha".to_string(),

            components: vec![],
        },
        metadata: CommandMetadata::now("test"),
    };
    apply_command(&envelope).expect("apply");
    assert_eq!(log_state_snapshot().size, 1);

    replace_with_empty("scene-1");
    let state = log_state_snapshot();
    assert_eq!(state.size, 0);
    assert!(is_dirty());
}

#[test]
fn mark_dirty_and_clear_dirty_round_trip() {
    clear_dirty();
    assert!(!is_dirty());
    mark_dirty();
    assert!(is_dirty());
    clear_dirty();
    assert!(!is_dirty());
}

#[test]
fn with_log_borrows_immutably() {
    load_default_scene();
    let size = with_log(|log| log.get_log_size());
    assert_eq!(size, 0);
}
