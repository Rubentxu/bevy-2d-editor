# Evidence map refresh v0.109.9 handoff

**Cycle**: `p-28fce7028ac3c497/evidence-map-refresh-v1099`
**Tag**: v0.109.9 (commit `6d8b5ec`)
**Closed at**: 2026-09-08, sequence 303→310

## TL;DR

B-direct doc-only cycle. Refreshes `docs/v1.0-stabilization-evidence-map.md`
to reflect actual state of the 9 v1.0 product gates at HEAD `c6d383e`.

**Coverage score: 3 ✅ / 3 🟡 / 3 🔴 → 5 ✅ / 2 🟡 / 2 🔴.**

## What shipped

### Single-file doc change

- `docs/v1.0-stabilization-evidence-map.md`: +92 / -69 (365 lines).
  - §2.1 Playwright specs: 78 → 80 (+2).
  - §2.2 Cargo tests: 702 → 784 points (+82).
  - §2.5 NEW: cycles-closed-since-baseline table.
  - §4 G1 cell: 🔴 → ✅ (with detailed evidence).
  - §4 G2 cell: 🟡 → ✅ (cross-link to G2 release-receipt).
  - §4 G8 cell: 🔴 → ✅ (cross-link to compat policy doc + G8 release-receipt).
  - §5.4 Extension API compat policy: 🔴 → ✅.
  - §6.1 Removed G2 and G8 from highest-impact gaps.
  - §7 Removed P4 (compat policy) from outstanding work.

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 303 | OPEN/build |
| phase.build.complete.b-direct | 304 | OPEN/verify |
| phase.verify.complete.b-direct | 306 | RELEASE_PENDING |
| release.complete | 308 | RELEASED |
| archive.complete | 310 | CLOSED |

## Validation evidence

```
$ git diff --stat docs/v1.0-stabilization-evidence-map.md
 docs/v1.0-stabilization-evidence-map.md | 161 ++++++++++++++++++--------------
 1 file changed, 92 insertions(+), 69 deletions(-)

$ ls -la docs/sddk/g2-git-friendly-roundtrip/release-receipt.md \
         docs/sddk/g8-extension-compat-policy-runtime/release-receipt.md \
         docs/compatibility-policy.md
-rw-r--r--. 15071 sep  7 09:12 docs/compatibility-policy.md
-rw-r--r--.  2506 sep  8 14:31 docs/sddk/g2-git-friendly-roundtrip/release-receipt.md
-rw-r--r--.  3112 sep  8 14:40 docs/sddk/g8-extension-compat-policy-runtime/release-receipt.md
```

All 6 acceptance criteria met. No code/test changes — no cargo
test invocation required.

## Lessons learned

1. **`sddk cycle start --path b-direct`** is required to avoid
   the A-full default. Without it, the cycle starts at Explore
   phase and you need a `cycle.block` step to restart with the
   right path.
2. **B-direct verify phase** still requires two gate receipts:
   `tests-pass` (trivial for doc-only) + `policy-compliant` (means
   docs-check is clean).
3. **Release transitions require both `release-receipt` AND
   `merge-receipt` artifacts** — even for doc-only cycles. The
   merge-receipt is short (it's just a record of the merge event).
4. **Coverage score is now 5/2/2 formally** — the map is the
   authoritative ledger of v1.0 progress, not the ROADMAP row.

## v1.0-stabilization gate status (formal)

| Gate | Status |
|------|--------|
| G1 (canonical playable sample) | ✅ |
| G2 (filesystem/Git workflow) | ✅ |
| G3 (browser-local workflow) | ✅ |
| G4 (round-trip/migration) | 🟡 |
| G5 (crash/data-loss recovery) | 🔴 |
| G6 (performance corpus) | 🔴 |
| G7 (accessibility critical paths) | 🟡 |
| G8 (extension compat policy) | ✅ |
| G9 (architecture fitness) | ✅ |

## Carry-forward

- **G4 🟡** (round-trip/migration corpus expansion).
- **G5 🔴** (crash recovery, high scope, ~1-2 days).
- **G6 🔴** (performance corpus, ~3 days, requires benchmark harness).
- **G7 🟡** (a11y critical paths enumeration, ~0.5 day).
- **BJ-5 contact-death** (out of scope, requires logic-graph runtime).
- **M-2, M-3** (minor debt, trivial).
