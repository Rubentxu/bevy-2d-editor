# G8 — Evidence map refresh (merge-receipt)

**Cycle**: `p-28fce7028ac3c497/evidence-map-refresh-v1099`
**Sequence**: 307
**Tag**: v0.109.9
**Phase**: release

## Merge to trunk

Single-commit direct push to `main`. Cycle started from `main` at
base `c6d383e` (== origin/main at cycle start).

## Pre-merge verification

- Single-file diff scope (`docs/v1.0-stabilization-evidence-map.md`
  +92/-69) confirmed via `git diff --stat`.
- Cross-link targets exist on disk:
  - `docs/sddk/g2-git-friendly-roundtrip/release-receipt.md` ✅
  - `docs/sddk/g8-extension-compat-policy-runtime/release-receipt.md` ✅
  - `docs/compatibility-policy.md` ✅
- All 6 acceptance criteria met.
- No code/test changes; no cargo test invocation required.

## Merge event

```
$ git push origin main
c6d383e..6d8b5ec  main -> main

$ git tag -a v0.109.9 -m "v0.109.9: refresh v1.0-stabilization evidence map"
$ git push origin v0.109.9
* [new tag]         v0.109.9 -> v0.109.9
```

## Trunk gate

```
$ git rev-parse HEAD; git rev-parse origin/main; git rev-parse v0.109.9^{commit}
6d8b5ecbc4c292fe3a807a1ce6bc921618264c31
6d8b5ecbc4c292fe3a807a1ce6bc921618264c31
6d8b5ecbc4c292fe3a807a1ce6bc921618264c31
```

HEAD == origin/main == v0.109.9^{commit} ✅
