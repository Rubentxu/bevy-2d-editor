# G1-step2 — Player Movement System (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/g1-step2-player-movement`
**Sequence**: 220 (2026-09-08)
**Tag**: v0.109.2
**Phase**: archive

## Artifacts persisted

| Path | Purpose |
|------|---------|
| `docs/sddk/g1-step2-player-movement/explore-report.md` | Cycle exploration evidence (125 lines) |
| `docs/sddk/g1-step2-player-movement/spec.md` | 7 acceptance criteria (107 lines) |
| `docs/sddk/g1-step2-player-movement/design.md` | Architecture + Bevy plugin pattern (166 lines) |
| `docs/sddk/g1-step2-player-movement/implementation-receipt.md` | Implementation summary (163 lines) |
| `docs/sddk/g1-step2-player-movement/verify-report.md` | AC verification (109 lines) |
| `docs/sddk/g1-step2-player-movement/debt-report.json` | 3 debt items, BJ-4 resolved |
| `docs/sddk/g1-step2-player-movement/release-receipt.md` | Release metadata (107 lines) |
| `docs/sddk/g1-step2-player-movement/merge-receipt.md` | Trunk merge evidence (51 lines) |
| `docs/sddk/g1-step2-player-movement/archive-manifest.md` | This document |

## Code changes (single commit)

```
98323a4 feat(bevy-harness): G1-step2 player movement plugin + integration tests (BJ-4)

 crates/examples-bevy-harness/src/lib.rs               |  2 +
 crates/examples-bevy-harness/src/movement.rs          | 52 +++++++++
 crates/examples-bevy-harness/tests/player_movement.rs| 225 ++++++++++++++++++++
 3 files changed, 279 insertions(+)
```

## Git evidence

| Check | Value |
|-------|-------|
| HEAD | `98323a4b0e6c5f4b8b0d8a4f1c2e9d7b5a8c3f1e` |
| origin/main | `98323a4` |
| Tag v0.109.2^{commit} | `98323a4` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |

## Knowledge artifacts

- No new ADRs needed (BJ-4 is a witness extension to G1-step1's existing
  contract, not a contract change).
- debt-report.json updated to reflect BJ-4 resolved at sequence 216.
- ROADMAP.md will be updated at session-end to record v0.109.2 release.

## Carry-forward debt

- BJ-3 (UI-creation Playwright test, P2) — open.
- BJ-5 (gameplay: jump, patrol, pickup, P3) — open.

These remain open in `debt-report.json` for future cycles.

## Tag-to-release-receipt linkage

- Release tag: `v0.109.2`
- Code-commit SHA: `98323a4`
- Release-receipt: `docs/sddk/g1-step2-player-movement/release-receipt.md`
- Merge-receipt: `docs/sddk/g1-step2-player-movement/merge-receipt.md`
- All four identifiers converge on `98323a4`.
