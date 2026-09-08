# G1-step2 — Player Movement System (merge-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step2-player-movement`
**Sequence**: 218 (2026-09-08)
**Tag**: v0.109.2
**Phase**: release

## Merge to trunk

Single-commit direct push to `main`. Cycle was started from `main` at
base `5923de8d` (== origin/main at cycle start).

## Pre-merge verification

- `cargo check --workspace --tests` ✅
- `cargo test -p editor-bevy --lib` → 414/0/1
- `cargo test -p examples-bevy-harness -- --ignored` → 4/4
- `bun run tools/archcheck/check.ts` → all assertions pass

## Merge event

```
$ git push origin main
5923de8..98323a4  main -> main

$ git tag -a v0.109.2 -m "G1-step2 player movement system (BJ-4 closed) ..."
$ git push origin v0.109.2
* [new tag]         v0.109.2 -> v0.109.2
```

## Post-merge verification

- `git fetch origin main` → HEAD == origin/main == `98323a4`
- Tag `v0.109.2^{commit}` == `98323a4` ✅

## Commit contents

```
98323a4 feat(bevy-harness): G1-step2 player movement plugin + integration tests (BJ-4)

 crates/examples-bevy-harness/src/lib.rs               |  2 +
 crates/examples-bevy-harness/src/movement.rs          | 52 +++++++++
 crates/examples-bevy-harness/tests/player_movement.rs| 225 ++++++++++++++++++++
 3 files changed, 279 insertions(+)
```

## Coherence

- No reviewer requested (single-commit A-lite with exhaustive test coverage).
- No approval gate needed (no production API change to existing types).
- No deploy step (workspace library crates only; WASM bundles unaffected).
