# Handoff — 2026-09-08 (Block I — Smoke Budget + Cargo Conformance Restore)

## TL;DR

Cerré el cleanup cycle `h2-5-fallback-cleanup` (sequence 167 → 171) por
supersede, documentando la decisión en **ADR-0064**: los 8
`*_FALLBACK` thread_locals se retienen como capa de compatibilidad
permanente, no como tech debt transitorio. Inicié un nuevo cycle
B-direct `smoke-budget-cargo-conformance-restore` (sequence 176 → 181)
que arregla dos pre-existing release-health blockers detectados al
investigar el cleanup:

1. **`cargo check --workspace --tests` fallaba** porque 3 integration
   test files (`state_migration_pr2`, `runtime_delta_wiring`,
   `state_unified`) definen sus propios `FakeSession`/`FakeSessionWithCap`
   que envuelven `support::FakeSession` pero no delegaban los 5 nuevos
   métodos `runtime_*_mut` que H2.5 Block A añadió al trait
   `EditorSessionPort`. La regresión fue enmascarada en Block H porque
   la verify-report solo ejerció la suite de parity tests, no la suite
   completa. **Bug pre-existente del handoff Block H**, no introducido
   por este cycle.

2. **Smoke cohort 60 s budget breach** porque
   `playwright.smoke.config.ts` listaba 6 specs por nombre en
   `testMatch`, incluyendo `engine.spec.ts` (~1.2 min) y otros specs
   `@full`/`@domain`. El evidence-map (P8) recomendaba mover
   `engine.spec.ts` al full cohort. **Pre-existing**, no
   introducido por este cycle.

## Cycle closures

| Cycle                                             | Sequence  | Status                |
|---------------------------------------------------|-----------|-----------------------|
| `h2-5-fallback-cleanup`                           | 167 → 171 | CLOSED via supersede (external-obsolete). Evidence: ADR-0064 + 13 files FALLBACK scope analysis. |
| `smoke-60s-budget-fix` (intermediate, abandoned)  | 172 → 175 | CLOSED via supersede (external-obsolete). Blocked by pre-existing cargo breakage. Smoke change preserved in working tree. |
| `smoke-budget-cargo-conformance-restore`          | 176 → 181 | ✅ RELEASED → v0.108.9 |

## Tag

`v0.108.9` → `b7683e798f1ebf293492b97df5c778b3e7f47cb7` (HEAD = origin/main).

## State at handoff

- **HEAD**: `b7683e7` (`chore(release): bump version 0.108.8 → 0.108.9 (Block I)`)
- **Tag**: `v0.108.9` points at `b7683e7` (verified).
- **archcheck-globals ratchet**: 29=29 (unchanged — `*_FALLBACK` symbols already excluded via B8 rule per ADR-0064).
- **Cycle**: `p-28fce7028ac3c497/smoke-budget-cargo-conformance-restore` is in RELEASED → archive phase; archive-manifest pending.

## What was actually fixed in v0.108.9

| Surface | Pre-v0.108.9 | Post-v0.108.9 |
|---------|--------------|---------------|
| `cargo check --workspace --tests` | FAIL (E0046 missing runtime_*_mut impls) | ✅ PASS |
| Playwright smoke cohort size | 6 files, 60+ tests, ~3 min | ✅ 4 files, 39 tests, expected <60s |
| `engine.spec.ts` cohort | smoke (60s budget breach) | ✅ full (296 tests in 40 files) |
| ADR coverage for `*_FALLBACK` | "transient, to be cleaned" (Block H handoff) | ✅ "permanent compat layer" (ADR-0064) |
| H2.5 milestone entry in ROADMAP | Missing | ✅ Added (v0.108.0 → v0.108.8 row) |

## Decision: ADR-0064 (`*_FALLBACK` as compat layer)

The 8 `*_FALLBACK` thread_locals — `COMMAND_BUS_FALLBACK`,
`EVENT_BUS_FALLBACK`, `HOT_RELOAD_BUS_FALLBACK`,
`PLAY_MODE_REQUEST_FALLBACK`, `KEYBOARD_STATE_FALLBACK`,
`PREVIEW_METRICS_FALLBACK`, `PREVIEW_MAPPING_FALLBACK`,
`PREVIEW_PROVENANCE_FALLBACK` — are **retained as a permanent
compatibility layer** for tests that exercise the bus surface without
installing an `EditorSession` or a Bevy `Resource`. The fallback path is
never exercised in production (canonical owner always wins). Removing it
would invalidate the parity tests' purpose and require updating 13
files for minimal benefit. Cost/benefit is poor.

See `docs/adr/0064-fallback-thread-locals-as-permanent-compat-layer.md`
for full rationale, considered options, and references.

## Discovered bugs (NOT fixed in v0.108.9 — follow-up for Block J)

1. **3 pre-existing `logic_evaluator` integration test failures**:
   `test_submit_and_drain`, `test_entity_bits_preserved_in_bus`,
   `test_end_to_end_actuator_pipeline` in
   `crates/editor-bevy/src/logic_evaluator.rs` (lines 1704/2026/2040).
   Root cause: `submit_actuator_output()` (in `actuator_bus.rs`)
   requires `editor_model::ports::with_session_mut(|s| …)` which
   returns `None` when no session is installed. The H2.5 Block A2
   refactor removed the FALLBACK for `actuator_outputs` without
   rewriting these tests to install a session. Block J should either
   add a FALLBACK for `actuator_outputs` or rewrite the tests.

2. **`archcheck` rule B1 false-positives**: regex `/bevy::/` matches
   `editor_bevy::command` substrings in 7 source files
   (`command.rs`, `operation_log.rs`, `scene_focus.rs`,
   `world_command.rs`, `asset_operation_log.rs`, `validation.rs`,
   `runtime/hot_reload.rs`, `session_port.rs`), all in doc-comments.
   Block J should refine the regex (e.g. negative lookbehind for
   `_`) or rewrite comments to use plain "the editor-bevy crate"
   language.

## Working tree state

- **Cycle's tracked files**: All committed in `c6535c7` + `b7683e7`.
- **Untracked (NOT mine — leave alone)**: concurrent-cycle artifacts
  from other agents:
  - `docs/sddk/application-stabilization-and-roadmap-convergence/`
  - `docs/sddk/archive/2026-07-21-scene-component-authoring-ux/`
  - `docs/sddk/semantic-editor-model-{adapter-contract,s2-impls,s3-migrations,s4-extension-bags}/`
  - `docs/sddk/wave-d1-editor-gateway-seam/`
  - `docs/sddk/world-workspace/`

## Next session — recommended direction

User's last directive was "según tu criterio". The path forward should
consider:

- **Block J**: Fix the 3 logic_evaluator test failures (small, bounded).
- **Block K**: Refine `archcheck` rule B1 regex (small, bounded).
- **G1 (canonical playable sample game)**: High-value v1.0-stabilization
  gate (large, A-full path). This is the biggest impact vs. cost gap.
- **Other H2.5 follow-ups**: see ROADMAP Active Work row.

The pattern of "discover pre-existing bug while doing scoped work" is
recurring (Block H handoff masked cargo breakage; Block I found 2 more
pre-existing issues). A future "test-infrastructure-health-check" cycle
might be worthwhile to surface all pre-existing failures in one shot.

## Reference artifacts

- Implementation receipt: `docs/sddk/smoke-budget-cargo-conformance-restore/implementation-receipt.md`
- Verify report: `docs/sddk/smoke-budget-cargo-conformance-restore/verify-report.md`
- Release receipt: `docs/sddk/smoke-budget-cargo-conformance-restore/release-receipt.md`
- ADR-0064: `docs/adr/0064-fallback-thread-locals-as-permanent-compat-layer.md`
- ROADMAP update: `docs/ROADMAP.md` (Active Work section, H2.5 + Block I rows)

## User preferences reaffirmed

- Auto mode: routing decisions without asking.
- SDDK discipline: explicit cycle start → phases → release → archive.
- B-direct for metadata/config changes.
- Manual `git push` + tag + `cycle supersede` pattern (dirty worktree is
  expected with concurrent cycles).
- Spanish for explanations, English for code/commands/identifiers.
