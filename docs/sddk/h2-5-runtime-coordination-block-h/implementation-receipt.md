# H2.5 Block H — Implementation Receipt

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-h`
**Path**: A-lite (explore → spec → design → build → verify → release → archive)
**Tag**: `v0.108.8` (to be released)
**Commit**: TBD (pending release)
**Date**: 2026-09-08

## Scope

Migrate the legacy `KEYBOARD_STATE` thread_local
(`RefCell<HashSet<String>>`) to a Bevy `Resource InputState` as the
canonical owner of keyboard state. The `KEYBOARD_STATE_FALLBACK`
thread_local is kept as the dual-write fallback for the legacy
`KeyPressedEvaluator` (no `NodeEvaluator` trait change required).

This is the **9th and final** H2.5 cell.

## Work units landed

| WU      | Files                                                                  | Lines |
|---------|------------------------------------------------------------------------|-------|
| WU-H-1  | `crates/editor-bevy/src/keyboard_state.rs` (new)                       | +91 / -0 |
| WU-H-2a | `crates/editor-bevy/src/lib.rs` (`mod keyboard_state;`)                 | +1 / -0 |
| WU-H-2b | `crates/editor-bevy/src/logic_evaluator.rs` (remove inline thread_local + update_keyboard_state; KeyPressedEvaluator reads FALLBACK) | +5 / -28 |
| WU-H-2c | `crates/editor-bevy/src/preview_runtime.rs` (system registration path) | +1 / -1 |
| WU-H-2d | `crates/editor-bevy/tests/play_mode.rs` (use path update, 4 sites)    | +5 / -5 |
| WU-H-3  | `crates/editor-bevy/tests/keyboard_input_state_parity.rs` (new, 5 tests) | +112 / -0 |
| WU-H-4  | `docs/architecture/state-ownership-matrix.md` (1 retired + progress to 9/9) | +5 / -5 |
| WU-H-5  | `tools/archcheck-globals/globals-inventory.yaml` (KEYBOARD_STATE → KEYBOARD_STATE_FALLBACK OPEN) | +4 / -4 |
| WU-H-6  | `Cargo.toml` (0.108.7 → 0.108.8)                                       | +1 / -1 |

**Net: +225 / -44** lines (excluding block-H artifacts).

## Code changes

### WU-H-1: New `keyboard_state.rs` module

```rust
use std::cell::RefCell;
use std::collections::HashSet;
use bevy::prelude::{ButtonInput, KeyCode, Res, ResMut, Resource};

#[derive(Resource, Default, Debug)]
pub struct InputState {
    pub held: HashSet<String>,
}

impl InputState {
    pub fn contains(&self, key: &str) -> bool { self.held.contains(key) }
    pub fn clear(&mut self) { self.held.clear(); }
    pub fn insert(&mut self, key: String) { self.held.insert(key); }
}

thread_local! {
    pub static KEYBOARD_STATE_FALLBACK: RefCell<HashSet<String>> =
        RefCell::new(HashSet::new());
}

pub fn update_keyboard_state(
    keys: Res<ButtonInput<KeyCode>>,
    input_state: Option<ResMut<InputState>>,  // backward-compat with tests
) {
    let held_keys: Vec<String> = keys.get_pressed().map(|k| format!("{:?}", k)).collect();
    if let Some(mut s) = input_state {  // Resource (canonical)
        s.clear();
        for k in &held_keys { s.insert(k.clone()); }
    }
    KEYBOARD_STATE_FALLBACK.with(|state| {  // FALLBACK (legacy)
        let mut held = state.borrow_mut();
        held.clear();
        for k in &held_keys { held.insert(k.clone()); }
    });
}
```

### WU-H-2: Consumer change (zero blast radius)

`KeyPressedEvaluator::evaluate` reads from
`crate::keyboard_state::KEYBOARD_STATE_FALLBACK` (same thread_local
path, just renamed). No `NodeEvaluator` trait change.

## Verification status

- `cargo check --workspace --locked` ✅
- `cargo check -p editor-model --target wasm32-unknown-unknown --locked` ✅
- `cargo test -p editor-bevy --test play_mode --locked` ✅ (4/4 — existing, FALLBACK path)
- `cargo test -p editor-bevy --test keyboard_input_state_parity --locked` ✅ (5/5 — new, Resource path)
- `archcheck-globals` ✅ (29 entries = 29 declarations)

## H2.5 progress after Block H

| Cell                        | Status                |
|-----------------------------|-----------------------|
| `ACTUATOR_OUTPUT_BUS`       | RETIRED (Block A2)    |
| `PREVIEW_METRICS`           | RETIRED (Block E)     |
| `PREVIEW_MAPPING`           | RETIRED (Block E)     |
| `PREVIEW_PROVENANCE`        | RETIRED (Block E)     |
| `COMMAND_BUS`               | RETIRED (Block F)     |
| `EVENT_BUS`                 | RETIRED (Block F)     |
| `HOT_RELOAD_BUS`            | RETIRED (Block G)     |
| `PLAY_MODE_REQUEST`         | RETIRED (Block G)     |
| `KEYBOARD_STATE`            | RETIRED (Block H)     |

**9 of 9 H2.5 cells retired — H2.5 COMPLETE.**

## Backward-compat note: `Option<ResMut<InputState>>`

The `update_keyboard_state` Bevy system declares
`input_state: Option<ResMut<InputState>>` (instead of bare
`ResMut<InputState>`). This is a defensive choice that:

1. Lets legacy tests that don't initialize the Resource continue
   to work (they exercise the FALLBACK path alone).
2. Avoids Bevy panics when `app.update()` runs without
   `app.init_resource::<InputState>()`.

New consumers should `app.init_resource::<InputState>()` to enable
the canonical dual-write path.

## Forward work after H (H2.5 COMPLETE)

- Update `BLOCK_*` patterns to verify the FALLBACK thread_locals
  can close (when parity tests prove they're unnecessary).
- Move on to next H2.x or H3.x work (per `MASTER_ROADMAP.md`).
