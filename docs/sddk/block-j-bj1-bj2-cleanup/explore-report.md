# Block J — BJ-1 + BJ-2 Cleanup (explore-report)

**Cycle**: `p-28fce7028ac3c497/block-j-bj1-bj2-cleanup`
**Sequence**: 197 (2026-09-08)
**Path**: A-min
**Phase**: explore

## 1. Origin

Pre-existing debt items discovered during prior cycles:

- **BJ-1**: 3 logic_evaluator integration tests fail (`test_submit_and_drain`,
  `test_entity_bits_preserved_in_bus`, `test_end_to_end_actuator_pipeline`).
  Block A2 left them without a session install — so `submit_actuator_output`
  is a no-op and `drain_actuator_outputs` returns `Vec::new()`.
  Documented in `docs/sddk/v1-g1-bevy-harness/debt-report.json` item BJ-1
  (P1).
- **BJ-2**: archcheck rule B1 (and rule B2) raised false positives on
  substring matches of `editor_bevy::` and `editor-bevy::` appearing in doc
  comments. Same debt item (BJ-2, P2).

Both surfaced as part of the G1 debt-report and the H2.5 handoff review.

## 2. Hypothesis

### BJ-1
The 4 `actuator_bus` integration tests work because
`actuator_bus::tests::install_fresh_session()` is private to that module.
The 3 failing tests in `logic_evaluator::integration_tests` were never
updated to install a session when the actuator bus path was migrated from
thread_locals to the editor session port in Block A2.

**Fix shape**: expose the `MinimalSession` + `install_fresh_session`
scaffolding at `pub(crate)` scope so `logic_evaluator` tests can call it.
Symmetric with the ADR-0064 thread_local fallback pattern: keep the
test-only scaffolding tiny, do not add new APIs to production code.

### BJ-2
Rule B1's regex `/bevy::/` matches any literal `bevy::` substring,
including those embedded in `editor_bevy::` (the `editor-bevy` crate name).
Rule B2 has the same shape.

**Fix shape**: refine the regex with a negative lookbehind
`/(?<![-_])bevy::/` (and `editor-bevy::` variant) so crate-name substrings
in doc comments are excluded. Verified locally:

```
"editor_bevy::Foo"        → no match  (negative lookbehind)
"bevy::prelude::Entity"   → match
"// see editor-bevy::Foo" → no match
```

## 3. Investigation

### BJ-1 trace

Trace from `crates/editor-bevy/src/actuator_bus.rs`:

```rust
pub fn submit_actuator_output(entity: Entity, field: &str, value: PortValue) {
    let bits = entity.to_bits();
    editor_model::ports::with_session_mut(|session| {
        let outputs = session.runtime_actuator_outputs_mut();
        // ...
    });
}
```

`with_session_mut` returns `None` if no session is registered. Then
`drain_actuator_outputs` returns an empty `Vec`. Test fails with
`assert_eq!(outputs.len(), 2)` getting 0.

**Test scaffolding comparison**:

| Test | Calls `install_fresh_session()`? | Passes? |
|------|----------------------------------|---------|
| `actuator_bus::tests::test_submit_and_drain_roundtrip` (L271) | yes | yes |
| `actuator_bus::tests::test_bus_empty_after_drain` (L294) | yes | yes |
| `logic_evaluator::integration_tests::test_submit_and_drain` (L1704) | **no** | **no** |
| `logic_evaluator::integration_tests::test_entity_bits_preserved_in_bus` (L2028) | **no** | **no** |
| `logic_evaluator::integration_tests::test_end_to_end_actuator_pipeline` (L2044) | **no** | **no** |

### BJ-2 trace

`tools/archcheck/check.ts` rules B1 and B2:

```ts
// Rule B1: editor-bevy must not import from bevy directly (deprecated path)
const bevyUse = scan(/\bbevy::/g, text);
```

The `\b` word-boundary does not consider `-` or `_` as word characters in
the TS regex flavor, so `editor_bevy::` and `editor-bevy::` substring
matches slip through.

`crates/editor-model/src/command.rs` had a doc comment:
`// wasm-bindgen provides js_sys::Date::now()` — this contained the literal
`js_sys::` substring which rule B2 also flagged.

## 4. Decision

### BJ-1 — Option B: expose `test_helpers`

I evaluated two paths:

- **(a) Symmetric fallback**: add `ACTUATOR_OUTPUTS_FALLBACK` thread_local
  mirroring the ADR-0064 pattern. Rejected: production behavior would
  diverge from production — the bus would always have *something* even
  without a session, which masks future regressions.
- **(b) Test-only scaffolding at `pub(crate)`**: move `MinimalSession` +
  `install_fresh_session` into a `pub(crate) mod test_helpers` in
  `actuator_bus.rs`, then call it from the 3 failing tests.

Picked **(b)**. Net change:

- New module `crates/editor-bevy/src/actuator_bus.rs::test_helpers`
  (125 lines, `pub(crate)`).
- `actuator_bus::tests` shrinks (no more duplicate scaffolding; just the
  two passing tests).
- 3-line additions in `logic_evaluator::integration_tests`.

### BJ-2 — Option A: regex refinement

Refined the two archcheck rules to use negative lookbehind:
`/(?<![-_])bevy::/` for B1 (and `editor-bevy::` pattern remains). Adjusted
the doc comment in `crates/editor-model/src/command.rs` to remove the
incidental `js_sys::Date::now()` reference (B2 hit).

## 5. Validation status

- `cargo test -p editor-bevy --lib` → **414 passed; 0 failed** (was 411
  passing + 3 failing).
- `cargo check --workspace` → green.
- `bun run tools/archcheck/check.ts` → `archcheck: all assertions pass`.

## 6. Out of scope (deferred)

- BJ-3 (UI-creation Playwright test, P2)
- BJ-4 (Bevy `play_mode` runtime assertions, P2)
- BJ-5 (gameplay assertions, P3)

These remain open in the G1 debt-report and are not in this cycle.

## 7. Open questions

None. Cycle ready to advance to spec.
