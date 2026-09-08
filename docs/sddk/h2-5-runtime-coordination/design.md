# Design: h2-5-runtime-coordination

> **Change:** `h2-5-runtime-coordination` · **Phase:** design · **Path:** A-full
> **Authority:** `state-ownership-matrix.md` § H2.5 + `proposal.md` + `spec.md`
> **Generated:** 2026-09-08 (orchestrator synthesis from 2026-09-07 material)
> **Predecessors:** H2.1 (#184), H2.2 (#185, #186), H2.3 (#187), H2.4 (#188)

---

## Decisions

### D1 — Naming convention for runtime fields
- `EditorSession.runtime` gains: `command_bus: LinearBus`, `event_bus: LinearBus`, `actuator_outputs: ActuatorBus`, `hot_reload_requests: Vec<HotReloadRequest>`, `play_mode_request: Option<PlayModeRequest>`
- Rationale: state-ownership-matrix § H2.5 names them at high level (`bus`, `events`); spec refines to specific names to match each cell's role. Concrete names preferred over generic to prevent future collisions (e.g., if a second `events` field emerges elsewhere).
- Conflict with matrix: matrix uses `bus` / `events` (shorter). Spec uses `command_bus` / `event_bus` (more specific). We follow spec.

### D2 — Type Movement to editor-model (REQ-A-4 correction from HANDOFF)
- **Pure types move to `editor-model/src/runtime/`**:
  - `LinearBus` (struct + impl) → `editor-model/src/runtime/linear_bus.rs`
  - `ActuatorBus` (struct + impl) + `ActuatorOutput` (struct) → `editor-model/src/runtime/actuator_bus.rs`
  - `HotReloadRequest`, `PlayModeRequest` → `editor-model/src/runtime/hot_reload.rs`
- **Bevy-conversion wrappers stay in editor-bevy**:
  - `submit_actuator_output(entity: Entity, field: &str, value: PortValue)` — does `Entity::to_bits()` conversion, calls editor-model API
  - `apply_actuator_outputs` Bevy system (converts `entity_bits` back to `Entity`, writes components) — stays in editor-bevy
- Rationale: editor-model must stay bevy-free per ADR-0030 + archcheck B1. The wrapper-with-Entity stays at the editor-bevy boundary.

### D3 — Group B decision (typed backing + JSON projection)
- `PreviewInspectorState` gains `metrics_typed: PreviewMetrics`, `mapping_typed: Vec<PreviewMappingEntry>`, `provenance_typed: BTreeMap<StableId, PreviewProvenance>` alongside existing JSON fields.
- New `dirty: bool` flag + new Bevy system `project_preview_inspector_json` regenerates JSON on demand when `dirty == true`.
- Rationale: keeps existing WASM/TS consumers working (JSON shape unchanged); typed fields allow test parity without parsing JSON; projection is a single Bevy system that can be tested in isolation.

### D4 — Group C (InputState) design
- New Bevy `Resource editor_bevy::InputState { held_keys: HashSet<KeyCode>, dirty: bool }`.
- Inline `App::init_resource::<InputState>()` at the existing `App::new()` site (`preview_runtime.rs:167`) rather than a new `Plugin` type — minimizes touch surface.
- `update_keyboard_state` signature changes from `(mut state: ResMut<KEYBOARD_STATE wrapper>)` to `(keys: Res<ButtonInput<KeyCode>>, mut input_state: ResMut<InputState>)`.
- `KEYBOARD_STATE.with(...)` reads in sensor nodes replaced by `Res<InputState>::held_keys` access.

### D5 — Access pattern
- Group A/B (writes/reads through EditorSession): use `editor_model::ports::with_session_mut(|sess| sess.runtime.command_bus_mut().push(...))` closure pattern.
- Precedent: existing `runtime_delta_buffer_mut` at `session.rs:1105` + `preview_inspector::record_rebuild_cause` at `preview_inspector.rs` (uses `with_session_mut`).
- Returns `None` if no session registered → silent no-op (test-friendly + handles early-boot case EC-A-1).

### D6 — EditorSessionPort trait extension
- Add 5 new methods: `runtime_command_bus_mut`, `runtime_event_bus_mut`, `runtime_actuator_outputs_mut`, `runtime_hot_reload_requests_mut`, `runtime_play_mode_request_mut`.
- Pattern: `impl EditorSessionPort for EditorSession { fn runtime_command_bus_mut(&mut self) -> &mut LinearBus { &mut self.runtime.command_bus } }` at `session.rs` (consistent with existing `runtime_delta_buffer_mut`).

### D7 — Work unit decomposition
- **Block A — Runtime buses (5 WUs):**
  - WU-A-1: Move `LinearBus` to `editor-model/src/runtime/linear_bus.rs` (new file)
  - WU-A-2: Add `command_bus` + `event_bus` fields to `EditorSession.runtime`; add 2 EditorSessionPort methods; trampolines in `lib.rs:407,408`
  - WU-A-3: Split `ActuatorBus`: pure types → `editor-model/src/runtime/actuator_bus.rs`; Bevy wrapper stays in editor-bevy; field + port method + trampoline
  - WU-A-4: Move `HotReloadRequest` + `PlayModeRequest` types to `editor-model/src/runtime/hot_reload.rs`; add 2 fields + 2 port methods + trampolines in `hot_reload_state.rs`
  - WU-A-5: Parity tests green: hot_reload.rs (4), apply_actuator_outputs_in_preview.rs (4), play_mode.rs (4 with deferred update)
- **Block B — Preview state (3 WUs):**
  - WU-B-1: Add typed backing fields (`metrics_typed`, `mapping_typed`, `provenance_typed`) + `dirty: bool` to `PreviewInspectorState`
  - WU-B-2: Trampolines in `preview_inspector.rs` write to typed backing + set `dirty = true`
  - WU-B-3: New `project_preview_inspector_json` Bevy system + update `rebuild_cause_unified.rs` fixture; new `preview_state_parity.rs` test (Group D)
- **Block C — InputState (3 WUs):**
  - WU-C-1: Add `InputState` Resource + `init_resource` inline at app builder
  - WU-C-2: Update `update_keyboard_state` signature to `(Res<ButtonInput<KeyCode>>, ResMut<InputState>)`; sensor nodes read via `Res<InputState>`
  - WU-C-3: Update 3 `play_mode.rs::test_update_keyboard_state_*` tests for new signature; new `keyboard_input_state.rs` boot order test
- **Block D — Inventory + matrix + parity (3 WUs):**
  - WU-D-1: `tools/archcheck-globals/globals-inventory.yaml` — remove 9 H2.5 entries
  - WU-D-2: `docs/architecture/state-ownership-matrix.md` § H2.5 — mark RETIRED with PR ref
  - WU-D-3: `cargo fmt --all -- --check` + `cargo check` 6 crates + `cargo test --workspace --all-targets --locked` + archcheck + docs-check all PASS; UAT-ARCH-003/004 verification

### D8 — Migration ordering
- Apply order: **A → B → C → D** (each block independently buildable + testable)
- Per block: write code + tests in same WU; commit per WU; block-level commit message: `refactor(arch): H2.5 [block] — collapse [n] thread_locals`
- Final commit: `docs(arch): H2.5 RETIRED — state-ownership-matrix § H2.5 closed`

### D9 — Pre-existing failures to ignore
- `validation_center_tests::wasm_validation_cycle_in_active_graph` — unchanged on main; parity tests must not regress it
- `frontend/src/wasm/editor_application.d.ts` TS errors (21) — unchanged
- Prettier failures in `importers.ts` + `ImportDialog.tsx` — unchanged

### D10 — Out of scope (explicit non-goals)
- Promoting typed preview data to WASM/TS surface (Group B keeps JSON projection)
- New `InputStatePlugin` Plugin type (inline `init_resource` is sufficient)
- H2.6 / H2.x catch-all id-minting (`C` global in scene_asset_catalog.rs) — separate change
- H1.x ADAPTERS carry-over — separate change
- H4.4 DISPATCH_VIA_KERNEL gate closure — separate change

---

## Architecture diagram

```
┌────────────────────────────────────────────────────────────────┐
│  EditorSession (editor-application/session.rs)                │
│  ├─ runtime: RuntimeSessionState                              │
│  │  ├─ command_bus: LinearBus  [from editor-model/runtime/]  │
│  │  ├─ event_bus: LinearBus                                 │
│  │  ├─ actuator_outputs: ActuatorBus                        │
│  │  ├─ hot_reload_requests: Vec<HotReloadRequest>           │
│  │  └─ play_mode_request: Option<PlayModeRequest>          │
│  └─ preview_state: PreviewSessionState                      │
│     └─ preview_inspector: PreviewInspectorState             │
│        ├─ metrics_typed: PreviewMetrics [NEW]               │
│        ├─ mapping_typed: Vec<PreviewMappingEntry> [NEW]    │
│        ├─ provenance_typed: BTreeMap<...> [NEW]            │
│        ├─ dirty: bool [NEW]                                  │
│        ├─ metrics: String (JSON projection, regenerated)    │
│        └─ ... existing JSON fields                            │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│  editor-bevy Bevy App                                          │
│  ├─ Resources                                                  │
│  │  └─ InputState { held_keys, dirty } [NEW]                │
│  └─ Systems                                                    │
│     ├─ update_keyboard_state(keys, mut input_state)         │
│     ├─ project_preview_inspector_json (if dirty)            │
│     └─ apply_actuator_outputs (Bevy-converts bits→Entity)   │
└────────────────────────────────────────────────────────────────┘

Access pattern (editor-bevy → editor-application):
  with_session_mut(|sess| sess.runtime.command_bus_mut().push(x))
```

---

## Constraint conformance

| Constraint | Status | Evidence |
|------------|--------|----------|
| ADR-0030 (bevy-free editor-model) | ✅ | Pure types only in editor-model/src/runtime/; Bevy wrappers stay in editor-bevy (D2) |
| ADR-0057 (single WASM composition root) | ✅ | No new ambient cells; `EDITOR_SESSION` seam unchanged |
| ADR-0059 (single transaction dispatch) | ✅ | Trampolines preserve existing dispatch entry points; no new paths |
| archcheck B1 (no bevy in editor-model) | ✅ | D2: only pure types move |
| archcheck B8 (wasm-pollution free) | ✅ | No new free fns; all access through EditorSessionPort |
| H2 series consistency | ✅ | Same pattern as H2.2/H2.3/H2.4 |

---

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Spawn stuck 25% (lesson 2026-09-07) | Medium | Fall back to inline execution per block if 2-3 spawns stuck |
| Parity test regression | Low | Each WU includes test update; D9 explicitly excludes pre-existing failures |
| 14 WUs is non-trivial | Medium | Block A→B→C→D decomposition with independent buildability per block |
| Naming mismatch matrix vs spec (D1) | Low | Documented in D1; matrix can be updated if reviewer prefers |

---

```yaml
status: success
verdict: design_approved
constraints_met: 6
work_units: 14
blocks: [A:5, B:3, C:3, D:3]
risks: 4 (all LOW-MEDIUM)
apply_strategy: "single-slice per block; commit per WU"
precedent_prs: [H2.2_#185, H2.2_#186, H2.3_#187, H2.4_#188]
out_of_scope: [WASM typed preview, new Plugin, H2.6, H1.x, H4.4]
next_recommended: sddk-tasks (skip — tasks.md already produced 2026-09-07) OR direct sddk-apply Block A
```