# G4 — Round-trip / migration corpus (release-receipt)

**Cycle**: `p-28fce7028ac3c497/g4-roundtrip-corpus`
**Sequence**: 326 → 327
**Tag**: v0.110.1
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle | p-28fce7028ac3c497/g4-roundtrip-corpus |
| Path | A-min |
| Tag | v0.110.1 |
| Code-commit SHA | `0c46663` |
| Trunk SHA (HEAD) | `0c46663` |
| Origin/main SHA | `0c46663` |
| HEAD == origin/main | ✅ |
| Tag points at commit | ✅ |
| Diff vs. v0.110.0 | +549 / -2 across 6 files |

## Files in this release

```
docs/v1-format-manifest.md                                             NEW (206 lines)
crates/editor-model/tests/migration_corpus.rs                          +158 / -2 (270 lines total)
docs/sddk/g4-roundtrip-corpus/explore-report.md                        NEW (85 lines)
docs/sddk/g4-roundtrip-corpus/specification.md                         NEW (87 lines)
docs/sddk/g4-roundtrip-corpus/implementation-receipt.md                NEW (99 lines)
docs/sddk/g4-roundtrip-corpus/verify-report.md                         NEW (85 lines)
```

## Test posture

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

## v1.0-stabilization gate status

| Gate | Before | After |
|------|--------|-------|
| G4 (round-trip/migration) | 🟡 | ✅ (5 types documented + 8 corpus tests) |

## Cycle closure

| Sequence | Event | Status |
|----------|-------|--------|
| 319 | cycle.start | OPEN/explore |
| 320 | phase.explore.complete | OPEN/specify |
| 322 | phase.specify.complete.a-min | OPEN/build |
| 324 | phase.build.complete | OPEN/verify |
| 326 | phase.verify.complete.a-min | RELEASE_PENDING |
| 327 | release.complete | (this commit) |
| (next) | archive.complete | CLOSED |

## Carry-forward

- **G5 🔴** (crash recovery, A-lite, ~1-2 days).
- **G6 🔴** (performance corpus, A-lite, ~3 days).
- **CP-5 🔴** (asset import trigger UI work — separate cycle).
- **CP-6 → CP-10** (canvas/hover/drag — future cycles).
- **BJ-5 contact-death** (out of scope).
- **M-2, M-3** (minor debt, trivial).
