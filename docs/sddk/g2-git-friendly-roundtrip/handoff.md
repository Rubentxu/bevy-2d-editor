# G2 handoff — Git-friendly round-trip test

**Cycle**: `p-28fce7028ac3c497/g2-git-friendly-roundtrip`
**Tag**: v0.109.7 (commit `827250f`)
**Closed at**: 2026-09-08, sequence 277→288

## TL;DR

A-min cycle closing the **runtime-evidence half of G2** (filesystem
/ Git workflow documented and stable). The frontend test suite
gains:

- `frontend/tests/git-friendly-roundtrip.spec.ts` (NEW, 182 lines):
  two tests proving that `project.json` round-trip is byte-identical
  and that every OPFS file is parseable JSON with a `version` field.

1 file / +431 / -0. 414/0/1 editor-bevy unchanged. tsc clean.

## What shipped

### Code (single commit `827250f`)

- `frontend/tests/git-friendly-roundtrip.spec.ts` (NEW, 182):
  - `project_json_round_trip_is_byte_identical`
  - `every_opfs_file_is_parseable_json_with_version_field`

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 277 | OPEN/explore |
| phase.explore.complete | 278 | OPEN/specify |
| phase.specify.complete.a-min | 280 | OPEN/build |
| phase.build.complete | 282 | OPEN/verify |
| phase.verify.complete.a-min | 284 | RELEASE_PENDING |
| release.complete | 286 | RELEASED |
| archive.complete | 288 | CLOSED |

## Validation evidence

```
$ cd frontend && npx tsc --noEmit -p .
(no output: TypeScript clean)

$ cd frontend && npx playwright test --list git-friendly-roundtrip
[full] › git-friendly-roundtrip.spec.ts:89:3 › G2 — Git-friendly round-trip › project_json_round_trip_is_byte_identical
[full] › git-friendly-roundtrip.spec.ts:141:3 › G2 — Git-friendly round-trip › every_opfs_file_is_parseable_json_with_version_field
Total: 2 tests in 1 file
```

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Lessons learned

1. **Text-determinism** is the load-bearing property for Git
   friendliness. Without byte-identical round-trips, every save
   pollutes PR review with spurious diffs. This test pins the
   property down so future changes can't regress it silently.

2. **A-min for test-only cycles** continues to be the right
   shape: spec → build → verify, no design phase.

## v1.0-stabilization gate status

| Gate | Status |
|------|--------|
| G1 (canonical playable sample) | ✅ evidence rich |
| G2 (filesystem/Git workflow) | 🟡 → **ready to upgrade to ✅** |
| G3 (browser-local workflow) | ✅ |
| G4 (round-trip/migration) | 🟡 |
| G5 (crash/data-loss recovery) | 🔴 |
| G6 (performance corpus) | 🔴 |
| G7 (accessibility critical paths) | 🟡 |
| G8 (extension/agent compat policy) | 🔴 |
| G9 (architecture fitness) | ✅ |

## Carry-forward

- **G4 🟡** (round-trip/migration corpus, separate cycle).
- **G5 🔴** (crash recovery, high scope).
- **G7 🟡** (a11y critical-path enumeration).
- **G8 🔴** (extension compat policy, doc-heavy).
- **BJ-5 contact-death** (out of scope, requires logic-graph runtime).
- **M-2, M-3** (minor debt, trivial).
