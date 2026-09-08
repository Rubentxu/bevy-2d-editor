# G8 handoff — Extension compat policy runtime evidence

**Cycle**: `p-28fce7028ac3c497/g8-extension-compat-policy-runtime`
**Tag**: v0.109.8 (commit `7c72072`)
**Closed at**: 2026-09-08, sequence 289→300

## TL;DR

A-min cycle closing the **runtime-evidence half of G8** (extension
and agent capability APIs have documented compatibility policy).
The `editor-model` crate gains:

- `crates/editor-model/tests/extension_compat.rs` (NEW, 180 lines):
  three tests proving the runtime contract declared by
  `docs/compatibility-policy.md` (Extension API + Capability tool
  surface sections).

1 file / +435 / -0. 313/0/0 editor-model lib unchanged. 414/0/1
editor-bevy unchanged.

## What shipped

### Code (single commit `7c72072`)

- `crates/editor-model/tests/extension_compat.rs` (NEW, 180):
  - `extension_manifest_json_round_trip_is_byte_identical`
  - `semver_parses_valid_and_rejects_malformed`
  - `capability_enum_has_builtin_categories`

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 289 | OPEN/explore |
| phase.explore.complete | 290 | OPEN/specify |
| phase.specify.complete.a-min | 292 | OPEN/build |
| phase.build.complete | 294 | OPEN/verify |
| phase.verify.complete.a-min | 296 | RELEASE_PENDING |
| release.complete | 298 | RELEASED |
| archive.complete | 300 | CLOSED |

## Validation evidence

```
$ cargo test -p editor-model --test extension_compat

running 3 tests
test semver_parses_valid_and_rejects_malformed ... ok
test capability_enum_has_builtin_categories ... ok
test extension_manifest_json_round_trip_is_byte_identical ... ok

test result: ok. 3 passed; 0 failed
```

```
$ cargo test -p editor-model --lib
test result: ok. 313 passed; 0 failed; 0 ignored

$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored

$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Lessons learned

1. **Always re-read the evidence map and actual code** before
   declaring a gate "red". `docs/compatibility-policy.md` was
   committed 2026-09-07 (`8590831`), one day before this cycle,
   but the evidence map (2026-09-06 baseline) didn't know about
   it. G8's documentation half was already complete; only runtime
   evidence was missing.

2. **A-min for test-only cycles** continues to be the right
   shape: spec → build → verify. Adding Rust integration tests
   is mechanical when the type definitions are stable.

3. **ADR-0040 lists 9 categories but the enum has 8** — known
   gap (no `menus/palette entries` variant yet). The test pins
   the 8 that exist without forbidding future additions
   (`#[non_exhaustive]` allows growth without breaking changes).

## v1.0-stabilization gate status

| Gate | Status |
|------|--------|
| G1 (canonical playable sample) | ✅ |
| G2 (filesystem/Git workflow) | ✅ (G2 cycle) |
| G3 (browser-local workflow) | ✅ |
| G4 (round-trip/migration) | 🟡 |
| G5 (crash/data-loss recovery) | 🔴 |
| G6 (performance corpus) | 🔴 |
| G7 (accessibility critical paths) | 🟡 |
| G8 (extension compat policy) | 🔴 → **ready for ✅** |
| G9 (architecture fitness) | ✅ |

**Expected coverage after this cycle: 5 ✅ / 2 🟡 / 2 🔴** once the
evidence map is refreshed to reflect G2 and G8's actual status.

## Carry-forward

- **G4 🟡** (round-trip/migration corpus expansion).
- **G5 🔴** (crash recovery, high scope).
- **G6 🔴** (performance corpus).
- **G7 🟡** (a11y critical paths enumeration).
- **BJ-5 contact-death** (out of scope, requires logic-graph runtime).
- **Evidence map refresh** (doc-only cycle).
- **M-2, M-3** (minor debt, trivial).
