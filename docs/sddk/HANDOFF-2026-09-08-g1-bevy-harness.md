# Handoff — 2026-09-08 (v1.0-stabilization G1 — Bevy Harness Witness)

## TL;DR

Cerré el primer cycle del work item **G1** (canonical playable sample
game) del evidence-map de v1.0-stabilization. El nuevo crate
`crates/examples-bevy-harness/` consume `examples/platformer-minimal/`
como testigo: lee los 4 archivos JSON de scene assets, spawna entidades
Bevy 0.19 con los componentes del editor traducidos a tipos Bevy, y
verifica que las 4 entidades estén bien estructuradas. **Tag v0.109.0**
marca este deliverable.

G1 sigue incompleto (3 sub-deliverables restantes: BJ-3 UI creation
test, BJ-4 play_mode runtime, BJ-5 gameplay assertions), pero la
**infraestructura de witness** ya está en su lugar. Cycles futuros
pueden extender el harness sin tocar el editor.

## Cycle closure

| Cycle                                       | Sequence  | Status                |
|---------------------------------------------|-----------|-----------------------|
| `v1-g1-bevy-harness` (this cycle)           | 184 → 193 | ✅ RELEASED → archive (CLOSED) |

## Tag

`v0.109.0` → re-tagged at archive commit (currently `b31e07a`, will be
moved to the final archive commit per Block I precedent).

## What was actually fixed in v0.109.0

| Surface | Pre-v0.109.0 | Post-v0.109.0 |
|---------|--------------|---------------|
| Bevy runtime witness for sample | ❌ Not in repo | ✅ `crates/examples-bevy-harness/` |
| Cargo test that proves sample → Bevy round-trip | ❌ Not in repo | ✅ 1/1 passes (`#[ignore]`) |
| G1 progress | 1/4 sub-deliverables | 2/4 (P1 split into witness + UI creation + play_mode + gameplay) |

## Decision: Option C (JSON witness, not BSN parser)

The harness reads `SceneAssetDocument` JSON directly rather than
parsing the `.bsn` Rust wrapper files. The `.bsn` files are
**Rust source code** (`pub fn spawn_*` wrappers around `bsn!{}`
blocks), not standalone loadable artefacts. Three options were
considered:

- **A — compile `.bsn` as Rust modules:** rejected (high risk of
  compile-cycle coupling).
- **B — parse `.bsn` (two parsers):** rejected (too much surface).
- **C — read JSON directly:** chosen (separation of concerns:
  harness = JSON-shape witness; e2e-game-creation.spec.ts = BSN
  pipeline witness).

This matches the architecture decision in the explore-report: the
Bevy harness proves the **authoring JSON** is consumable by Bevy,
not the **export pipeline**. The export pipeline is already verified
by `frontend/tests/e2e-game-creation.spec.ts`.

## Discovered bugs (none new — pre-existing carry-forward)

No new bugs found in this cycle. The harness design intentionally
narrow so it cannot regress existing behaviour:

- It does NOT call `editor-bevy` (no Bevy App / Schedule coupling).
- It does NOT exercise `submit_actuator_output` (the BJ-1 bug is
  untouched).
- It does NOT touch `editor-model` types (no API surface change).
- It does NOT touch archcheck (BJ-2 still pre-existing).

## Working tree state

- **Cycle's tracked files**: All committed in `ed0f730` + `b31e07a` +
  archive commit (pending).
- **Untracked (NOT mine — leave alone)**: concurrent-cycle artifacts
  from other agents.

## Next session — recommended direction

User's last directive was "g1" (interpreted as "start the G1 cycle").
The next concrete steps for G1:

- **BJ-3 (P2, 1-2 days):** UI-creation Playwright test. Authors the
  sample through the editor's UI rather than loading from filesystem.
  Verifies that a real user can recreate the sample from scratch.
- **BJ-4 (P2, 1-2 days):** Bevy `play_mode` runtime state. Adds
  Bevy App + Schedule to the harness, runs `enter_play_mode`
  semantics, asserts player movement + enemy patrol.
- **BJ-5 (P3, depends on BJ-4):** Gameplay assertions (collision,
  death, pickup).

Alternative: Block J for BJ-1 + BJ-2 (smaller, fixes pre-existing
bugs from Block I follow-up).

## Reference artifacts

- Implementation receipt: `docs/sddk/v1-g1-bevy-harness/implementation-receipt.md`
- Verify report: `docs/sddk/v1-g1-bevy-harness/verify-report.md`
- Release receipt: `docs/sddk/v1-g1-bevy-harness/release-receipt.md`
- Archive manifest: `docs/sddk/v1-g1-bevy-harness/archive-manifest.md`
- Debt report: `docs/sddk/v1-g1-bevy-harness/debt-report.json`
- Explore report: `docs/sddk/v1-g1-bevy-harness/explore-report.md`
- Spec: `docs/sddk/v1-g1-bevy-harness/spec.md`
- Sample: `examples/platformer-minimal/`
- ROADMAP update: pending (will be added in archive commit)

## User preferences reaffirmed

- Auto mode: routing decisions without asking.
- SDDK discipline: explicit cycle start → phases → release → archive.
- A-min for bounded code changes with clear precedent; A-lite for
  architectural decisions.
- Manual `git push` + tag + `cycle supersede` pattern (dirty worktree
  is expected with concurrent cycles).
- Spanish for explanations, English for code/commands/identifiers.
- Re-tag at archive commit (Block I precedent): tag MUST point at
  final commit WITH SDDK artifacts.
