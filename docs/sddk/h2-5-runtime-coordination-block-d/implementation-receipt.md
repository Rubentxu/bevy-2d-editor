# H2.5 Block D — Implementation Receipt

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-d`
**Path**: B-direct
**Tag**: `v0.108.4`
**Commit**: `1e10aae` (`main`)
**Date**: 2026-09-08

## Scope

Update the H0.3 global-state inventory ratchet to reflect Block A2's
migration of `ACTUATOR_OUTPUT_BUS`, update the H2.5 section of the
state-ownership matrix, and add ratchet parity tests that prove the
inventory ratchet correctly catches the four canonical H0.3 failure
modes.

## Work units landed

| WU      | Files changed                                                                  | Lines |
|---------|--------------------------------------------------------------------------------|-------|
| WU-D-1  | `tools/archcheck-globals/globals-inventory.yaml`                                | +18 / -5   |
| WU-D-2  | `docs/architecture/state-ownership-matrix.md` § H2.5                            | +14 / -1   |
| WU-D-3  | `tools/archcheck-globals/check.test.ts` (ratchet parity tests)                  | +92 / -0   |
| WU-D-x  | `tools/archcheck-globals/check.ts` (orphanEntries type interface fix)           | +1 / -1    |

Total: **+125 / -7** across 3 source files plus the version bump in
`Cargo.toml` (`0.108.3 → 0.108.4`).

## WU-D-1 — Inventory retirement

Removed `ACTUATOR_OUTPUT_BUS` from the `entries:` list (it no longer
exists in code after Block A2). Consolidated the previously duplicated
`retired:` section into a single list with milestone attribution
(H2.4 first sweep + H2.5 Block A2). The retired entry now carries:

- `retirement_pr: h2-5-runtime-coordination-block-a2`
- `replacement: editor_model::runtime::ActuatorBus (accessed via
  editor_model::ports::with_session_mut)`

## WU-D-2 — State-ownership matrix

Updated `docs/architecture/state-ownership-matrix.md` § H2.5:

- `ACTUATOR_OUTPUT_BUS` row converted to strikethrough with
  `RETIRED (H2.5 Block A2)` status.
- Added a "Progress" note documenting: **1 of 9 retired**, the
  PortValue canonicalization, and the remaining 8 H2.5 cells with
  names + scheduled follow-up (Block E+).

## WU-D-3 — Ratchet parity tests

Added four `Block D ratchet: …` test cases to
`tools/archcheck-globals/check.test.ts`:

1. **Matched**: code declares X, inventory has X at the same path →
   neither orphan nor new (the steady state).
2. **Untracked**: code declares X+Y, inventory has only X → X matches,
   Y surfaces as `newDeclarations` (H0.3 "forbid new globals" gate).
3. **Orphan**: inventory claims X exists at line 5, code does not
   declare X → surfaces as `orphanEntries` (the "I forgot to delete
   the inventory entry" failure mode from Block A2).
4. **Re-introduced**: same name appears in code AND inventory (after a
   previous retirement) → must match by name, not flag as new.

Each case is independent and uses synthetic inventories to avoid
cluttering the real committed inventory. They run in the existing
`npm test` harness in `tools/archcheck-globals/`.

## Bonus — pre-existing type fix

The `RatchetResult.orphanEntries` field in `check.ts` was declared as
`Array<{ entry: InventoryEntry }>` but the implementation returned
`InventoryEntry[]` directly. Tests in this cycle surfaced the
discrepancy (a `TypeError` at runtime). Fixed by aligning the
interface with the implementation. No behavioural change; consumers
that read the field correctly were never affected.

## Validation evidence

### Ratchet parity suite (Block D)

```
$ cd tools/archcheck-globals && npm test
> archcheck-globals@0.1.0 test
> tsx check.test.ts

archcheck-globals tests: all pass
```

### Inventory ↔ workspace sync

```
$ cd tools/archcheck-globals && npm run check
> archcheck-globals@0.1.0 check
> tsx check.ts

archcheck-globals: 29 declarations, all match inventory (29 entries)
```

29 declarations (down from 30 in v0.108.3 because `ACTUATOR_OUTPUT_BUS`
no longer exists in code after Block A2's commit `05cde01`).

### Cargo workspace

```
$ cargo check --workspace --locked
…
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.82s
```

OK (161 warnings in `editor-bevy` — pre-existing, unrelated to Block D).

## Closes

- **H0.3** (global-state inventory ratchet) — Block D side: the
  ratchet now correctly tracks the retirement of one H2.5 cell and
  rejects untracked declarations.
- **H2.5 Block D** (this cycle): inventory + matrix + ratchet parity
  tests all landed.

## Does NOT close (forward work)

The actual migration of the **remaining 8 H2.5 thread_locals**
(`COMMAND_BUS`, `EVENT_BUS`, `PREVIEW_METRICS`, `PREVIEW_MAPPING`,
`PREVIEW_PROVENANCE`, `HOT_RELOAD_BUS`, `PLAY_MODE_REQUEST`,
`KEYBOARD_STATE`) is tracked as **Block E+** in the
state-ownership-matrix. Each will be a separate H2.5 follow-up cycle
because they touch different sub-systems (preview rebuild, hot-reload
scheduler, play-mode queue, Bevy input resource).

## Compatibility

- No public API change.
- No migration of consumer code required (the retired entry was an
  internal `thread_local!`).
- `KEYBOARD_STATE` deliberately remains in inventory and in code
  pending the Block C follow-up (Bevy `Resource InputState`).

## Rollback

Revert commit `1e10aae` (single commit, no follow-ups). The change is
metadata + tests only; reverting it restores the inventory entry but
does not reintroduce the thread_local (it was removed in Block A2,
commit `05cde01`).
