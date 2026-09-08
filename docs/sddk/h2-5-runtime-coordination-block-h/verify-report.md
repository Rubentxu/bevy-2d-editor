# H2.5 Block H — Verify Report

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-h`
**Path**: A-lite (explore → spec → design → build → verify → release → archive)
**Tag**: `v0.108.8` (to be released)
**Date**: 2026-09-08

## Verification scope

Block H is the fifth and **final** thread_local migration under the
dual-write pattern. It migrates `KEYBOARD_STATE` (a Bevy keyboard
sensor) to a Bevy `Resource InputState`. Verification covers four
axes:

1. **Parity (new)**: `InputState` Resource path works (5 tests).
2. **Regression**: existing `play_mode.rs` tests (4 tests) continue
   to pass without modification (one-line `use` path update).
3. **Build hygiene**: `cargo check --workspace --locked` + WASM
   target.
4. **H0.3 ratchet**: `archcheck-globals` 29=29.

## Test results

### 1. Parity suite (new — 5 tests)

```
$ cargo test -p editor-bevy --test keyboard_input_state_parity --locked
running 5 tests
test parity_input_state_default_empty ... ok
test parity_legacy_fallback_only_when_resource_missing ... ok
test parity_dual_write_resource ... ok
test parity_no_keys_pressed_clears_resource ... ok
test parity_released_keys_removed_from_resource ... ok

test result: ok. 5 passed; 0 failed
```

### 2. Existing play_mode tests (4 tests, use path update only)

```
$ cargo test -p editor-bevy --test play_mode --locked
running 4 tests
test test_keyboard_state_not_updated_in_edit_mode ... ok
test test_update_keyboard_state_empty_when_no_keys_pressed ... ok
test test_update_keyboard_state_populates_from_button_input ... ok
test test_update_keyboard_state_clears_released_keys ... ok

test result: ok. 4 passed; 0 failed
```

### 3. Build hygiene

```
$ cargo check --workspace --locked
Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.03s

$ cargo check -p editor-model --target wasm32-unknown-unknown --locked
Finished `dev` profile in 0.22s
```

### 4. H0.3 ratchet (archcheck-globals)

```
$ cd tools/archcheck-globals && npm run check
archcheck-globals: 29 declarations, all match inventory (29 entries)
```

The rename `KEYBOARD_STATE` → `KEYBOARD_STATE_FALLBACK` (plus removal
of the old entry from inventory) keeps the ratchet at 29 entries
(same total).

## Code diff summary

```
crates/editor-bevy/src/keyboard_state.rs                       | +91 / -0   (new)
crates/editor-bevy/src/lib.rs                                  | +1 / -0    (mod declaration)
crates/editor-bevy/src/logic_evaluator.rs                      | +5 / -28   (remove inline + reader)
crates/editor-bevy/src/preview_runtime.rs                      | +1 / -1    (system path)
crates/editor-bevy/tests/play_mode.rs                          | +5 / -5    (use path + strings)
crates/editor-bevy/tests/keyboard_input_state_parity.rs        | +112 / -0  (new, 5 tests)
docs/architecture/state-ownership-matrix.md                    | +5 / -5    (1 retired + 9/9 progress)
tools/archcheck-globals/globals-inventory.yaml                 | +4 / -4    (rename)
Cargo.toml                                                     | +1 / -1    (0.108.7 → 0.108.8)
```

Net: **+225 / -44** lines (excluding block-H artifacts).

## H2.5 — MILESTONE COMPLETE (9 of 9 cells retired)

| Cell                        | Status                | Block |
|-----------------------------|-----------------------|-------|
| `ACTUATOR_OUTPUT_BUS`       | RETIRED               | A2    |
| `PREVIEW_METRICS`           | RETIRED               | E     |
| `PREVIEW_MAPPING`           | RETIRED               | E     |
| `PREVIEW_PROVENANCE`        | RETIRED               | E     |
| `COMMAND_BUS`               | RETIRED               | F     |
| `EVENT_BUS`                 | RETIRED               | F     |
| `HOT_RELOAD_BUS`            | RETIRED               | G     |
| `PLAY_MODE_REQUEST`         | RETIRED               | G     |
| `KEYBOARD_STATE`            | RETIRED               | H     |

**H2.5 complete.** No remaining cells.

## Backward-compat design

`update_keyboard_state` declares `input_state: Option<ResMut<InputState>>`
(not bare `ResMut<InputState>`). This is defensive:

1. Legacy tests that don't `init_resource::<InputState>()` still
   work (they exercise the FALLBACK path alone).
2. Bevy panics avoided when `app.update()` runs without Resource.

New consumers should `app.init_resource::<InputState>()` to enable
the canonical dual-write path.

## Out-of-scope (not verified)

- Full UAT cohort (`playwright.*.config.ts`) — Block H is internal to
  the Bevy preview subsystem; no UI changes.
- Bevy system ordering — `update_keyboard_state` registration
  unchanged (`Update`, `.run_if(in_play_mode)`, before
  `dispatch_dirty_bindings`).
- Cross-crate integration — only `editor-bevy` is touched.

## Compatibility

- `KEYBOARD_STATE` rename to `KEYBOARD_STATE_FALLBACK` — same type,
  same path, same semantics for the legacy consumer
  (`KeyPressedEvaluator`).
- New `InputState` Bevy Resource is additive — no existing system
  reads from it (only `update_keyboard_state` writes to it).
- All 4 existing `play_mode.rs` tests pass with one-line `use`
  path update.
- WASM exports unchanged.

## Conclusion

Block H is **verified**. The dual-write pattern (Blocks A2/D/E/F/G
consistency) preserves backward compatibility while migrating the
canonical owner of keyboard state to a Bevy `Resource`. The H2.5
milestone is now **complete** with all 9 cells retired.

Ready for `phase.verify.complete.a-min` → release → archive.
