# H2.5 Block H — Explore Report

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-h`
**Path**: A-lite (architectural — Bevy Resource target, not EditorSession field)
**Date**: 2026-09-08

## Scope

Migrate the legacy `KEYBOARD_STATE` thread_local (a
`RefCell<HashSet<String>>`) to a Bevy `Resource InputState` so the
keyboard state is owned by the Bevy world (lifetime-bound to input
frames, not to a thread-local). This is the **last** H2.5 cell.

## Key facts

### Current state

`crates/editor-bevy/src/logic_evaluator.rs:1046`:

```rust
thread_local! {
    pub static KEYBOARD_STATE: RefCell<std::collections::HashSet<String>> =
        RefCell::new(std::collections::HashSet::new());
}
```

### Producers

`update_keyboard_state` (Bevy system at `logic_evaluator.rs:1062`)
runs in Update schedule (registered in `preview_runtime.rs:226`,
`.run_if(in_play_mode).before(logic_dispatch::dispatch_dirty_bindings)`):

```rust
pub fn update_keyboard_state(keys: Res<ButtonInput<KeyCode>>) {
    KEYBOARD_STATE.with(|state| {
        let mut held = state.borrow_mut();
        held.clear();
        for key in keys.get_pressed() {
            held.insert(format!("{:?}", key));
        }
    });
}
```

### Consumers

`KeyPressedEvaluator::evaluate` (logic node evaluator at
`logic_evaluator.rs:888`):

```rust
let is_pressed = KEYBOARD_STATE.with(|state| state.borrow().contains(key));
```

### Tests

`crates/editor-bevy/tests/play_mode.rs` (165 lines, 4 tests):
- `test_update_keyboard_state_populates_from_button_input`
- `test_update_keyboard_state_clears_released_keys`
- `test_update_keyboard_state_empty_when_no_keys_pressed`
- `test_keyboard_state_not_updated_in_edit_mode`

All 4 tests use the thread_local directly (`KEYBOARD_STATE.with(...)`).
The migration must keep these tests passing — either via the FALLBACK
thread_local (dual-write pattern) or by updating tests to query the
new Bevy Resource directly.

## Why this is A-lite, not A-min

Unlike Blocks A2/D/E/F/G (which target `EditorSession.runtime.*` or
extend `editor_model::runtime::*`), Block H targets a **Bevy
Resource**. The migration is bounded but architectural:

1. **New type**: `InputState` Bevy Resource (lives in
   `editor-bevy` crate — same crate as the Bevy systems). The
   matrix says target = `Bevy Resource InputState`, no editor_model
   involvement needed (the Resource is consumer-side Bevy-specific,
   unlike the `editor_model::runtime::*` types which are pure data
   shared with editor-application).
2. **Reader needs access to the Bevy world**: `KeyPressedEvaluator`
   currently takes `&self, node: &LogicNode, inputs: &[PortValue]`
   — it has no `Res<InputState>`. To read the Resource, the
   evaluator needs `Res<InputState>` plumbed through the logic
   dispatch path, OR the evaluator reads via a `World::resource` /
   `FromWorld` helper, OR we add a system that mirrors Resource →
   thread_local (legacy FALLBACK only).
3. **Dual-write fallback**: keep `KEYBOARD_STATE` (renamed to
   `KEYBOARD_STATE_FALLBACK`) as the legacy thread_local so the 4
   existing tests pass without modification.

### Two viable sub-paths

**Sub-path H.1: Pure Resource, no thread_local**
- Drop `KEYBOARD_STATE` thread_local entirely.
- Add `pub struct InputState(pub HashSet<String>)` + `#[derive(Resource)]`.
- `update_keyboard_state` writes to the Resource.
- `KeyPressedEvaluator` takes `Res<InputState>` (signature change
  in `NodeEvaluator` trait — propagates to every evaluator).
- All 4 existing tests must be updated to use `App` + `Res<InputState>`
  instead of `KEYBOARD_STATE.with(...)`.
- Risk: large blast radius (`NodeEvaluator` trait change). Likely A-full.

**Sub-path H.2: Resource as canonical, thread_local as FALLBACK (dual-write, Block G style)**
- Rename `KEYBOARD_STATE` → `KEYBOARD_STATE_FALLBACK`.
- Add `InputState` Bevy Resource as canonical owner.
- `update_keyboard_state` writes to BOTH (session-first dual-write
  pattern — except "session" here = Bevy Resource).
- `KeyPressedEvaluator` reads from Resource (no signature change if we
  pass `Res<InputState>` via a thread-local handle, or via
  `from_world`). For minimal blast radius, use a small helper:
  ```rust
  fn keyboard_state_contains(key: &str) -> bool {
      // Resource first (production), FALLBACK otherwise (tests).
      INPUT_STATE_HANDLE.with(|h| {
          if let Some(handle) = *h.borrow() {
              // deref handle... but Res can't be cloned cheaply
          }
          KEYBOARD_STATE_FALLBACK.with(|s| s.borrow().contains(key))
      })
  }
  ```
  This is awkward because `Res<T>` is not `Clone`/`Send`. The
  cleanest is to have `update_keyboard_state` populate both the
  Resource and the FALLBACK thread_local every frame (dual-write),
  and `KeyPressedEvaluator` continues to read from the thread_local
  (which is now `KEYBOARD_STATE_FALLBACK`).
- The Resource becomes the "public" canonical owner (visible to
  Bevy tooling, queryable via `World::resource::<InputState>()`) but
  the thread_local is the legacy consumer path.

### Decision: Sub-path H.2 (Resource + FALLBACK dual-write)

Rationale:
1. **Zero blast radius on `NodeEvaluator` trait**: keeps the
   existing 4 tests working unmodified, keeps every other evaluator
   (`GateEvaluator`, etc.) working.
2. **The Resource is the new canonical owner** for any future
   Bevy-tooling integration (e.g. bevy_inspector_egui, system
   observers).
3. **Mirrors Blocks A2/D/E/F/G**: same dual-write pattern, same
   `_FALLBACK` rename, same ratchet discipline.

The "canonical owner = Bevy Resource" answer is what the matrix
already documents. The FALLBACK thread_local is a migration aid, not
the permanent design.

## Path: A-lite (1 apply phase + architectural decision in spec)

**Rationale**: the type extension (`InputState` new Resource) is
bounded but architecturally meaningful. A-min would imply a pure
rename with no new types — not the case here. A-full would imply
multiple architectural changes — not the case either. A-lite is
the right level.

## Block H work plan (5 WUs)

| WU      | Files                                                                   | Lines |
|---------|-------------------------------------------------------------------------|-------|
| WU-H-1  | `crates/editor-bevy/src/keyboard_state.rs` (new) — define `InputState` Bevy Resource + FALLBACK rename | +60 / -0 |
| WU-H-2  | `crates/editor-bevy/src/logic_evaluator.rs` — update_keyboard_state dual-write (Resource + FALLBACK); KeyPressedEvaluator reads from FALLBACK | +15 / -5 |
| WU-H-3  | `crates/editor-bevy/tests/keyboard_input_state_parity.rs` (new) — 4-6 tests covering Resource path | +120 / -0 |
| WU-H-4  | `docs/architecture/state-ownership-matrix.md` — KEYBOARD_STATE RETIRED, progress 9 of 9 retired | +5 / -5 |
| WU-H-5  | `tools/archcheck-globals/globals-inventory.yaml` — KEYBOARD_STATE → KEYBOARD_STATE_FALLBACK (OPEN) | +4 / -4 |
| WU-H-6  | `Cargo.toml` — 0.108.7 → 0.108.8                                       | +1 / -1 |

## Compatibility

- All 4 existing `play_mode.rs` tests pass unmodified (FALLBACK path).
- WASM exports unchanged.
- `update_keyboard_state` Bevy system signature unchanged.
- `KeyPressedEvaluator` continues to read from the (renamed)
  thread_local — no trait change.
- New Bevy Resource `InputState` is added for new consumers to read
  via `Res<InputState>` in Bevy systems.

## Forward work (after H — all H2.5 cells retired)

- Update `BLOCK_*` patterns to verify the FALLBACK thread_locals
  close (when parity tests prove they're unnecessary).
- Move on to next H2.x or H3.x work (per `MASTER_ROADMAP.md`).
