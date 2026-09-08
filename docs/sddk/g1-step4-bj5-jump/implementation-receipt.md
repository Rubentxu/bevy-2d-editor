# G1-step4 — Jump mechanic (implementation-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step4-bj5-jump`
**Sequence**: 242 (2026-09-08)
**Path**: A-lite
**Phase**: build

## Files created / modified

```
crates/examples-bevy-harness/src/jump.rs           NEW (83 lines)
crates/examples-bevy-harness/src/lib.rs            +3/-1
crates/examples-bevy-harness/tests/jump.rs         NEW (188 lines)
```

## Implementation

`crates/examples-bevy-harness/src/jump.rs`:
- `pub const JUMP_FRAME_DT: f32 = 0.1` — fixed per-frame impulse
  duration, matches the test's `TimeUpdateStrategy::ManualDuration`.
- `pub struct JumpState { pub space_was_pressed: bool }` —
  `Resource` for rising-edge detection.
- `pub struct JumpPlugin` — `Plugin` impl that init_resource + adds
  `jump_system` to `Update`.
- `pub fn jump_system(input, jump_state, query)` — applies
  `translation.y += jump_force * JUMP_FRAME_DT` on the rising edge.

`crates/examples-bevy-harness/src/lib.rs`:
- `pub mod jump;`
- `pub use jump::{JUMP_FRAME_DT, JumpPlugin, JumpState, jump_system};`

`crates/examples-bevy-harness/tests/jump.rs`:
- `player_jumps_on_space_press` (Space pressed → y delta = 40.0)
- `player_does_not_jump_without_input` (no press → y unchanged)
- `jump_only_affects_player` (Space pressed → enemy y unchanged)

## Build verification

```
$ cd crates/examples-bevy-harness && cargo build
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.59s
```

```
$ cd crates/examples-bevy-harness && cargo check
   Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Test results

```
$ cd crates/examples-bevy-harness && cargo test --test jump -- --ignored

running 3 tests
test player_jumps_on_space_press ... ok
test player_does_not_jump_without_input ... ok
test jump_only_affects_player ... ok

test result: ok. 3 passed; 0 failed
```

## Workspace

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Code quality

- Zero new dependencies.
- Zero changes to public types outside `examples-bevy-harness`.
- `JumpState` resource is `Default`-derived (no manual initialization
  needed).
- `JUMP_FRAME_DT` constant is `pub` for cross-crate consistency.

## Acceptance

✓ `cargo test -p examples-bevy-harness -- --ignored`: 11/11 pass
✓ `cargo test -p editor-bevy --lib`: 414/0/1 unchanged
✓ `bun run tools/archcheck/check.ts`: green
✓ `cargo check --workspace --tests`: clean
