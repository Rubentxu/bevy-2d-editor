# Archive Manifest — `fix-ux-welcome-full-cohort`

> **Cycle:** `p-28fce7028ac3c497/fix-ux-welcome-full-cohort`
> **Path:** A-min
> **Sequence:** 404 → 405 (explore) → 406 (specify) → 406→407 (build) → 407→408 (verify) → 408→409 (release) → 409→410 (`archive.complete`)
> **Tag:** `v0.110.10` (annotated, peeled to the archive commit — pending)
> **Phase:** Archive — **`status=CLOSED` (terminal, pending `archive.complete` transition)**
> **Delivery:** local (single-commit direct push via primary checkout + sidecar docs commit)
> **Manifest SHA-256 (final):** pending

---

## Closure Summary

| Field | Value |
|-------|-------|
| Verdict | RELEASED — `archive.complete` eligible |
| Path | A-min |
| Branch | `main` |
| Tag | `v0.110.10` (annotated: "fix(test): ux-welcome.spec.ts @full cohort — use page.goto(\"/\") not page.reload() to drop ?skip-welcome=1") |
| Cycle commit (carries product diff) | `3df1682fcffb2075aaa9a579adc3df05d349eca1` |
| Impl sidecar commit | `3a62725` (implementation-receipt.md) |
| Verify sidecar commit | `b63134e` (verify-report.md + verify-findings.json) |
| Release sidecar commit | `6dd9617` (release-report.md + release-receipt.json + merge-receipt.md) |
| Archive commit (this commit) | pending — created by archive phase (lands this manifest + ROADMAP/evidence-map refresh) |
| Base SHA | `a72104f76da48e7f6dc1ecf31e18702fe1728f03` (v0.110.9 archive) |
| Diff digest | `+10/-3` across 1 file in cycle commit (`frontend/tests/ux-welcome.spec.ts`) |
| Tests | 3/3 new (`ux-welcome.spec.ts` `@full` cohort was 0/3 pre-existing); 5/5 regression (`tutorial-walkthrough` + `tour-completed-persistence`) pass; 871/871 rust unit tests pass |
| Verify verdict | `PASS` (subject `3df1682`) |
| Debt verdict | `PASS` (no `ponytail:` markers introduced; 0 findings) |
| Files (cycle delta) | 1 — `+10/-3` (1 MOD TS-test: `frontend/tests/ux-welcome.spec.ts`) |
| Carry-forward closed | **v0.110.6 carry-forward #3** (P3: `ux-welcome.spec.ts` `@full` cohort — 3 tests, was 0/3 pre-existing) |

## Release Receipt (machine authority)

`docs/sddk/fix-ux-welcome-full-cohort/release-receipt.json` — SHA-256
pending final hash.

| Field | Value |
|-------|-------|
| Schema | `sddk.release-receipt/v1` |
| Release tag | `v0.110.10` |
| Tag object (initial) | `0b660badeb978395f59d9880ebe1ac8b60dbba3c` |
| Annotated | true |
| Tag target SHA (cycle commit, initial placement) | `3df1682fcffb2075aaa9a579adc3df05d349eca1` |
| Tag target SHA (final, after archive retag) | archive commit (this commit's SHA, pending) |
| Published HEAD / origin/main (pre-archive) | `6dd9617f2d87c17edeed7fc39f62f88681215dea` (release sidecar) |
| `git.push` receipt | `git.push:3df1682-push-a72104f-3df1682-main-pending` (exit `0`) |
| `git.tag` receipt | `git.tag:v0.110.10-tag-3df1682-pending` (exit `0`) |
| Postconditions | `head_equals_origin_main: true`, `remote_tag_peels_to_archive_commit_pending: "tag currently peels to cycle commit 3df1682; archive phase will move it to the archive commit"`, `remote_annotated_tag_object_matches_local: true`, `base_is_ancestor_of_head: true` |

```text
HEAD              = 6dd9617 (release sidecar; archive commit pending)
origin/main       = 6dd9617
v0.110.10^{}        = 3df1682 (cycle commit; retag to archive commit during archive phase)
v0.110.10 (object) = 0b660badeb978395f59d9880ebe1ac8b60dbba3c
```

`HEAD == origin/main == 6dd9617` ✅
`v0.110.10 annotated peel == 3df1682 (cycle commit)` ✅ (final placement at archive commit happens during this archive phase)
`base (a72104f) ⊏ 3df1682 ⊏ 3a62725 ⊏ b63134e ⊏ 6dd9617 ⊏ archive_commit` ✅

The cycle commit (`3df1682`) carries the product diff; the sidecar
commits register the receipts (`3a62725` impl, `b63134e` verify,
`6dd9617` release); the **archive commit** (this commit) lands the
SDDK artifacts (`docs/sddk/fix-ux-welcome-full-cohort/archive-manifest.md`)
and refreshes `docs/ROADMAP.md` + `docs/v1.0-stabilization-evidence-map.md`.
The tag is placed at the **archive commit** per the repository's
tag-at-cycle-archive pattern (matching the prior pattern: v0.110.6 →
`9546284`, v0.110.7 → `47117df`, v0.110.8 → `01b0e50`, v0.110.9 →
`da0faac`).

## Verify Receipt

`docs/sddk/fix-ux-welcome-full-cohort/verify-report.md` — SHA-256
pending final hash. Verdict `PASS` (subject SHA
`3df1682fcffb2075aaa9a579adc3df05d349eca1`).

All 4 verify gates passed:

- `tests-pass` — receipt `gate-tests-pass-e47990e5cfeb7e97-1` (8/8 in-scope
  Playwright tests pass; 871/871 rust unit tests pass)
- `policy-compliant` — receipt `gate-policy-compliant-e47990e5cfeb7e97-1`
  (tsc clean, eslint clean)
- `debt-severity-assigned` — receipt
  `gate-debt-severity-assigned-e47990e5cfeb7e97-1` (0 introduced
  `ponytail:` markers across the 1 in-scope file)
- `debt-priority-assigned` — receipt
  `gate-debt-priority-assigned-e47990e5cfeb7e97-1` (no debt findings
  to triage)

## File-level delta (cycle commit `3df1682`)

```text
frontend/tests/ux-welcome.spec.ts | 13 ++++++++++---
1 file changed, 10 insertions(+), 3 deletions(-)
```

## Carry-forwards opened / closed by this cycle

### Closed

- **v0.110.6 carry-forward #3 (P3)** — `ux-welcome.spec.ts` `@full`
  cohort (3 tests, all failing) — now 3/3 passing.

### Opened / tracked forward

None new. The pre-existing 5 P3 smoke failures (app-characterization ×4
+ editor-ready S2) are unchanged from v0.110.9; the Bevy↔JS mutex
architectural cleanup is unchanged from v0.110.9; the production-build
sample fallback is unchanged from v0.110.8. No new carry-forwards
opened by this cycle.

## ROADMAP and evidence map refresh

This archive commit updates two aggregate documents so the cumulative
state matches `HEAD`:

- `docs/ROADMAP.md` — adds the cycle 398 row (line 79) registering
  `fix-ux-welcome-full-cohort` as `✅ RELEASED — v0.110.10`, with the
  full changelog (root cause: `page.reload()` preserves
  `?skip-welcome=1`; fix: `page.goto("/")`).
- `docs/v1.0-stabilization-evidence-map.md` — adds the
  `### Refresh 2026-09-08 (Fix ux-welcome full cohort)` block (§2.1
  spec count, §4 P3 carry-forward closed, §6.1 carry-forward list
  updated). Coverage score unchanged: **9 ✅ / 0 🟡 / 0 🔴**.

## Preconditions for `archive.complete` transition

- [x] All 4 verify gates passed (see Verify Receipt above)
- [x] `release-receipt.json` exists with `head_equals_origin_main: true`
- [x] `merge-receipt.md` exists with `git.push` + `git.tag` receipts
- [x] `archive-manifest.md` written (this file)
- [x] `docs/ROADMAP.md` refresh committed
- [x] `docs/v1.0-stabilization-evidence-map.md` refresh committed
- [ ] `sddk cycle evaluate-gate --gate ledger-valid --cycle ... --transition archive.complete`
- [ ] `sddk cycle evaluate-gate --gate vault-index-current --cycle ... --transition archive.complete`
- [ ] `sddk cycle transition --transition archive.complete --gate-receipt <both> --artifact archive-manifest=docs/sddk/fix-ux-welcome-full-cohort/archive-manifest.md`

## Out-of-cycle references

- Bug origin: `2e0e5ec` (Aug 19, P3 Wave B cohort tagging sweep)
- Production code readers (NOT changed): `frontend/src/components/WelcomeOverlay.tsx:175-180`
  (`urlSkip` synchronous reader), `frontend/src/components/WelcomeOverlay.tsx:228`
  (render guard)
- Carry-forward tracker: `docs/v1.0-stabilization-evidence-map.md` §6.1
- ADR-0051 cohort tagging convention (referenced by `2e0e5ec`'s sweep
  intent; the cohort tagging itself is correct, only the application to
  `ux-welcome.spec.ts` was incorrect)
