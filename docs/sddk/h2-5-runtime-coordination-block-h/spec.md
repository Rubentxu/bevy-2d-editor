# H2.5 Block H — Spec

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-h`
**Path**: A-lite (explore → spec → build → verify → release → archive)
**Tag**: `v0.108.8` (to be released)
**Date**: 2026-09-08

## Goals

1. Add `InputState` Bevy `Resource` (canonical owner of keyboard
   state).
2. Rename `KEYBOARD_STATE` thread_local → `KEYBOARD_STATE_FALLBACK`
   (dual-write fallback for legacy tests + non-Bevy entrypoints).
3. `update_keyboard_state` Bevy system writes to BOTH the Resource
   and the FALLBACK (dual-write, mirroring Blocks A2/D/E/F/G).
4. `KeyPressedEvaluator` continues to read from
   `KEYBOARD_STATE_FALLBACK` (zero blast radius on
   `NodeEvaluator` trait).
5. Pass all 4 existing `play_mode.rs` tests without modification.

## Scenarios

### §S-H-1: Bevy Resource `InputState` exists with empty initial state

**Given** the Bevy `App` is initialized
**When** `world.resource::<InputState>()` is queried
**Then** the resource contains an empty `HashSet<String>`
(`InputState::default()` semantics)

### §S-H-2: `update_keyboard_state` populates both Resource and FALLBACK

**Given** Bevy `ButtonInput<KeyCode>` reports KeyW, Space, ArrowUp
pressed (and ArrowDown released)
**When** `update_keyboard_state` runs
**Then** the `InputState` Resource contains {"KeyW", "Space",
"ArrowUp"}
**And** `KEYBOARD_STATE_FALLBACK` contains the same set

### §S-H-3: `KeyPressedEvaluator` reads from FALLBACK

**Given** `KEYBOARD_STATE_FALLBACK` contains {"KeyW"}
**When** a `LogicNode` with `key="KeyW"` is evaluated via
`KeyPressedEvaluator`
**Then** it emits `PortValue::Action("pressed")`

### §S-H-4: `KeyPressedEvaluator` emits empty when key not held

**Given** `KEYBOARD_STATE_FALLBACK` is empty
**When** a `LogicNode` with `key="KeyW"` is evaluated
**Then** it emits `PortValue::Action("")`

### §S-H-5: 4 existing `play_mode.rs` tests pass unmodified

**Given** the 4 existing tests query `KEYBOARD_STATE` directly (via
`KEYBOARD_STATE.with(...)`)
**When** the tests run
**Then** they pass without code modification (because they exercise
the renamed `KEYBOARD_STATE_FALLBACK` — same path as the legacy
behaviour, but the read happens on the renamed symbol)

### §S-H-6: archcheck-globals ratchet

**Given** the inventory is updated with `KEYBOARD_STATE_FALLBACK`
entry (OPEN)
**When** `npm run check` runs in `tools/archcheck-globals`
**Then** 31 entries = 31 declarations (29 from Block G + 2 = 31)

Wait — actually: Block G left 29 entries (net zero). Block H adds 1
new entry (`KEYBOARD_STATE_FALLBACK`) and removes 1 old
(`KEYBOARD_STATE`). Net = 29. Let me correct the spec:

### §S-H-6 (corrected): archcheck-globals ratchet stays at 29

**Given** the inventory is updated with `KEYBOARD_STATE_FALLBACK`
(added) and `KEYBOARD_STATE` (removed)
**When** `npm run check` runs in `tools/archcheck-globals`
**Then** 29 entries = 29 declarations (no net change)

## Out-of-scope

- Bevy system signature changes (dual-write keeps the existing
  `update_keyboard_state(keys: Res<ButtonInput<KeyCode>>)` signature).
- `NodeEvaluator` trait change (consumer reads from FALLBACK).
- Refactor `play_mode.rs` tests (they exercise the FALLBACK path
  after rename).

## Type semantics

### `editor_bevy::keyboard_state::InputState` (new)

```rust
use std::collections::HashSet;
use bevy::prelude::Resource;

/// Canonical keyboard state — Bevy Resource owned by the world.
/// Updated by `update_keyboard_state` each frame.
///
/// H2.5 Block H: the Bevy Resource is the new canonical owner of
/// keyboard state. The legacy `KEYBOARD_STATE_FALLBACK`
/// thread_local is dual-written for backward compatibility with
/// the existing `KeyPressedEvaluator` (which reads from
/// thread_local ��� no NodeEvaluator trait change).
#[derive(Resource, Default, Debug)]
pub struct InputState {
    /// Set of currently held key names (e.g. "KeyW", "Space",
    /// "ArrowUp"). Keys are Bevy `KeyCode` Debug representations.
    pub held: HashSet<String>,
}

impl InputState {
    pub fn contains(&self, key: &str) -> bool {
        self.held.contains(key)
    }
}
```

### `editor_bevy::keyboard_state::KEYBOARD_STATE_FALLBACK` (renamed)

```rust
thread_local! {
    /// Dual-write fallback for `InputState` Resource.
    ///
    /// Production Bevy systems populate BOTH this thread_local AND
    /// the `InputState` Resource each frame (mirroring the
    /// Blocks A2/D/E/F/G pattern). Legacy consumers
    /// (`KeyPressedEvaluator`) read from this thread_local — no
    /// NodeEvaluator trait change required.
    ///
    /// H2.5 Block H — OPEN until parity tests prove the Resource
    /// path covers all consumers.
    pub static KEYBOARD_STATE_FALLBACK: RefCell<std::collections::HashSet<String>> =
        RefCell::new(std::collections::HashSet::new());
}
```

## Files to be modified

| File | Change |
|------|--------|
| `crates/editor-bevy/src/keyboard_state.rs` (new) | Define `InputState` Resource + `KEYBOARD_STATE_FALLBACK` thread_local |
| `crates/editor-bevy/src/lib.rs` | `mod keyboard_state;` + re-export `KEYBOARD_STATE_FALLBACK` (drop `KEYBOARD_STATE`) |
| `crates/editor-bevy/src/logic_evaluator.rs` | Remove the inline thread_local + `update_keyboard_state`; use `keyboard_state::KEYBOARD_STATE_FALLBACK`; `KeyPressedEvaluator` reads from `keyboard_state::KEYBOARD_STATE_FALLBACK` |
| `crates/editor-bevy/src/state.rs` | Update re-export if needed |
| `crates/editor-bevy/tests/keyboard_input_state_parity.rs` (new) | 4-6 parity tests covering the Resource path (§S-H-1, §S-H-2) |
| `crates/editor-bevy/tests/support/mod.rs` | Optional: add `InputState` field to `FakeSession` for parity tests (Bevy `World::resource` access in tests) |
| `docs/architecture/state-ownership-matrix.md` | `KEYBOARD_STATE` row → RETIRED (Block H); progress note: 8 of 9 → **9 of 9 retired (H2.5 COMPLETE)** |
| `tools/archcheck-globals/globals-inventory.yaml` | `KEYBOARD_STATE` → `KEYBOARD_STATE_FALLBACK` (OPEN) |
| `Cargo.toml` | 0.108.7 → 0.108.8 |

## Compatibility

- `KEYBOARD_STATE` rename to `KEYBOARD_STATE_FALLBACK` — same
  type, same path, same tests. The 4 existing `play_mode.rs`
  tests need to update their `use` statement from
  `KEYBOARD_STATE` to `KEYBOARD_STATE_FALLBACK` (one-line
  change, not modifying test logic).
- New `InputState` Resource is additive — no existing system reads
  from it.
- `update_keyboard_state` now dual-writes (Resource + FALLBACK).
  Existing Bevy schedule unchanged.
- WASM exports unchanged.
