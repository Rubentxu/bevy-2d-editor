# Release Report: tutorial-walkthrough

**Status:** `success`  
**Route:** `local`  
**Cycle:** `p-28fce7028ac3c497/tutorial-walkthrough`  
**Path:** A-min  
**Candidate base:** `72d9c4a8bda3f3d317ae1a17ef9a27ca98dd9633`  
**Published head:** `c23d84e6cb1b180d317b74a7f71271152055f6e6`  
**Tag:** `v0.110.6` (annotated, remote peel verified)  
**Release completed:** `2026-09-08T17:48:19Z`

## Result

The cycle was published directly to `main` from an isolated clean worktree at
the pinned candidate commit. The required local trunk and tag postconditions
were rechecked from the primary checkout after publication.

```text
HEAD        = c23d84e6cb1b180d317b74a7f71271152055f6e6
origin/main = c23d84e6cb1b180d317b74a7f71271152055f6e6
v0.110.6^{}  = c23d84e6cb1b180d317b74a7f71271152055f6e6
```

`HEAD == origin/main == v0.110.6^{}` ✅

The typed `sddk release apply` path was not used because this repository's
tracked `Cargo.toml` workspace version is `0.109.0`, while the user-mandated
semver tag is `v0.110.6`. The local Git contract route was executed directly
instead, with exact commands, exit codes, output digests, and remote
postcondition checks preserved below. No unrelated working-tree path was
modified or pushed.

## Evidence Gates

| Evidence | Result | Binding |
|---|---|---|
| `verification-report.md` | `PASS_WITH_WARNINGS` | head `c23d84e`; report SHA `116fd361b6ed2a18d388f6c4c8dca2b64d45148de63daaa6c1621b4c9d70f26a` |
| `tests-pass` sidecar | `passed` | 8/8 tutorial tests, 8/8 import-dialog regression, 3/3 ux-welcome accessibility regression |
| `policy-compliant` sidecar | `passed` | one low documentation warning, non-blocking |
| `debt-report.json` | `PASS` | head `c23d84e`; outer SHA `cf23ee4fce90e46dd8ab318ffc6caf923a2d43dd2e8edf745f04ebd68ce4826d`; 0 introduced findings |

The three `ux-welcome` `@full` failures are pre-existing at the declared base
and were reproduced there. They do not affect the passing release evidence.

## Publication Receipts

### Merge

- Receipt: `docs/sddk/tutorial-walkthrough/merge-receipt.md`
- SHA-256: `e4d65ebf5372b41befb731deb74fbd49411460636c720fbd1f069b1491d47ccd`
- Capability: `git.push`
- Receipt ID: `git.push:0bebdee874245efba9f6a3570f745b9937423ce581cb6dd5fde15e84bd44afd8`
- Command: `git push origin c23d84e6cb1b180d317b74a7f71271152055f6e6:refs/heads/main`
- Exit: `0`
- Output digest: `sha256:0bebdee874245efba9f6a3570f745b9937423ce581cb6dd5fde15e84bd44afd8`

### Release

- Machine authority: `docs/sddk/tutorial-walkthrough/release-receipt.json`
- SHA-256: `c08348a521c82f9d7cd1d4ca0c9aa9c2dff448d20880573fa1e3e01da6d69efd`
- Capability: `git.tag`
- Receipt ID: `git.tag:1df18e7ff8f015874c0fbd4015fb5394d7e197df9e99bfa6abdd92cb57ea14ae`
- Command: `git tag -a v0.110.6 c23d84e6cb1b180d317b74a7f71271152055f6e6 -m "feat: guided tutorial walkthrough"; git push origin refs/tags/v0.110.6`
- Exit: `0`
- Output digest: `sha256:1df18e7ff8f015874c0fbd4015fb5394d7e197df9e99bfa6abdd92cb57ea14ae`
- Tag object: `a8d444c47cf141bdf3ea79531db2e84f3cf8facc`

## Files Inventory

Source: `/home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/inventory.json` (`sddk.inventory/v1`).

| Bucket | Added | Modified | Deleted | Renamed |
|---|---:|---:|---:|---:|
| prompts/ | 0 | 0 | 0 | 0 |
| agents/ | 0 | 0 | 0 | 0 |
| skills/ | 0 | 0 | 0 | 0 |
| assets/ | 0 | 0 | 0 | 0 |
| tools/ | 0 | 0 | 0 | 0 |
| docs/ | 68 | 0 | 0 | 0 |
| tests/ | 0 | 0 | 0 | 0 |
| `untagged_project/<segment>` | 0 | 0 | 0 | 0 |

Top 25 paths sorted by status / bucket:

| Status | Bucket | Path | Renamed from | SHA-256 |
|---|---|---|---|---|
| A | docs | `docs/sddk/application-stabilization-and-roadmap-convergence/apply-progress.md` | — | inventory |
| A | docs | `docs/sddk/archive/2026-07-21-scene-component-authoring-ux/archive-report.md` | — | inventory |
| A | docs | `docs/sddk/archive/2026-07-21-scene-component-authoring-ux/source/design.md` | — | inventory |
| A | docs | `docs/sddk/archive/2026-07-21-scene-component-authoring-ux/source/proposal.md` | — | inventory |
| A | docs | `docs/sddk/archive/2026-07-21-scene-component-authoring-ux/source/spec.md` | — | inventory |
| A | docs | `docs/sddk/archive/2026-07-21-scene-component-authoring-ux/source/tasks.md` | — | inventory |
| A | docs | `docs/sddk/g5-crash-recovery/design.md` | — | inventory |
| A | docs | `docs/sddk/g5-crash-recovery/explore-report.md` | — | inventory |
| A | docs | `docs/sddk/g5-crash-recovery/specification.md` | — | inventory |
| A | docs | `docs/sddk/g8-extension-compat-policy-runtime/debt-report.json` | — | inventory |
| A | docs | `docs/sddk/g8-extension-compat-policy-runtime/verify-report.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-adapter-contract/debt-report.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-adapter-contract/design.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-adapter-contract/explore-report.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-adapter-contract/proposal.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-adapter-contract/spec.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-adapter-contract/tasks.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-adapter-contract/verify-report.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-s2-impls/archive-manifest.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-s2-impls/debt-report.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-s2-impls/design.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-s2-impls/explore-report.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-s2-impls/proposal.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-s2-impls/spec.md` | — | inventory |
| A | docs | `docs/sddk/semantic-editor-model-s2-impls/tasks.md` | — | inventory |

The inventory comparison is `stage-and-working-tree-vs-head` and therefore
includes unrelated sibling-cycle documents present in the primary workspace.
The cycle's own committed implementation delta remains the authoritative four
files listed below.

## Cycle Delta

| Status | Path | Change |
|---|---|---|
| A | `frontend/src/components/TutorialStepper.tsx` | five-step guided walkthrough component |
| M | `frontend/src/components/AppShell.tsx` | mounts stepper and handles Take the tour |
| M | `frontend/src/styles.css` | `.tour-stepper` positioning and presentation |
| A | `frontend/tests/tutorial-walkthrough.spec.ts` | four scenarios across two projects, 8 tests |

Diff: `+431 -1`, digest
`cbc5ffdba033aa7caf2caffe0fe26e76c652e7c34418503adb240dfd9a084621`.

## Quality Signals

- TypeScript: `npx tsc --noEmit -p .` passed.
- Production ESLint: `npx eslint --max-warnings=0 src/components/TutorialStepper.tsx src/components/AppShell.tsx` passed.
- Test ESLint: `npx eslint --max-warnings=0 tests/tutorial-walkthrough.spec.ts` passed.
- Tutorial Playwright: 8/8 passed.
- Import-dialog regression: 8/8 passed.
- ux-welcome accessibility regression: 3/3 passed.
- The 3/3 ux-welcome `@full` failures reproduce at base `72d9c4a` and are not cycle-introduced.

## No Pending Effects

- `origin/main` is at the published SHA.
- The remote annotated tag object matches the local tag object.
- The remote tag peels to the published SHA.
- Optional CI/CD, hosted release, asset, signing, and distribution effects are
  explicitly out of scope and were not awaited.
- Archive transition was intentionally not run. The next phase is `archive`.

## Release Envelope

```yaml
status: success
route: local
change: tutorial-walkthrough
cycle_id: p-28fce7028ac3c497/tutorial-walkthrough
main_sha: c23d84e6cb1b180d317b74a7f71271152055f6e6
tag: v0.110.6
merge_receipt: docs/sddk/tutorial-walkthrough/merge-receipt.md
release_receipt: docs/sddk/tutorial-walkthrough/release-receipt.json
runtime_status: RELEASED
next_phase: archive
lease_after_transition: absent
optional_distribution: not_requested
blockers: []
inventory:
  path: /home/rubentxu/.local/share/sddk/projects/p-28fce7028ac3c497/cycle-artifacts/p-28fce7028ac3c497/inventory.json
  sha256: cdcdb6a912f0d5a21a1ca999bb7376168039d5449aef1e3691dbcace895d16b7
  unavailable_reason: null
```

This report covers the release phase only. It does not claim archive closure and
creates no archive transition, archive manifest, or ROADMAP/evidence-map
reorganization.
