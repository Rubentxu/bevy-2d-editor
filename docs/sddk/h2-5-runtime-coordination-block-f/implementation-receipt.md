# H2.5 Block F — Implementation Receipt

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-f`
**Path**: A-min
**Tag**: `v0.108.6` (to be released)
**Commit**: TBD (pending release)
**Date**: 2026-09-08

## Scope

Migrate the legacy `COMMAND_BUS` / `EVENT_BUS` thread_locals to
`EditorSession` via the established dual-write fallback pattern
(Block A2 ActuatorBus, Block E preview inspector). Two thread_locals
retired; production path goes through the session, fallback
thread_locals renamed (`*_FALLBACK`) for legacy tests. Also removes
the duplicated local `LinearBus` struct (~70 lines) and now uses
`editor_model::runtime::LinearBus`.

## Work units landed

| WU     | Files                                                            | Lines |
|--------|------------------------------------------------------------------|-------|
| WU-F-1 | `crates/editor-bevy/src/lib.rs` (rename + dual-write doc)        | +12 / -8 |
| WU-F-2 | `crates/editor-bevy/src/lib.rs` (delete local LinearBus + BUS_CAPACITY) | +1 / -70 |
| WU-F-3 | `crates/editor-bevy/src/preview_runtime.rs` (callers + .expect()) | +5 / -5 |
| WU-F-4 | `crates/editor-bevy/tests/runtime_buses_parity.rs` (new, 5 tests) | +136 / -0 |
| WU-F-5 | `docs/architecture/state-ownership-matrix.md` § H2.5 (2 retired + progress) | +5 / -5 |
| WU-F-6 | `tools/archcheck-globals/globals-inventory.yaml` (2 retired → 2 FALLBACK OPEN) | +4 / -4 |
| WU-F-7 | `Cargo.toml` (0.108.5 → 0.108.6)                                 | +1 / -1 |

## Code changes

### WU-F-1: Thread-local rename with dual-write docs

`crates/editor-bevy/src/lib.rs`:

```rust
// Dual-write fallback: production code writes through the session
// (`EditorSession.runtime.command_bus` via `with_session_mut`).
// This thread_local only exists so legacy tests + non-session
// entrypoints (preview_runtime WASM exports) keep working while the
// migration lands. OPEN until §S-F parity tests prove full coverage.
pub(crate) static COMMAND_BUS_FALLBACK: RefCell<Option<editor_model::runtime::LinearBus>> =
    RefCell::new(None);

pub(crate) static EVENT_BUS_FALLBACK: RefCell<Option<editor_model::runtime::LinearBus>> =
    RefCell::new(None);
```

### WU-F-2: Delete duplicated local LinearBus + BUS_CAPACITY

Removed ~70 lines. Now `editor_model::runtime::LinearBus` is the
single source of truth (already used by `EditorSession.runtime`).

### WU-F-3: Update preview_runtime.rs callers

All `crate::COMMAND_BUS.with` → `crate::COMMAND_BUS_FALLBACK.with`
(same for `EVENT_BUS`). Error messages updated to reflect new name.

### WU-F-4: Parity tests (5 tests, 100% pass)

`crates/editor-bevy/tests/runtime_buses_parity.rs`:

- `parity_command_bus_session_path` — write + drain via session port
- `parity_event_bus_session_drain_fifo` — 3 events in FIFO order
- `parity_drain_empty` — empty drain returns `Vec::new()`
- `parity_reset_clears_write_offset` — drain resets write offset
- `parity_two_session_isolation` — sessions don't leak state

All 5 tests pass:

```
running 5 tests
test parity_command_bus_session_path ... ok
test parity_drain_empty ... ok
test parity_event_bus_session_drain_fifo ... ok
test parity_reset_clears_write_offset ... ok
test parity_two_session_isolation ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

### WU-F-5: Matrix update

`docs/architecture/state-ownership-matrix.md` § H2.5:

- `COMMAND_BUS` row: OPEN → RETIRED (H2.5 Block F) — fallback only
- `EVENT_BUS` row: OPEN → RETIRED (H2.5 Block F) — fallback only
- Progress note: 4 of 9 → **6 of 9 retired** (after Block F)
- Remaining 3 cells: `HOT_RELOAD_BUS`, `PLAY_MODE_REQUEST`, `KEYBOARD_STATE`

### WU-F-6: Inventory update

`tools/archcheck-globals/globals-inventory.yaml`:

- `COMMAND_BUS` → `COMMAND_BUS_FALLBACK` (declared_at 365, OPEN)
- `EVENT_BUS` → `EVENT_BUS_FALLBACK` (declared_at 375, OPEN)

Both FALLBACK entries documented as "dual-write fallback; production
code uses EditorSession; OPEN until §S-F parity tests prove full
coverage".

### WU-F-7: Version bump

`Cargo.toml`: 0.108.5 → 0.108.6.

## Verification status

- `cargo check --workspace --locked` ✅
- `cargo test -p editor-bevy --test runtime_buses_parity --locked` ✅ (5/5)
- `cargo check -p editor-model --target wasm32-unknown-unknown --locked` ✅

## Forward work after F

- **Block G**: `HOT_RELOAD_BUS` + `PLAY_MODE_REQUEST` (`hot_reload_state.rs:32,35`)
- **Block H**: `KEYBOARD_STATE` → Bevy `Resource InputState` (`logic_evaluator.rs:1049`)
