# G1-step5 — Enemy patrol (merge-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`
**Sequence**: 260 (2026-09-08)
**Tag**: v0.109.5
**Phase**: release

## Merge to trunk

Single-commit direct push to `main`. Cycle started from `main` at base
`c0fb6dc` (== origin/main at cycle start).

## Pre-merge verification

- `cargo check --workspace --tests` ✅
- `cargo test -p editor-bevy --lib` → 414/0/1
- `cargo test -p examples-bevy-harness -- --ignored` → 15/15 (4 new +
  11 prior)
- `bun run tools/archcheck/check.ts` → all assertions pass
- `cargo clippy -p examples-bevy-harness --tests --no-deps` → no new
  warnings in enemy_patrol.rs or tests/enemy_patrol.rs

## Merge event

```
$ git push origin main
c0fb6dc..7006dc7  main -> main

$ git tag -a v0.109.5 -m "G1-step5 enemy patrol (BJ-5 partial) ..."
$ git push origin v0.109.5
* [new tag]         v0.109.5 -> v0.109.5
```

## Trunk gate

```
$ git rev-parse HEAD; git rev-parse origin/main; git rev-parse v0.109.5^{commit}
7006dc72a513bba055eba8d0a6144a74da59fdd8
7006dc72a513bba055eba8d0a6144a74da59fdd8
7006dc72a513bba055eba8d0a6144a74da59fdd8
```

HEAD == origin/main == v0.109.5^{commit} ✅
