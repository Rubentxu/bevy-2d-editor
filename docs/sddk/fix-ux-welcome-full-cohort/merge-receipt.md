# Merge Receipt — fix-ux-welcome-full-cohort

**Cycle:** `p-28fce7028ac3c497/fix-ux-welcome-full-cohort`
**Path:** A-min
**Route:** local Git
**Release tag:** `v0.110.10`
**Base:** `a72104f76da48e7f6dc1ecf31e18702fe1728f03` (v0.110.9 archive)
**Published SHA (cycle commit):** `3df1682fcffb2075aaa9a579adc3df05d349eca1`
**Current trunk tip:** `b63134e` (verify-report docs sidecar; HEAD is at `b63134e` after the verify sidecar commit, but the cycle commit `3df1682` is the published diff)
**Published at:** `2026-09-08T21:41:00Z`

## Publication

Publication ran from the primary checkout (no isolated worktree this
cycle; unrelated staged and untracked sibling-cycle documents were left
untouched).

```text
$ git push origin main
To https://github.com/Rubentxu/bevy-2d-editor.git
   a72104f..b63134e  main -> main
```

- Exit code: `0`
- Capability: `git.push`
- Receipt ID: `git.push:3df1682-push-a72104f-b63134e-main-2026-09-08T21:41:00Z`
- Output digest: `sha256:3df1682-push-a72104f-b63134e-main-2026-09-08T21:41:00Z`

The push included:
- Cycle commit `3df1682` (1 file changed, `+10/-3`) — carries the
  product diff (cycle 398 fix).
- Implementation receipt commit `3a62725` (docs sidecar).
- Verify report + findings commit `b63134e` (docs sidecar).

Tag was created and pushed separately:

```text
$ git tag -a v0.110.10 3df1682fcffb2075aaa9a579adc3df05d349eca1 \
    -m "fix(test): ux-welcome.spec.ts @full cohort — use page.goto(\"/\") not page.reload() to drop ?skip-welcome=1"
$ git push origin refs/tags/v0.110.10
To https://github.com/Rubentxu/bevy-2d-editor.git
 * [new tag]         v0.110.10 -> v0.110.10
```

- Tag object ID: <pending — captured by release-receipt.json>
- Tag target SHA: `3df1682fcffb2075aaa9a579adc3df05d349eca1` (cycle
  commit; will be moved to archive commit during archive phase per the
  repo's tag-at-cycle-archive pattern).

## Postconditions

- `HEAD == origin/main` ✅ (at the verify-report sidecar `b63134e`; the
  product diff lives at the cycle commit `3df1682` which is the
  tag-target).
- `v0.110.10` annotated peel == `3df1682` (cycle commit) ✅ — final
  placement at the archive commit happens during the archive phase.
- `base (a72104f)` is an ancestor of `3df1682` ✅.
- `cargo test --workspace --lib` passes 871/871 ✅.
- Playwright in-scope tests pass 8/8 ✅ (3 from ux-welcome.spec.ts +
  5 regression).

## Carried-forward smoke failures (P3)

- `app-characterization.spec.ts:56` P2: multi-select
- `app-characterization.spec.ts:126` P4: scene operations
- `app-characterization.spec.ts:149` P5: composition root
- `app-characterization.spec.ts:211` P8: welcome overlay
- `editor-ready.spec.ts:78` S2: pre-ready action feedback

All 5 verified pre-existing at base `a72104f` (this cycle's base).
Tracked in `docs/v1.0-stabilization-evidence-map.md` as P3
carry-forward. Out of scope for cycle 398.
