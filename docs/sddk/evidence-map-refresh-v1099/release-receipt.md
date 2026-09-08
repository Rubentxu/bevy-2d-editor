# G8 — Evidence map refresh (release-receipt)

**Cycle**: `p-28fce7028ac3c497/evidence-map-refresh-v1099`
**Sequence**: 306 → 307
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/evidence-map-refresh-v1099 |
| Path | B-direct (doc-only refresh) |
| Phase | verify → release transition |
| Tag | v0.109.9 |
| Code-commit SHA | `6d8b5ec` |
| Trunk SHA (HEAD) | `6d8b5ec` |
| Origin/main SHA | `6d8b5ec` |
| HEAD == origin/main | ✅ |
| Tag points at commit | ✅ |
| Diff vs. v0.109.8 | +304 / -69 across 4 files |

## Files in this release

```
docs/v1.0-stabilization-evidence-map.md            +92 / -69 (365 lines)
docs/sddk/evidence-map-refresh-v1099/specification.md             NEW
docs/sddk/evidence-map-refresh-v1099/implementation-receipt.md    NEW
docs/sddk/evidence-map-refresh-v1099/verify-report.md             NEW
```

## Test posture

No code changes. No cargo test required. (B-direct doc-only cycle.)

## Static analysis

```
$ git diff --stat docs/v1.0-stabilization-evidence-map.md
 docs/v1.0-stabilization-evidence-map.md | 161 ++++++++++++++++++--------------
 1 file changed, 92 insertions(+), 69 deletions(-)
```

Single-file scope discipline confirmed.

## Material content changes

1. G1 cell: 🔴 → ✅ with detailed evidence (canonical platformer
   sample + Bevy harness + 15 runtime tests + BJ-3).
2. G2 cell: 🟡 → ✅ with cross-link to G2 release-receipt.
3. G8 cell: 🔴 → ✅ with cross-link to compat policy doc + G8
   release-receipt + 3 test descriptions.
4. Coverage score: 3 ✅ / 3 🟡 / 3 🔴 → **5 ✅ / 2 🟡 / 2 🔴**.
5. Inventory counts updated for HEAD `c6d383e`.
6. NEW §2.5 "Cycles closed since previous baseline" table.

## Cycle closure (sequence → status)

| Sequence | Event | Status |
|----------|-------|--------|
| 303 | cycle.start | OPEN |
| 304 | phase.build.complete.b-direct | OPEN/verify |
| 306 | phase.verify.complete.b-direct | RELEASE_PENDING |
| 307 | release.complete | (this commit) |
| (next) | archive.complete | CLOSED |

## Carry-forward

- **G4 🟡** (round-trip/migration corpus expansion).
- **G5 🔴** (crash recovery, high scope).
- **G6 🔴** (performance corpus).
- **G7 🟡** (a11y critical paths enumeration).
- **BJ-5 contact-death** (out of scope, requires logic-graph runtime).
- **M-2, M-3** (minor debt, trivial).
