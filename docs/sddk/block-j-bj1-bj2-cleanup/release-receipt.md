# Block J — BJ-1 + BJ-2 Cleanup (release-receipt)

**Cycle**: `p-28fce7028ac3c497/block-j-bj1-bj2-cleanup`
**Sequence**: 204 (2026-09-08)
**Tag**: v0.109.1
**Phase**: release

## Release summary

| Item | Value |
|------|-------|
| Cycle name | block-j-bj1-bj2-cleanup |
| Path | A-min |
| Phase at release | verify → release transition |
| Tag | v0.109.1 |
| Tag SHA | `eb28ce7` |
| Trunk SHA (HEAD) | `eb28ce7` |
| Origin/main SHA | `eb28ce7` |
| HEAD == origin/main | ✅ |
| Tag points at trunk | ✅ |
| Diff from prior (v0.109.0) | +57 / -27 lines across 4 files |

## Files in release

```
crates/editor-bevy/src/actuator_bus.rs    | +38 / -19 lines
crates/editor-bevy/src/logic_evaluator.rs | +6 / 0 lines
crates/editor-model/src/command.rs        | +3 / -3 lines
tools/archcheck/check.ts                  | +3 / -3 lines
```

## Test posture

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

Net change vs. v0.109.0: +3 tests now passing (was 411 + 3 failing).

## Static analysis

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Workspace

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Public API

**No production API surface changes.** The diff is entirely:

- 1 new `pub(crate) mod test_helpers` block (125 lines, `#[cfg(test)]`).
- 3 × 1-line additions to test bodies (`install_fresh_session()`).
- 1 × 1-character regex refinement (archcheck rule B1).
- 1 doc comment drift fix (archcheck rule B2).

## Out-of-scope carried forward

- BJ-3 (UI-creation Playwright test, P2)
- BJ-4 (Bevy play_mode runtime assertions, P2)
- BJ-5 (gameplay assertions, P3)

## Lessons

- Test scaffolding that needs to be shared across `mod tests` blocks
  should be hoisted to a `pub(crate) mod test_helpers` from the start —
  moving it later requires the symmetric `use super::test_helpers::…`
  refactor.
- Archcheck regex with `\b` does not honor `-`/`_` as word boundaries
  in the JS regex flavor. Negative lookbehind is the standard tool.
