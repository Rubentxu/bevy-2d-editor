//! H2.5 Block H — keyboard state module.
//!
//! Owns the Bevy `Resource InputState` (canonical keyboard state) and
//! the dual-write fallback thread_local `KEYBOARD_STATE_FALLBACK`.
//!
//! Production code (the `update_keyboard_state` Bevy system) writes to
//! BOTH the Resource and the FALLBACK each frame. The legacy
//! `KeyPressedEvaluator` reads from `KEYBOARD_STATE_FALLBACK` — same
//! thread_local path as before Block H, no `NodeEvaluator` trait
//! change required.
//!
//! **H2.5 Block H — OPEN until §S-H parity tests prove full coverage.**

use std::cell::RefCell;
use std::collections::HashSet;

use bevy::prelude::{ButtonInput, KeyCode, Res, ResMut, Resource};

/// Canonical keyboard state — Bevy Resource owned by the world.
///
/// Updated each frame by `update_keyboard_state`. Bevy auto-inserts
/// `InputState::default()` (empty set) when first accessed via
/// `Res<InputState>`.
#[derive(Resource, Default, Debug)]
pub struct InputState {
    /// Set of currently held key names (e.g. "KeyW", "Space",
    /// "ArrowUp"). Keys are Bevy `KeyCode` Debug representations.
    pub held: HashSet<String>,
}

impl InputState {
    /// Returns true if the given key name is currently held.
    pub fn contains(&self, key: &str) -> bool {
        self.held.contains(key)
    }

    /// Clears all held keys.
    pub fn clear(&mut self) {
        self.held.clear();
    }

    /// Inserts a key name into the held set.
    pub fn insert(&mut self, key: String) {
        self.held.insert(key);
    }
}

thread_local! {
    /// Dual-write fallback for `InputState` Resource.
    ///
    /// Production Bevy systems populate BOTH this thread_local AND
    /// the `InputState` Resource each frame (mirroring the
    /// Blocks A2/D/E/F/G pattern). Legacy consumers
    /// (`KeyPressedEvaluator`) read from this thread_local — no
    /// `NodeEvaluator` trait change required.
    ///
    /// H2.5 Block H — OPEN until §S-H parity tests prove the Resource
    /// path covers all consumers.
    pub static KEYBOARD_STATE_FALLBACK: RefCell<HashSet<String>> =
        RefCell::new(HashSet::new());
}

/// Bevy system: updates BOTH `Res<InputState>` and
/// `KEYBOARD_STATE_FALLBACK` from `ButtonInput<KeyCode>`.
///
/// H2.5 Block H: dual-write mirrors Blocks A2/D/E/F/G. The Resource
/// is the canonical owner (visible to Bevy tooling, queryable via
/// `World::resource::<InputState>()`); the thread_local is the legacy
/// consumer path.
///
/// Uses `Option<ResMut<InputState>>` so legacy tests that don't
/// initialize the Resource still work (the FALLBACK path is exercised
/// alone). New tests should `app.init_resource::<InputState>()` to
/// enable the dual-write.
pub fn update_keyboard_state(
    keys: Res<ButtonInput<KeyCode>>,
    input_state: Option<ResMut<InputState>>,
) {
    // Collect held key names from Bevy's button input.
    let held_keys: Vec<String> = keys.get_pressed().map(|k| format!("{:?}", k)).collect();

    // Resource first (canonical owner) — only if Resource is initialized.
    if let Some(mut input_state) = input_state {
        input_state.clear();
        for k in &held_keys {
            input_state.insert(k.clone());
        }
    }

    // FALLBACK (legacy consumer path) — always written.
    KEYBOARD_STATE_FALLBACK.with(|state| {
        let mut held = state.borrow_mut();
        held.clear();
        for k in &held_keys {
            held.insert(k.clone());
        }
    });
}
