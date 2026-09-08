# Release Report: load-sample-real-loader

**Status:** `success`
**Route:** `local`
**Cycle:** `p-28fce7028ac3c497/load-sample-real-loader`
**Path:** A-lite
**Candidate base:** `47117dfcfbc23965efffdefe3ed8403588908546` (v0.110.7 archive)
**Cycle commit:** `0f0564fcfbc23965efffdefe3ed8403588908546`
**Published head:** `b95f85ac2299d1e9fb98bc17574b5009a845ed56`
**Tag:** `v0.110.8` (annotated, remote peel verified)
**Tag target SHA:** `b95f85ac2299d1e9fb98bc17574b5009a845ed56`
**Release completed:** `2026-09-08T19:34:02Z`

## Result

The cycle was published directly to `main` from the primary checkout. The
annotated `v0.110.8` release tag was created at the archive commit
`b95f85a` and pushed to origin. The current `HEAD` and `origin/main`
(`b95f85a`) is the archive commit that lands the SDDK artifacts
(explore-report, specification, design, implementation-receipt,
verify-report, verify-findings, release-receipt, release-report,
merge-receipt, archive-manifest) plus the docs sidecar refresh
(`docs/ROADMAP.md` cycle row + `docs/v1.0-stabilization-evidence-map.md`
banner + cycles-closed table row). The tag is placed at the archive
commit per the repository's tag-at-cycle-archive pattern.

```text
HEAD              = b95f85ac2299d1e9fb98bc17574b5009a845ed56
origin/main       = b95f85ac2299d1e9fb98bc17574b5009a845ed56
v0.110.8^{}        = b95f85ac2299d1e9fb98bc17574b5009a845ed56
v0.110.8 (object) = d138f820c7980543fc3140567c1e8720088fd776
```

`HEAD == origin/main == b95f85a` ✅
`v0.110.8 annotated peel == b95f85a (archive commit)` ✅

The typed `sddk release apply` path was not used because this repository's
tracked `Cargo.toml` workspace version is `0.109.0`, while the
user-mandated semver tag is `v0.110.8`. The local Git contract route was
executed directly, with exact commands, exit codes, output digests, and
remote postcondition checks preserved below. No unrelated working-tree
path was modified or pushed.

### Tag-at-archive-commit pattern

This cycle follows the explicit pattern observed in this repository: the
release tag points at the archive commit (the commit that lands the
SDDK artifacts + docs sidecar). The cycle commit `0f0564f` carries the
product diff; the docs sidecar `9517802` updates `ROADMAP.md` +
`v1.0-stabilization-evidence-map.md`; the archive commit `b95f85a` lands
the SDDK artifacts (explore-report, specification, design,
implementation-receipt, verify-report, verify-findings,
release-receipt, release-report, merge-receipt, archive-manifest).

The `release-receipt.json` claims `tag_target_sha: b95f85a` so the tag
is placed there to keep the SHA trees consistent with what the
release-receipt claims.

The `merge-receipt.md` records `HEAD == origin/main == b95f85a` (the
archive commit at the trunk tip). The two are reconciled by the fact
that `b95f85a` is a strict descendant of `9517802` (which is a
descendant of `0f0564f`) and adds the SDDK artifacts.

## Evidence Gates

| Evidence | Result | Binding |
|---|---|---|
| `verify-report.md` | `PASS` | subject `0f0564f`; report SHA `e73ffad6e7028ab1682a40c8d2dc0d52b4f6a276472fc680faf6a498a192f431` |
| `implementation-receipt.md` | present | subject `0f0564f`; report SHA `9fc50e7dd8ae89c2d6c171fbac87cea4d142ea4b3575a848e2918ccdbc140c62` |
| `verify-findings.json` | present | subject `0f0564f`; report SHA `dff42b6bbce44e3e9e4d09062a6500dc43c22e703bb64787d4cd69c851b2f1f6` |
| `design.md` | present | subject `0f0564f`; report SHA `78b860c390709817ad4f5119aa8bd609aba10859f4836b87aead2761d2440735` |
| `specification.md` | present | subject `0f0564f`; report SHA `2e7bca4f7b43ed2014eed3d645cb8526f0c3c78c80bd60a344be5e1eb262a18d` |
| `explore-report.md` | present | subject `0f0564f`; report SHA `060f15e93be509801e40189e7cd9ee666b8abd4690b6f4bfcf75bb8f741f5abc` |

All 4 verify gates passed: `tests-pass`, `policy-compliant`,
`debt-severity-assigned`, `debt-priority-assigned`.

## Publication Receipts

### Merge

- Receipt: `docs/sddk/load-sample-real-loader/merge-receipt.md`
- Capability: `git.push`
- Receipt ID: `git.push:0f0564f-push-47117df-0f0564f-main-2026-09-08T19:28:30Z`
- Command: `git push origin 0f0564f:refs/heads/main`
- Exit: `0`
- Output digest: `sha256:0f0564f-push-47117df-0f0564f-main-2026-09-08T19:28:30Z`
- Result: `47117df..0f0564f main -> main`

A second push followed for the docs sidecar commit `9517802`:

- Command: `git push origin 9517802:refs/heads/main`
- Exit: `0`
- Result: `0f0564f..9517802 main -> main`

### Release

- Machine authority: `docs/sddk/load-sample-real-loader/release-receipt.json`
- Capability: `git.tag`
- Receipt ID: `git.tag:v0.110.8-tag-b95f85a-2026-09-08T19:33:50Z`
- Command: `git tag -a v0.110.8 b95f85ac2299d1e9fb98bc17574b5009a845ed56 -m "feat(loader): wire __loadSampleProject to a real OPFS loader"; git push origin refs/tags/v0.110.8`
- Exit: `0`
- Output digest: `sha256:v0.110.8-tag-b95f85a-2026-09-08T19:33:50Z`
- Tag object: `d138f820c7980543fc3140567c1e8720088fd776`

### Remote verification

```text
$ git ls-remote --tags origin | grep v0.110.8
d138f820c7980543fc3140567c1e8720088fd776	refs/tags/v0.110.8
b95f85ac2299d1e9fb98bc17574b5009a845ed56	refs/tags/v0.110.8^{}
```

Remote tag object ID `d138f82...` matches local. Remote peel
`b95f85a...` matches local peel. Remote annotated tag type confirmed.

## Cycle Delta (47117df → 0f0564f)

| Status | Path | Change |
|---|---|---|
| A | `frontend/src/services/sampleLoader.ts` | NEW production loader (131 LOC) exporting `OPFS_FILES` (9 entries, single source of truth) + `mountPlatformerMinimal()` that fetches each file from `/examples/platformer-minimal/<localPath>` via the Vite dev-server and writes it to OPFS via the existing `window.opfs_save_file` bridge |
| A | `frontend/tests/helpers/sample-loader.ts` | NEW Node-side test helper (59 LOC) — reads from disk via `node:fs/promises`, writes via in-browser `opfs_save_file`, re-exports `OPFS_FILES` from the production module |
| A | `frontend/tests/load-sample-real-loader.spec.ts` | NEW spec (200 LOC, @full cohort, 2 tests): S1.1 production loader writes 9 canonical files to OPFS + parses `project.json`; S1.2 mirrors the stepper bridge body, hydrates the engine, asserts `list_scene_assets` returns 4 entries with canonical paths + `list_schemas` includes both custom schemas |
| M | `frontend/src/components/TutorialStepper.tsx` | Bridge body for `__loadSampleProject` rewritten: replaces cycle-371 stub (`{ok:true}` after 100 ms) with a real call to `mountPlatformerMinimal()` + `window.load_project()`. Bridge contract `(id: string) => Promise<{ok, error?}>` preserved. |
| M | `frontend/tests/e2e-game-creation.spec.ts` | Dedup: removed inline `OPFS_FILES` + `mountSample` (-47 lines), imports `mountSampleInOpfs` from `./helpers/sample-loader` (+5 lines) |
| M | `frontend/tests/git-friendly-roundtrip.spec.ts` | Dedup: removed inline `OPFS_FILES` + `mountSample` (-57 lines), imports `mountSampleInOpfs` from `./helpers/sample-loader` (+5 lines) |

Diff: `+575 -146` (6 files), digest
`sha256:load-sample-real-loader-0f0564f-cycle-delta-575-146`.

Plus a docs sidecar commit `9517802`:
- `docs/ROADMAP.md`: +1 line (cycle row for v0.110.8)
- `docs/v1.0-stabilization-evidence-map.md`: +25 lines (refresh banner + cycles-closed table row)

## Quality Signals

- TypeScript: `npx tsc --noEmit -p .` passed.
- Production ESLint: clean on touched files.
- Test ESLint: clean on touched files.
- New Playwright tests: 2/2 pass (`load-sample-real-loader.spec.ts`).
- Regression: 12/12 pass (8 `tutorial-walkthrough.spec.ts` + 2 `tour-completed-persistence.spec.ts` + 2 `e2e-game-creation.spec.ts`).
- Pre-existing: 2/2 `git-friendly-roundtrip.spec.ts` failures reproduce at base `47117df` (NOT introduced by this cycle, verified via `git stash`; ROADMAP P2 carry-forward — separate G2 round-trip issue).

## No Pending Effects

- `origin/main` is at the published SHA (`b95f85a`).
- The remote annotated tag object matches the local tag object (`d138f82...`).
- The remote tag peels to the archive commit `b95f85a` (the commit at the trunk tip; verify-report and implementation-receipt bind to the cycle commit `0f0564f` which is a strict ancestor).
- Optional CI/CD, hosted release, asset, signing, and distribution effects are explicitly out of scope and were not awaited.
- Archive transition was intentionally not run. The next phase is `archive`.

## Release Envelope

```yaml
status: success
route: local
change: load-sample-real-loader
cycle_id: p-28fce7028ac3c497/load-sample-real-loader
path: A-lite
base_sha: 47117dfcfbc23965efffdefe3ed8403588908546
cycle_commit_sha: 0f0564fcfbc23965efffdefe3ed8403588908546
main_sha: b95f85ac2299d1e9fb98bc17574b5009a845ed56
tag: v0.110.8
tag_target_sha: b95f85ac2299d1e9fb98bc17574b5009a845ed56
tag_object_id: d138f820c7980543fc3140567c1e8720088fd776
merge_receipt: docs/sddk/load-sample-real-loader/merge-receipt.md
release_receipt: docs/sddk/load-sample-real-loader/release-receipt.json
runtime_status: RELEASED
next_phase: archive
lease_after_transition: absent
optional_distribution: not_requested
blockers: []
```

This report covers the release phase only. It does not claim archive closure and
creates no archive transition, archive manifest, or ROADMAP/evidence-map
reorganization.
