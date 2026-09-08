# G8 — Extension compat policy runtime evidence (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/g8-extension-compat-policy-runtime`
**Sequence**: 298 (2026-09-08)
**Tag**: v0.109.8
**Phase**: archive

## Archive summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/g8-extension-compat-policy-runtime |
| Status | CLOSED |
| Path | A-min |
| Tag | v0.109.8 |
| Code-commit SHA | `7c72072` |
| Trunk SHA (HEAD) | `7c72072` (will become archive commit) |
| Origin/main SHA | `7c72072` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Cycle span (sequence) | 289 → 298 (10 events) |

## Artifacts archived

```
docs/sddk/g8-extension-compat-policy-runtime/explore-report.md
docs/sddk/g8-extension-compat-policy-runtime/spec.md
docs/sddk/g8-extension-compat-policy-runtime/specification.md (alias)
docs/sddk/g8-extension-compat-policy-runtime/implementation-receipt.md
docs/sddk/g8-extension-compat-policy-runtime/verify-report.md
docs/sddk/g8-extension-compat-policy-runtime/debt-report.json
docs/sddk/g8-extension-compat-policy-runtime/release-receipt.md
docs/sddk/g8-extension-compat-policy-runtime/merge-receipt.md
```

## Code released

```
crates/editor-model/tests/extension_compat.rs  NEW (180 lines, 3 tests)
```

## v1.0-stabilization gate status after this cycle

| Gate | Status |
|------|--------|
| G1 (canonical playable sample) | ✅ |
| G2 (filesystem/Git workflow) | ✅ (ADR-0045 + this round's G2 test + branch protection) |
| G3 (browser-local workflow) | ✅ |
| G4 (round-trip/migration) | 🟡 |
| G5 (crash/data-loss recovery) | 🔴 |
| G6 (performance corpus) | 🔴 |
| G7 (accessibility critical paths) | 🟡 |
| G8 (extension compat policy) | 🔴 → **ready for ✅** (compat policy doc + this round's runtime test) |
| G9 (architecture fitness) | ✅ |

**Coverage score after this cycle: 4 ✅ / 2 🟡 / 3 🔴 → expected 5 ✅ / 2 🟡 / 2 🔴**
once the evidence map is refreshed to reflect G2 and G8's actual
status.

## Carry-forward

- **G4 🟡** (round-trip/migration corpus expansion).
- **G5 🔴** (crash recovery, high scope).
- **G7 🟡** (a11y critical paths enumeration).
- **G6 🔴** (performance corpus).
- **BJ-5 contact-death** (out of scope, requires logic-graph runtime).
- **Evidence map refresh** (doc-only cycle to reflect G2/G8 status).
- **M-2, M-3** (minor debt, trivial).

## Cycle closure event sequence

| Event | Sequence |
|-------|----------|
| cycle.start | 289 |
| phase.explore.complete | 290 |
| phase.specify.complete.a-min | 292 |
| phase.build.complete | 294 |
| phase.verify.complete.a-min | 296 |
| release.complete | 298 |
| archive.complete | (this commit) |

## Lessons (final)

1. **Re-read the evidence map and actual code** before declaring a
   gate "red". G8's documentation half (`docs/compatibility-policy.md`,
   323 lines) was committed 2026-09-07 but the map (2026-09-06)
   didn't know about it. Only runtime evidence was missing.

2. **A-min for test-only cycles** continues to be the right
   shape: spec → build → verify. The 3-test extension_compat
   suite took ~10 minutes to author because the type definitions
   are stable.

3. **ADR-0040 lists 9 categories but the enum has 8** — known
   gap (no `menus/palette entries` variant yet). The test pins
   the 8 that exist without forbidding future additions
   (`#[non_exhaustive]` allows growth without breaking changes).
