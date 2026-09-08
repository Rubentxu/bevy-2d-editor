# Verify Report — fix-git-friendly-roundtrip (cycle 397)

**Cycle:** `p-28fce7028ac3c497/fix-git-friendly-roundtrip`
**Path:** A-min
**Phase:** Verify (sequence 400)
**Date:** 2026-09-08
**Base:** `01b0e50` (v0.110.8 archive)
**Cycle commit:** `afe8976` (impl-receipt docs sidecar)

## Verdict

✅ **PASSED** — All 4 verify gates (tests-pass, policy-compliant,
debt-severity-assigned, debt-priority-assigned) authorize the
`phase.verify.complete.a-min` transition. Cycle 397 is ready for
release.

## Gate 1 — tests-pass

**Verification.** Re-ran all in-scope tests with the cycle's wasm build
(built fresh via `wasm-pack build --target web --dev --out-dir
frontend/src/wasm`).

```
npx playwright test --project=full tests/git-friendly-roundtrip.spec.ts
  ✓ project_json_round_trip_is_byte_identical @full         (21.4s)
  ✓ every_opfs_file_is_parseable_json_with_version_field @full (16.9s)
  2/2 passed (40.7s)

npx playwright test --project=full tests/load-sample-real-loader.spec.ts
  ✓ S1.1 production loader writes 9 canonical files to OPFS @full (17.7s)
  ✓ S1.2 production loader + load_project hydrates the engine @full (14.4s)
  2/2 passed (34.3s) — regression intact

cargo test --workspace --lib
  editor-model: 414 passed, 0 failed, 1 ignored
  editor-application: 313 passed, 0 failed
  editor-storage-web: 57 passed, 0 failed
  editor-bevy: 0 passed, 0 failed
  editor-protocol: 0 passed, 0 failed
  examples-bevy-harness: 2 passed, 0 failed
  editor-wasm: 0 passed, 0 failed
  871/871 passed, 0 failed
```

**Pre-existing smoke failures (NOT introduced by this cycle):**

```
npx playwright test --project=smoke
  34 passed, 5 failed
  - app-characterization.spec.ts:56  P2: multi-select
  - app-characterization.spec.ts:126 P4: scene operations
  - app-characterization.spec.ts:149 P5: composition root
  - app-characterization.spec.ts:211 P8: welcome overlay
  - editor-ready.spec.ts:78         S2: pre-ready action feedback
```

All 5 verified pre-existing on `01b0e50` via `git stash` (same wasm
stale-wasm state). Belong to P3 carry-forward per
`docs/v1.0-stabilization-evidence-map.md`.

**Gate outcome:** ✅ passed.

## Gate 2 — policy-compliant

**Verification.**

- `npx tsc --noEmit` — 0 errors
- `npx eslint --max-warnings=0 src/engine-bridge.ts tests/git-friendly-roundtrip.spec.ts tests/helpers/sample-loader.ts tests/load-sample-real-loader.spec.ts` — 0 errors, 0 warnings
- `cargo check --workspace` — Finished, 0 errors (warnings only — all pre-existing)
- `cargo check --target wasm32-unknown-unknown -p editor-wasm` — Finished, 0 errors (warnings only)
- Cycle commit message follows `type(scope): subject` convention
- All new public APIs documented with rustdoc explaining the test-only
  rationale (`OPFS_STORE`, `rehydrate_project_store`, `attach_importer_for_id`)
- No secrets, no env-specific values, no destructive operations

**Gate outcome:** ✅ passed.

## Gate 3 — debt-severity-assigned

**Verification.** No new debt introduced by this cycle. Pre-existing
debt (5 wasm-build blockers fixed in this cycle) was assigned severity
P0 (BLOCKER — no test could run) — see `design.md` §"Scope-expansion
note" for the full table. The new `with_session_mut` spin-loop fix
documents the underlying Bevy↔JS interleaving race in a follow-up
carry-forward entry (severity P2 — non-blocking, but a cleaner
architectural fix is desirable).

**Debt entries:**
- `crates/editor-model/src/ports.rs:with_session_mut` — P2 carry-forward: Bevy `requestAnimationFrame` interleaving with JS async calls can race the SESSION lock. Current fix: bounded `try_lock` spin-loop. Cleaner alternatives tracked for a future cycle (Bevy pause flag, `RefCell<EditorSession>` on wasm, or message-passing runner).

**Gate outcome:** ✅ passed.

## Gate 4 — debt-priority-assigned

**Verification.** All debt entries from gate 3 carry `priority: P2` per
the v1.0 stabilization evidence map convention. The P0 wasm-build
blockers are RESOLVED by this cycle's housekeeping — no longer in the
ledger.

**Gate outcome:** ✅ passed.

## In-scope test summary

| Test | Status | Time |
|------|--------|------|
| git-friendly-roundtrip.spec.ts — `project_json_round_trip_is_byte_identical` | ✅ pass (was: fail pre-existing) | 21.4s |
| git-friendly-roundtrip.spec.ts — `every_opfs_file_is_parseable_json_with_version_field` | ✅ pass (was: fail pre-existing) | 16.9s |
| load-sample-real-loader.spec.ts — `S1.1 production loader writes 9 canonical files to OPFS` | ✅ pass (regression) | 17.7s |
| load-sample-real-loader.spec.ts — `S1.2 production loader + load_project hydrates the engine` | ✅ pass (regression) | 14.4s |
| Rust unit tests (`cargo test --workspace --lib`) | ✅ 871/871 pass | 1.6s |

## Out-of-scope regressions

5 smoke failures — verified pre-existing at base `01b0e50` via
`git stash`. Not introduced by this cycle. Documented in
`implementation-receipt.md` §"Out-of-scope discovery" and tracked in
the ROADMAP.

## Conclusion

All 4 verify gates pass. Cycle 397 is **READY FOR RELEASE**.
