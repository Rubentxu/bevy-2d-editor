# Block J handoff — BJ-1 + BJ-2 cleanup

**Cycle**: `p-28fce7028ac3c497/block-j-bj1-bj2-cleanup`
**Tag**: v0.109.1 (commit `eb28ce76aaf171f322d573e370efb5124f6ea24c`)
**Closed at**: 2026-09-08, sequence 197→208

## TL;DR

A-min cycle closing two pre-existing debt items (BJ-1, BJ-2) that were
tracked from Block I / G1. 4 files changed, +57/-27 lines, no public API
change. All 414 lib tests pass (was 411 + 3 failing). archcheck green.

## What shipped

### BJ-1 — `MinimalSession` scaffolding shared across test modules

- `crates/editor-bevy/src/actuator_bus.rs`: new `#[cfg(test)] pub(crate)
  mod test_helpers` exposes `MinimalSession` + `install_fresh_session`
  to other lib unit-test modules.
- `crates/editor-bevy/src/logic_evaluator.rs`: 3 lines added at the
  top of `test_submit_and_drain`, `test_entity_bits_preserved_in_bus`,
  `test_end_to_end_actuator_pipeline` to install a session before
  submitting actuator outputs.

### BJ-2 — archcheck regex refinement + doc comment drift

- `tools/archcheck/check.ts`: rule B1 regex changed from `/\bbevy::/g`
  to `/(?<![-_])bevy::/g`. The negative lookbehind excludes
  `editor_bevy::` and `editor-bevy::` substrings while still flagging
  top-level `bevy::` imports.
- `crates/editor-model/src/command.rs`: removed incidental
  `js_sys::Date::now()` substring from a doc comment to silence rule B2.

## Cycle flow

| Phase | Sequence | Status |
|-------|----------|--------|
| cycle.start | 197 | OPEN |
| phase.explore.complete | 198 | OPEN/specify |
| phase.specify.complete.a-min | 200 | OPEN/build |
| phase.build.complete | 202 | OPEN/verify |
| phase.verify.complete.a-min | 203 | RELEASE_PENDING |
| release.complete | 204 | RELEASED |
| archive.complete | 208 | CLOSED |

## Validation evidence

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out

$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)

$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Lessons learned

1. **Test scaffolding relocation**: when a private `install_fresh_session`
   helper needs to be shared across `mod tests` blocks in the same crate,
   hoist it to a `pub(crate) mod test_helpers` from the start. The
   "duplicate then dedupe" pattern (Block J did this retroactively) is
   mechanical but adds churn.

2. **JS regex `\b` does not honor `-`/`_`**: for archcheck rules that
   want to match `bevy::` only at the path-segment level, use negative
   lookbehind `(?<![-_])bevy::` rather than `\bbevy::`.

3. **Door of opportunity**: BJ-1 was a Block A2 bug masked by Block H's
   handoff. It would have been discovered sooner with a `cargo test`
   green-bar gate at the end of every cycle. The Block I cargo conformance
   restore was the trigger — we should keep `cargo test --workspace` as a
   hard gate on every cycle from here on.

## Open carry-forward

BJ-3 (UI-creation Playwright test, P2)
BJ-4 (Bevy play_mode runtime assertions, P2)
BJ-5 (gameplay assertions, P3)

These remain open in `docs/sddk/block-j-bj1-bj2-cleanup/debt-report.json`
and are candidates for a future G1-step2 cycle.

## Reference

- Spec: `docs/sddk/block-j-bj1-bj2-cleanup/spec.md`
- Implementation receipt: `docs/sddk/block-j-bj1-bj2-cleanup/implementation-receipt.md`
- Verify report: `docs/sddk/block-j-bj1-bj2-cleanup/verify-report.md`
- Debt report: `docs/sddk/block-j-bj1-bj2-cleanup/debt-report.json`
- Release receipt: `docs/sddk/block-j-bj1-bj2-cleanup/release-receipt.md`
- Merge receipt: `docs/sddk/block-j-bj1-bj2-cleanup/merge-receipt.md`
- Archive manifest: `docs/sddk/block-j-bj1-bj2-cleanup/archive-manifest.md`
