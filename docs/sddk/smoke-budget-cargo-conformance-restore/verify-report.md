# Block I — Verify Report

**Cycle**: `p-28fce7028ac3c497/smoke-budget-cargo-conformance-restore`
**Path**: B-direct (light verify, single lens: direct-acceptance)
**Date**: 2026-09-08

## Verdict: PASS_WITH_WARNINGS

### Acceptance criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `cargo check --workspace --tests` exits 0 | ✅ PASS | Output: `Finished dev profile in 5.28s` (was failing with E0046 "missing `runtime_*_mut` in impl"). |
| 3 fixed test files pass their cargo test runs | ✅ PASS | `state_migration_pr2` (3/3), `runtime_delta_wiring` (5/5), `state_unified` (4/4) — 12/12 tests pass. |
| Playwright smoke config lists only `@smoke` specs | ✅ PASS | `npx playwright test --config=playwright.smoke.config.ts --list` → `Total: 39 tests in 4 files`. Files: smoke.spec.ts, app-characterization.spec.ts, capabilities-smoke.spec.ts, editor-ready.spec.ts. |
| Removed specs are reachable via full/domain cohorts | ✅ PASS | `npx playwright test --config=playwright.full.config.ts --list` → `Total: 296 tests in 40 files`. engine.spec.ts, ux-dock.spec.ts, mode-context-bar.spec.ts, mode-headers.spec.ts all present. |
| FRONTEND-001 (TypeScript check) green | ✅ PASS | `cd frontend && npx tsc -p .` silent (no errors). |
| ADR-0064 ratified and filed | ✅ PASS | `docs/adr/0064-fallback-thread-locals-as-permanent-compat-layer.md` created, 160 lines, references all 8 FALLBACK thread_locals + cycle 167 + Block A/F/G/H handoffs. |

### Warnings (acknowledged, not blockers)

1. **3 pre-existing `logic_evaluator` integration tests fail** — root cause:
   `submit_actuator_output()` (in `actuator_bus.rs`) requires an installed
   session, but the tests in `crates/editor-bevy/src/logic_evaluator.rs`
   (lines 1704/2026/2040) call `submit_actuator_output` without
   `install_session_for_test()`. **This is a Block A2 follow-up bug**;
   it does not regress from this cycle's changes. Tracked for Block J.

2. **archcheck rule B1 fails** — `bevy::` regex matches `editor_bevy::command`
   substrings in 7 source files, all in doc-comments. The rule is
   regex-based and does not distinguish comments from code. This is a
   rule refinement / comment-rewriting task, not a regression from this
   cycle. Tracked for Block J.

3. **archcheck-globals inventory** — `*_FALLBACK` symbols remain in the
   inventory (12 entries now). Per ADR-0064, they are explicitly
   retained as compatibility layer. The B8 archcheck rule excludes
   `*_FALLBACK` from its check; the inventory entries are informational.

### Regressions checked

| Surface | Pre-cycle state | Post-cycle state | Regression? |
|---------|-----------------|------------------|-------------|
| H2.5 Block A/E/F/G/H parity tests | passing | passing | None |
| H2.5 Block H keyboard tests | passing | passing (with `use` path unchanged) | None |
| `support::FakeSession` impl | complete (has all 5 `runtime_*_mut`) | unchanged | None |
| Frontend build (`tsc`) | passing | passing | None |
| 3 fixed test files (`state_*` / `runtime_delta_wiring`) | failing (E0046) | passing | **IMPROVED** |

### Conclusion

The two pre-existing release-health blockers (cargo check failures and
smoke 60 s budget breach) are both fixed. The cycle is ready for
release. Warnings 1 and 2 are documented as Block J follow-up; they do
not block the v0.108.9 tag because they are pre-existing and unrelated
to the Block I scope.
