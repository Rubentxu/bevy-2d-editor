# Block J — BJ-1 + BJ-2 Cleanup (spec)

**Cycle**: `p-28fce7028ac3c497/block-j-bj1-bj2-cleanup`
**Sequence**: 198 (2026-09-08)
**Path**: A-min
**Phase**: specify

## Acceptance criteria

### AC-J1.1 — `test_submit_and_drain` passes

```
$ cd crates/editor-bevy
$ cargo test --lib -- test_submit_and_drain --nocapture

test logic_evaluator::integration_tests::test_submit_and_drain ... ok
```

**Given**: The test is run with no editor session registered (clean state).
**When**: `submit_actuator_output(entity, "x", PortValue::Float(1.0))` is
called twice with two different fields, then `drain_actuator_outputs()` is
called.
**Then**: The returned `Vec<ActuatorOutput>` has length 2, the entity bits
are preserved, and a second drain returns empty.

### AC-J1.2 — `test_entity_bits_preserved_in_bus` passes

```
$ cargo test --lib -- test_entity_bits_preserved_in_bus --nocapture

test logic_evaluator::integration_tests::test_entity_bits_preserved_in_bus ... ok
```

**Given**: A session is installed via `install_fresh_session()`.
**When**: One output is submitted with `entity_bits=777` and field `"vx"`.
**Then**: Drain returns a 1-element `Vec`, the entity bits match, and the
field name matches.

### AC-J1.3 — `test_end_to_end_actuator_pipeline` passes

```
$ cargo test --lib -- test_end_to_end_actuator_pipeline --nocapture

test logic_evaluator::integration_tests::test_end_to_end_actuator_pipeline ... ok
```

**Given**: A `LogicGraphAsset` is registered with one actuator node.
**When**: The graph is evaluated with a translation actuator.
**Then**: The actuator output bus receives an entry with the correct
entity bits and field name.

### AC-J1.4 — Full lib test suite green

```
$ cd crates/editor-bevy
$ cargo test --lib

test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

(Was 411 passing + 3 failing before Block J.)

### AC-J1.5 — Workspace check green

```
$ cargo check --workspace
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### AC-J2.1 — archcheck green

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

The rule B1 regex refined with `(?<![-_])bevy::` (negative lookbehind)
excludes `editor_bevy::` and `editor-bevy::` substrings while still
matching top-level `bevy::` imports.

### AC-J2.2 — Doc comment drift fixed

`crates/editor-model/src/command.rs` no longer mentions `js_sys::Date::now()`
inline; rule B2 no longer matches that file.

## Behavior preserved

- Production API surface unchanged. `MinimalSession` and
  `install_fresh_session` are `pub(crate)` test-only.
- ADR-0064 thread_local fallbacks untouched.
- No new dependencies, no new files in `src/`.
- Only test scaffolding relocation + 3-line additions in 3 tests.

## Out of scope

- BJ-3, BJ-4, BJ-5 remain open.
- No edits to `tests/support/mod.rs` (the `FakeSession` harness is for
  integration tests in `tests/`, not unit tests inside the lib).
