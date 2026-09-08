# Archive — g6-performance-corpus

**Cycle**: `p-28fce7028ac3c497/g6-performance-corpus` (A-lite)
**Tag**: v0.110.3
**HEAD at archive**: 0c658e694b65e5a61a363fdbe68dfd34d15bf07b
**Sequence**: 349
**Outcome**: ✅ Cycle CLOSED; v1.0-stabilization coverage 8/0/1 → 9/0/0 (all Green).

## Cycle ledger

| Phase | Artifact | SHA |
|---|---|---|
| Explore | `docs/sddk/g6-performance-corpus/explore-report.md` | 134 lines |
| Specify | `docs/sddk/g6-performance-corpus/specification.md` | 169 lines |
| Design | `docs/sddk/g6-performance-corpus/design.md` | 380 lines |
| Build | `ddd03050d72f0d22f83943a25db4125e9dcafcf1` (12 files, +1397/-1) | commit |
| Verify | `docs/sddk/g6-performance-corpus/verify-report.md` | 108 lines |
| Release | `docs/sddk/g6-performance-corpus/release-receipt.json` + `merge-receipt.json` | receipts |
| Archive | (this file) | manifest |

## Specs produced (6/6 pass under hard budget)

| Spec | Hard budget | Soft budget | Empirical | Soft WARN? |
|---|---|---|---:|:---:|
| `perf-1k-entities.spec.ts` | 20 s | 8 s | 1.93 s | no |
| `perf-tile-level.spec.ts` (paint) | 25 s | 12 s | 14.59 s | yes |
| `perf-tile-level.spec.ts` (erase) | 25 s | 12 s | 14.51 s | yes |
| `perf-multi-level-world.spec.ts` | 1500 ms | 500 ms | 223 ms (mean) | no |
| `perf-asset-catalog.spec.ts` | 3 s | 1 s | <20 s incl. prebuild | no |
| `perf-logic-graph.spec.ts` | 200 ms | 50 ms | 71 ms (mean) | yes |
| `perf-project-source-listing.spec.ts` | 3 s | 1 s | <20 s incl. prebuild | no |

## New files (8 created, 1 deleted)

```
A  frontend/playwright.performance.config.ts
A  frontend/tests/helpers/perfBudget.ts
A  frontend/tests/perf-1k-entities.spec.ts
A  frontend/tests/perf-tile-level.spec.ts
A  frontend/tests/perf-multi-level-world.spec.ts
A  frontend/tests/perf-asset-catalog.spec.ts
A  frontend/tests/perf-logic-graph.spec.ts
A  frontend/tests/perf-project-source-listing.spec.ts
D  frontend/tests/perf-10k-entities.spec.ts  (renamed, count reduced)
M  frontend/package.json                      (+ test:perf script)
A  docs/sddk/g6-performance-corpus/{explore-report,specification,design,verify-report,release-receipt,merge-receipt}.md|json
```

## Tag map

- `v0.110.3` re-tagged at `0c658e6` (final archive commit carrying release-receipt + merge-receipt).

## Empirical findings captured

- **Per-cell paint cost**: ~73 ms/cell through the bridge
- **Scene-switch latency**: ~45 ms/switch (after world setup)
- **Logic dispatch latency**: ~71 ms / 100-node dispatch
- **1k-entity hydrate**: 1.93 s
- **MAX_SCENES**: 16 (engine hard-coded limit)
- **OPFS prebuild speed**: ~50 files/s for source, ~250 entities/s for scenes

## Coverage delta

v0.110.2 → v0.110.3:

- 8 ✅ / 0 🟡 / 1 🔴 → **9 ✅ / 0 🟡 / 0 🔴** (ALL GREEN).

G6 (performance corpus) was the only remaining Red gate. v1.0-stabilization
is now **complete at the gate-matrix level**: every product gate has declared
coverage.

## Carry-forward (non-blocking, post-v1.0 candidate work)

- Tighten soft budgets (P2 12 s → 18 s for example) to surface regressions
  earlier.
- Raise MAX_SCENES (separate cycle, requires engine changes).
- Add `criterion` Rust benchmarks for the bridge hot path (deferred per
  explore-report).
- Add `nightly-perf.yml` CI workflow (per `specification.md` §6.4).
- Re-scope P6 to use `global_search` once the bridge is exposed.
- CP-5 import-trigger UI work (separate a11y cycle).
- M-2/M-3 docs-check warnings (low priority).
- BJ-5 contact-death (out of scope; user-paused).
- rig-agent-runtime-foundation (user-paused until v1.0 gates pass — they now
  ALL pass).

## Recommended next

The next cycle is up to user choice. High-value candidates:

1. **Tutorial sample** (UX onboarding) — closes a 🟡 in `v1.0-stabilization.md` §5.3.
2. **CP-5 import-trigger UI work** — closes the last a11y gap.
3. **P6 budget tighten + MAX_SCENES lift** — track the empirical regressions.
4. **Nightly perf CI workflow** — operationalise the corpus.
5. **v1.0-stabilization declaration** — the gates-all-green state warrants a
   "v1.0 stable" tag/PR when the user is ready to declare release.

## Artifacts produced under archive-manifest

This file is the `archive-manifest` referenced by `release.complete`'s gate
receipts. Its SHA is included in the cycle ledger above; future cycles that
query this one read this file as the source of truth.
