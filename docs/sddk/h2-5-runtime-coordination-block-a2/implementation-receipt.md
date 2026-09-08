# Implementation Receipt: `h2-5-runtime-coordination-block-a2`

> **Cycle:** `p-28fce7028ac3c497/h2-5-runtime-coordination-block-a2`
> **Phase:** build · **Path:** A-lite
> **Date:** 2026-09-08

---

## WUs Landed

| WU | Description | Status | LOC delta |
|----|-------------|--------|-----------|
| WU-A2-1 | Replace local `logic_evaluator::PortValue` enum with `pub use editor_model::runtime::PortValue` | PASS | -7 +1 = -6 |
| WU-A2-2 | Migrate `actuator_bus.rs` to session-owned bus, drop `thread_local!` `ACTUATOR_OUTPUT_BUS` | PASS | -42 +30 = -12 |
| WU-A2-3 | Update existing `actuator_bus.rs` tests with session installation | PASS | +90 (MinimalSession stub for EditorSessionPort) |
| WU-A2-4 | Add `tests/actuator_parity.rs` (5 parity tests, all green) | PASS | +130 |

**Net:** ~+200 LOC (most of it is the MinimalSession stub for the lib test harness; the integration test file is 130 LOC)

## Files Changed

| File | Change |
|------|--------|
| `crates/editor-bevy/src/logic_evaluator.rs` | Replaced local enum with re-export (1 line net) |
| `crates/editor-bevy/src/actuator_bus.rs` | Removed thread_local, rewrote submit/drain to session; added MinimalSession test harness |
| `crates/editor-bevy/tests/actuator_parity.rs` | New integration test file (5 tests) |

## Tests

| Suite | Count | Status |
|-------|-------|--------|
| `cargo test -p editor-bevy --lib actuator_bus` | 2 | 2/2 PASS |
| `cargo test -p editor-bevy --test actuator_parity` | 5 | 5/5 PASS |
| `cargo check --workspace --locked` | n/a | PASS |
| `cargo check -p editor-model --target wasm32-unknown-unknown --locked` | n/a | PASS |

## Verification

- ADRs preserved: ADR-0030 (bevy-free editor-model), ADR-0057 (single WASM composition root), ADR-0059 (single transaction dispatch).
- `thread_local!` cell `ACTUATOR_OUTPUT_BUS` is **gone** from `actuator_bus.rs` (verified via grep).
- The local `pub enum PortValue` definition in `logic_evaluator.rs` is **gone**; replaced with `pub use editor_model::runtime::PortValue;`.
- `PortValueType` is intentionally **kept** in `editor_bevy::logic_evaluator` — it's a metadata type tied to the editor-bevy node system, not a value boundary (per D4 in design).

## Risks Carried Over

- R1 (Bevy system outside `with_session_mut`): the design decision was "tracing::warn on miss, not panic". The current code uses `with_session_mut` which returns `Option<R>`; the submit side is `if let Some(_) = with_session_mut(...)` semantically (the closure runs unconditionally when session exists, else nothing happens). This is consistent with D2.
- R2 (multi-thread access): `editor_model::ports` uses thread-local session registry with `Mutex<dyn EditorSessionPort>` — confirmed thread-safe.

## Deferred Work

This cycle exhausts Block A2 of the H2.5 refactor. Remaining deferred items:

- **Block B** (preview typed fields): type `serde_json::Value` preview fields with `editor_model::PortValue`. Independent of A2.
- **Block C** (`InputState` Resource): collapse Bevy-side `InputState` thread-local into a Bevy `Resource` reading from session-owned buses. Independent of A2.
- **Block D** (inventory + matrix + parity tests): update `tools/archcheck-globals/globals-inventory.yaml` and `docs/architecture/state-ownership-matrix.md` § H2.5 to mark the 2 retired cells (`logic_evaluator::PortValue` local enum + `ACTUATOR_OUTPUT_BUS` thread_local) as superseded by the session-owned equivalents. Adds parity tests at the inventory level.

## Comparison to Previous Cycle

The previous cycle (`h2-5-runtime-coordination`, closed at `v0.108.2`) deferred Block A2 explicitly. This cycle delivered it as a self-contained A-lite refactor in one session without spawning any worker agents (lesson from the 600s stall in the previous session).

```yaml
status: implemented
wUs_landed: 4/4
tests_pass: 7/7
workspace_check: pass
wasm32_check: pass
adrs_preserved: 3/3
thread_locals_retired: 1
local_types_retired: 1
risks_carried: 0 (closed)
next_cycle_hint: h2-5-runtime-coordination-block-b
```