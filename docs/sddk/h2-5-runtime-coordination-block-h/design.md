# H2.5 Block H — Design

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-h`
**Path**: A-lite
**Date**: 2026-09-08

## Design overview

Block H migrates `KEYBOARD_STATE` (a `RefCell<HashSet<String>>`
thread_local) to a Bevy `Resource InputState`. The architectural
choice is **dual-write** (canonical = Bevy Resource, fallback =
renamed thread_local) — same pattern as Blocks A2/D/E/F/G.

### Type architecture

```
                   ┌──────────────────────────────────────┐
                   │  Bevy World                          │
                   │  ┌──────────────────────────────┐    │
                   │  │  Res<InputState>             │◄────┼── canonical owner
                   │  │  held: HashSet<String>       │    │   (new)
                   │  └──────────────────────────────┘    │
                   └────────▲────────────▲─────────────────┘
                            │            │
        update_keyboard_state Bevy system writes both
                            │            │
                   ┌────────┴────────────┴─────────────────┐
                   │  Thread-local                         │
                   │  KEYBOARD_STATE_FALLBACK              │◄── legacy fallback
                   │  RefCell<HashSet<String>>             │    (renamed)
                   └────────▲──────────────────────────────┘
                            │
                            │ KeyPressedEvaluator reads
                            │ (no NodeEvaluator trait change)
```

### Module placement

| File | Role |
|------|------|
| `crates/editor-bevy/src/keyboard_state.rs` (new) | Owns `InputState` Resource + `KEYBOARD_STATE_FALLBACK` thread_local + dual-write helper |
| `crates/editor-bevy/src/logic_evaluator.rs` | `update_keyboard_state` becomes a thin wrapper around `keyboard_state::update`; `KeyPressedEvaluator` reads `keyboard_state::KEYBOARD_STATE_FALLBACK` |

### `keyboard_state.rs` API

```rust
use std::cell::RefCell;
use std::collections::HashSet;
use bevy::prelude::{ButtonInput, KeyCode, Resource, World};

/// Canonical keyboard state — Bevy Resource.
#[derive(Resource, Default, Debug)]
pub struct InputState {
    pub held: HashSet<String>,
}

impl InputState {
    pub fn contains(&self, key: &str) -> bool {
        self.held.contains(key)
    }
}

/// Dual-write fallback thread_local (legacy consumer path).
thread_local! {
    pub static KEYBOARD_STATE_FALLBACK: RefCell<HashSet<String>> =
        RefCell::new(HashSet::new());
}

/// Bevy system: updates BOTH `Res<InputState>` and
/// `KEYBOARD_STATE_FALLBACK` from `ButtonInput<KeyCode>`.
///
/// H2.5 Block H: dual-write mirrors Blocks A2/D/E/F/G.
pub fn update_keyboard_state(
    keys: Res<ButtonInput<KeyCode>>,
    mut input_state: ResMut<InputState>,
) {
    let held_keys: Vec<String> = keys
        .get_pressed()
        .map(|k| format!("{:?}", k))
        .collect();

    // Resource first (canonical).
    input_state.held.clear();
    for k in &held_keys {
        input_state.held.insert(k.clone());
    }

    // FALLBACK (legacy consumer path).
    KEYBOARD_STATE_FALLBACK.with(|state| {
        let mut held = state.borrow_mut();
        held.clear();
        for k in &held_keys {
            held.insert(k.clone());
        }
    });
}
```

### `KeyPressedEvaluator` change

Before:
```rust
let is_pressed = KEYBOARD_STATE.with(|state| state.borrow().contains(key));
```

After:
```rust
let is_pressed = crate::keyboard_state::KEYBOARD_STATE_FALLBACK
    .with(|state| state.borrow().contains(key));
```

Same semantics, no NodeEvaluator trait change.

## Files & line estimates

| File | Lines | Net |
|------|-------|-----|
| `crates/editor-bevy/src/keyboard_state.rs` (new) | +60 | +60 |
| `crates/editor-bevy/src/logic_evaluator.rs` | +5 / -10 | -5 (move update + reader to keyboard_state) |
| `crates/editor-bevy/src/lib.rs` | +1 / -1 | 0 |
| `crates/editor-bevy/tests/keyboard_input_state_parity.rs` (new) | +130 | +130 |
| `crates/editor-bevy/tests/play_mode.rs` | +1 / -1 | 0 (use path update) |
| `docs/architecture/state-ownership-matrix.md` | +5 / -5 | 0 |
| `tools/archcheck-globals/globals-inventory.yaml` | +4 / -4 | 0 |
| `Cargo.toml` | +1 / -1 | 0 |

**Net: +206 / -22** lines (excluding block-H artifacts).

## Compatibility plan

1. **Existing tests** (`crates/editor-bevy/tests/play_mode.rs`):
   - Update `use editor_bevy::logic_evaluator::{KEYBOARD_STATE, ...}`
     to `use editor_bevy::keyboard_state::KEYBOARD_STATE_FALLBACK`.
   - All 4 tests pass without logic changes.

2. **`update_keyboard_state` system signature change**:
   - Adds `mut input_state: ResMut<InputState>` parameter.
   - Bevy `Res<T>` system parameter rules: `Res` + `ResMut` of
     different types is allowed (no conflict).
   - System registration in `preview_runtime.rs:226` unchanged.

3. **New `InputState` Resource initialization**:
   - Bevy auto-inserts `InputState::default()` when first accessed
     via `Res<InputState>` (Resource derive Default + auto-init).
   - No explicit `App::init_resource::<InputState>()` required.

4. **`play_mode.rs` test environment**:
   - Tests use `App::new().add_systems(Update, ...)` then run.
   - `update_keyboard_state` now requires `ResMut<InputState>` —
     Bevy auto-inserts default. No test changes beyond `use` path.

## Risks

1. **Bevy Resource auto-init**: if a test creates an App without
   `init_resource::<InputState>()`, Bevy will panic with "missing
   resource". Mitigation: `update_keyboard_state` declares
   `ResMut<InputState>` which triggers auto-init via `FromWorld`.
   Verify in test runs.
2. **Concurrent Bevy systems**: `update_keyboard_state` runs in
   Update, no other system writes `InputState`. No conflicts.
3. **`play_mode.rs` test signatures**: tests use Bevy's test
   helpers (`minimal_app` or `App::new()`). If `ResMut<InputState>`
   requires explicit init, the test setup may need
   `app.init_resource::<InputState>()`.

## Implementation order

1. Create `crates/editor-bevy/src/keyboard_state.rs` (the new
   module).
2. Add `mod keyboard_state;` to `lib.rs`.
3. Update `logic_evaluator.rs`:
   - Remove the inline `thread_local!` block.
   - Remove the inline `update_keyboard_state` (moved to
     `keyboard_state.rs`).
   - Update `KeyPressedEvaluator::evaluate` to read from
     `keyboard_state::KEYBOARD_STATE_FALLBACK`.
4. Update `play_mode.rs` test use path.
5. Verify `cargo check` + `cargo test --test play_mode`.
6. Create `keyboard_input_state_parity.rs` (new tests).
7. Update matrix + inventory + Cargo.toml.

## Design rationale summary

- **Why dual-write**: zero blast radius on `NodeEvaluator` trait +
  100% backward compat with the 4 existing tests + same pattern as
  Blocks A2/D/E/F/G.
- **Why Bevy Resource (not EditorSession)**: matrix already documents
  this as the target; the lifecycle is "lifetime-bound to the Bevy
  world (input frames)" per `KEYBOARD_STATE`'s row in the matrix.
  EditorSession would be the wrong layer (logic dispatch happens in
  Bevy context anyway).
- **Why rename to `KEYBOARD_STATE_FALLBACK`**: same naming convention
  as Blocks A2/D/E/F/G; signals dual-write intent in the inventory.

## Out-of-scope design choices

- **`FromWorld` impl**: not needed. Bevy auto-inserts
  `InputState::default()`.
- **Custom system sets**: not needed. `update_keyboard_state` runs
  in Update, before logic dispatch — same as before.
- **Multiple keys per frame**: Bevy `ButtonInput<KeyCode>::get_pressed()`
  returns all currently held keys. No batching needed.
- **`.run_if(in_play_mode)`**: same as before (registered in
  `preview_runtime.rs:226`). Not changed by Block H.
