# G4 — Round-trip / migration corpus (merge-receipt)

**Cycle**: `p-28fce7028ac3c497/g4-roundtrip-corpus`
**Sequence**: 327
**Tag**: v0.110.1
**Phase**: release

## Merge to trunk

Single-commit direct push to `main`. Cycle started from `main` at
base `7b938ae` (== origin/main at cycle start).

## Pre-merge verification

- 8/8 migration corpus tests pass (4 original + 4 new).
- 313/0/0 editor-model lib tests pass.
- 414/0/1 editor-bevy lib tests pass.
- archcheck green.

## Merge event

```
$ git push origin main
7b938ae..0c46663  main -> main

$ git tag -a v0.110.1 -m "G4 round-trip / migration corpus"
$ git push origin v0.110.1
* [new tag]         v0.110.1 -> v0.110.1
```

## Trunk gate

```
$ git rev-parse HEAD; git rev-parse origin/main; git rev-parse v0.110.1^{commit}
0c46663db4e7452ed9ff7e35b808d38096f57e32
0c46663db4e7452ed9ff7e35b808d38096f57e32
0c46663db4e7452ed9ff7e35b808d38096f57e32
```

HEAD == origin/main == v0.110.1^{commit} ✅
