# G1-step3 — Pickup collision (merge-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step3-bj5-pickup-patrol`
**Sequence**: 232 (2026-09-08)
**Tag**: v0.109.3
**Phase**: release

## Merge to trunk

Single-commit direct push to `main`. Cycle started from `main` at base
`8e7be64` (== origin/main at cycle start).

## Pre-merge verification

- `cargo check --workspace --tests` ✅
- `cargo test -p editor-bevy --lib` → 414/0/1
- `cargo test -p examples-bevy-harness -- --ignored` → 8/8 (3 new +
  5 prior)
- `bun run tools/archcheck/check.ts` → all assertions pass

## Merge event

```
$ git push origin main
8e7be64..c46effc  main -> main

$ git tag -a v0.109.3 -m "G1-step3 pickup collision system (BJ-5 partial) ..."
$ git push origin v0.109.3
* [new tag]         v0.109.3 -> v0.109.3
```

## Post-merge verification

- `git fetch origin main` → HEAD == origin/main == `c46effc`
- Tag `v0.109.3^{commit}` == `c46effc` ✅

## Commit contents

```
c46effc feat(bevy-harness): G1-step3 pickup collision plugin + integration tests (BJ-5 partial)

 crates/examples-bevy-harness/src/collision.rs       |  55 +++++++++++
 crates/examples-bevy-harness/src/components.rs      |  11 +++
 crates/examples-bevy-harness/src/lib.rs             |   4 +-
 crates/examples-bevy-harness/src/loader.rs          |  11 +-
 crates/examples-bevy-harness/tests/pickup_collision.rs | 186 ++++++++++++++++++++
 5 files changed, 264 insertions(+), 3 deletions(-)
```

## Coherence

- No reviewer requested (single-commit A-lite with exhaustive test coverage).
- No approval gate needed (no production API change to existing types).
- No deploy step (workspace library crates only; WASM bundles unaffected).
