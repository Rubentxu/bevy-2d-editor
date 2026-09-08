# H2.5 Block F — Verify Report

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-f`
**Path**: A-min
**Tag**: `v0.108.6` (to be released)
**Date**: 2026-09-08

## Verification scope

Block F is the third thread_local migration under the dual-write
fallback pattern (Block A2 ActuatorBus, Block E preview inspector,
Block F runtime buses). The verification covers four axes:

1. **Parity**: the new session-backed behaviour matches the legacy
   thread_local semantics for all 5 test cases (command bus session
   path, event bus FIFO drain, empty drain, drain reset, two-session
   isolation).
2. **Regression**: existing lib tests (`metrics_round_trip`,
   `mapping_round_trip`, `mapping_contains_no_bevy_entity_id_field`)
   continue to pass without modification.
3. **Build hygiene**: `cargo check --workspace --locked` and the WASM
   target check both succeed.
4. **H0.3 ratchet**: `archcheck-globals` 29 declarations = 29 inventory
   entries.

This is **mid verify** (A-min path), not full verify. No integration
tests beyond the parity tests, no UAT.

## Test results

### 1. Parity suite (new — 5 tests)

```
$ cargo test -p editor-bevy --test runtime_buses_parity --locked
running 5 tests
test parity_command_bus_session_path ... ok
test parity_drain_empty ... ok
test parity_event_bus_session_drain_fifo ... ok
test parity_reset_clears_write_offset ... ok
test parity_two_session_isolation ... ok

test result: ok. 5 passed; 0 failed
```

### 2. Lib tests (existing — 3 tests, unchanged)

```
$ cargo test -p editor-bevy --lib preview_inspector --locked
running 3 tests
test preview_inspector::tests::metrics_round_trip ... ok
test preview_inspector::tests::mapping_contains_no_bevy_entity_id_field ... ok
test preview_inspector::tests::mapping_round_trip ... ok

test result: ok. 3 passed; 0 failed
```

### 3. Build hygiene

```
$ cargo check --workspace --locked
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.76s

$ cargo check -p editor-model --target wasm32-unknown-unknown --locked
Finished `dev` profile in 0.24s
```

### 4. H0.3 ratchet (archcheck-globals)

```
$ cd tools/archcheck-globals && npm run check
archcheck-globals: 29 declarations, all match inventory (29 entries)
```

The rename `COMMAND_BUS` → `COMMAND_BUS_FALLBACK` (line 365) and
`EVENT_BUS` → `EVENT_BUS_FALLBACK` (line 375) keeps the ratchet at
29 entries (same total — old entries retired, new FALLBACK entries
added, both OPEN per the dual-write fallback contract).

## Code diff summary

```
crates/editor-bevy/src/lib.rs              | +14 / -78  (rename + doc + delete local LinearBus)
crates/editor-bevy/src/preview_runtime.rs | +5 / -5    (callers + .expect() messages)
crates/editor-bevy/tests/runtime_buses_parity.rs | +136 / -0   (new, 5 tests)
crates/editor-bevy/tests/support/mod.rs   | unchanged    (FakeSession already had command_bus/event_bus)
docs/architecture/state-ownership-matrix.md | +5 / -5    (2 retired + progress note)
tools/archcheck-globals/globals-inventory.yaml | +4 / -4  (2 retired → 2 FALLBACK)
Cargo.toml                                 | +1 / -1    (0.108.5 → 0.108.6)
```

Net: **+165 / -93** lines.

## H2.5 progress after Block F

| Cell                        | Status                |
|-----------------------------|-----------------------|
| `ACTUATOR_OUTPUT_BUS`       | RETIRED (Block A2)    |
| `PREVIEW_METRICS`           | RETIRED (Block E)     |
| `PREVIEW_MAPPING`           | RETIRED (Block E)     |
| `PREVIEW_PROVENANCE`        | RETIRED (Block E)     |
| `COMMAND_BUS`               | RETIRED (Block F)     |
| `EVENT_BUS`                 | RETIRED (Block F)     |
| `HOT_RELOAD_BUS`            | OPEN (Block G planned) |
| `PLAY_MODE_REQUEST`         | OPEN (Block G planned) |
| `KEYBOARD_STATE`            | OPEN (Block H planned) |

**6 of 9 H2.5 cells retired.** Two-thirds complete.

## Out-of-scope (not verified)

- Full UAT cohort (`playwright.*.config.ts`) — Block F is internal to
  the Bevy preview subsystem (thread_locals only); no UI changes.
- Bevy system ordering — no Bevy systems touched.
- Cross-crate integration — only `editor-bevy` and `editor-model` are
  touched (no public API change).

## Compatibility

- No public API change.
- No migration of consumer code required.
- WASM exports unchanged.
- The renamed `*_FALLBACK` thread_locals are private to
  `editor-bevy::lib`; no other crate references them by name.

## Conclusion

Block F is **verified**. The dual-write fallback pattern preserves
backward compatibility while migrating the canonical owner to the
session. The 9 H2.5 cells are now two-thirds retired (6 of 9), with
the remaining 3 logically grouped for Blocks G (hot-reload +
play-mode) and H (Bevy input `KEYBOARD_STATE`).

Ready for `phase.verify.complete.a-min` → release → archive.
