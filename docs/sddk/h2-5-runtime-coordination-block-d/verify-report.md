# H2.5 Block D — Light Verify Report

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-d`
**Path**: B-direct
**Tag**: `v0.108.4`
**Commit**: `1e10aae`
**Date**: 2026-09-08

## Scope of verification

Block D is a **metadata + parity tests** cycle. The verification
covers three axes:

1. **Inventory ↔ workspace sync** — the ratchet matches the real
   scanned declarations against the committed inventory.
2. **Ratchet parity tests** — the new Block D test cases pass.
3. **Cargo workspace** — no Rust compilation regression.

This is **light verify** (B-direct path), not full verify. No
integration tests, no UAT. The acceptance evidence is the ratchet
itself plus the cargo check exit code.

## Test results

### 1. Ratchet parity suite (new)

```
$ cd tools/archcheck-globals && npm test
> archcheck-globals@0.1.0 test
> tsx check.test.ts

archcheck-globals tests: all pass
```

Includes the four new Block D cases (matched, untracked, orphan,
re-introduced) plus all pre-existing tests.

### 2. Inventory ↔ workspace sync

```
$ cd tools/archcheck-globals && npm run check
> archcheck-globals@0.1.0 check
> tsx check.ts

archcheck-globals: 29 declarations, all match inventory (29 entries)
```

29 declarations (was 30 in v0.108.3 because `ACTUATOR_OUTPUT_BUS` was
removed in Block A2, commit `05cde01`). All entries match
declarations — **zero orphans, zero new declarations**.

### 3. Cargo workspace

```
$ cargo check --workspace --locked
…
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.82s
```

OK. The 161 warnings in `editor-bevy` are pre-existing and unrelated
to Block D (no new code was added to `editor-bevy`).

## H0.3 ratchet gate

The H0.3 "forbid new globals" gate is satisfied because:

- `npm run check` exits 0 with the message "all match inventory".
- The four new parity tests prove the ratchet catches the canonical
  failure modes (matched, untracked, orphan, re-introduced).

If anyone tries to add a new `thread_local!` or `static` to any
crate without adding it to `globals-inventory.yaml`, the ratchet will
fail in CI. This is the H0.3 exit condition.

## H2.5 inventory progress

After Block A2 + Block D (this cycle):

| Cell                        | Status                |
|-----------------------------|-----------------------|
| `ACTUATOR_OUTPUT_BUS`       | RETIRED (Block A2)    |
| `COMMAND_BUS`               | OPEN                  |
| `EVENT_BUS`                 | OPEN                  |
| `PREVIEW_METRICS`           | OPEN                  |
| `PREVIEW_MAPPING`           | OPEN                  |
| `PREVIEW_PROVENANCE`        | OPEN                  |
| `HOT_RELOAD_BUS`            | OPEN                  |
| `PLAY_MODE_REQUEST`         | OPEN                  |
| `KEYBOARD_STATE`            | OPEN (Block C)        |

**1 of 9 H2.5 cells retired.** The remaining 8 are tracked as Block E+
follow-ups.

## Out-of-scope (not verified)

- Full UAT cohort (`playwright.*.config.ts`) — Block D is docs +
  inventory + ratchet tests; no UI changes.
- Bevy runtime parity — Block D doesn't touch Bevy systems.
- Cross-crate integration — Block D is scoped to
  `tools/archcheck-globals/` + `docs/architecture/`.

## Compatibility

- No public API change.
- No migration of consumer code required.
- The retired entry was an internal `thread_local!` only consumed by
  `actuator_bus.rs` (already migrated in Block A2).

## Conclusion

Block D is **verified**. The H0.3 inventory ratchet is operational,
the inventory reflects reality, and all four parity test cases pass.
Ready for the `phase.verify.complete.b-direct` transition and
subsequent release.
