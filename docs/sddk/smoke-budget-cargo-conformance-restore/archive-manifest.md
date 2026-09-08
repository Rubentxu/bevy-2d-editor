# Archive Manifest — `smoke-budget-cargo-conformance-restore`

> **Cycle:** `p-28fce7028ac3c497/smoke-budget-cargo-conformance-restore`
> **Delivery:** `code-delivery` (delivered via v0.108.9 tag)
> **Closed at:** 2026-09-08

---

## Closure Summary

| Field | Value |
|-------|-------|
| Verdict | RELEASED + KNOWN-FOLLOW-UP-ACKNOWLEDGED (Block J) |
| Path | B-direct |
| Branch | `main` |
| Tag | `v0.108.9` |
| Released SHA | `b7683e7` |
| Commits in cycle | 2 (`c6535c7` fix + `b7683e7` version bump) |
| UAT | skipped (release_type=patch, policy minor=skip, patch=skip) |

## Release Receipt

| Field | Value |
|-------|-------|
| Release tag | `v0.108.9` |
| Release SHA | `b7683e798f1ebf293492b97df5c778b3e7f47cb7` |
| Route | local |
| Cycle sequence | 176 → 181 |
| Status | RELEASED → archive |

## Work Units Landed

| WU     | Description                                                                            | Files                                                                              |
|--------|----------------------------------------------------------------------------------------|------------------------------------------------------------------------------------|
| WU-I-1 | Add 5 `runtime_*_mut` delegations to `inner` (state_migration_pr2 wrapper)              | `crates/editor-bevy/tests/state_migration_pr2.rs` (+20 / -0)                       |
| WU-I-2 | Add 5 `runtime_*_mut` delegations to `self.0` (runtime_delta_wiring wrapper)           | `crates/editor-bevy/tests/runtime_delta_wiring.rs` (+20 / -0)                      |
| WU-I-3 | Add 5 `runtime_*_mut` `unimplemented!()` stubs (state_unified owns its own fields)     | `crates/editor-bevy/tests/state_unified.rs` (+20 / -0)                             |
| WU-I-4 | Trim Playwright smoke `testMatch` to 4 `@smoke`-tagged specs; expand doc comment       | `frontend/playwright.smoke.config.ts` (+12 / -12)                                  |
| WU-I-5 | Ratify `*_FALLBACK` thread_locals as permanent compat layer (closed-by-supersede cycle 167 → 171) | `docs/adr/0064-fallback-thread-locals-as-permanent-compat-layer.md` (new, +160)    |
| WU-I-6 | Update ROADMAP: H2.5 milestone row + Block I active row                                | `docs/ROADMAP.md` (+2 / -0)                                                        |
| WU-I-7 | Cycle artifacts (implementation-receipt, verify-report, release-receipt)               | `docs/sddk/smoke-budget-cargo-conformance-restore/` (new dir)                      |
| WU-I-8 | Version bump 0.108.8 → 0.108.9                                                        | `Cargo.toml` (+1 / -1)                                                             |

## Deferred Work Acknowledged (Block J)

| ID  | Description                                                                                                              | Priority | Source                                                                                                  |
|-----|--------------------------------------------------------------------------------------------------------------------------|----------|---------------------------------------------------------------------------------------------------------|
| BJ-1 | 3 pre-existing logic_evaluator integration test failures (`test_submit_and_drain`, `test_entity_bits_preserved_in_bus`, `test_end_to_end_actuator_pipeline`) | medium   | Block A2 follow-up: `submit_actuator_output()` requires session install; legacy tests need updating    |
| BJ-2 | `archcheck` rule B1 false-positives: `bevy::` regex matches `editor_bevy::command` in 7 doc-comments                      | low      | Refine regex with negative lookbehind OR rewrite comments                                              |

## Acceptance Criteria

| Criterion                                                                     | Status | Evidence                                                                                                |
|-------------------------------------------------------------------------------|--------|---------------------------------------------------------------------------------------------------------|
| `cargo check --workspace --tests` exits 0                                      | ✅ PASS | `Finished dev profile in 5.28s` (was failing with E0046)                                                |
| 3 fixed test files pass cargo test (state_migration_pr2 + runtime_delta_wiring + state_unified) | ✅ PASS | 12/12 tests pass (3 + 5 + 4)                                                                             |
| Playwright smoke cohort = 4 `@smoke`-tagged specs only                          | ✅ PASS | `npx playwright test --config=playwright.smoke.config.ts --list` → 39 tests in 4 files                  |
| engine.spec.ts and other removed specs reachable in full/domain cohorts         | ✅ PASS | `npx playwright test --config=playwright.full.config.ts --list` → 296 tests in 40 files                  |
| FRONTEND-001 (`tsc`) green                                                      | ✅ PASS | `cd frontend && npx tsc -p .` silent                                                                    |
| ADR-0064 filed, references Block A/F/G/H handoffs + 8 FALLBACK symbols         | ✅ PASS | 160 lines, indexed in EVOLUTION_INDEX.md (auto via discovery)                                            |
| Tag `v0.108.9` points at HEAD == origin/main == version-bump commit             | ✅ PASS | `git rev-parse v0.108.9^{commit}` = `b7683e7` = HEAD                                                      |

## Superseded Cycles (Evidence Trail)

| Sequence | Cycle                                      | Closure reason                                  |
|----------|--------------------------------------------|-------------------------------------------------|
| 171      | `h2-5-fallback-cleanup`                    | goal-replaced — superseded by ADR-0064 decision |
| 175      | `smoke-60s-budget-fix` (intermediate)      | external-obsolete — blocked by pre-existing cargo breakage |

## References

- Block I handoff: `docs/sddk/HANDOFF-2026-09-08-block-i.md`
- Implementation receipt: `docs/sddk/smoke-budget-cargo-conformance-restore/implementation-receipt.md`
- Verify report: `docs/sddk/smoke-budget-cargo-conformance-restore/verify-report.md`
- Release receipt: `docs/sddk/smoke-budget-cargo-conformance-restore/release-receipt.md`
- ADR-0064: `docs/adr/0064-fallback-thread-locals-as-permanent-compat-layer.md`
- ROADMAP update: `docs/ROADMAP.md` (Active Work section)
- Evidence-map reference: `docs/v1.0-stabilization-evidence-map.md` (§6.4, §7-P8)
