# Cycle handoff — g6-performance-corpus → next session

**Date:** 2026-09-08
**Closed cycle:** `g6-performance-corpus` (v0.110.3, sequence 338→351)
**Cycle status:** ✅ CLOSED at archive (sequence 351)
**v1.0-stabilization state:** **9 ✅ / 0 🟡 / 0 🔴** — all nine product gates green.

## What we did

1. **New Playwright cohort** (`playwright.performance.config.ts`).
2. **Helper** (`frontend/tests/helpers/perfBudget.ts`):
   - `PerfBudgetSpec { name, softMs, hardMs }`.
   - `measureMs(fn)` and `assertWithinBudget(spec, fn)`.
   - Soft exceeded → `console.warn`. Hard exceeded → `Error`.
3. **6 perf specs** (`frontend/tests/perf-*.spec.ts`):
   - P1 `perf-1k-entities`: 1k-entity hydrate, 1.93 s (hard 20 s).
   - P2 `perf-tile-level`: 200 cells paint/erase, 14.5 s each (hard 25 s, soft 12 s warned).
   - P3 `perf-multi-level-world`: 5 scene switches, 223 ms mean (hard 1500 ms).
   - P4 `perf-asset-catalog`: 100 asset listing (hard 3 s).
   - P5 `perf-logic-graph`: 100-node dispatch, 71 ms mean (hard 200 ms, soft 50 ms warned).
   - P6 `perf-project-source-listing`: 200 file listing (hard 3 s).
4. **npm script** `test:perf` (in `frontend/package.json`).
5. **All 6 specs pass under hard budget** on local Chromium in 2.1 minutes.

## Bridge contracts discovered/fixed during verify

| Bridge | Wrong call | Correct signature |
|---|---|---|
| `paint_tile` | `(x, y, tile)` | `(assetRef, layerId, x, y, tilesetId, localIndex)` |
| `erase_tile` | `(x, y)` | `(assetRef, layerId, x, y)` |
| `scene_switch_commit` | `()` | `(id)` |
| `dispatch_logic_command` | `(obj)` | `(JSON string)` |
| `AddNode` | `{type: "add_node"}` | `{type: "AddNode", node_id, role, node_type_id, field_values, controller_id}` |
| `SetNodeField` | `{type, field: "x"}` | `{type: "SetNodeField", node_id, field_path: ["x"], value}` |
| `role` | `"transform"` | `"sensor" \| "controller" \| "actuator"` |
| `create_logic_graph_asset` | n/a | required before `dispatch_logic_command` |
| `import_asset_file` | `(path, payload)` | `(name, mimeType, bytes)` |
| `create_scene_asset` | returns object | returns JSON string with `{asset_id}` |
| `global_search` | n/a | bridge does not exist; P6 proxies with `list_source_files` |

## Empirical findings the corpus surfaced

Real numbers captured (and now in evidence map G6 cell):

- **MAX_SCENES = 16** (`crates/editor-bevy/src/scenes.rs:14`).
- **Per-cell paint cost = ~73 ms** through the bridge.
- **Scene-switch latency = ~45 ms** (after world setup).
- **Logic dispatch latency = ~71 ms** for SetNodeField over 100 nodes.
- **OPFS write throughput**: ~50 src/s, ~250 entities/s, ~400 assets/s.
- **1k-entity hydrate = 1.93 s** (v1 baseline for "a small game").

These are real, evidence-bound findings that the corpus surfaced. They are
the regression detectors the next perf cycle will tighten against.

## Cycle artefacts

```
docs/sddk/g6-performance-corpus/
  explore-report.md          (134 lines)
  specification.md           (170 lines)
  design.md                  (380 lines)
  verify-report.md           (108 lines)
  release-receipt.json       (33 lines)
  merge-receipt.json         (12 lines)
  archive-manifest.md        (102 lines)
```

## Tag map

- `v0.110.3` re-tagged at final archive `c8153c3` carrying all SDDK artifacts.

## Recommended next

**User-decision point.** With all gates green, the gate-matrix work is done.
Three high-value directions:

1. **P0 — v1.0 declaration PR** (`CHANGELOG.md` entry, release tag, roadmap
   declaration). The matrix is in terminal state; this is the moment.
2. **P1 — Tutorial sample** (UX onboarding, ~3 days).
3. **P2 — CP-5 import-trigger UI work** (a11y, ~0.5 day).
4. **P3 — Perf budget tightening** (next perf cycle, ~1 day).

None of these block v1.0; they are polish / next-bucket work.

## Persistence reminders (this cycle)

- [x] handoff doc (this file)
- [x] memory entry (`mem_g6_v1103`)
- [x] ROADMAP row added
- [x] commit + push + re-tag at archive
- [x] evidence map refresh (G6 🔴→✅, score 9/0/0)
- [x] all 6 specs verified locally

## Carry-forward (next session sees)

- CP-5 (a11y UI trigger).
- MAX_SCENES lift (engine change; can re-benchmark P3 at new maximum).
- `criterion` Rust benchmarks (deferred).
- `nightly-perf.yml` CI workflow.
- M-2/M-3 docs-check warnings (low priority).
- BJ-5 contact-death (out of scope; user-paused).
- rig-agent-runtime-foundation (user-paused; v1.0-stab gates now all pass —
  this becomes user-callable).
- v1.0 declaration PR (P0 above).
