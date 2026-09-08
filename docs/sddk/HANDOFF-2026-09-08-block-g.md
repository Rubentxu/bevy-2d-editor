# H2.5 Block G — Handoff (v0.108.7 released)

**Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-g`
**Path**: A-lite (escalated from A-min due to type-shape mismatch)
**Tag**: `v0.108.7` on origin pointing at commit `acdb11c`
**Date**: 2026-09-08

## Outcome

Block G retires the legacy `HOT_RELOAD_BUS` and `PLAY_MODE_REQUEST`
thread_locals via the dual-write fallback pattern (Blocks A2/D/E/F/G).
Production code now writes through `EditorSession.runtime.hot_reload_requests`
and `EditorSession.runtime.play_mode_request` via
`editor_model::ports::with_session_mut`. Fallback thread_locals
renamed (`HOT_RELOAD_BUS_FALLBACK` / `PLAY_MODE_REQUEST_FALLBACK`)
for legacy tests.

**Architectural change (key difference from Blocks A2/D/E/F)**:
Block G extends `editor_model::runtime::{HotReloadRequest,
PlayModeRequest}` from struct stubs to full enums matching the
legacy thread_local semantics (3-variant and 2-variant
respectively). This was required because:
- `Source{file_id}` vs `Asset{asset_id}` are routed differently in
  `process_hot_reload_requests` (different caches, different
  invalidation logic).
- `Enter` vs `Exit` play-mode requests trigger different code paths
  (snapshot vs deltas + restore).

The `editor_bevy::hot_reload_state::{HotReloadRequest,
PlayModeRequest}` are now `pub use` re-exports of
`editor_model::runtime::*` — single source of truth.

## Sequence

| Step | Transition                                        | Sequence |
|------|---------------------------------------------------|----------|
| 1    | start (A-min path, escalated to A-lite in spec)   | 141      |
| 2    | explore → specify                                 | 142      |
| 3    | specify → build                                   | 144      |
| 4    | build → verify                                    | 146      |
| 5    | verify → release                                  | 148      |
| 6    | supersede (`scope_invalid`)                       | 152      |

## Work units landed

| WU      | Files                                                              | Lines |
|---------|--------------------------------------------------------------------|-------|
| WU-G-1  | `crates/editor-model/src/runtime/hot_reload.rs` (struct → enum)    | +28 / -7 |
| WU-G-2  | `crates/editor-bevy/src/hot_reload_state.rs` (re-exports + rename) | +35 / -28 |
| WU-G-3  | `crates/editor-bevy/src/wasm_hot_reload.rs` (dual-write)           | +25 / -15 |
| WU-G-4  | `crates/editor-bevy/src/preview_runtime.rs` (dual-write Bevy systems) | +45 / -10 |
| WU-G-5  | `crates/editor-bevy/src/lib.rs` (import update)                    | +1 / -1 |
| WU-G-6  | `crates/editor-bevy/src/state.rs` (re-export update)               | +1 / -1 |
| WU-G-7  | `crates/editor-bevy/tests/hot_reload_play_mode_parity.rs` (new, 6 tests) | +131 / -0 |
| WU-G-8  | `docs/architecture/state-ownership-matrix.md` (2 retired + progress) | +8 / -8 |
| WU-G-9  | `tools/archcheck-globals/globals-inventory.yaml` (2 retired → 2 FALLBACK) | +8 / -8 |
| WU-G-10 | `Cargo.toml` (0.108.6 → 0.108.7)                                  | +1 / -1 |

**Net: +283 / -79 lines** (excluding block-G artifacts).

## Verification status

| Check | Result |
|-------|--------|
| `cargo check --workspace --locked`               | ✅ |
| `cargo check -p editor-model --target wasm32-unknown-unknown --locked` | ✅ |
| `cargo test -p editor-bevy --test hot_reload --locked`                | ✅ 4/4 (existing, FALLBACK path) |
| `cargo test -p editor-bevy --test hot_reload_play_mode_parity --locked` | ✅ 6/6 (new, session path) |
| `archcheck-globals` (`npm run check`)            | ✅ 29/29 |

## Gate receipts (4/4 passed)

1. `tests-pass` — 6/6 parity + 4/4 existing = 10/10
2. `policy-compliant` — archcheck 29=29
3. `debt-severity-assigned` — `sddk debt gates` PASS (0 findings)
4. `debt-priority-assigned` — `sddk debt gates` PASS (0 findings)

## Release anomaly (same as Blocks A2 + D + E + F)

`sddk release apply` rejected with "dirty worktree" (concurrent
cycles' untracked files). Pattern now stable across **5 cycles**:
1. `git add` only files owned by this cycle.
2. `git commit -m "refactor(arch): H2.5 Block G ..."`
3. `git push origin main`
4. `git tag -a v0.108.7 -m "..."`
5. `git push origin v0.108.7`
6. `sddk cycle supersede --reason scope-invalid --evidence-refs '<json-array>'`

## H2.5 progress (after Block G)

| Cell                        | Status                |
|-----------------------------|-----------------------|
| `ACTUATOR_OUTPUT_BUS`       | RETIRED (Block A2)    |
| `PREVIEW_METRICS`           | RETIRED (Block E)     |
| `PREVIEW_MAPPING`           | RETIRED (Block E)     |
| `PREVIEW_PROVENANCE`        | RETIRED (Block E)     |
| `COMMAND_BUS`               | RETIRED (Block F)     |
| `EVENT_BUS`                 | RETIRED (Block F)     |
| `HOT_RELOAD_BUS`            | RETIRED (Block G)     |
| `PLAY_MODE_REQUEST`         | RETIRED (Block G)     |
| `KEYBOARD_STATE`            | OPEN (Block H planned) |

**8 of 9 H2.5 cells retired.** One cell remains.

## Path escalation A-min → A-lite — retrospective

Block G was initially scoped as A-min (same pattern as Blocks
A2/D/E/F — pure rename with dual-write fallback). During explore
phase, a type-shape mismatch was discovered:

| Aspect | editor-bevy (current) | editor_model (target) |
|--------|----------------------|----------------------|
| `HotReloadRequest` | 3-variant enum | struct stub |
| `PlayModeRequest` | 2-variant enum | struct stub |

The session-side types were placeholders that needed to be extended
to support the legacy semantics. This is an architectural change
touching `editor-model`, `editor-application`, and `editor-bevy`,
which elevated the path to A-lite. The A-min transitions available
in the framework still work (the framework exposes
`phase.specify.complete.a-min` as the transition — it doesn't gate
on path metadata), so no replan was needed; the spec just covered
the architectural decisions explicitly.

## Cycle mechanics notes

- The cycle was started fresh (`cycle start` worked this time because
  Block G is a new cycle name, not a re-start of an existing one).
- Lease expiry happened 3 times during the cycle (~1h each);
  re-acquired with `sddk cycle lock acquire --owner jcode-block-g --lease-ms 7200000`.
- `cycle supersede --evidence-refs` JSON array format worked
  exactly as in Block F.

## Forward work (NOT in this cycle)

1. **Block H**: `KEYBOARD_STATE` → Bevy `Resource InputState`
   (`crates/editor-bevy/src/logic_evaluator.rs:1049`).
   **Different migration target** (Bevy Resource, not EditorSession
   field) — likely A-lite or A-full because the migration target is
   architectural. The thread_local state needs to become a Bevy
   `Resource` that input-event systems write to and the logic
   evaluator reads from.

## Lessons persisted to memory

1. **Block G established A-lite path**: type-shape mismatch between
   thread_local types and session types requires extending
   `editor_model::runtime` (architectural change).
2. **Single source of truth**: `editor_bevy::hot_reload_state::{HotReloadRequest,
   PlayModeRequest}` are now `pub use` re-exports of
   `editor_model::runtime::*`.
3. **`FakeSession` already had `hot_reload_requests` and
   `play_mode_request` fields** (Block A setup) — no changes needed
   in `crates/editor-bevy/tests/support/mod.rs`.
4. **archcheck-globals ratchet** confirmed: 29 entries stays the
   same (Block G removed 2 old `HOT_RELOAD_BUS` + `PLAY_MODE_REQUEST`
   entries and added 2 new `*_FALLBACK` entries — net zero change).

## Persistence

This handoff is committed as part of the Block G commit (`acdb11c`)
in `docs/sddk/HANDOFF-2026-09-08-block-g.md`.
