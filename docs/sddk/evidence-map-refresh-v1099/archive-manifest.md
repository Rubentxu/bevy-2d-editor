# G8 — Evidence map refresh (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/evidence-map-refresh-v1099`
**Sequence**: 308 (release) → 309 (archive)
**Tag**: v0.109.9
**Phase**: archive

## Archive summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/evidence-map-refresh-v1099 |
| Status | CLOSED |
| Path | B-direct |
| Tag | v0.109.9 |
| Code-commit SHA | `6d8b5ec` |
| Trunk SHA (HEAD) | `6d8b5ec` (will become archive commit) |
| Origin/main SHA | `6d8b5ec` |
| HEAD == origin/main | ✅ |
| Tag points at commit | ✅ |
| Cycle span | 303 → 309 (7 events) |

## Artifacts archived

```
docs/sddk/evidence-map-refresh-v1099/specification.md             NEW
docs/sddk/evidence-map-refresh-v1099/implementation-receipt.md    NEW
docs/sddk/evidence-map-refresh-v1099/verify-report.md             NEW
docs/sddk/evidence-map-refresh-v1099/release-receipt.md           NEW
docs/sddk/evidence-map-refresh-v1099/merge-receipt.md             NEW
docs/v1.0-stabilization-evidence-map.md                            UPDATED (+92/-69)
```

## What changed

`docs/v1.0-stabilization-evidence-map.md` was rewritten to reflect
the actual state of the 9 v1.0 product gates at HEAD `c6d383e`
(v0.109.8 + G8 archive commit).

### Material changes

- §2.1 Playwright specs: 78 → 80 (+2: ui-entity-creation BJ-3, git-friendly-roundtrip G2).
- §2.2 Cargo tests: 702 → 784 total (+82: jump, enemy-patrol, extension_compat).
- §2.5 NEW: "Cycles closed since previous baseline" table.
- §4 G1 cell: 🔴 → ✅.
- §4 G2 cell: 🟡 → ✅ with cross-link.
- §4 G8 cell: 🔴 → ✅ with cross-link to compat policy doc + G8 release-receipt + 3 test descriptions.
- §5.4 Extension API compat policy: 🔴 → ✅.
- §6.1 Removed G2 and G8 from highest-impact gaps.
- §7 Removed P4 (compat policy) from outstanding work.
- **Coverage score: 5 ✅ / 2 🟡 / 2 🔴** (was 3/3/3).

## v1.0-stabilization gate status after this cycle

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

**Formal coverage: 5 ✅ / 2 🟡 / 2 🔴** ✅ (matches expected after G2 + G8 + this refresh).

## Cycle closure event sequence

| Event | Sequence |
|-------|----------|
| cycle.start | 303 |
| phase.build.complete.b-direct | 304 |
| phase.verify.complete.b-direct | 306 |
| release.complete | 308 |
| archive.complete | (this commit) |

## Lessons

1. **B-direct path is correct for doc-only refreshes**: no
   code/test/build gates apply. The verify phase reduces to
   "diff scope + cross-links exist + acceptance criteria met".
2. **`cycle.start` defaults to A-full when no `--path` flag is
   passed** — explicitly pass `--path b-direct` to avoid an
   extra `cycle.block` step.
3. **Release transitions require both `release-receipt` AND
   `merge-receipt` artifacts** — even for doc-only cycles. The
   merge-receipt can be a short 30-line doc (it's just a record
   of the merge event, not a full PR workflow).
4. **Always re-archive after release**: the tag MUST point at the
   archive commit (which contains the SDDK artifacts), not at
   the code commit alone.

## Carry-forward (post-this-cycle)

- **G4 🟡** (round-trip/migration corpus expansion).
- **G5 🔴** (crash recovery, high scope, ~1-2 days).
- **G6 🔴** (performance corpus, ~3 days, requires benchmark harness).
- **G7 🟡** (a11y critical paths enumeration, ~0.5 day doc + tests).
- **BJ-5 contact-death** (out of scope, requires logic-graph runtime).
- **M-2, M-3** (minor debt, trivial).
