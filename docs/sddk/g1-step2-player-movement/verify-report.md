# G1-step2 — Player Movement System (verify-report)

**Cycle**: `p-28fce7028ac3c497/g1-step2-player-movement`
**Sequence**: 216 (2026-09-08)
**Path**: A-lite
**Phase**: verify

## Acceptance criteria — verification

### AC-PM1.1 — Player translates right when ArrowRight pressed ✅

```
$ cd crates/examples-bevy-harness
$ cargo test --test player_movement player_translates_right -- --ignored --nocapture

test player_translates_right_on_arrow_right ... ok
```

**PASS.** Player moves `x ≈ 20.0` (200.0 speed × 0.1 s).

### AC-PM1.2 — Player stays still when no input ✅

```
test player_stays_still_with_no_input ... ok
```

**PASS.** `translation.x == 0.0` after 0.1 s with no input.

### AC-PM1.3 — Player reverses on ArrowLeft ✅

```
test player_translates_left_on_arrow_left ... ok
```

**PASS.** Player moves `x ≈ -20.0`.

### AC-PM1.4 — System is gated by `With<PlayerController>` ✅

```
test enemy_is_unaffected_by_player_input ... ok
```

**PASS.** Enemy `translation.x == 0.0` despite ArrowRight pressed
(the query `(&mut Transform, &PlayerController)` filters by
`With<PlayerController>`).

### AC-PM1.5 — No regressions in existing test suite ✅

```
$ cargo test --workspace
```

All test crates report `test result: ok. <n> passed; 0 failed`.
Notable counts (only the lib tests; ignored are separate):

| Crate | Pass | Fail | Ignored |
|-------|------|------|---------|
| `editor-bevy` (lib) | 414 | 0 | 1 |
| `editor-wasm` | 26 | 0 | 0 |
| `editor-model` (lib) | 313 | 0 | 0 |
| `examples-bevy-harness` (lib + integration) | 4 + 1 | 0 | 4 |

### AC-PM1.6 — Workspace check green ✅

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### AC-PM1.7 — archcheck green ✅

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Side checks

### `cargo test -p editor-bevy --lib`

```
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

No regressions vs. Block J baseline (414 / 0 / 1).

### `cargo test -p examples-bevy-harness -- --ignored`

```
running 4 tests
test player_stays_still_with_no_input ... ok
test player_translates_right_on_arrow_right ... ok
test player_translates_left_on_arrow_left ... ok
test enemy_is_unaffected_by_player_input ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The four new tests pass deterministically.

## Risks identified

None. The Bevy 0.19 test-time gotchas (MinimalPlugins missing
InputPlugin, Time<Real> warm-up) are documented in the implementation
receipt and handled in the test helpers.

## Summary

**PASS** — 7/7 acceptance criteria met. Ready to advance to release.
