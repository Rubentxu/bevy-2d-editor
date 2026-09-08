# G8 — Extension compat policy runtime evidence (merge-receipt)

**Cycle**: `p-28fce7028ac3c497/g8-extension-compat-policy-runtime`
**Sequence**: 296 (2026-09-08)
**Tag**: v0.109.8
**Phase**: release

## Merge to trunk

Single-commit direct push to `main`. Cycle started from `main` at base
`862cfbf` (== origin/main at cycle start).

## Pre-merge verification

- `cargo test -p editor-model --test extension_compat` → 3/3 pass
- `cargo test -p editor-model --lib` → 313/0/0
- `cargo test -p editor-bevy --lib` → 414/0/1
- `bun run tools/archcheck/check.ts` → all assertions pass

## Merge event

```
$ git push origin main
862cfbf..7c72072  main -> main

$ git tag -a v0.109.8 -m "G8 extension compat policy runtime evidence"
$ git push origin v0.109.8
* [new tag]         v0.109.8 -> v0.109.8
```

## Trunk gate

```
$ git rev-parse HEAD; git rev-parse origin/main; git rev-parse v0.109.8^{commit}
7c72072bc19e3b104252fab86a487204d2c14f3a
7c72072bc19e3b104252fab86a487204d2c14f3a
7c72072bc19e3b104252fab86a487204d2c14f3a
```

HEAD == origin/main == v0.109.8^{commit} ✅
