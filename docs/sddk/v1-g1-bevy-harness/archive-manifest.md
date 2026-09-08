# Archive Manifest — `v1-g1-bevy-harness`

> **Cycle:** `p-28fce7028ac3c497/v1-g1-bevy-harness`
> **Delivery:** `code-delivery` (delivered via v0.109.0 tag)
> **Closed at:** 2026-09-08

---

## Closure Summary

| Field | Value |
|-------|-------|
| Verdict | RELEASED + KNOWN-FOLLOW-UP-ACKNOWLEDGED |
| Path | A-min |
| Branch | `main` |
| Tag | `v0.109.0` |
| Released SHA | `b31e07a` (will be re-tagged to archive commit) |
| Commits in cycle | 2 (`ed0f730` feature + `b31e07a` version bump) |
| UAT | skipped (release_type=minor, policy self-attest) |

## Release Receipt

| Field | Value |
|-------|-------|
| Release tag | `v0.109.0` |
| Release SHA | `b31e07a` (re-tagged to archive commit) |
| Route | local |
| Cycle sequence | 184 → 193 |
| Status | RELEASED → archive |

## Work Units Landed

| WU       | Description                                                                          | Files                                                                              |
|----------|--------------------------------------------------------------------------------------|------------------------------------------------------------------------------------|
| WU-G1-1  | Add `crates/examples-bevy-harness` to workspace members                              | `Cargo.toml` (+1 / -1)                                                             |
| WU-G1-2  | New harness crate manifest (Bevy 0.19, editor-model, serde_json)                    | `crates/examples-bevy-harness/Cargo.toml` (new, 20 lines)                          |
| WU-G1-3  | Public API: re-exports + `spawn_all_assets`                                          | `crates/examples-bevy-harness/src/lib.rs` (new, 30 lines)                          |
| WU-G1-4  | Bevy component mirrors of editor schemas                                             | `crates/examples-bevy-harness/src/components.rs` (new, 68 lines)                   |
| WU-G1-5  | SceneAssetDocument → Bevy World translation                                          | `crates/examples-bevy-harness/src/loader.rs` (new, 200 lines)                      |
| WU-G1-6  | `#[ignore]` integration test (sample round-trips into Bevy world)                    | `crates/examples-bevy-harness/tests/load_sample.rs` (new, 142 lines)               |
| WU-G1-7  | Cycle explore-report                                                                 | `docs/sddk/v1-g1-bevy-harness/explore-report.md` (new, 143 lines)                  |
| WU-G1-8  | Cycle specification                                                                  | `docs/sddk/v1-g1-bevy-harness/spec.md` (new, 179 lines)                            |
| WU-G1-9  | Cycle implementation receipt                                                         | `docs/sddk/v1-g1-bevy-harness/implementation-receipt.md` (new, 102 lines)          |
| WU-G1-10 | Cycle verify report                                                                  | `docs/sddk/v1-g1-bevy-harness/verify-report.md` (new, 78 lines)                    |
| WU-G1-11 | Cycle debt report                                                                    | `docs/sddk/v1-g1-bevy-harness/debt-report.json` (new, 70 lines)                    |
| WU-G1-12 | Cycle release receipt                                                                | `docs/sddk/v1-g1-bevy-harness/release-receipt.md` (new, 59 lines)                  |
| WU-G1-13 | Cycle archive manifest                                                               | `docs/sddk/v1-g1-bevy-harness/archive-manifest.md` (this file)                     |

## Deferred Work Acknowledged (Block J+ for G1)

| ID   | Description                                                                                | Priority | Source                                                              |
|------|--------------------------------------------------------------------------------------------|----------|---------------------------------------------------------------------|
| BJ-3 | UI-creation Playwright test (author sample via editor UI)                                 | P2       | Evidence-map §7-P1 second sub-deliverable                            |
| BJ-4 | Bevy `play_mode` runtime state assertions                                                  | P2       | Evidence-map §7-P1 fourth sub-deliverable                            |
| BJ-5 | Gameplay assertions (movement, collision, pickup)                                           | P3       | G1 description (gameplay not required, only structural correctness) |

## Carry-Forward from Block I

| ID   | Description                                                  | Priority |
|------|--------------------------------------------------------------|----------|
| BJ-1 | 3 pre-existing logic_evaluator integration test failures     | P1       |
| BJ-2 | archcheck rule B1 + B2 false-positives                        | P2       |

## Acceptance Criteria (closed)

| Criterion                                                                  | Status | Evidence |
|----------------------------------------------------------------------------|--------|----------|
| `crates/examples-bevy-harness/` exists and is in workspace members        | ✅ PASS | `Cargo.toml` line 2 lists the new crate. |
| `cargo check -p examples-bevy-harness` exits 0                            | ✅ PASS | `Finished dev profile in 49.58s`. |
| `cargo build -p examples-bevy-harness` exits 0                            | ✅ PASS | `cargo test -p examples-bevy-harness -- --ignored` compiles and runs. |
| `cargo test -p examples-bevy-harness -- --ignored` runs ≥1 test that passes | ✅ PASS | `test result: ok. 1 passed; 0 failed`. |
| Test loads 4 scene asset JSON files into `SceneAssetDocument`               | ✅ PASS | Test asserts 1 entity per asset + matching logical_path. |
| Test spawns 4 Bevy entities via `spawn_all_assets`                         | ✅ PASS | `spawned == 4`. |
| Test asserts `Name` values match `{Enemy, Ground, Pickup, Player}`         | ✅ PASS | `names == expected`. |
| Test asserts Player entity carries harness `PlayerController` component    | ✅ PASS | `player.speed == 200.0`, `player.jump_force == 400.0`. |
| Existing `cargo check --workspace --tests` (Block I) still green           | ✅ PASS | `Finished dev profile in 5.37s`. |

## Decision Rationale

The harness reads `SceneAssetDocument` JSON directly (Option C in the
explore report) rather than parsing the `.bsn` Rust wrapper files.
This separation of concerns makes the harness a **witness** for the
editor's authoring JSON, while the existing
`frontend/tests/e2e-game-creation.spec.ts` continues to witness the
editor's BSN export pipeline.

## References

- `docs/sddk/v1-g1-bevy-harness/explore-report.md`
- `docs/sddk/v1-g1-bevy-harness/spec.md`
- `docs/sddk/v1-g1-bevy-harness/implementation-receipt.md`
- `docs/sddk/v1-g1-bevy-harness/verify-report.md`
- `docs/sddk/v1-g1-bevy-harness/debt-report.json`
- `docs/sddk/v1-g1-bevy-harness/release-receipt.md`
- `examples/platformer-minimal/README.md`
- `docs/v1.0-stabilization-evidence-map.md` §6.1, §7-P1
- `docs/roadmaps/v1.0-stabilization.md` §G1
- ADR-0005 (Scene Asset as the BSN-aligned reusable scene model)
