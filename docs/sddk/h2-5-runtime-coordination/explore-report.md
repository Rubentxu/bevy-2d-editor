# Exploration Report: `h2-5-runtime-coordination`

> **Change:** `h2-5-runtime-coordination` · **Phase:** explore · **Path:** A-full
> **Generated:** 2026-09-08 (orchestrator synthesis from session 2026-09-07 material)
> **Authority:** `state-ownership-matrix.md` § H2.5 · **Predecessors:** H2.1 (#184), H2.2 (#185, #186), H2.3 (#187), H2.4 (#188)
> **Context quality:** C3 (intent + scope + cells + group decomposition + spec correction all already documented in proposal.md, spec.md, HANDOFF-2026-09-07.md)

---

## Summary

| Field | Value |
|-------|-------|
| Status | exploration complete (synthesized from 2026-09-07 in-session artifacts) |
| Cells in scope | 9 `thread_local!` in `crates/editor-bevy/src/` |
| Groups | A (runtime buses, 5 WUs), B (preview typed fields, 3 WUs), C (InputState Resource, 3 WUs), D (inventory + matrix + parity, 3 WUs) |
| Total WUs | 14 |
| Path | A-full (architectural change with multi-lens verification) |
| Context quality | C3 |
| Taxonomy | architectural-collapse (retire thread_locals, route through EditorSession/EditorSessionPort/InputState Resource) |

---

## Problem statement

Nine `thread_local!` cells in `crates/editor-bevy/src/` hold live runtime, preview-inspector and hot-reload coordination state that has no business being thread-local:

| # | Cell | `declared_at` | Group |
|---|------|----------------|-------|
| 1 | `COMMAND_BUS` | `crates/editor-bevy/src/lib.rs:407` | A |
| 2 | `EVENT_BUS` | `crates/editor-bevy/src/lib.rs:408` | A |
| 3 | `ACTUATOR_OUTPUT_BUS` | `crates/editor-bevy/src/actuator_bus.rs:53` | A |
| 4 | `HOT_RELOAD_BUS` | `crates/editor-bevy/src/hot_reload_state.rs:32` | A |
| 5 | `PLAY_MODE_REQUEST` | `crates/editor-bevy/src/hot_reload_state.rs:35` | A |
| 6 | `PREVIEW_METRICS` | `crates/editor-bevy/src/preview_inspector.rs:60` | B |
| 7 | `PREVIEW_MAPPING` | `crates/editor-bevy/src/preview_inspector.rs:68` | B |
| 8 | `PREVIEW_PROVENANCE` | `crates/editor-bevy/src/preview_inspector.rs:72` | B |
| 9 | `KEYBOARD_STATE` | `crates/editor-bevy/src/logic_evaluator.rs:1049` | C |

Each is marked `owner: H2.5` in `tools/archcheck-globals/globals-inventory.yaml`. Total inventory has 65 entries, 30 with `owner: H.x` (15 already retired by H2.1–H2.4).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/editor-application/src/session.rs` | modified | Add `runtime.{command_bus, event_bus, actuator_outputs, hot_reload_requests, play_mode_request}` fields (Group A) |
| `crates/editor-application/src/session.rs` | modified | Extend `preview_state.preview_inspector` with `metrics_typed`, `mapping_typed`, `provenance_typed` + `dirty: bool` (Group B) |
| `crates/editor-model/src/session_port.rs` | modified | Add 5 new methods for Group A fields; pattern follows existing `runtime_delta_buffer_mut` precedent |
| `crates/editor-model/src/runtime/` (new module) | new | Pure runtime types moved here (LinearBus, ActuatorBus pure struct, HotReloadRequest, PlayModeRequest) |
| `crates/editor-model/src/runtime/actuator_bus.rs` | new | Pure ActuatorBus + ActuatorOutput types (per REQ-A-4 correction) |
| `crates/editor-bevy/src/actuator_bus.rs` | modified | Wrapper stays; Bevy `Entity`-conversion on producer-side API |
| `crates/editor-bevy/src/preview_inspector.rs` | modified | Trampolines + new `project_preview_inspector_json` Bevy system (Group B) |
| `crates/editor-bevy/src/hot_reload_state.rs` | modified | Trampolines via `with_session_mut` (Group A) |
| `crates/editor-bevy/src/logic_evaluator.rs` | modified | Trampolines via `with_session_mut` (Group A actuator bus); KEYBOARD_STATE reads via `Res<InputState>` (Group C) |
| `crates/editor-bevy/src/preview_runtime.rs` | modified | `App::init_resource::<InputState>()` inline at app builder (Group C); `update_keyboard_state` signature change to `(Res<ButtonInput<KeyCode>>, ResMut<InputState>)` |
| `tools/archcheck-globals/globals-inventory.yaml` | modified | Remove 9 H2.5 entries |
| `docs/architecture/state-ownership-matrix.md` | modified | § H2.5 marked RETIRED with PR ref |
| Tests (parity) | modified | `crates/editor-bevy/tests/hot_reload.rs` (4), `play_mode.rs` (4 with signature updates), `apply_actuator_outputs_in_preview.rs` (4), `rebuild_cause_unified.rs` (4 fixture updates) |
| Tests (new) | new | `crates/editor-bevy/tests/keyboard_input_state.rs` boot order (Group C); `preview_state_parity.rs` (Group B) |

## Approaches Considered

### Approach 1: Group A into `EditorSession.runtime` + Group B into typed preview fields + Group C into Bevy Resource (RECOMMENDED)
- **Pros**: Mirrors H2.2/H2.3/H2.4 pattern; consistent with `state-ownership-matrix.md`; clean dep direction (editor-application is the only owner, editor-bevy routes through it); `InputState` Resource has strictly Bevy-bound lifetime (correct for input state)
- **Cons**: 14 WUs is non-trivial; touches 4 parity test files; requires pre-investigation on EditorSession layout (done in HANDOFF)
- **Effort**: ~3-5 days (1 PR large or 4 PRs one-per-block)

### Approach 2: All cells into a single `editor_bevy::RuntimeCoordinator` struct
- **Pros**: Fewer files touched
- **Cons**: Violates dep direction (editor-application → editor-bevy would be required to access coordinator); contradicts ADR-0057 (single composition root); introduces cross-crate coupling not present in H2.2-H2.4
- **Effort**: ~2 days but with high architectural debt

### Approach 3: Keep thread_locals, add Documentation-only changes
- **Pros**: Zero risk
- **Cons**: Doesn't address the actual problem (multi-session re-entrancy, test isolation, hidden coupling); state-ownership-matrix § H2.5 commitment is to retire, not document
- **Effort**: <1 day but doesn't ship H2.5

## Recommendation

**Approach 1** — same pattern as H2.2 (#185, #186), H2.3 (#187), H2.4 (#188) precedents. The HANDOFF-2026-09-07 already contains a complete pre-investigation including:
- Group B decision (option b — typed backing + JSON projection via `dirty` flag + new `project_preview_inspector_json` system)
- REQ-A-4 correction (ActuatorBus split: pure types to editor-model, Bevy wrapper stays in editor-bevy)
- Pre-investigation WU decomposition (Block A=5 WUs, B=3 WUs, C=3 WUs, D=3 WUs)

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Spec says `ActuatorBus` moves to editor-model but the module-level producer API uses Bevy `Entity` | Solved | HANDOFF REQ-A-4 correction: pure types to editor-model, Bevy wrapper stays in editor-bevy |
| Spawn reliability 25% (lesson from 2026-09-07) | Medium | If spawn stuck 2-3 times, fall back to inline execution per process lessons |
| `update_keyboard_state` signature change breaks 3 play_mode.rs tests | Low | Update tests alongside the signature change; same WU-C-3 |
| `rebuild_cause_unified.rs` fixture changes when PREVIEW_PROVENANCE moves | Low | Update fixture in same WU-B-2; parity test proves equivalence |
| `LinearBus` and `ActuatorBus` use bevy types in `apply_actuator_outputs` | Low | System stays in editor-bevy; only pure structs move to editor-model |

## Pre-investigation (from HANDOFF-2026-09-07)

**EditorSession layout** (`crates/editor-application/src/session.rs`):
- `preview_state: PreviewSessionState` with `preview_inspector: PreviewInspectorState` (lines 234-276)
- `runtime: RuntimeSessionState` with `runtime_delta_buffer: VecDeque` + `tunable_baselines: BTreeMap` (lines 366-372)

**EditorSessionPort trait** (`crates/editor-model/src/session_port.rs`):
- 20 existing methods; pattern `self.runtime.<field>_mut()` already used (e.g. `runtime_delta_buffer_mut`)
- For Block A need 5 NEW methods; pattern precedent exists at `session.rs:1105`

**Access seam** (`crates/editor-model/src/ports.rs`):
- `EDITOR_SESSION: RefCell<Option<Arc<Mutex<dyn EditorSessionPort>>>>` registered once at WASM startup
- `with_session_mut(|sess| ...)` closure pattern — precedent at `preview_inspector::record_rebuild_cause`

**Parity tests** (16 total, MUST pass):
- `crates/editor-bevy/tests/hot_reload.rs` (4)
- `crates/editor-bevy/tests/play_mode.rs` (4) — 3 require `update_keyboard_state_*` signature update
- `crates/editor-bevy/tests/apply_actuator_outputs_in_preview.rs` (4)
- `crates/editor-bevy/tests/rebuild_cause_unified.rs` (4) — fixture touched by Block B

## Ready for Proposal

**Yes.** Proposal (`proposal.md`) + Spec (`spec.md`) + Tasks (`tasks.md`) + HANDOFF (`HANDOFF-2026-09-07.md`) already on disk from 2026-09-07. Skipping re-proposal/spec/tasks generation; orchestrator proceeds to specify → apply.

## Process notes (from 2026-09-07)

- **Model id**: use `minimax-coding-plan/MiniMax-M3` (not `MiniMax-M3`; latter needs `OPENROUTER_API_KEY`)
- **FRAMEWORK path**: `FRAMEWORK=$(sddk version | awk '/^resolved:/{print $2}')` at start of every spawn prompt
- **Recovery**: after 2+ stuck spawns, kill agent and do phase work inline

---

```yaml
status: success
verdict: exploration_complete
context_quality: C3
taxonomy: [architectural-collapse, state-ownership-matrix-h2, thread_local-retirement]
cells_in_scope: 9
groups: [A:5_WUs, B:3_WUs, C:3_WUs, D:3_WUs]
total_work_units: 14
parity_tests_required: 16 (4 files)
precedent_prs: [H2.2_#185, H2.2_#186, H2.3_#187, H2.4_#188]
risks: 5 (1 SOLVED, 4 LOW-MEDIUM)
next_recommended: sddk-spec (skipped — spec.md already approved 2026-09-07) OR direct sddk-tasks refinement
```