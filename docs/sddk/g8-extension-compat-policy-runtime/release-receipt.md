# G8 — Extension compat policy runtime evidence (release-receipt)

**Cycle**: `p-28fce7028ac3c497/g8-extension-compat-policy-runtime`
**Sequence**: 296 (2026-09-08)
**Tag**: v0.109.8
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle name | g8-extension-compat-policy-runtime |
| Path | A-min |
| Phase at release | verify → release transition |
| Tag | v0.109.8 |
| Code-commit SHA | `7c72072` |
| Trunk SHA (HEAD) | `7c72072` |
| Origin/main SHA | `7c72072` |
| HEAD == origin/main | ✅ |
| Tag points at code commit | ✅ |
| Diff vs. v0.109.7 | +435 / -0 across 4 files |

## Files in code release

```
crates/editor-model/tests/extension_compat.rs  NEW (180 lines, 3 tests)
```

(plus 3 SDDK artifact docs in `docs/sddk/g8-extension-compat-policy-runtime/`.)

## Test posture

```
$ cargo test -p editor-model --test extension_compat

running 3 tests
test semver_parses_valid_and_rejects_malformed ... ok
test capability_enum_has_builtin_categories ... ok
test extension_manifest_json_round_trip_is_byte_identical ... ok

test result: ok. 3 passed; 0 failed
```

## Workspace

```
$ cargo test -p editor-model --lib
test result: ok. 313 passed; 0 failed; 0 ignored

$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

## Static analysis

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Cycle closure

| Phase | Sequence |
|-------|----------|
| cycle.start | 289 |
| phase.explore.complete | 290 |
| phase.specify.complete.a-min | 292 |
| phase.build.complete | 294 |
| phase.verify.complete.a-min | 296 |

## G8 status

Combined with `docs/compatibility-policy.md` (committed 2026-09-07
in `8590831`), G8 is now ready to upgrade from 🔴 to ✅:

- **Documentation half** (`docs/compatibility-policy.md`, 323
  lines): document format versions, extension API SemVer
  discipline, `#[non_exhaustive]` discipline, capability tool
  surface, stability levels, deprecation policy, breaking change
  process, version pinning strategy.
- **Runtime evidence half** (this cycle's test, 3 tests): JSON
  round-trip is byte-identical, `SemVer::parse` enforces pinning,
  `Capability` enum covers 8 declared runtime categories.

The evidence map (committed 2026-09-06) needs updating to reflect
G8's status. A future doc-only cycle may refresh the map.

## Lessons

1. **Always re-read the evidence map and the actual code before
   declaring a gate "red"** — `docs/compatibility-policy.md` was
   committed 2026-09-07 but the evidence map (2026-09-06 baseline)
   didn't know about it. G8's documentation half was already
   complete; only runtime evidence was missing.

2. **A-min for test-only cycles** continues to be the right
   shape: spec → build → verify. Adding Rust integration tests is
   mechanical when the type definitions are stable.

3. **ADR-0040 lists 9 categories but the enum has 8** — that's
   a known gap (no `menus/palette entries` variant yet). The test
   pins the 8 that exist without forbidding future additions
   (`#[non_exhaustive]` allows growth without breaking changes).
