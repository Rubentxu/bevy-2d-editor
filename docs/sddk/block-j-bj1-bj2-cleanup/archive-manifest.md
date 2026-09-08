# Block J — BJ-1 + BJ-2 Cleanup (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/block-j-bj1-bj2-cleanup`
**Sequence**: 206 (2026-09-08)
**Tag**: v0.109.1
**Phase**: archive

## Artifacts persisted

| Path | Purpose |
|------|---------|
| `docs/sddk/block-j-bj1-bj2-cleanup/explore-report.md` | Cycle exploration evidence (146 lines) |
| `docs/sddk/block-j-bj1-bj2-cleanup/spec.md` | 7 acceptance criteria (98 lines) |
| `docs/sddk/block-j-bj1-bj2-cleanup/implementation-receipt.md` | Implementation summary (127 lines) |
| `docs/sddk/block-j-bj1-bj2-cleanup/verify-report.md` | AC verification (116 lines) |
| `docs/sddk/block-j-bj1-bj2-cleanup/debt-report.json` | 5 debt items, BJ-1+BJ-2 resolved |
| `docs/sddk/block-j-bj1-bj2-cleanup/release-receipt.md` | Release metadata (77 lines) |
| `docs/sddk/block-j-bj1-bj2-cleanup/merge-receipt.md` | Trunk merge evidence (55 lines) |
| `docs/sddk/block-j-bj1-bj2-cleanup/archive-manifest.md` | This document |

## Code changes (single commit)

```
eb28ce7 fix(tests): expose MinimalSession scaffolding + archcheck B1 regex (Block J BJ-1+BJ-2)

 crates/editor-bevy/src/actuator_bus.rs    | 38 +++++++++++++++++++++---------
 crates/editor-bevy/src/logic_evaluator.rs |  6 ++++
 crates/editor-model/src/command.rs        |  6 ++--
 tools/archcheck/check.ts                  |  6 ++--
 4 files changed, 57 insertions(+), 27 deletions(-)
```

## Ledger events

```
196 cycle.created       p-28fce7028ac3c497/block-j-bj1-bj2
197 cycle.created       p-28fce7028ac3c497/block-j-bj1-bj2-cleanup
198 cycle.transitioned  phase.explore.complete
200 cycle.transitioned  phase.specify.complete.a-min
202 cycle.transitioned  phase.build.complete
203 cycle.transitioned  phase.verify.complete.a-min
204 cycle.transitioned  release.complete
205 cycle.released      archive
206 cycle.transitioned  archive.complete (pending)
```

## Git evidence

| Check | Value |
|-------|-------|
| HEAD | `eb28ce76aaf171f322d573e370efb5124f6ea24c` |
| origin/main | `eb28ce76aaf171f322d573e370efb5124f6ea24c` |
| Tag v0.109.1 | `eb28ce76aaf171f322d573e370efb5124f6ea24c` |
| HEAD == origin/main | ✅ |
| Tag points at trunk SHA | ✅ |

## Knowledge artifacts

- ADR-0064 (`*_FALLBACK` thread_locals) — untouched.
- No new ADRs introduced (cycle is mechanical: scaffolding relocation +
  regex refinement + doc fix).
- debt-report.json updated to reflect BJ-1+BJ-2 resolved at sequence 202.
- ROADMAP.md will be updated at session-end to record v0.109.1 release.

## Carry-forward debt

- BJ-3 (UI-creation Playwright test, P2)
- BJ-4 (Bevy play_mode runtime assertions, P2)
- BJ-5 (gameplay assertions, P3)

These remain open; not in this cycle's scope.

## Tag-to-release-receipt linkage

- Release tag: `v0.109.1`
- Release-receipt: `docs/sddk/block-j-bj1-bj2-cleanup/release-receipt.md`
- Merge-receipt: `docs/sddk/block-j-bj1-bj2-cleanup/merge-receipt.md`
- All three refer to commit `eb28ce76aaf171f322d573e370efb5124f6ea24c`.
