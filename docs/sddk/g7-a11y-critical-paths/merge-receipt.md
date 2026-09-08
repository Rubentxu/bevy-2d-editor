# G7 — A11y critical paths corpus (merge-receipt)

**Cycle**: `p-28fce7028ac3c497/g7-a11y-critical-paths`
**Sequence**: 313
**Tag**: v0.110.0
**Phase**: release

## Merge to trunk

Single-commit direct push to `main`. Cycle started from `main` at
base `885d488` (== origin/main at cycle start).

## Pre-merge verification

- `npx tsc --noEmit -p .` clean.
- 4 new Playwright tests registered in both @a11y and @full cohorts.
- `docs/a11y-critical-paths.md` declares 5 critical paths (CP-1 → CP-5).
- CP-5 explicitly deferred with grep-verified rationale.

## Merge event

```
$ git push origin main
885d488..6d470e4  main -> main

$ git tag -a v0.110.0 -m "G7 a11y critical paths corpus"
$ git push origin v0.110.0
* [new tag]         v0.110.0 -> v0.110.0
```

## Trunk gate

```
$ git rev-parse HEAD; git rev-parse origin/main; git rev-parse v0.110.0^{commit}
6d470e4fe025b7c03fc7732b75074fc5649c47ed
6d470e4fe025b7c03fc7732b75074fc5649c47ed
6d470e4fe025b7c03fc7732b75074fc5649c47ed
```

HEAD == origin/main == v0.110.0^{commit} ✅
