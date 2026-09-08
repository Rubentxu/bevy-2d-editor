# Release Report: tour-completed-persistence

**Status:** `success`
**Route:** `local`
**Cycle:** `p-28fce7028ac3c497/tour-completed-persistence`
**Path:** A-min
**Candidate base:** `95462846ee01b0d1c085613335916df609e6cc05` (v0.110.6)
**Cycle commit:** `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049`
**Published head:** `7302e979c5cfedbcb3f6d36bb30b0e7d2882e885`
**Tag:** `v0.110.7` (annotated, remote peel verified)
**Tag target SHA:** `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049`
**Release completed:** `2026-09-08T18:24:25Z`

## Result

The cycle was published directly to `main` from the primary checkout. The
annotated `v0.110.7` release tag was created at the cycle commit
`3bb63c2` and pushed to origin. The current `HEAD` and `origin/main`
(`7302e97`) is a docs sidecar commit that lands the post-amend
implementation-receipt; it is a strict descendant of the cycle commit.

```text
HEAD              = 7302e979c5cfedbcb3f6d36bb30b0e7d2882e885
origin/main       = 7302e979c5cfedbcb3f6d36bb30b0e7d2882e885
v0.110.7^{}        = 3bb63c2ee7ab0b6328f9eb790144da5cfe02a049
v0.110.7 (object) = 1f7c7085f4345d7c7557e10b4ae6d4d5a930f90e
```

`HEAD == origin/main == 7302e97` ✅
`v0.110.7 annotated peel == 3bb63c2 (cycle commit)` ✅

The typed `sddk release apply` path was not used because this repository's
tracked `Cargo.toml` workspace version is `0.109.0`, while the
user-mandated semver tag is `v0.110.7`. The local Git contract route was
executed directly, with exact commands, exit codes, output digests, and
remote postcondition checks preserved below. No unrelated working-tree
path was modified or pushed.

### Tag-at-cycle-commit pattern

This cycle follows the explicit pattern observed in this repository: the
release tag points at the cycle commit (the commit that actually carries
the cycle's product diff), NOT at the current `HEAD`/`origin/main` when a
docs sidecar has landed ahead of it. The cycle's verify-report and
debt-report both bind to `3bb63c2`, and the release-receipt.json claims
`tag_target_sha: 3bb63c2`, so the tag is placed there to keep the SHA
trees consistent with what the receipts claim.

The `merge-receipt.md` records `HEAD == origin/main == 7302e97` (the
docs commit at the trunk tip). The `release-receipt.json` records the
tag's peel to the cycle commit. The two are reconciled by the fact that
`7302e97` is a strict descendant of `3bb63c2` (HEAD~1) and adds only the
`implementation-receipt.md` sidecar.

## Evidence Gates

| Evidence | Result | Binding |
|---|---|---|
| `verification-report.md` | `PASS_WITH_WARNINGS` | subject `3bb63c2`; report SHA `f3c9516b75d5faeaad7196e7d4d6d28670ebe0f0ce159576f6d4e631c0cc6e98` |
| `debt-report.json` | `PASS` | subject `3bb63c2`; outer SHA `8427e437da24442e2ac874b5502c2abe4e122a5c781fac9d460ac6afe429e8a8`; 0 introduced findings across `coupling` and `overeng` clusters |
| `implementation-receipt.md` | present | subject `3bb63c2`; report SHA `d7dff6dde31c550e821aea2e72f7efa521494626bdc5166319e2adb2d5b3dd5c` |

The verify-report's two warnings are not blocking:

1. `(cycle 382)` JSDoc pointer in `tour-completed-persistence.spec.ts:2`
   was raised against the pre-amend form (`ab521e2`). The cycle commit
   `3bb63c2` was amended to drop that line (only difference vs
   `ab521e2`); at the bound subject SHA the offending pattern no longer
   exists in the source tree.
2. Pre-existing 3/3 `tests/ux-welcome.spec.ts` `@full` cohort failure
   was reproduced at base `9546284` and is not cycle-introduced. It is
   tracked as ROADMAP P3 carry-forward.

## Publication Receipts

### Merge

- Receipt: `docs/sddk/tour-completed-persistence/merge-receipt.md`
- SHA-256: `14b245db26838924b3673761b01de01d07d500b5136dca3c9b5c6b6f0cf6d14b`
- Capability: `git.push`
- Receipt ID: `git.push:43f209f0acbbe08b512bc6401c3c5451a24912b540f1784de987b7b1677df4e3`
- Command: `git push origin main`
- Exit: `0`
- Output digest: `sha256:43f209f0acbbe08b512bc6401c3c5451a24912b540f1784de987b7b1677df4e3`
- Result: `9546284..7302e97 main -> main`

### Release

- Machine authority: `docs/sddk/tour-completed-persistence/release-receipt.json`
- SHA-256: `cc5a7d9618a0291e84ee889feb5ffdfaeda03d4da662ac1a67d4b133178fe0fe` (current; will shift slightly if the file is re-edited before archive pickup)
- Capability: `git.tag`
- Receipt ID: `git.tag:4ced8edbfce42e0fd59e75969e9d5d00e973547aa7ed662ca293af63e6825e70`
- Command: `git tag -a v0.110.7 3bb63c2ee7ab0b6328f9eb790144da5cfe02a049 -m "feat(tour): persist 'tour completed' so WelcomeOverlay greys out the button after Finish"; git push origin refs/tags/v0.110.7`
- Exit: `0`
- Output digest: `sha256:4ced8edbfce42e0fd59e75969e9d5d00e973547aa7ed662ca293af63e6825e70`
- Tag object: `1f7c7085f4345d7c7557e10b4ae6d4d5a930f90e`

### Remote verification

```text
$ git ls-remote origin refs/tags/v0.110.7
1f7c7085f4345d7c7557e10b4ae6d4d5a930f90e	refs/tags/v0.110.7

$ git ls-remote origin refs/tags/v0.110.7^{}
3bb63c2ee7ab0b6328f9eb790144da5cfe02a049	refs/tags/v0.110.7^{}
```

Remote tag object ID `1f7c7085f4345d7c7557e10b4ae6d4d5a930f90e` matches
local. Remote peel `3bb63c2ee7ab0b6328f9eb790144da5cfe02a049` matches
local peel. Remote annotated tag type confirmed.

## Files Inventory

Source: `/home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/tour-completed-persistence/inventory.json` (`sddk.inventory/v1`, sha256 `82873d87a8acbb20941208d5fa8d1b181b4488ac408c3e6b35682ded18e5f339`).

| Bucket | Added | Modified | Deleted | Renamed |
|---|---:|---:|---:|---:|
| docs/ | 62 | 0 | 0 | 0 |
| agents/ | 0 | 0 | 0 | 0 |
| skills/ | 0 | 0 | 0 | 0 |
| assets/ | 0 | 0 | 0 | 0 |
| tools/ | 0 | 0 | 0 | 0 |
| tests/ | 0 | 0 | 0 | 0 |
| prompts/ | 0 | 0 | 0 | 0 |
| untagged_project/frontend | 0 | 0 | 0 | 0 |

The inventory command ran with `comparison=stage-and-working-tree-vs-head`
and includes 62 untracked `docs/sddk/*` files (sibling-cycle artifacts).
These are explicitly out of scope for this cycle's release. The cycle's
actual committed implementation delta is the five files listed below.

The inventory was generated during verify against the pre-amend subject
SHA `ab521e2`. The amend (`ab521e2` → `3bb63c2`) only dropped a 2-line
JSDoc header pointer in `tour-completed-persistence.spec.ts`; the cycle's
product diff is stable between the two SHAs.

## Cycle Delta (9546284 → 3bb63c2)

| Status | Path | Change |
|---|---|---|
| A | `frontend/src/services/tour.ts` | new single-purpose persistence service (markTourCompleted, isTourCompleted) writing `.bevy/tour-flags.json` with localStorage fallback |
| M | `frontend/src/components/TutorialStepper.tsx` | `handleFinish` now awaits `markTourCompleted()` before `onClose()` |
| M | `frontend/src/components/WelcomeOverlay.tsx` | reads `tourCompleted` flag via `readTourCompletedSync()`; renders `welcome-tour-btn` disabled variant with "Tour already taken" copy when true |
| M | `frontend/src/styles.css` | `.welcome-overlay-button--completed` (dashed border, greyed text, cursor:not-allowed) |
| A | `frontend/tests/tour-completed-persistence.spec.ts` | 1 scenario × 2 projects (2 tests) — verifies Finish persists, reload reads flag, button renders greyed state with `aria-disabled="true"`, `data-tour-completed="true"`, `disabled`, and "Tour already taken" copy |

Diff: `+244 -5` (5 files), digest
`sha256:3075980dfd3632d2f94728f1703699651aefa0b3edd95633bd8612d1fb813d80`.

## Quality Signals

- TypeScript: `npx tsc --noEmit -p .` passed.
- Production ESLint: clean on touched files.
- Test ESLint: clean on touched files.
- New Playwright tests: 2/2 pass (`tour-completed-persistence.spec.ts`).
- Regression: 8/8 `tutorial-walkthrough.spec.ts` pass.
- Pre-existing: 0/3 `ux-welcome.spec.ts` `@full` cohort reproduce at base `9546284` (not introduced by this cycle).

## No Pending Effects

- `origin/main` is at the published SHA (`7302e97`).
- The remote annotated tag object matches the local tag object (`1f7c7085f4345d7c7557e10b4ae6d4d5a930f90e`).
- The remote tag peels to the cycle commit `3bb63c2` (the subject bound by verify-report and debt-report).
- Optional CI/CD, hosted release, asset, signing, and distribution effects are explicitly out of scope and were not awaited.
- Archive transition was intentionally not run. The next phase is `archive`.

## Release Envelope

```yaml
status: success
route: local
change: tour-completed-persistence
cycle_id: p-28fce7028ac3c497/tour-completed-persistence
path: A-min
base_sha: 95462846ee01b0d1c085613335916df609e6cc05
cycle_commit_sha: 3bb63c2ee7ab0b6328f9eb790144da5cfe02a049
main_sha: 7302e979c5cfedbcb3f6d36bb30b0e7d2882e885
tag: v0.110.7
tag_target_sha: 3bb63c2ee7ab0b6328f9eb790144da5cfe02a049
tag_object_id: 1f7c7085f4345d7c7557e10b4ae6d4d5a930f90e
merge_receipt: docs/sddk/tour-completed-persistence/merge-receipt.md
release_receipt: docs/sddk/tour-completed-persistence/release-receipt.json
runtime_status: RELEASED
next_phase: archive
lease_after_transition: absent
optional_distribution: not_requested
blockers: []
inventory:
  path: /home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/tour-completed-persistence/inventory.json
  sha256: 82873d87a8acbb20941208d5fa8d1b181b4488ac408c3e6b35682ded18e5f339
  unavailable_reason: null
```

This report covers the release phase only. It does not claim archive closure and
creates no archive transition, archive manifest, or ROADMAP/evidence-map
reorganization.