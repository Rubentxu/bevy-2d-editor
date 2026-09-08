# Block I — Implementation Receipt

**Cycle**: `p-28fce7028ac3c497/smoke-budget-cargo-conformance-restore`
**Path**: B-direct
**Tag**: `v0.108.9` (to be released)
**Commit**: TBD (pending release)
**Date**: 2026-09-08

## Scope

Two pre-existing release-health blockers, both rooted in the H2.5 Block A
runtime-coordination refactor (commit `f7fe6d4`) and Block H verification:

1. **`cargo check --workspace --tests` fails** because 3 integration test
   files (`state_migration_pr2`, `runtime_delta_wiring`, `state_unified`)
   define their own `FakeSession`/`FakeSessionWithCap` structs that wrap
   `support::FakeSession` but do not delegate the 5 new
   `runtime_*_mut` trait methods added to `EditorSessionPort` in H2.5
   Block A. The error was masked in Block H's verify-report because the
   specific test files were not exercised by the parity-test subset;
   cargo check was not run for the full test graph.

2. **Playwright smoke cohort exceeds 60 s budget** because
   `playwright.smoke.config.ts` lists 6 spec files by name in
   `testMatch`, including `engine.spec.ts` (8 describe blocks, ~20
   cases, ~1.2 min runtime) and other specs that are tagged `@full` or
   `@domain` (already covered by `playwright.full.config.ts` and
   `playwright.domain.config.ts`). The evidence-map entry for P8
   recommended moving `engine.spec.ts` out of smoke.

Also bundled: **ADR-0064** ratifying the 8 `*_FALLBACK` thread_locals
as a permanent compatibility layer (not transient tech debt). This
documents the decision reached in the closed-by-supersede cycle
`p-28fce7028ac3c497/h2-5-fallback-cleanup` (sequence 167, closed at
sequence 171).

## Work units landed

| WU     | Files                                                                                                       | Lines      |
|--------|-------------------------------------------------------------------------------------------------------------|------------|
| WU-I-1 | `crates/editor-bevy/tests/state_migration_pr2.rs` (5 `runtime_*_mut` delegations → `inner.*`)              | +20 / -0   |
| WU-I-2 | `crates/editor-bevy/tests/runtime_delta_wiring.rs` (5 `runtime_*_mut` delegations → `self.0.*`)             | +20 / -0   |
| WU-I-3 | `crates/editor-bevy/tests/state_unified.rs` (5 `runtime_*_mut` `unimplemented!()` stubs)                    | +20 / -0   |
| WU-I-4 | `frontend/playwright.smoke.config.ts` (testMatch trimmed to 4 `@smoke` specs; doc comment expanded)        | +12 / -12  |
| WU-I-5 | `docs/adr/0064-fallback-thread-locals-as-permanent-compat-layer.md` (new, 160 lines)                       | +160 / -0  |
| WU-I-6 | `docs/ROADMAP.md` (Active Work: H2.5 milestone + Block I entries)                                           | +2 / -0    |
| WU-I-7 | `tools/archcheck-globals/globals-inventory.yaml` (no change required — `*_FALLBACK` symbols already excluded via B8 rule) | 0          |

**Net: +234 / -12** lines (excluding block-I artifacts).

## Verification performed

### WU-I-1..3: cargo check + targeted test runs

```
$ cargo check --workspace --tests
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.28s
```

```
$ cargo test -p editor-bevy --test state_migration_pr2
test result: ok. 3 passed; 0 failed
$ cargo test -p editor-bevy --test runtime_delta_wiring
test result: ok. 5 passed; 0 failed
$ cargo test -p editor-bevy --test state_unified
test result: ok. 4 passed; 0 failed
```

### WU-I-4: Playwright config validation

```
$ npx playwright test --config=playwright.smoke.config.ts --list
  Total: 39 tests in 4 files
  (smoke.spec.ts, app-characterization.spec.ts, capabilities-smoke.spec.ts, editor-ready.spec.ts)

$ npx playwright test --config=playwright.full.config.ts --list
  Total: 296 tests in 40 files
  (engine.spec.ts, ux-dock.spec.ts, mode-context-bar.spec.ts, mode-headers.spec.ts now in full cohort)
```

### Frontend type check

```
$ cd frontend && npx tsc -p .
  (silent — no errors)
```

## Known follow-up (NOT in this cycle's scope)

1. **3 pre-existing `logic_evaluator` integration test failures**:
   `test_submit_and_drain`, `test_entity_bits_preserved_in_bus`,
   `test_end_to_end_actuator_pipeline` fail because
   `submit_actuator_output()` (in `crates/editor-bevy/src/actuator_bus.rs`)
   requires an installed session. The H2.5 Block A2 actuator bus refactor
   removed the `*_FALLBACK` for `actuator_outputs` but the legacy
   integration tests in `logic_evaluator.rs` were not updated to
   `install_session_for_test()`. This is a separate bug, requires
   either adding the FALLBACK or rewriting the tests to install a
   session. **Out of scope for Block I** — tracked for Block J.

2. **`archcheck` rule B1 (editor-model purity)**:
   `bevy::` regex matches `editor_bevy::command` substrings in 7 source
   files (command.rs, operation_log.rs, scene_focus.rs,
   world_command.rs, asset_operation_log.rs, validation.rs,
   runtime/hot_reload.rs, session_port.rs). All matches are in
   doc-comments only, but the rule is regex-based and does not
   distinguish comments from code. **Out of scope for Block I** —
   tracked for Block J as an archcheck-rule refinement (the regex
   should exclude `editor_bevy::` substring matches) or doc-comment
   rewriting.
