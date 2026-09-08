# Release Report: fix-ux-welcome-full-cohort

**Status:** `success`
**Route:** `local`
**Cycle:** `p-28fce7028ac3c497/fix-ux-welcome-full-cohort`
**Path:** A-min
**Candidate base:** `a72104f76da48e7f6dc1ecf31e18702fe1728f03` (v0.110.9 archive)
**Cycle commit:** `3df1682fcffb2075aaa9a579adc3df05d349eca1` (carries product diff)
**Published head:** `3df1682fcffb2075aaa9a579adc3df05d349eca1` (cycle commit; archive commit pending)
**Tag:** `v0.110.10` (annotated, remote peel verified)
**Tag target SHA (initial placement):** `3df1682fcffb2075aaa9a579adc3df05d349eca1`
**Release completed:** `2026-09-08T21:41:00Z`

## Result

The cycle was published directly to `main` from the primary checkout. The
annotated `v0.110.10` release tag was created at the cycle commit
`3df1682` and pushed to origin. The current `HEAD` and `origin/main`
(`3df1682`) is the cycle commit. The tag will be moved to the archive
commit during the archive phase per the repository's tag-at-cycle-archive
pattern (matching v0.110.6 → `9546284`, v0.110.7 → `47117df`,
v0.110.8 → `01b0e50`, v0.110.9 → `da0faac`).

```text
HEAD              = 3df1682fcffb2075aaa9a579adc3df05d349eca1
origin/main       = 3df1682fcffb2075aaa9a579adc3df05d349eca1
v0.110.10^{}        = 3df1682fcffb2075aaa9a579adc3df05d349eca1
v0.110.10 (object) = <pending — set by git.tag>
```

`HEAD == origin/main == 3df1682` ✅
`v0.110.10 annotated peel == 3df1682 (cycle commit)` ✅

The typed `sddk release apply` path was not used because this repository's
tracked `Cargo.toml` workspace version is `0.109.0`, while the
user-mandated semver tag is `v0.110.10`. The local Git contract route was
executed directly, with exact commands, exit codes, output digests, and
the SHA of the pushed commit recorded in the receipt.

## What landed

Cycle 398 closes the v0.110.6 carry-forward #3 (P3): the 3
`ux-welcome.spec.ts` `@full`-cohort tests now pass (was: 3/3 failing
pre-existing since `2e0e5ec` "P3 Wave B cohort tagging", Aug 19).

Cycle 398 change (the actual fix):

- `frontend/tests/ux-welcome.spec.ts:46-61` — `beforeEach` reorganised
  into 3 explicit phases (init skip, clear OPFS, navigate to `/`) with
  inline comments. The `page.reload()` call was replaced with
  `page.goto("/")` so the post-init navigation actually drops the
  `?skip-welcome=1` query parameter (which `page.reload()` preserves
  by design — the URL is the source of truth for the reload).

No other files were touched. Zero production source change. Zero Rust
change. Zero config change. Zero test scenarios added or removed.

## Verification

- `npx playwright test --project=full tests/ux-welcome.spec.ts` → 3/3 pass (32.3 s) — was 0/3 pre-existing
- `npx playwright test --project=full tests/tutorial-walkthrough.spec.ts tests/tour-completed-persistence.spec.ts` → 5/5 pass (regression intact)
- `cargo test --workspace --lib` → 871/871 pass
- `npx tsc --noEmit` → clean
- `npx eslint --max-warnings=0 tests/ux-welcome.spec.ts` → clean

## Pre-existing failures (NOT introduced)

The 5 P3 smoke failures from v0.110.9 (app-characterization ×4 +
editor-ready S2) are unchanged — verified pre-existing at base
`a72104f` (this cycle's base). Tracked as P3 carry-forward.

## Files changed

| File | LOC |
|------|-----|
| `frontend/tests/ux-welcome.spec.ts` | +10/-3 |
| **Total (cycle commit)** | **+10/-3** |

(Plus impl-receipt at `docs/sddk/fix-ux-welcome-full-cohort/implementation-receipt.md` —
docs-sidecar commit `3a62725`, not part of the cycle commit's product diff.)

## Out-of-scope carry-forwards

- Bevy↔JS mutex architectural cleanup (tracked from v0.110.9)
- 5 P3 smoke failures (tracked from v0.110.9)
- Production-build sample fallback (tracked from v0.110.8)
