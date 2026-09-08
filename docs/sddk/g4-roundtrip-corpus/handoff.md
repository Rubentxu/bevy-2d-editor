# G4 round-trip / migration corpus — handoff

**Cycle**: `p-28fce7028ac3c497/g4-roundtrip-corpus`
**Tag**: v0.110.1 (commit `0c46663`)
**Closed at**: 2026-09-08, sequence 319→330

## TL;DR

A-min cycle closing **G4 🟡 → ✅** (round-trip/migration corpus).
Declares the v1 format registry and proves v0→v1 migration for all
5 durable document types with 8 corpus tests.

## What shipped

### Documentation

- `docs/v1-format-manifest.md` (206 lines, NEW):
  - §1 Scope.
  - §2 Declared v1 format list (5-row table).
  - §3 Version constants (canonical reference).
  - §4 Migration policy (forward + backward compatibility).
  - §5 v0→v1 detailed migration steps.
  - §6 Carry-forward (v2 plan).
  - §7 Cross-references.

### Tests

- `crates/editor-model/tests/migration_corpus.rs` (+158 lines, 4 new tests):
  - `corpus_v0_scene_asset_document_migrates` (V0 → V1, idempotent).
  - `corpus_v0_world_document_migrates` (V0 → V1, idempotent).
  - `corpus_v0_logic_graph_asset_migrates` (V0 → V1, idempotent).
  - `corpus_all_types_reject_future_version` (cross-type table-driven).

### SDDK artifacts

- `docs/sddk/g4-roundtrip-corpus/explore-report.md` (85 lines).
- `docs/sddk/g4-roundtrip-corpus/specification.md` (87 lines).
- `docs/sddk/g4-roundtrip-corpus/implementation-receipt.md` (99 lines).
- `docs/sddk/g4-roundtrip-corpus/verify-report.md` (85 lines).
- `docs/sddk/g4-roundtrip-corpus/release-receipt.md` (74 lines).
- `docs/sddk/g4-roundtrip-corpus/merge-receipt.md` (40 lines).
- `docs/sddk/g4-roundtrip-corpus/archive-manifest.md` (104 lines).
- `docs/sddk/g4-roundtrip-corpus/handoff.md` (this file).

### Evidence map refresh

- `docs/v1.0-stabilization-evidence-map.md`: G4 row 🟡 → ✅,
  cycles table row added, §6.1 G4 removed, coverage score
  6/1/2 → **7/0/2**.

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 319 | OPEN/explore |
| phase.explore.complete | 320 | OPEN/specify |
| phase.specify.complete.a-min | 322 | OPEN/build |
| phase.build.complete | 324 | OPEN/verify |
| phase.verify.complete.a-min | 326 | RELEASE_PENDING |
| release.complete | 328 | RELEASED |
| archive.complete | 330 | CLOSED |

## Validation evidence

```
$ cargo test -p editor-model --test migration_corpus
test result: ok. 8 passed; 0 failed; 0 ignored

$ cargo test -p editor-model --lib
test result: ok. 313 passed; 0 failed; 0 ignored

$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored

$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Lessons learned

1. **A-min transition naming**: A-min uses plain transition names
   (`phase.build.complete`), NOT `phase.build.complete.a-min`. The
   suffix exists in some contexts but A-min doesn't use it.
2. **Lowercase serde enums**: 3 of 4 new tests required fixture
   iteration because `SceneAssetRole` and `LayoutPolicy::Kind` use
   `rename_all = "lowercase"`. Captured in the implementation
   receipt's "Discoveries" section.
3. **Cross-type table-driven test**: a single
   `corpus_all_types_reject_future_version` test exercises all 5
   rejection arms in 30 lines. Cheaper than 5 separate negative tests.
4. **A-min verify requires 4 gates**: tests-pass, policy-compliant,
   debt-severity-assigned, debt-priority-assigned. The debt gates
   are trivial for doc-only cycles but must still be evaluated.

## v1.0-stabilization gate status (formal)

| Gate | Status |
|------|--------|
| G1 (canonical playable sample) | ✅ |
| G2 (filesystem/Git workflow) | ✅ |
| G3 (browser-local workflow) | ✅ |
| G4 (round-trip/migration) | ✅ |
| G5 (crash/data-loss recovery) | 🔴 |
| G6 (performance corpus) | 🔴 |
| G7 (accessibility critical paths) | ✅ |
| G8 (extension compat policy) | ✅ |
| G9 (architecture fitness) | ✅ |

**Coverage score: 7 ✅ / 0 🟡 / 2 🔴** (up from 6/1/2).

## Carry-forward

- **G5 🔴** (crash recovery corpus, A-lite, ~1-2 days).
- **G6 🔴** (performance corpus, A-lite, ~3 days).
- **CP-5 🔴** (asset import trigger UI work).
- **CP-6 → CP-10** (canvas/hover/drag — future cycles).
- **BJ-5 contact-death** (out of scope).
- **M-2, M-3** (minor debt, trivial).
