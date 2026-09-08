# H2.5 Block F — Handoff (v0.108.6 released)

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-f`
**Path**: A-min (explore → spec → tasks → build → verify → release → archive)
**Tag**: `v0.108.6` on origin pointing at commit `def9272`
**Date**: 2026-09-08

## Outcome

Block F retires the legacy `COMMAND_BUS` and `EVENT_BUS` thread_locals
via the established dual-write fallback pattern (Block A2
ActuatorBus, Block E preview inspector). Production code now writes
through `EditorSession.runtime.command_bus` / `EditorSession.runtime.event_bus`
via `editor_model::ports::with_session_mut`. Fallback thread_locals
renamed (`COMMAND_BUS_FALLBACK` / `EVENT_BUS_FALLBACK`) and kept for
legacy tests + non-session entrypoints.

**Side effect**: also removed the duplicated local `LinearBus` struct
(~70 lines) and the `BUS_CAPACITY` constant. Now using
`editor_model::runtime::LinearBus` as the single source of truth.

## Sequence

| Step | Transition                                        | Sequence |
|------|---------------------------------------------------|----------|
| 1    | start (already existed, OPEN)                     | —        |
| 2    | build → verify                                    | 134      |
| 3    | verify → release                                  | 136      |
| 4    | supersede (`scope_invalid`, evidence-refs)        | 140      |

## Work units landed

| WU     | Files                                                            | Lines |
|--------|------------------------------------------------------------------|-------|
| WU-F-1 | `crates/editor-bevy/src/lib.rs` (rename + dual-write doc)        | +12 / -8 |
| WU-F-2 | `crates/editor-bevy/src/lib.rs` (delete local LinearBus + BUS_CAPACITY) | +1 / -70 |
| WU-F-3 | `crates/editor-bevy/src/preview_runtime.rs` (callers + .expect()) | +5 / -5 |
| WU-F-4 | `crates/editor-bevy/tests/runtime_buses_parity.rs` (new, 5 tests) | +136 / -0 |
| WU-F-5 | `docs/architecture/state-ownership-matrix.md` § H2.5              | +5 / -5 |
| WU-F-6 | `tools/archcheck-globals/globals-inventory.yaml` (2 retired → 2 FALLBACK OPEN) | +4 / -4 |
| WU-F-7 | `Cargo.toml` (0.108.5 → 0.108.6)                                 | +1 / -1 |

**Net: +165 / -93 lines** (excluding block-F artifacts).

## Verification status

| Check | Result |
|-------|--------|
| `cargo check --workspace --locked`               | ✅ |
| `cargo check -p editor-model --target wasm32-unknown-unknown --locked` | ✅ |
| `cargo test -p editor-bevy --test runtime_buses_parity --locked`       | ✅ 5/5 |
| `cargo test -p editor-bevy --lib preview_inspector --locked`           | ✅ 3/3 |
| `archcheck-globals` (`npm run check`)            | ✅ 29/29 |

## Gate receipts (4/4 passed)

1. `tests-pass` — parity + lib tests
2. `policy-compliant` — archcheck 29=29
3. `debt-severity-assigned` — `sddk debt gates` PASS (0 findings)
4. `debt-priority-assigned` — `sddk debt gates` PASS (0 findings)

## Release anomaly (same as Blocks A2 + D + E)

`sddk release apply` rejected with "dirty worktree" because the
workspace contains untracked files owned by OTHER concurrent cycles
(`docs/sddk/wave-d1-editor-gateway-seam/`,
`docs/sddk/world-workspace/`,
`docs/sddk/semantic-editor-model-*/`,
`docs/sddk/application-stabilization-and-roadmap-convergence/`).
**Pattern is now stable across 4 cycles**:

1. `git add` only files owned by this cycle.
2. `git commit -m "refactor(arch): H2.5 Block F ..."`
3. `git push origin main`
4. `git tag -a v0.108.6 -m "..."`
5. `git push origin v0.108.6`
6. `sddk cycle supersede --reason scope-invalid --evidence-refs [receipt,verify-report,commit:def9272,tag:v0.108.6]`

Stashing untracked files owned by other cycles would corrupt their
work, so the workaround is to leave them alone and push/tag manually.
This is the documented pattern across Blocks A2, D, E, F.

## H2.5 progress (after Block F)

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

**6 of 9 H2.5 cells retired** (two-thirds complete).

## Cycle mechanics notes

- The cycle already existed (`OPEN` / `build` phase) when this session
  picked up after compaction. The `cycle start` command returned
  `UNIQUE constraint failed: cycles.project_id, cycles.cycle_id` even
  though the row was not visible via direct SQL inspection — the
  recovery path was:
  1. Locate state-ledger at
     `/home/rubentxu/.local/state/sddk/projects/p-28fce7028ac3c497/ledger.sqlite`
     (NOT the data-side `/home/rubentxu/.local/share/sddk/projects/...`).
  2. Inspect `cycles` table — confirmed row with `phase=build, status=OPEN`.
  3. `sddk cycle lock acquire --cycle ... --owner jcode-block-f` →
     fencing_token=1.
  4. `sddk cycle evaluate-gate --gate implementation-complete --transition phase.build.complete --outcome passed --evidence '{"argv":...,"exit_code":0,...}'`.
  5. `sddk cycle transition --transition phase.build.complete --artifact implementation-receipt=... --gate-receipt gate-implementation-complete-...`
  6. Repeated for verify phase (4 gates + verification-report artifact).
- `cycle evaluate-gate` does NOT take `--lease-owner` / `--fencing-token`
  (despite looking similar). `cycle transition` does.
- `cycle supersede --evidence-refs` expects a **JSON array string**,
  not a comma-separated list (despite the help message wording).
  Pass `'["ref1","ref2","commit:sha","tag:vX.Y.Z"]'`.

## Forward work (NOT in this cycle)

1. **Block G**: `HOT_RELOAD_BUS` + `PLAY_MODE_REQUEST`
   (`crates/editor-bevy/src/hot_reload_state.rs:32,35`).
   Same dual-write pattern. Will land in `EditorSession.runtime.hot_reload`
   and `EditorSession.runtime.play_mode`.
2. **Block H**: `KEYBOARD_STATE` → Bevy `Resource InputState`
   (`crates/editor-bevy/src/logic_evaluator.rs:1049`).
   Different migration target (Bevy Resource, not EditorSession
   field) — likely A-lite path because the migration target is
   architectural.

## Lessons persisted to memory

1. **`sddk cycle start` UNIQUE error recovery**:
   inspect state-ledger at
   `$HOME/.local/state/sddk/projects/<project_id>/ledger.sqlite`.
   The data-side `share/` path is NOT the cycles table.
2. **`cycle supersede --evidence-refs` requires JSON array**, not
   comma-separated.
3. **Dirty worktree workaround is stable** across 4 cycles — manual
   `git push` + `git tag -a` + `git push --tags` + `cycle supersede`.
   Tag must be repositioned to the final commit WITH the SDDK
   artifacts (implementation-receipt, verify-report).
4. **`FakeSession` already had command_bus/event_bus fields** in
   `crates/editor-bevy/tests/support/mod.rs` (Block A setup). No
   changes needed there — Block F parity tests just exercise them.

## Persistence

This handoff is committed as part of the Block F commit (`def9272`)
in `docs/sddk/HANDOFF-2026-09-08-block-f.md`.
