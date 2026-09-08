# G2 — Git-friendly round-trip test (merge-receipt)

**Cycle**: `p-28fce7028ac3c497/g2-git-friendly-roundtrip`
**Sequence**: 284 (2026-09-08)
**Tag**: v0.109.7
**Phase**: release

## Merge to trunk

Single-commit direct push to `main`. Cycle started from `main` at base
`3499520` (== origin/main at cycle start).

## Pre-merge verification

- `cd frontend && npx tsc --noEmit -p .` ✅
- `cd frontend && npx playwright test --list git-friendly-roundtrip`
  registers both tests ✅
- `cargo test -p editor-bevy --lib` → 414/0/1
- `bun run tools/archcheck/check.ts` → all assertions pass

## Merge event

```
$ git push origin main
3499520..827250f  main -> main

$ git tag -a v0.109.7 -m "G2 Git-friendly round-trip test"
$ git push origin v0.109.7
* [new tag]         v0.109.7 -> v0.109.7
```

## Trunk gate

```
$ git rev-parse HEAD; git rev-parse origin/main; git rev-parse v0.109.7^{commit}
827250f751c6a1ffe80f9ac2a70abbe56b838c47
827250f751c6a1ffe80f9ac2a70abbe56b838c47
827250f751c6a1ffe80f9ac2a70abbe56b838c47
```

HEAD == origin/main == v0.109.7^{commit} ✅
