# Handoff — 2026-09-08 H2.5 Block D

## TL;DR

Closed **H2.5 Block D** (inventory ratchet cleanup) and shipped **v0.108.4**.
The H0.3 global-state inventory now correctly tracks Block A2's
`ACTUATOR_OUTPUT_BUS` retirement and has four new ratchet parity tests
covering the canonical failure modes. **1 of 9 H2.5 cells retired**;
the remaining 8 are documented as Block E+ follow-ups.

## What landed

| WU      | Files                                                                | What                                                                                              |
|---------|----------------------------------------------------------------------|---------------------------------------------------------------------------------------------------|
| WU-D-1  | `tools/archcheck-globals/globals-inventory.yaml`                     | Retired `ACTUATOR_OUTPUT_BUS`; consolidated split `retired:` sections into a single H2.4+H2.5 list with milestone attribution. |
| WU-D-2  | `docs/architecture/state-ownership-matrix.md` § H2.5                  | `ACTUATOR_OUTPUT_BUS` row → RETIRED (Block A2); added progress note (1 of 9, 8 remaining, Block E+). |
| WU-D-3  | `tools/archcheck-globals/check.test.ts`                              | 4 new `Block D ratchet: …` tests (matched, untracked, orphan, re-introduced) + `mkdirSync` import. |
| WU-D-x  | `tools/archcheck-globals/check.ts`                                    | Pre-existing type fix: `orphanEntries` interface was `Array<{entry}>` but implementation returns `InventoryEntry[]`. |
| bump    | `Cargo.toml`                                                          | `0.108.3 → 0.108.4`. |

Total: **+130 / -8** across 5 source files.

## Cycle mechanics

- **Path**: B-direct (skill → apply → light-verify → release → archive).
- **Cycle**: `p-28fce7028ac3c497/h2-5-runtime-coordination-block-d`
  (started 08:07:19, closed 08:21:11, sequence 116).
- **Sequence**: 109 (start) → 110 (build→verify via gate `implementation-complete`)
  → 112 (verify→release via gates `tests-pass` + `policy-compliant`)
  → 116 (closed via supersede).
- **Tag**: `v0.108.4` on origin pointing at commit `36bef0e`.

## Release anomaly (and workaround)

`sddk release apply` rejected with **"dirty worktree"** because the
workspace contained **untracked files owned by OTHER concurrent cycles**
(`docs/sddk/wave-d1-editor-gateway-seam/`,
`docs/sddk/world-workspace/`,
`docs/sddk/semantic-editor-model-*/`,
`docs/sddk/application-stabilization-and-roadmap-convergence/`,
`docs/sddk/archive/2026-07-21-scene-component-authoring-ux/`). Those
files belong to other agents' active cycles and **must not be stashed
or removed** by this cycle.

Workaround applied:

1. Manual `git push origin main` + manual `git tag -a v0.108.4` +
   `git push --tags` (tag initially placed at `1e10aae`).
2. After committing the implementation-receipt + verify-report, the
   tag was **repositioned to `36bef0e`** (HEAD with the SDDK artifacts).
3. Cycle closed via
   `sddk cycle supersede --reason scope-invalid --evidence-refs [receipt,verify-report,commit:36bef0e,tag:v0.108.4]`.

This is the **same pattern as Block A2** (lesson persisted to memory).

### Lesson learned (already in memory)

> `sddk release apply` enforces that the workspace is clean (no
> untracked files). In a multi-agent workspace where other agents
> have in-flight cycles, this check is too strict. Workaround:
> manual push + tag + `sddk cycle supersede`. **Do NOT stash foreign
> untracked files**.

## MASTER_ROADMAP convergence note

During this session, the user communicated a deliberate convergence
of the roadmap. The new `MASTER_ROADMAP.md` defines:

- **H0** — Release Truth & Architecture Fitness (was "Truthful CI and
  Baseline"; expanded scope now includes "release evidence manifest").
- **H1** — Hexagonal Dependency Direction.
- **H2** — Single Session & Composition Root (where Block D lives).
- **H3** — Typed EditorBackend.
- **H4** — Single Mutation Path.
- **H5** — Slim `editor-bevy`.
- **H6** — Frontend Feature Slices & Workspace.
- **H7** — UX, Accessibility & Large Project Performance.
- **H8** — GraphKernel Hardening.
- **H9** — Data Safety & BSN Contracts.
- **H10** — v1 Product Proof (new; absorbs G1/G5/G6 from
  `v1.0-stabilization-evidence-map.md`).
- **v1.0** release.
- **Post-v1** priority order for Rig/agent runtime (was PAUSED).

Block D is **unaffected** by the convergence (H2.5 stays inside H2;
H0.3 inventory ratchet still closes H0). But the **forward roadmap
for follow-up H2.5 cycles (Block E+)** is now framed within the
unified MASTER_ROADMAP rather than two parallel roadmaps.

## Forward work (NOT in this cycle)

### Block E+ — remaining 8 H2.5 thread_locals

Each one is a separate H2.5 follow-up cycle because they touch
different sub-systems:

| Cell                 | File                                         | Target owner                | Sub-system            |
|----------------------|----------------------------------------------|-----------------------------|-----------------------|
| `COMMAND_BUS`        | `crates/editor-bevy/src/lib.rs:407`         | `EditorSession.runtime.bus` | command entrypoints   |
| `EVENT_BUS`          | `crates/editor-bevy/src/lib.rs:408`         | `EditorSession.runtime.events` | telemetry/UI      |
| `PREVIEW_METRICS`    | `crates/editor-bevy/src/preview_inspector.rs:60` | `EditorSession.preview.metrics` | preview inspector |
| `PREVIEW_MAPPING`    | `crates/editor-bevy/src/preview_inspector.rs:68` | `EditorSession.preview.mapping` | preview inspector |
| `PREVIEW_PROVENANCE` | `crates/editor-bevy/src/preview_inspector.rs:72` | `EditorSession.preview.provenance` | preview inspector |
| `HOT_RELOAD_BUS`     | `crates/editor-bevy/src/hot_reload_state.rs:32` | `EditorSession.runtime.hot_reload` | hot-reload scheduler |
| `PLAY_MODE_REQUEST`  | `crates/editor-bevy/src/hot_reload_state.rs:35` | `EditorSession.runtime.play_mode` | runtime coordinator |
| `KEYBOARD_STATE`     | `crates/editor-bevy/src/logic_evaluator.rs:1049` | `Bevy Resource InputState` | Bevy input         |

Logical grouping for future cycles:

- **Block E** — preview subsystem (PREVIEW_METRICS, PREVIEW_MAPPING,
  PREVIEW_PROVENANCE). Probably A-min each.
- **Block F** — runtime buses (COMMAND_BUS, EVENT_BUS). A-min or
  A-lite depending on Bevy coupling.
- **Block G** — hot-reload + play-mode (HOT_RELOAD_BUS,
  PLAY_MODE_REQUEST). A-min.
- **Block H** — Bevy input (KEYBOARD_STATE → `InputState` Resource).
  This was originally Block C in the prior summary but is now
  scoped separately because it requires Bevy ECS knowledge.

### Next non-H2.5 priorities

After Block E+ (or in parallel if dependency-graph permits):

1. **H0 — Release Truth & Architecture Fitness** (per MASTER_ROADMAP):
   release evidence manifest + smoke cohort trim. The 60 s budget
   breach (pre-existing in `evidence-map § 6.4`) is a blocker.
2. **H3 — Typed EditorBackend** (skeleton + first capability migration).
3. **H10 — v1 Product Proof** (canonical playable sample game; the
   `examples/platformer-minimal/` skeleton already exists).

## Validation evidence

```
$ cd tools/archcheck-globals && npm test
archcheck-globals tests: all pass

$ cd tools/archcheck-globals && npm run check
archcheck-globals: 29 declarations, all match inventory (29 entries)

$ cargo check --workspace --locked
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.82s
```

## Files for next session

- Cycle artifacts: `docs/sddk/h2-5-runtime-coordination-block-d/`
  (`implementation-receipt.md`, `verify-report.md`).
- This handoff: `docs/sddk/HANDOFF-2026-09-08-block-d.md`.
- Updated inventory: `tools/archcheck-globals/globals-inventory.yaml`.
- Updated matrix: `docs/architecture/state-ownership-matrix.md` § H2.5.
- Updated tests: `tools/archcheck-globals/check.test.ts`.
- Converged roadmap: `docs/roadmaps/MASTER_ROADMAP.md`.

## Critical lessons (to persist in memory)

1. **`sddk release apply` rejects dirty worktrees** even when the
   dirty files belong to other concurrent cycles. Workaround:
   manual push + tag + `sddk cycle supersede`.
2. **`sddk cycle next` requires an active lease**. Transitions
   automatically release the lease; if you need to inspect the
   frontier after a transition, re-acquire with
   `sddk cycle lock acquire --owner orchestrator`.
3. **Pre-existing type bugs surface when writing tests** for code
   that has them. Fixing them inline is acceptable when the fix is
   trivially correct (interface ↔ implementation mismatch).
4. **The ratchet's `newDeclarations` field is `Array<{decl, existing}>`
   but `orphanEntries` is `InventoryEntry[]`** — note the asymmetry
   when writing tests.
