# Block J — BJ-1 + BJ-2 Cleanup (verify-report)

**Cycle**: `p-28fce7028ac3c497/block-j-bj1-bj2-cleanup`
**Sequence**: 202 (2026-09-08)
**Path**: A-min
**Phase**: verify

## Acceptance criteria — verification

### AC-J1.1 — `test_submit_and_drain` passes ✅

```
$ cd crates/editor-bevy
$ cargo test --lib -- test_submit_and_drain --nocapture

test logic_evaluator::integration_tests::test_submit_and_drain ... ok
```

PASS — observed in 414-test full run.

### AC-J1.2 — `test_entity_bits_preserved_in_bus` passes ✅

```
$ cargo test --lib -- test_entity_bits_preserved_in_bus --nocapture

test logic_evaluator::integration_tests::test_entity_bits_preserved_in_bus ... ok
```

PASS.

### AC-J1.3 — `test_end_to_end_actuator_pipeline` passes ✅

```
$ cargo test --lib -- test_end_to_end_actuator_pipeline --nocapture

test logic_evaluator::integration_tests::test_end_to_end_actuator_pipeline ... ok
```

PASS.

### AC-J1.4 — Full lib test suite green ✅

```
$ cargo test --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

Baseline before Block J: 411 passing + 3 failing. After: 414 + 0 + 1
ignored (unchanged). Net +3 tests now passing.

### AC-J1.5 — Workspace check green ✅

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.21s
```

### AC-J2.1 — archcheck green ✅

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

B1 regex refined to `/(?<![-_])bevy::/` excludes `editor_bevy::` and
`editor-bevy::` substrings. Verified in node REPL:

```js
> "editor_bevy::Foo".match(/(?<![-_])bevy::/g)
null
> "bevy::prelude::Entity".match(/(?<![-_])bevy::/g)
[ 'bevy::' ]
> "editor-bevy::Scene".match(/(?<![-_])bevy::/g)
null
```

### AC-J2.2 — Doc comment drift fixed ✅

`crates/editor-model/src/command.rs` line 53 now reads:

```
// The WASM bridge provides a date function — used as fallback when
// SystemTime::now() is unavailable in WASM.
```

Rule B2 (`js_sys::` substring match) no longer triggers.

## Side checks

### `cargo check --workspace --tests` ✅

No compile errors across workspace with `--tests` flag.

### `cargo build -p editor-bevy` ✅

Build succeeds (not run explicitly — covered by `cargo test --lib`).

### No regressions in `actuator_bus` tests ✅

The two original `actuator_bus::tests` tests still pass (they use the
relocated helper via `use super::test_helpers::install_fresh_session`):

```
test actuator_bus::tests::test_submit_and_drain_roundtrip ... ok
test actuator_bus::tests::test_bus_empty_after_drain ... ok
```

## Risks identified

None. All acceptance criteria pass. No production API surface change.
No new thread_locals. No dependency change.

## Summary

**PASS** — 7/7 acceptance criteria met. Ready to advance to debt-verify
or release (whichever is the next A-min phase).
