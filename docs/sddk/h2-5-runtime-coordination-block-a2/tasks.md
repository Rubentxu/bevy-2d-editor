# Tasks: `h2-5-runtime-coordination-block-a2`

> **Cycle:** `p-28fce7028ac3c497/h2-5-runtime-coordination-block-a2`
> **Phase:** build · **Path:** A-lite
> **Date:** 2026-09-08

---

## WU-A2-1: Replace local PortValue with re-export

**Status:** pending
**REQs:** REQ-A2-1, REQ-A2-4
**Estimated LOC:** -7 +1 = -6 (replacement)

### Steps
1. Open `crates/editor-bevy/src/logic_evaluator.rs`
2. Delete lines 30-37 (the local `pub enum PortValue { ... }` block)
3. Insert `pub use editor_model::runtime::PortValue;` at the same location (after the existing doc comment on lines 28-29)
4. Run `cargo check -p editor-bevy --locked` and verify zero new warnings

### Done when
- [ ] `cargo check -p editor-bevy --locked` passes
- [ ] `grep -n "pub enum PortValue" crates/editor-bevy/src/logic_evaluator.rs` returns empty
- [ ] `grep -n "pub use editor_model::runtime::PortValue" crates/editor-bevy/src/logic_evaluator.rs` returns the new line

---

## WU-A2-2: Migrate actuator_bus.rs to session, drop thread_local

**Status:** pending
**REQs:** REQ-A2-2
**Depends on:** WU-A2-1
**Estimated LOC:** ~30

### Steps
1. Open `crates/editor-bevy/src/actuator_bus.rs`
2. Delete the `thread_local! { static ACTUATOR_OUTPUT_BUS ... }` block (line 51-54)
3. Delete the `actuator_bus()` helper function (lines 56-62)
4. Rewrite `submit_actuator_output` to use `editor_model::ports::with_session_mut`
5. Rewrite `drain_actuator_outputs` to use `editor_model::ports::with_session_mut`
6. Update `editor_model` import (add `editor_model::ports` use)
7. Add tracing import for graceful no-op
8. Run `cargo check -p editor-bevy --locked`

### Done when
- [ ] No `thread_local!` cell in `actuator_bus.rs`
- [ ] `submit_actuator_output` and `drain_actuator_outputs` route via session
- [ ] `cargo check -p editor-bevy --locked` passes
- [ ] `grep "thread_local" crates/editor-bevy/src/actuator_bus.rs` returns empty

---

## WU-A2-3: Update existing tests in actuator_bus.rs

**Status:** pending
**REQs:** REQ-A2-2 (parity)
**Depends on:** WU-A2-2
**Estimated LOC:** ~20

### Steps
1. Open `crates/editor-bevy/src/actuator_bus.rs`
2. Find the existing tests `submit_drain_basic`, `drain_returns_empty`, `submit_preserves_order` (lines 192-228)
3. Update them to set up a session before submit/drain and tear it down after
4. Use `editor_model::ports::install_session_for_test(...)` or equivalent test harness
5. Run `cargo test -p editor-bevy --lib --locked`

### Done when
- [ ] All existing tests in actuator_bus.rs pass
- [ ] Tests demonstrate session-backed bus behavior matches original thread-local contract

---

## WU-A2-4: Add parity test file `editor-bevy/tests/actuator_parity.rs`

**Status:** pending
**REQs:** REQ-A2-3
**Depends on:** WU-A2-3
**Estimated LOC:** ~80

### Steps
1. Create new file `crates/editor-bevy/tests/actuator_parity.rs`
2. Write test `actuator_parity_submit_drain_fifo` (S-A2-3.1)
3. Write test `actuator_parity_submit_drain_n_sequential` (S-A2-3.2)
4. Write test `actuator_parity_drain_when_empty` (EC-A2-2)
5. Use `install_session_for_test` helper or inline session setup
6. Run `cargo test -p editor-bevy --tests --locked`

### Done when
- [ ] New file passes `cargo test -p editor-bevy --tests --locked`
- [ ] All 3 parity tests green
- [ ] No flaky behavior (run 3 times)

---

## Implementation Order

```
WU-A2-1 ──> WU-A2-2 ──> WU-A2-3 ──> WU-A2-4
                                   (parity test file)
```

Strict sequential. Each WU gates the next.

## Rollback Plan

If WU-A2-2 breaks at `cargo check`, revert with `git checkout -- crates/editor-bevy/src/actuator_bus.rs`. The PortValue rename in WU-A2-1 is independent and can be kept if WU-A2-2 is reverted.

## Risks (carried over from design.md)

- **R1**: Bevy system `apply_actuator_outputs` running outside `with_session_mut` context. Mitigation: tracing::warn on miss.
- **R2**: Multi-thread access. Mitigation: session is already designed thread-local in editor-model::ports.

```yaml
status: planned
wus: 4
estimated_loc: ~125 (with deletions and new test file)
dependencies: strict_sequential
rollback: per-WU git checkout
```
