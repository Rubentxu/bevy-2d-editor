# Block I — Release Receipt

**Cycle**: `p-28fce7028ac3c497/smoke-budget-cargo-conformance-restore`
**Tag**: `v0.108.9`
**Commit**: `b7683e798f1ebf293492b97df5c778b3e7f47cb7`
**Path**: B-direct
**Release date**: 2026-09-08

## Released commits

```
b7683e7 chore(release): bump version 0.108.8 → 0.108.9 (Block I)
c6535c7 fix(test+playwright): restore cargo check + trim smoke cohort (Block I)
```

## Tag

`v0.108.9` → `b7683e798f1ebf293492b97df5c778b3e7f47cb7`

Verified: `git rev-parse v0.108.9^{commit}` == `git rev-parse HEAD` == `git rev-parse origin/main`.

## Files released (this cycle)

| Path                                                                | Change                                  |
|---------------------------------------------------------------------|-----------------------------------------|
| `crates/editor-bevy/tests/state_migration_pr2.rs`                   | +20 / -0  (5 runtime_*_mut delegations) |
| `crates/editor-bevy/tests/runtime_delta_wiring.rs`                  | +20 / -0  (5 runtime_*_mut delegations) |
| `crates/editor-bevy/tests/state_unified.rs`                         | +20 / -0  (5 runtime_*_mut unimplemented) |
| `frontend/playwright.smoke.config.ts`                               | +12 / -12 (testMatch trimmed, doc expanded) |
| `docs/ROADMAP.md`                                                   | +2 / -0   (H2.5 + Block I active rows)  |
| `docs/adr/0064-fallback-thread-locals-as-permanent-compat-layer.md` | new (160 lines)                         |
| `docs/sddk/smoke-budget-cargo-conformance-restore/`                 | new dir (implementation-receipt + verify-report) |
| `Cargo.toml`                                                        | +1 / -1   (version bump)                |

## Verification gates

- `cargo check --workspace --tests` → exit 0 (was failing with E0046).
- `cargo test -p editor-bevy --test state_migration_pr2 --test runtime_delta_wiring --test state_unified` → 12/12 pass.
- `npx playwright test --config=playwright.smoke.config.ts --list` → 39 tests in 4 files (was 60+ tests in 6 files).
- `npx playwright test --config=playwright.full.config.ts --list` → 296 tests in 40 files (engine.spec.ts, ux-dock.spec.ts, mode-context-bar.spec.ts, mode-headers.spec.ts now correctly in full cohort).
- `cd frontend && npx tsc -p .` → silent (no errors).

## Known follow-ups (Block J)

1. **3 logic_evaluator integration tests fail** (`test_submit_and_drain`,
   `test_entity_bits_preserved_in_bus`, `test_end_to_end_actuator_pipeline`)
   because `submit_actuator_output()` requires session install — the
   H2.5 Block A2 actuator bus refactor removed the FALLBACK without
   rewriting these tests. Block J will either add a FALLBACK or rewrite
   the tests to install a session.

2. **archcheck rule B1 false-positives** — `bevy::` regex matches
   `editor_bevy::command` substrings in 7 source files, all in
   doc-comments. Block J will refine the regex (e.g. require word
   boundary not preceded by `_`) or rewrite the comments.

3. **`cargo test --release` budget (C-3)** — pre-existing, out of scope.
4. **M-2, M-3, M-5** — pre-existing low-priority docs/archcheck warnings, out of scope.
