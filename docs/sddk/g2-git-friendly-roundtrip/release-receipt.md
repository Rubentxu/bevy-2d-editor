# G2 — Git-friendly round-trip test (release-receipt)

**Cycle**: `p-28fce7028ac3c497/g2-git-friendly-roundtrip`
**Sequence**: 284 (2026-09-08)
**Tag**: v0.109.7
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle name | g2-git-friendly-roundtrip |
| Path | A-min |
| Phase at release | verify → release transition |
| Tag | v0.109.7 |
| Code-commit SHA | `827250f` |
| Trunk SHA (HEAD) | `827250f` |
| Origin/main SHA | `827250f` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Diff vs. v0.109.6 | +431 / -0 across 4 files |

## Files in code release

```
frontend/tests/git-friendly-roundtrip.spec.ts  NEW (182 lines, 2 tests)
```

(plus 3 SDDK artifact docs in `docs/sddk/g2-git-friendly-roundtrip/`.)

## Test posture

```
$ cd frontend && npx tsc --noEmit -p .
(no output: TypeScript clean)

$ cd frontend && npx playwright test --list git-friendly-roundtrip
[full] › git-friendly-roundtrip.spec.ts:89:3 › G2 — Git-friendly round-trip › project_json_round_trip_is_byte_identical
[full] › git-friendly-roundtrip.spec.ts:141:3 › G2 — Git-friendly round-trip › every_opfs_file_is_parseable_json_with_version_field
Total: 2 tests in 1 file
```

## Workspace

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

## Static analysis

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Cycle closure

| Phase | Sequence |
|-------|----------|
| cycle.start | 277 |
| phase.explore.complete | 278 |
| phase.specify.complete.a-min | 280 |
| phase.build.complete | 282 |
| phase.verify.complete.a-min | 284 |

## G2 status

ADR-0045 + this test + branch protection rule = G2 should now
upgrade from 🟡 to ✅. The runtime-evidence half is closed.

## Lessons

1. **Text-determinism is the load-bearing property for Git
   friendliness**: without byte-identical round-trips, every save
   pollutes PR review with spurious diffs. This test pins the
   property down so future changes can't regress it silently.

2. **A-min for test-only cycles** continues to be the right
   shape: spec → build → verify, no design phase. Adding tests
   for already-declared ADRs is mechanical when the harness is
   mature.

3. **Reusing the OPFS_FILES table and mountSample helper** from
   `e2e-game-creation.spec.ts` keeps the new spec consistent. The
   duplication of the file list is a minor wart — a future cycle
   could extract a shared helper.
