# Implementation Receipt: `h2-5-runtime-coordination` (Partial Block A)

> **Change:** `h2-5-runtime-coordination` · **Phase:** build · **Path:** A-full
> **Cycle:** `p-28fce7028ac3c497/h2-5-runtime-coordination`
> **Branch:** `h2-5-runtime-coordination-cycle` · **Commit:** `43c2bef`
> **Date:** 2026-09-08

---

## Summary

| Field | Value |
|-------|-------|
| Block | A (runtime buses) — partial |
| WUs complete | **1 / 5** (WU-A-1 only) |
| WUs deferred | 4 (WU-A-2, WU-A-3, WU-A-4, WU-A-5) |
| Commits landed | 1 (`43c2bef`) |
| LOC added | +116 (3 files) |
| LOC removed | 0 |
| Net change | +116 |
| Tests added | 0 |
| Tests passing | unchanged from baseline |

---

## WU-A-1 — Move LinearBus to editor-model/src/runtime/ ✅

**Commit:** `43c2bef refactor(arch): H2.5 A-1 — move LinearBus to editor-model/src/runtime/`

**Files changed:**
- `crates/editor-model/src/runtime/linear_bus.rs` (new, 102 LOC)
- `crates/editor-model/src/runtime/mod.rs` (new, 10 LOC)
- `crates/editor-model/src/lib.rs` (+2 lines: `pub mod runtime;` + `pub use runtime::LinearBus;`)

**Verified:**
- `cargo check -p editor-model --target wasm32-unknown-unknown --locked` PASS (1.76s)
- `cargo check -p editor-bevy --locked` PASS (3.72s)
- No new bevy / wasm_bindgen imports in editor-model/src/
- `editor_model::LinearBus` re-exported at crate root
- archcheck B1 (no bevy in editor-model) preserved

**What it lands:**
- `LinearBus` (fixed-capacity linear binary bus, used by COMMAND_BUS / EVENT_BUS) now lives in `editor-model/src/runtime/linear_bus.rs` as a pure, bevy-free type.
- ADR-0030 (bevy-free editor-model) satisfied.
- ADR-0057 (single composition root) preserved.

---

## Deferred Work (4 WUs)

### WU-A-2 — command_bus + event_bus fields + EditorSessionPort methods + trampolines
**Status:** NOT LANDED
**Reason for deferral:** Worker session_hatchling ran 653s without delivering verified envelope (lesson 2026-09-07). Manual review revealed legacy code in `crates/editor-bevy/src/preview_runtime.rs:1080/1114` (`crate::COMMAND_BUS.with(...)`, `crate::EVENT_BUS.with(...)`) and `crates/editor-bevy/src/lib.rs:432/433/1098/1105/1113/1120` that requires trampoline rewrites through `editor_model::ports::with_session_mut`. Scope expansion beyond original WU.

### WU-A-3 — ActuatorBus split per REQ-A-4
**Status:** NOT LANDED
**Reason for deferral:** Worker attempted the split but introduced a **double-PortValue identity issue**: `editor_model::runtime::PortValue` (created in new `port_value.rs`) coexists with `crate::logic_evaluator::PortValue` (existing private type used by 6 evaluators in logic_evaluator.rs). Type-equal but identity-different causes E0308 mismatched types at all `submit_actuator_output(...)` call sites. Resolution: rename `logic_evaluator::PortValue` to `LogicPortValue` first, then move `PortValue` to editor-model cleanly. Scope creep beyond WU-A-3.

### WU-A-4 — HotReloadRequest + PlayModeRequest movement
**Status:** NOT LANDED
**Reason for deferral:** Same scope-reduction reasoning as WU-A-3. The new types were created in `editor-model/src/runtime/hot_reload.rs` but not wired through `EditorSession.runtime` or the `EditorSessionPort` trait.

### WU-A-5 — Parity tests for hot_reload + apply_actuator_outputs_in_preview
**Status:** NOT LANDED
**Reason for deferral:** Depends on WU-A-2..A-4 completing. Cannot meaningfully test parity without the runtime state actually being moved.

---

## Next Iteration Plan

To resume Block A in a future cycle (`h2-5-runtime-coordination-block-a2`):

1. **Pre-step:** Rename `crates/editor-bevy/src/logic_evaluator.rs::PortValue` → `LogicPortValue` (or similar). Update all 6 evaluator impls + callers in `logic_evaluator.rs`. Verify build.
2. **WU-A-2:** Add `command_bus: LinearBus` + `event_bus: LinearBus` fields to `EditorSession.runtime`. Add 2 EditorSessionPort methods. Add trampolines in `editor-bevy/src/preview_runtime.rs:1080/1114` and `lib.rs:432/433/1098/1105/1113/1120` that route through `editor_model::ports::with_session_mut`. Verify cargo test --test hot_reload.
3. **WU-A-3:** Split `ActuatorBus`: pure types → `editor_model::runtime/actuator_bus.rs`; Bevy wrapper stays in `editor-bevy/src/actuator_bus.rs`. Verify cargo test --test apply_actuator_outputs_in_preview.
4. **WU-A-4:** Move `HotReloadRequest`, `PlayModeRequest` types to `editor_model::runtime/hot_reload.rs`. Add 2 fields + 2 port methods. Verify cargo test --test hot_reload.
5. **WU-A-5:** Update parity tests as needed.

### Then Block B / C / D

Block B (preview typed fields): simpler than Block A — PreviewMetrics, PreviewMappingEntry, PreviewProvenance move from editor-bevy to editor-model (3 WUs, no double-type issue if done in isolation).

Block C (InputState Resource): KEYBOARD_STATE → Bevy Resource (3 WUs, no type movement across crate boundary).

Block D: inventory.yaml -9 entries + matrix RETIRED + gates (3 WUs, pure docs + archcheck).

---

## Risks for Future Block A Iteration

| Risk | Mitigation |
|------|------------|
| Worker session can hang without delivering (25% success rate observed 2026-09-07 + 0% this session) | Apply 600s threshold kill + manual revert + scope reduction. Don't trust worker session to deliver envelope. |
| `logic_evaluator::PortValue` rename can break tests using `editor_model::PortValue` (if introduced prematurely) | Pre-step FIRST; Block A2 cannot proceed without it. |
| Archcheck pre-existing failures (asset_operation_log.rs wasm_bindgen) | Unchanged from main baseline; not blocking. Documented in debt-report. |

---

```yaml
status: partial
verdict: block_a_slice_landed
slice_landed: [WU-A-1]
slice_deferred: [WU-A-2, WU-A-3, WU-A-4, WU-A-5]
commit: 43c2bef
branch: h2-5-runtime-coordination-cycle
files_changed: 3
loc_added: 116
tests_added: 0
build_status: pass
wasm32_check: pass
archcheck: pre-existing_failures_unchanged
next_iteration: h2-5-runtime-coordination-block-a2
blocker_for_continuation: logic_evaluator::PortValue rename
risks: 3
```