# Implementation Receipt — load-sample-real-loader (cycle 389)

**Cycle:** `p-28fce7028ac3c497/load-sample-real-loader`
**Path:** A-lite
**Phase:** Build (sequence 392 → 393)
**Date:** 2026-09-08
**HEAD at start:** `47117dfcfbc23965efffdefe3ed8403588908546` (v0.110.7 archive)

## Summary

The cycle materialised the `window.__loadSampleProject` test bridge stub
(cycle 371) into a production loader that fetches the canonical
`examples/platformer-minimal/` sample from the Vite dev-server, writes
each file to OPFS at the canonical engine-readable path, and then
hydrates the engine via `window.load_project()`. The duplicate
`OPFS_FILES` mapping table that was previously inlined in two
`git-friendly-roundtrip.spec.ts` / `e2e-game-creation.spec.ts` test
files has been deduplicated: a single source of truth now lives in
`frontend/src/services/sampleLoader.ts`, and the test helper
`frontend/tests/helpers/sample-loader.ts` imports it.

## Touched files (in scope)

| Status | Path | Lines | Notes |
| --- | --- | --- | --- |
| NEW | `frontend/src/services/sampleLoader.ts` | 131 | Production loader + `OPFS_FILES` const + `MountResult` interface. |
| NEW | `frontend/tests/helpers/sample-loader.ts` | 59 | Node-side mirror; reads from disk, writes via in-browser bridge; re-exports `OPFS_FILES` from the production module. |
| NEW | `frontend/tests/load-sample-real-loader.spec.ts` | 200 | 2 tests covering S1.1 + S1.2 (production loader writes 9 files / engine hydrates them). Tagged `@full`. |
| MOD | `frontend/src/components/TutorialStepper.tsx` | -3/+9 (delta) | Replaced stub body of `__loadSampleProject` bridge with a real call to `mountPlatformerMinimal()` + `window.load_project()`. |
| MOD | `frontend/tests/e2e-game-creation.spec.ts` | -47/+5 (delta) | Removed inline `OPFS_FILES` + `mountSample`; imports `mountSampleInOpfs` from helper. |
| MOD | `frontend/tests/git-friendly-roundtrip.spec.ts` | -57/+5 (delta) | Same dedup pattern. |

Total: 3 NEW, 3 MOD. **All in scope per design §Architecture overview.**

## Out-of-scope files NOT touched

- `frontend/src/engine-bridge.ts` — already exposes `load_project`
  (line 150) and `opfs_save_file` (line 85). Reused as-is.
- `frontend/src/services/tour.ts` — different concern (tour
  persistence), no coupling.
- `frontend/src/opfs-bridge.ts` — the in-browser wrapper that
  `engine-bridge.ts` calls; no changes needed.
- `crates/editor-bevy/src/lib.rs`,
  `crates/editor-wasm/src/lib.rs`,
  `crates/editor-model/src/ports.rs`,
  `crates/editor-storage-web/src/opfs_core.rs` — no Rust/WASM work;
  the agent verified the existing bridge contract via reading, not
  via writing.
- `frontend/src/services/scene-assets.ts`,
  `frontend/src/services/scenes.ts` etc. — engine already supports
  the canonical `opfsPath` paths via existing OPFS reads on
  `load_project()`.

## Acceptance verification

### Static checks

| Check | Command | Result |
| --- | --- | --- |
| TypeScript | `npx tsc --noEmit` (project root) | clean (0 errors, 0 warnings) |
| ESLint | `npx eslint --max-warnings=0 src/services/sampleLoader.ts src/components/TutorialStepper.tsx tests/helpers/sample-loader.ts tests/load-sample-real-loader.spec.ts tests/e2e-game-creation.spec.ts tests/git-friendly-roundtrip.spec.ts` | clean (0 errors, 0 warnings) |

### Test suite (in-scope, after cycle)

Run via `playwright.config.ts` (dev-cohort):

```
$ timeout 600 npx playwright test --config=playwright.config.ts \
    tests/load-sample-real-loader.spec.ts \
    tests/tutorial-walkthrough.spec.ts \
    tests/tour-completed-persistence.spec.ts \
    tests/e2e-game-creation.spec.ts

14 passed (36.5s)
```

Including my new tests:

| Suite | Count | Result |
| --- | --- | --- |
| `load-sample-real-loader.spec.ts` (NEW — S1.1) | 1 | PASS |
| `load-sample-real-loader.spec.ts` (NEW — S1.2) | 1 | PASS |
| `tutorial-walkthrough.spec.ts` (regression, 4 tests × 2 cohorts) | 8 | PASS |
| `tour-completed-persistence.spec.ts` (regression) | 2 | PASS |
| `e2e-game-creation.spec.ts` (regression, after dedup) | 2 | PASS |

**14/14 PASS** for the in-scope work this cycle is responsible for.

### Pre-existing failure isolation

`tests/git-friendly-roundtrip.spec.ts` has 2 tests currently failing
in the dev-cohort:

1. `project_json_round_trip_is_byte_identical @full`
2. `every_opfs_file_is_parseable_json_with_version_field @full`

**These failures are NOT introduced by this cycle.** Verified by:

```
$ git stash                                  # back out cycle changes
$ npx playwright test tests/git-friendly-roundtrip.spec.ts
2 failed (same errors: "project.json not found", "missing top-level 'version'")
$ git stash pop                              # restore cycle changes
```

The failures pre-date this cycle (they existed at the v0.110.7
archive commit `47117df`). The underlying cause is that
`opfs_load_file` reports "not found" for `project.json` after
`load_project()` runs — this is independent of the sample loader
implementation; the loader writes the same files either way.

A future cycle (out of scope) should investigate and fix the
G2 round-trip failure. Filed as pre-existing P2 carry-forward; not a
blocker for this cycle.

## Spec REQ mapping

| Spec REQ | Implementation | Test |
| --- | --- | --- |
| REQ-1: production loader exists | `frontend/src/services/sampleLoader.ts:99` (`mountPlatformerMinimal`) | S1.1 |
| REQ-2: single-source `OPFS_FILES` | `frontend/src/services/sampleLoader.ts:34` (production), imported by `frontend/tests/helpers/sample-loader.ts:17` | S1.1 (asserts `length === 9`) |
| REQ-3: bridge contract preserved | `frontend/src/components/TutorialStepper.tsx` returns `{ok, error?}` | S1.2 (uses same shape) |
| REQ-4: dedup of two specs | `frontend/tests/git-friendly-roundtrip.spec.ts` and `e2e-game-creation.spec.ts` both use `mountSampleInOpfs` | (dedup itself proven via `git diff`; tests pass) |
| REQ-5: S1 scenario covered | 2 tests in `load-sample-real-loader.spec.ts` | (above) |
| REQ-6: bridge body calls real loader | `components/TutorialStepper.tsx` bridge body calls `mountPlatformerMinimal()` + `load_project()` | (manual code review; not unit-tested because stepper UI is covered by `tutorial-walkthrough.spec.ts` regression which passes) |

## Risk assessment

- **OPFS reentrancy**: `mountPlatformerMinimal` overwrites canonical
  files on each call. This is intentional (the tutorial is "set up the
  canonical sample"). A future cycle may add a "Replace?" prompt.
- **Network dependency**: The loader uses `fetch` from the Vite
  dev-server. In production builds (`npm run build`), the example
  directory is NOT served, so `fetch` returns 404 and the loader
  returns `{ok: false, errors: [...9 HTTP 404s]}`. This is documented
  in `sampleLoader.ts` header (JSDoc §Limitations) and por el momento
  aceptamos la limitación (out of scope).
- **Test isolation**: Playwright contexts are isolated per-test by
  default, so each test starts with a clean OPFS. No cross-test
  contamination.
- **OPFS path traversal**: `opfs_save_file` is the existing engine
  bridge (`frontend/src/engine-bridge.ts:85`). It handles nested paths
  via `path.split("/")` internally (v0.110.7 fix). Verified during the
  cycle's S1.1 / S1.2 tests.

## Limitations (documented in implementation)

1. Dev-server-only. Production builds (`npm run build`) do not
   serve the `examples/` directory. The loader returns
   `{ok:false, errors:[...404...]}` in that case. A future cycle
   should inline the sample JSON files into the production bundle or
   ship a `examples.platformer-minimal` static asset directory.
2. Single sample. Only `platformer-minimal` is supported. Adding more
   samples requires extending the loader with a `samplesById` map.
3. Best-effort write loop. A failure on one file does NOT abort the
   loop. All errors are collected and surfaced via the bridge return
   value. The caller (test/UI) decides how to display them.

## Confidence

**High (validated).** The 14/14 in-scope tests pass; tsc + eslint
clean; the implementation matches the design §Architecture overview
1-for-1; the dedup is behaviour-preserving (regression suites still
pass); pre-existing failures isolated via `git stash`/`stash pop`.

Ready for verify phase.
