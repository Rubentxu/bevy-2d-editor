# v1.0-stabilization G1 Bevy Harness — Implementation Receipt

**Cycle**: `p-28fce7028ac3c497/v1-g1-bevy-harness`
**Path**: A-min (explore → spec → build → verify → debt-verify → release → archive)
**Tag**: `v0.109.0` (to be released)
**Commit**: TBD (pending release)
**Date**: 2026-09-08

## Scope

Close one bounded step of v1.0-stabilization G1 (canonical playable
sample game) by creating a Bevy 2D runtime harness that consumes
`examples/platformer-minimal/` as a witness.

## Work units landed

| WU      | Files                                                                                     | Lines    |
|---------|-------------------------------------------------------------------------------------------|----------|
| WU-G1-1 | `Cargo.toml` (workspace members) — added `crates/examples-bevy-harness`                   | +1 / -1  |
| WU-G1-2 | `crates/examples-bevy-harness/Cargo.toml` (new)                                          | +20 / -0 |
| WU-G1-3 | `crates/examples-bevy-harness/src/lib.rs` (new) — public API re-exports + `spawn_all_assets` | +30 / -0 |
| WU-G1-4 | `crates/examples-bevy-harness/src/components.rs` (new) — Bevy mirrors of custom schemas  | +68 / -0 |
| WU-G1-5 | `crates/examples-bevy-harness/src/loader.rs` (new) — SceneAssetDocument → World translation | +200 / -0 |
| WU-G1-6 | `crates/examples-bevy-harness/tests/load_sample.rs` (new) — `#[ignore]` integration test | +142 / -0 |
| WU-G1-7 | `docs/sddk/v1-g1-bevy-harness/explore-report.md`                                         | +143 / -0 |
| WU-G1-8 | `docs/sddk/v1-g1-bevy-harness/spec.md`                                                   | +179 / -0 |

**Net: +783 / -1** lines (excluding cycle artifacts).

## Verification performed

### Build (cargo check + cargo test)

```
$ cargo check -p examples-bevy-harness
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 49.58s
```

```
$ cargo test -p examples-bevy-harness --test load_sample -- --ignored
running 1 test
test sample_round_trips_into_bevy_world ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Regression check

```
$ cargo check --workspace --tests
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.37s
```

`cargo check --workspace --tests` remains green (was fixed in Block I).
The `examples-bevy-harness` crate adds no regression to any other
crate.

### Archcheck

```
$ bun run tools/archcheck/check.ts
archcheck: 2 assertion(s) failed
  - editor-model purity (pre-existing in operation_log.rs, command.rs, ...)
  - editor-model wasm_bindgen/web_sys imports (pre-existing)
```

Both failures are **pre-existing** (carry-forward from H2.5 Block H
handoff, documented in Block I as BJ-2). The new harness crate
preserves `editor-model` purity — it depends on `editor-model` (no
Bevy, no WASM, no JS), and imports Bevy itself in the harness, not
the model.

## Acceptance criteria

| Criterion                                                                  | Status | Evidence |
|----------------------------------------------------------------------------|--------|----------|
| `crates/examples-bevy-harness/` exists and is in workspace members        | ✅ PASS | `Cargo.toml` line 2 lists the new crate. |
| `cargo check -p examples-bevy-harness` exits 0                            | ✅ PASS | Output: `Finished dev profile in 49.58s`. |
| `cargo build -p examples-bevy-harness` exits 0                            | ✅ PASS | `cargo test -p examples-bevy-harness -- --ignored` compiles and runs. |
| `cargo test -p examples-bevy-harness -- --ignored` runs ≥1 test that passes | ✅ PASS | `1 passed; 0 failed` for `sample_round_trips_into_bevy_world`. |
| Existing `cargo check --workspace --tests` (Block I) still green           | ✅ PASS | Output ends with `Finished dev profile`. |
| Existing `playwright.full.config.ts` still passes                          | 🔄 PENDING | Verify phase will run; not regressed by the harness crate. |
| `archcheck` all assertions pass                                            | ❌ FAIL (pre-existing) | Out of scope (BJ-2). Not a regression. |

## Decision rationale (recap from spec)

The harness reads `SceneAssetDocument` JSON directly (Option C in the
explore report) rather than parsing the `.bsn` Rust wrapper files.
This separation of concerns makes the harness a **witness** for the
editor's authoring JSON, while the existing
`frontend/tests/e2e-game-creation.spec.ts` continues to witness the
editor's BSN export pipeline.

## Known follow-up (Block J+ for G1)

1. **UI-creation Playwright test** — does not author the sample from
   the editor's UI; only loads it from filesystem. Out of scope for
   this cycle.
2. **Gameplay assertions** — the harness proves entity structure, not
   gameplay (no jumping, no enemy patrol, no pickup collision).
3. **Bevy `play_mode` runtime state** — does not exercise
   `enter_play_mode()` semantics. Tracked for the next G1 cycle.
