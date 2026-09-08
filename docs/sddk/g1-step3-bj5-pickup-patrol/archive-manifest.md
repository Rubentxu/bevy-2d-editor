# G1-step3 — Pickup collision (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/g1-step3-bj5-pickup-patrol`
**Sequence**: 234 (2026-09-08)
**Tag**: v0.109.3
**Phase**: archive

## Artifacts persisted

| Path | Purpose |
|------|---------|
| `docs/sddk/g1-step3-bj5-pickup-patrol/explore-report.md` | Cycle exploration evidence (167 lines) |
| `docs/sddk/g1-step3-bj5-pickup-patrol/spec.md` | 7 acceptance criteria (110 lines) |
| `docs/sddk/g1-step3-bj5-pickup-patrol/implementation-receipt.md` | Implementation summary (161 lines) |
| `docs/sddk/g1-step3-bj5-pickup-patrol/verify-report.md` | AC verification (96 lines) |
| `docs/sddk/g1-step3-bj5-pickup-patrol/debt-report.json` | 2 debt items, BJ-5 partial-resolved |
| `docs/sddk/g1-step3-bj5-pickup-patrol/release-receipt.md` | Release metadata (104 lines) |
| `docs/sddk/g1-step3-bj5-pickup-patrol/merge-receipt.md` | Trunk merge evidence (54 lines) |
| `docs/sddk/g1-step3-bj5-pickup-patrol/archive-manifest.md` | This document |

## Code changes (single commit)

```
c46effc feat(bevy-harness): G1-step3 pickup collision plugin + integration tests (BJ-5 partial)

 crates/examples-bevy-harness/src/collision.rs       |  55 +++++++++++
 crates/examples-bevy-harness/src/components.rs      |  11 +++
 crates/examples-bevy-harness/src/lib.rs             |   4 +-
 crates/examples-bevy-harness/src/loader.rs          |  11 +-
 crates/examples-bevy-harness/tests/pickup_collision.rs | 186 ++++++++++++++++++++
 5 files changed, 264 insertions(+), 3 deletions(-)
```

## Git evidence

| Check | Value |
|-------|-------|
| HEAD | `c46effc` |
| origin/main | `c46effc` |
| Tag v0.109.3^{commit} | `c46effc` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |

## Knowledge artifacts

- No new ADRs needed (BJ-5 partial is a witness extension, not a
  contract change).
- debt-report.json updated to reflect BJ-5 partial-resolved at seq 230.
- ROADMAP.md will be updated at session-end to record v0.109.3 release.

## Carry-forward debt

- BJ-3 (UI-creation Playwright test, P2) — open.
- BJ-5 remaining: enemy patrol, jump, contact-death logic-graph
  runtime — open.

## Tag-to-release-receipt linkage

- Release tag: `v0.109.3`
- Code-commit SHA: `c46effc`
- Release-receipt: `docs/sddk/g1-step3-bj5-pickup-patrol/release-receipt.md`
- Merge-receipt: `docs/sddk/g1-step3-bj5-pickup-patrol/merge-receipt.md`
- All four identifiers converge on `c46effc`.
