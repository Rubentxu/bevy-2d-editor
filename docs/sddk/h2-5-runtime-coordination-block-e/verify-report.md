# H2.5 Block E — Verify Report

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-e`
**Path**: A-min
**Tag**: `v0.108.5`
**Commit**: `6c7c0fc` (`main`)
**Date**: 2026-09-08

## Verification scope

Block E is a thread_local migration with a dual-write fallback. The
verification covers three axes:

1. **Parity**: the new session-backed behaviour matches the legacy
   thread_local semantics for all 6 test cases (round-trip metrics,
   mapping, provenance; increment_rebuild_count; fallback without
   session; two-session isolation).
2. **Regression**: existing lib tests (`metrics_round_trip`,
   `mapping_round_trip`, `mapping_contains_no_bevy_entity_id_field`)
   continue to pass without modification.
3. **Build hygiene**: `cargo check --workspace --locked` and the WASM
   target check both succeed.

This is **mid verify** (A-min path), not full verify. No integration
tests beyond the parity tests, no UAT.

## Test results

### 1. Parity suite (new — 6 tests)

```
$ cargo test -p editor-bevy --test preview_inspector_parity --locked
running 6 tests
test parity_fallback_path_without_session ... ok
test parity_metrics_round_trip_via_session ... ok
test parity_increment_rebuild_count_via_session ... ok
test parity_mapping_round_trip_via_session ... ok
test parity_two_session_isolation ... ok
test parity_provenance_round_trip_via_session ... ok

test result: ok. 6 passed; 0 failed
```

### 2. Lib tests (existing — 3 tests, unchanged)

```
$ cargo test -p editor-bevy --lib preview_inspector --locked
running 3 tests
test preview_inspector::tests::metrics_round_trip ... ok
test preview_inspector::tests::mapping_round_trip ... ok
test preview_inspector::tests::mapping_contains_no_bevy_entity_id_field ... ok

test result: ok. 3 passed; 0 failed
```

### 3. Build hygiene

```
$ cargo check --workspace --locked
Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.03s

$ cargo check -p editor-model --target wasm32-unknown-unknown --locked
Finished `dev` profile in 0.30s
```

### 4. Inventory sync

```
$ cd tools/archcheck-globals && npm run check
archcheck-globals: 29 declarations, all match inventory (29 entries)

$ npm test
archcheck-globals tests: all pass
```

## H0.3 ratchet gate

Still satisfied after Block E:

- `npm run check` exits 0.
- The 4 parity tests added in Block D (matched, untracked, orphan,
  re-introduced) continue to pass.
- The 6 new parity tests for Block E (preview-inspector migration)
  pass.

Adding a new `thread_local!` or `static` to any crate without adding
it to `globals-inventory.yaml` still fails CI.

## H2.5 progress after Block E

| Cell                        | Status                |
|-----------------------------|-----------------------|
| `ACTUATOR_OUTPUT_BUS`       | RETIRED (Block A2)    |
| `PREVIEW_METRICS`           | RETIRED (Block E)     |
| `PREVIEW_MAPPING`           | RETIRED (Block E)     |
| `PREVIEW_PROVENANCE`        | RETIRED (Block E)     |
| `COMMAND_BUS`               | OPEN (Block F planned) |
| `EVENT_BUS`                 | OPEN (Block F planned) |
| `HOT_RELOAD_BUS`            | OPEN (Block G planned) |
| `PLAY_MODE_REQUEST`         | OPEN (Block G planned) |
| `KEYBOARD_STATE`            | OPEN (Block H planned) |

**4 of 9 H2.5 cells retired.** Halfway through.

## Out-of-scope (not verified)

- Full UAT cohort (`playwright.*.config.ts`) — Block E is internal to
  the Bevy preview subsystem; no UI changes.
- Bevy system ordering — `preview_inspector.rs` has no Bevy systems.
- Cross-crate integration — only `editor-bevy` and `editor-model` are
  touched.

## Compatibility

- No public API change.
- No migration of consumer code required.
- WASM exports unchanged.
- The renamed `*_FALLBACK` thread_locals are private to
  `preview_inspector.rs`; no other crate references them.

## Conclusion

Block E is **verified**. The dual-write fallback pattern preserves
backward compatibility while migrating the canonical owner to the
session. The 9 H2.5 cells are now halfway retired (4 of 9), with
the remaining 5 logically grouped for Blocks F/G/H.

Ready for `phase.build.complete` → verify (already complete) → release
→ archive.
