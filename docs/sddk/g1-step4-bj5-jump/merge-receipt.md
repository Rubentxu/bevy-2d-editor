# G1-step4 — Jump mechanic (merge-receipt)

**Cycle**: `p-28fce7028ac3c497/g1-step4-bj5-jump`
**Sequence**: 246 (2026-09-08)
**Tag**: v0.109.4
**Phase**: release

## Merge to trunk

Single-commit direct push to `main`. Cycle started from `main` at base
`658eb14` (== origin/main at cycle start).

## Pre-merge verification

- `cargo check --workspace --tests` ✅
- `cargo test -p editor-bevy --lib` → 414/0/1
- `cargo test -p examples-bevy-harness -- --ignored` → 11/11 (3 new +
  8 prior)
- `bun run tools/archcheck/check.ts` → all assertions pass
- `cargo clippy -p examples-bevy-harness --tests --no-deps` → no new
  warnings in jump.rs (the only new warning introduced was the
  `clone on Copy` warning at line 127 of jump.rs, fixed inline
  before commit)

## Merge event

```
$ git push origin main
658eb14..d2fbff6  main -> main

$ git tag -a v0.109.4 -m "G1-step4 jump mechanic (BJ-5 partial) ..."
$ git push origin v0.109.4
* [new tag]         v0.109.4 -> v0.109.4
```

## Trunk gate

```
$ git rev-parse HEAD; git rev-parse origin/main; git rev-parse v0.109.4^{commit}
d2fbff62ee16b972f102083756df3eed28786c71
d2fbff62ee16b972f102083756df3eed28786c71
d2fbff62ee16b972f102083756df3eed28786c71
```

HEAD == origin/main == v0.109.4^{commit} ✅
