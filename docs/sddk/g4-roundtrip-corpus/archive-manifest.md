# G4 — Round-trip / migration corpus (archive-manifest)

**Cycle**: `p-28fce7028ac3c497/g4-roundtrip-corpus`
**Sequence**: 328 (release) → 329 (archive)
**Tag**: v0.110.1
**Phase**: archive

## Archive summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/g4-roundtrip-corpus |
| Status | CLOSED |
| Path | A-min |
| Tag | v0.110.1 |
| Code-commit SHA | `0c46663` |
| Trunk SHA (HEAD) | `0c46663` (will become archive commit) |
| Origin/main SHA | `0c46663` |
| HEAD == origin/main | ✅ |
| Tag points at commit | ✅ |
| Cycle span | 319 → 329 (11 events) |

## Artifacts archived

```
docs/v1-format-manifest.md                                            NEW (206 lines)
docs/sddk/g4-roundtrip-corpus/explore-report.md                       NEW (85 lines)
docs/sddk/g4-roundtrip-corpus/specification.md                        NEW (87 lines)
docs/sddk/g4-roundtrip-corpus/implementation-receipt.md               NEW (99 lines)
docs/sddk/g4-roundtrip-corpus/verify-report.md                        NEW (85 lines)
docs/sddk/g4-roundtrip-corpus/release-receipt.md                      NEW (74 lines)
docs/sddk/g4-roundtrip-corpus/merge-receipt.md                        NEW (40 lines)
crates/editor-model/tests/migration_corpus.rs                         +158 / -2 (270 lines total)
```

## v1.0-stabilization gate status after this cycle

| Gate | Status |
|------|--------|
| G1 (canonical playable sample) | ✅ |
| G2 (filesystem/Git workflow) | ✅ |
| G3 (browser-local workflow) | ✅ |
| G4 (round-trip/migration) | 🟡 → **✅** |
| G5 (crash/data-loss recovery) | 🔴 |
| G6 (performance corpus) | 🔴 |
| G7 (accessibility critical paths) | ✅ |
| G8 (extension compat policy) | ✅ |
| G9 (architecture fitness) | ✅ |

**Formal coverage: 7 ✅ / 0 🟡 / 2 🔴** (up from 6/1/2). Only G5 and G6 remain.

## Migration corpus (final)

8 tests covering all 5 durable types:

- `corpus_v0_project_metadata_migrates` (existing)
- `corpus_v0_scene_document_migrates` (existing)
- `corpus_current_version_round_trip_noop` (existing)
- `corpus_future_version_rejected` (existing)
- `corpus_v0_scene_asset_document_migrates` (new)
- `corpus_v0_world_document_migrates` (new)
- `corpus_v0_logic_graph_asset_migrates` (new)
- `corpus_all_types_reject_future_version` (new, cross-type table-driven)

## Cycle closure event sequence

| Event | Sequence |
|-------|----------|
| cycle.start | 319 |
| phase.explore.complete | 320 |
| phase.specify.complete.a-min | 322 |
| phase.build.complete | 324 |
| phase.verify.complete.a-min | 326 |
| release.complete | 328 |
| archive.complete | (this commit) |

## Lessons

1. **A-min transition naming**: A-min uses the **plain** transition
   names (e.g. `phase.build.complete`), NOT `phase.build.complete.a-min`.
   Discovered via `sddk cycle next` showing the canonical IDs.
2. **Lowercase serde enums**: `SceneAssetRole` and
   `LayoutPolicy::Kind` use `rename_all = "lowercase"`. V0 fixtures
   need `"role": "screen"` (not `"Screen"`), `"kind": "grid"`
   (not `"Grid"`).
3. **Required fields with defaults**: `WorldDocument.updated_at`
   is `u64` (not Option). V0 fixtures include `"updated_at": 0`.
4. **Cross-type table-driven tests**: a single
   `corpus_all_types_reject_future_version` test exercises all 5
   rejection arms in 30 lines — cheaper than 5 separate tests
   and proves the rejection contract is uniform.
5. **A-min verify gates**: requires 4 gates
   (tests-pass, policy-compliant, debt-severity-assigned,
   debt-priority-assigned). The debt gates are trivial for
   doc-only cycles but must still be evaluated.

## Carry-forward

- **G5 🔴** (crash recovery corpus, A-lite, ~1-2 days).
- **G6 🔴** (performance corpus, A-lite, ~3 days).
- **CP-5 🔴** (asset import trigger UI work).
- **CP-6 → CP-10** (canvas/hover/drag — future cycles).
- **BJ-5 contact-death** (out of scope).
- **M-2, M-3** (minor debt, trivial).
