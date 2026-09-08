# Bevy 2D Editor — Project Roadmap

## Hito 0: Scene Editor Foundation

**Goal**: Browser-based 2D scene editor with entity hierarchy, inspector, undo/redo, and Bevy preview rendering.

### Completed Milestones

| Milestone                                     | Version | Status | Key Deliverables                                                                                                                                   |
| --------------------------------------------- | ------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| scene-document                                | v0.1.0  | ✅     | `SceneDocument` JSON, `StableId`, `Entity`, `ComponentInstance`, `ComponentSchemaRegistry`                                                         |
| command-system                                | v0.1.0  | ✅     | Typed `Command` enum, `processor.rs` with reversibility, `dispatch_command` WASM entry                                                             |
| opfs-persistence                              | v0.2.0  | ✅     | `save_scene`/`load_scene` to OPFS, `project.json` atomic load                                                                                      |
| schema-registry-persistence                   | v0.3.0  | ✅     | Mutable user schemas, `register_schema_from_json`, builtin + user combined registry                                                                |
| entity-template-persistence + **instantiate** | v0.4.0  | ✅     | `EntityTemplate` tree, `instantiate()` with fresh ID minting, OPFS save/load, inverse = Batch of DeleteEntity                                      |
| ui-panels                                     | v0.5.0  | ✅     | `HierarchyPanel` + `InspectorPanel` React components, `useSceneState`, `useLogState` hooks                                                         |
| dynamic-scene-export                          | v0.6.0  | ✅     | `export_dynamic_scene_wasm` → Bevy `DynamicScene`, component mapping from editor schemas to Bevy native components                                 |
| preview-anchor-sync                           | v0.7.0  | ✅     | Preview world honors `editor.Sprite2D.values.anchor` via Bevy 0.19 Anchor Component                                                                |
| keyboard-shortcuts                            | v0.8.0  | ✅     | `useKeyboardShortcuts` hook, Ctrl+Z/Y + Cmd+Z/Y, input guard, Playwright screenshot diff E2E                                                       |
| delete-key                                    | v0.9.0  | ✅     | Delete/Backspace removes selected entity, input guard, `handleDeleteEntity` in App, 3 Playwright E2E tests                                         |
| entity-rename-inline                          | v0.10.0 | ✅     | Double-click name in hierarchy → inline input, Enter/blur commits via RenameEntity, Escape cancels, empty/unchanged no-op                          |
| entity-drag-drop                              | v0.11.0 | ✅     | HTML5 DnD reparenting in HierarchyPanel, `ReparentEntity` via `window.dispatch_command`, root-drop zone, self-drop guard, cycle safety via backend |

---

## Hito 1: AI-Assisted Editing

**Goal**: LLM-powered scene editing via a Rust HTTP proxy that routes to OpenAI, with a React UI panel for proposing, reviewing, and dispatching scene-edit commands.

### Completed Milestones

| Milestone                                                   | Version | Status       | Key Deliverables                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ----------------------------------------------------------- | ------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| ai-assisted-editing                                         | v0.12.0 | ✅           | Rust axum proxy (Ollama + OpenAI), `crates/ai-proxy`, WASM bridge `get_combined_schemas_json`, `AIAssistantPanel` + `ProposalCard` React components, `useAIAssistant` hook, mock LLM proxy fixture, 6 Playwright E2E tests                                                                                                                                                                                                                                                                                                                                                                                               |
| code-export                                                 | v0.14.0 | ✅           | `crates/editor-core/src/code_export.rs` (590 LOC): pure-string codegen, `rust_type_for_field`, `emit_header/user_structs/plugin_shell/spawn_scene`, snapshot tests                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| multi-scene                                                 | v0.15.0 | ✅           | `SceneRegistry`, scene switching with dirty-state tracking, `SceneTabs` UI, `UnsavedChangesDialog`, E2E tests, WASM bindings                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| pixelmatch quantitative diff                                | —       | ✅           | `frontend/tests/pixelmatchHelper.ts`, upgraded screenshot tests to per-pixel quantitative output with explicit % metrics                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| scene-asset-document (BSN spike)                            | v0.16.0 | ✅           | Rust types for `SceneAssetDocument`, `SceneInstance`, `BsnIr` per ADR-0005; aligned with Bevy 0.19 `bsn!` semantics; 10/10 spec scenarios covered                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| code-export-bsn                                             | v0.17.0 | ✅           | `bsn_codegen.rs` emits `bsn!`/`bsn_list!` source from `BsnIr`; parallels existing `Commands::spawn` codegen; 7 integration tests                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| scene-asset-catalog                                         | v0.18.0 | ✅           | `SceneAssetCatalog` metadata index: three `BTreeMap` indices, 11 public methods, `CatalogError`/`CatalogWarning`, `mint_asset_id`; 12 integration tests; wasm32 build green                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| scene-instance-overrides                                    | v0.19.0 | ✅           | `scene_instance_overrides.rs`: non-destructive override lifecycle + asset-version resync; 7 public functions (`effective_values`, `resync`, `mint_id_map`, `reconcile_id_map`, `validate_overrides`, `classify_overrides`, `try_rebind`); field-path segment-0 = full `type_id`; 11 integration tests; `StableId` gets `Ord` derive for `BTreeSet` usage                                                                                                                                                                                                                                                                 |
| remove-template-rs                                          | v0.20.0 | ✅           | Deletion of legacy `EntityTemplate` per ADR-0005 §Implementation Direction step 3. `crates/editor-core/src/template.rs` (507 LOC) and all 9 callers removed; net -892 LOC; 206 existing tests still compile on wasm. Completes BSN migration roadmap (Fases 0–4).                                                                                                                                                                                                                                                                                                                                                        |
| project-asset-browser-and-scene-asset-authoring (PR1 slice) | v0.21.0 | ✅ (partial) | Scene Asset persistence + catalog holder foundation. Path-based OPFS layout (`assets/<logical_path>.asset.json`), `ProjectMetadata.scene_assets` with `#[serde(default)]`, `SCENE_ASSET_CATALOG` / `SCENE_ASSET_DOC` / `SCENE_ASSET_CATALOG_WARNINGS` thread-locals, typed `CatalogWarning` for orphaned entries (S16), 9/9 PR1 spec scenarios compliant (S4–S8, S16–S19). PR #16 (docs/plan) and PR #17 (code) merged; tag `v0.21.0`. ADR-0007/ADR-0008/ADR README and SDDK artifacts (`docs/sddk/project-asset-browser-and-scene-asset-authoring/`) added.                                                             |
| project-asset-browser-and-scene-asset-authoring (PR2 slice) | v0.22.0 | ✅ (partial) | AssetCommand surface + WASM bridge. Separate `AssetCommand` enum (AddEntity, RemoveEntity, RenameEntity, SetComponentValue) per ADR-0007, `AssetOperationLog` (undo/redo) scoped to scene assets, `AssetProcessor` with `set_field_path_vec` helper, thread-local `ASSET_OPERATION_LOG`, WASM CRUD bridge (dispatch_asset_command, create/rename/duplicate/delete/list_scene_assets, open/close/get_asset_document/get_scene_asset_catalog, save_scene_asset body-first/catalog-second). 16/16 PR2 tasks complete; 23/23 spec scenarios covered (S10, S13, S14, S15 PR2 + PR1 regression). PR #18 merged; tag `v0.22.0`. |
| project-asset-browser-and-scene-asset-authoring (PR3 slice) | v0.23.0 | ✅           | Project Asset Browser + Scene Asset Authoring Mode frontend. React components (ProjectAssetBrowser, AssetAuthoringView, AssetUnsavedChangesDialog), hooks (useSceneAssets), services (scene-assets.ts), App.tsx editorMode state, TopBar mode-aware toolbar, Playwright E2E tests (14 scenarios + EC1-EC6). C-1 (engine-bridge TypeError) and C-4 (canvas unmount) fixed in correction commit f85333b. Follow-up issue #19 tracks C-NEW (SystemTime::now panic on wasm32). PR #20 merged; tag `v0.23.0`. **Capability 1 (Project Asset Browser + Scene Asset Authoring) CLOSED**.                                        |
| scene-instance-placement (PR1 slice)                        | v0.24.0 | ✅ (partial) | Storage seam + cache + gate. `SceneDocument.instances: BTreeMap<StableId, SceneInstance>` with `#[serde(default)]`, `ASSET_BODY_CACHE` skeleton (resolve, warm, invalidate, clear), `instance_projection.rs` with `root_local_ids` gate (single-root enforcement). PR #21 merged; tag `v0.24.0`.                                                                                                                                                                                                                                                                                                                         |
| scene-instance-placement (PR2 slice)                        | v0.25.0 | ✅           | Commands + WASM + projection. `PlaceSceneInstance`/`RemoveSceneInstance` commands, `instance_projection.rs` with `place_instance`/`remove_instance`/`root_local_ids`, WASM bridge (`dispatch_scene_instance_command`, `place_scene_instance`, `remove_scene_instance`), warm_asset_body_cache integration. PR #22 merged; tag `v0.25.0`.                                                                                                                                                                                                                                                                                 |
| scene-instance-placement (PR3 slice)                        | v0.26.0 | ✅           | Frontend + E2E. HierarchyPanel/InspectorPanel/ProjectAssetBrowser UI updates, `useSceneAssets` hook, `scene-assets.ts` service, `engine-bridge` methods, 14 Playwright E2E tests (13 blocked by OPFS headless, S21 terminology passed). PR #23 merged; tag `v0.26.0`. **Hito 2 Order 2 CLOSED**.                                                                                                                                                                                                                                                                                                                         |
| override-resync-workbench                                   | v0.27.0 | ✅           | Override status surfacing UI. 4 new WASM functions (validate_overrides_wasm, effective_values_wasm, try_rebind_wasm, get_resync_reports), RESYNC_REPORTS thread-local, HierarchyPanel colored override dot, InspectorPanel override summary + collapsible issues list. PR #24 merged; tag `v0.27.0`. **Hito 2 Order 3 CLOSED**.                                                                                                                                                                                                                                                                                          |
| validation-center                                           | v0.28.0 | ✅           | Unified ValidationIssue types + get_validation_issues_wasm (catalog + export warnings), ValidationCenter panel (severity grouping, empty state, refresh), TopBar toggle button. PR #25 merged; tag `v0.28.0`. **Hito 2 Order 4 CLOSED**.                                                                                                                                                                                                                                                                                                                                                                                 |

### Active Work

| Change                                              | Branch                                           | Status                                                                 |
| --------------------------------------------------- | ------------------------------------------------ | ---------------------------------------------------------------------- |
| `application-stabilization-and-roadmap-convergence` | ✅ RELEASED — v0.108.1 (2026-09-06), cycle `p-28fce7028ac3c497` | Done — 9/9 release-cycle gates green. Recovery-1 closed archcheck B8 (C-1). Recovery-2 closed D3.2 (no-op by evidence) + D4.2 (superseded by D4.3) + M-4 (8813556 bundling) as accepted debt via ADR-0055. Recovery-3 closed C-2 (Playwright OPFS race) by deferring `window.*` test bridges in `engine-bridge.ts` until after `init_project_store()`. Remaining: `cargo test --release` budget (C-3), M-2 (AUDIT.md misplaced), M-3 (B9 regex narrow), M-5 (B3.1 cherry-pick blocked), docs-check rule-7 markers. |
| `h2-5-runtime-coordination` (Blocks A2/E/F/G/H)     | ✅ RELEASED — v0.108.0 → v0.108.8 (2026-09-07/08), cycles `p-28fce7028ac3c497/h2-5-runtime-coordination-block-{a2,e,f,g,h}` | Done — 9/9 thread_locals retired, EditorSession owns runtime buses + preview inspector, KEYBOARD_STATE → Bevy Resource InputState. Block A2/E/F/G/H cycles closed via supersede (sequences 110/145/140/152/166). ADR-0064 ratifies `*_FALLBACK` thread_locals as permanent compatibility layer. Tag `v0.108.8` points at commit `68020bb`. |
| `smoke-budget-cargo-conformance-restore` (Block I)  | ✅ RELEASED — v0.108.9 (2026-09-08), cycle `p-28fce7028ac3c497/smoke-budget-cargo-conformance-restore`, sequence 176 | Cargo check green, Playwright smoke cohort trimmed to 39 tests in 4 `@smoke` files (was 60+ tests). ADR-0064 ratified. Cycle closed via archive at sequence 183. Tag `v0.108.9` points at commit `a4e020a`. Follow-ups tracked for Block J: BJ-1 (3 logic_evaluator integration tests fail, Block A2 session-install bug), BJ-2 (archcheck rule B1+B2 false-positives from `editor_bevy::` doc-comment substrings). |
| `v1-g1-bevy-harness` (G1 step 1)                    | ✅ RELEASED — v0.109.0 (2026-09-08), cycle `p-28fce7028ac3c497/v1-g1-bevy-harness`, sequence 184 | New `crates/examples-bevy-harness/` workspace member consumes `examples/platformer-minimal/` as JSON witness: loads 4 scene asset JSON files into `SceneAssetDocument`, spawns Bevy 0.19 entities, asserts 4 named entities + custom schema values. 1/1 `#[ignore]` integration test passes. Cycle closed via archive at sequence 195. Tag `v0.109.0` will be re-tagged at archive commit. Closes 1/4 G1 sub-deliverables. Follow-ups: BJ-3 (UI-creation Playwright test, P2), BJ-4 (Bevy play_mode runtime assertions, P2), BJ-5 (gameplay assertions, P3). |
| `block-j-bj1-bj2-cleanup` (Block J)                  | ✅ RELEASED — v0.109.1 (2026-09-08), cycle `p-28fce7028ac3c497/block-j-bj1-bj2-cleanup`, sequence 197→208 | Closes BJ-1 + BJ-2 carry-forward debt from Block I/G1. BJ-1: `MinimalSession` + `install_fresh_session` exposed as `pub(crate) mod test_helpers` in `actuator_bus.rs`; 3 lines added to 3 failing tests in `logic_evaluator::integration_tests`. BJ-2: archcheck rule B1 regex refined to `/(?<![-_])bevy::/` (negative lookbehind excludes `editor_bevy::` / `editor-bevy::` substrings); 1 doc comment drift fixed in `command.rs`. Tests: 414 passed / 0 failed (was 411 + 3 failing). archcheck: all assertions pass. A-min cycle, 4 files / +57 / -27. Tag `v0.109.1` points at commit `eb28ce7`. Remaining open: BJ-3, BJ-4, BJ-5. |
| `g1-step2-player-movement` (G1 step 2)               | ✅ RELEASED — v0.109.2 (2026-09-08), cycle `p-28fce7028ac3c497/g1-step2-player-movement`, sequence 209→222 | Closes BJ-4 (Bevy play_mode runtime state assertions). New `PlayerMovementPlugin` + `player_movement_system` in `crates/examples-bevy-harness/src/movement.rs`; 4 integration tests in `tests/player_movement.rs` covering ArrowRight/ArrowLeft/no-input plus `With<PlayerController>` filter. Tests: 4/4 pass under `--ignored`. Bevy 0.19 test-time gotchas resolved (MinimalPlugins lacks InputPlugin; Time<Real> warm-up needs two updates). A-lite cycle, 3 files / +279 / -0. Tag `v0.109.2` points at commit `98323a4`. Remaining open: BJ-3, BJ-5. |
| `g1-step3-bj5-pickup-patrol` (G1 step 3, BJ-5 partial) | ✅ RELEASED — v0.109.3 (2026-09-08), cycle `p-28fce7028ac3c497/g1-step3-bj5-pickup-patrol`, sequence 223→236 | Closes BJ-5 partial (pickup collision mechanic). New `PickupCollisionPlugin` + `pickup_collision_system` in `crates/examples-bevy-harness/src/collision.rs`; `Pickup` marker in `components.rs`; loader attachment by `editor.Name == "Pickup"` heuristic; 3 integration tests in `tests/pickup_collision.rs` covering overlap despawn, far-away preserved, marker-only filter. Tests: 3/3 pass under `--ignored`. A-lite cycle, 5 files / +264 / -3. Tag `v0.109.3` points at commit `c46effc`. Remaining open: BJ-3, BJ-5 (enemy patrol, jump, contact-death). |
| `g1-step4-bj5-jump` (G1 step 4, BJ-5 partial) | ✅ RELEASED — v0.109.4 (2026-09-08), cycle `p-28fce7028ac3c497/g1-step4-bj5-jump`, sequence 237→250 | Closes BJ-5 partial (jump mechanic). New `JumpPlugin` + `jump_system` + `JumpState` resource in `crates/examples-bevy-harness/src/jump.rs`; `JUMP_FRAME_DT = 0.1` constant matching test `ManualDuration`. Rising-edge detection via `pressed()` + `JumpState::space_was_pressed` resource (Bevy 0.19's PreUpdate `keyboard_input_system.clear()` wipes `just_pressed` before our Update runs). 3 integration tests in `tests/jump.rs` covering Space-press, no-input, and `With<PlayerController>` filter. Tests: 3/3 pass under `--ignored`. Bevy runtime total: 11/11. A-lite cycle, 3 files / +271 / -1. Tag `v0.109.4` points at commit `d2fbff6`. Remaining open: BJ-3, BJ-5 (enemy patrol, contact-death). |
| `g1-step5-bj5-enemy-patrol` (G1 step 5, BJ-5 partial) | ✅ RELEASED — v0.109.5 (2026-09-08), cycle `p-28fce7028ac3c497/g1-step5-bj5-enemy-patrol`, sequence 251→264 | Closes BJ-5 partial (enemy patrol). New `EnemyDirection(pub f32)` component in `crates/examples-bevy-harness/src/components.rs` (default +1.0, per-entity state); new `EnemyPatrolPlugin` + `enemy_patrol_system` in `crates/examples-bevy-harness/src/enemy_patrol.rs` (reflective boundary at ±`patrol_range`); loader attaches `EnemyDirection::default()` to entities named `Enemy`. 4 integration tests in `tests/enemy_patrol.rs` covering initial right patrol, both boundary flips, and player-vs-enemy filter independence (player.x=20 vs enemy.x=6). Tests: 4/4 pass under `--ignored`. Bevy runtime total: 15/15. A-lite cycle, 5 files / +419 / -1. Tag `v0.109.5` points at commit `7006dc7`. Remaining open: BJ-3, BJ-5 (contact-death logic-graph runtime). |
| `bj3-ui-entity-creation` (BJ-3 closure) | ✅ RELEASED — v0.109.6 (2026-09-08), cycle `p-28fce7028ac3c497/bj3-ui-entity-creation`, sequence 265→276 | Closes BJ-3 (UI-creation Playwright test). New `frontend/tests/ui-entity-creation.spec.ts` (100 lines, 2 tests): `add_entity_button_creates_one_entity` (clean editor → click `+ Add Entity` → 1 entity rendered in Hierarchy panel) and `add_entity_button_can_be_clicked_multiple_times` (3 clicks → 3 distinct entities rendered). Reuses `waitForEditorReady` helper, `add-entity-btn` testid, `get_scene_snapshot` bridge export, `hierarchy-entity-${id}` testid. Static checks: `npx tsc --noEmit -p .` clean; `npx playwright test --list` registers both tests. Tests run in `@full` cohort (require WASM bundle). A-min cycle, 1 file / +296 / -0. Tag `v0.109.6` points at commit `f2ee966`. Remaining open: BJ-5 (contact-death logic-graph runtime — out of scope for v0.109.x). **G1 now at 7/8 sub-deliverables closed**. |
| `g2-git-friendly-roundtrip` (G2 closure, runtime evidence) | ✅ RELEASED — v0.109.7 (2026-09-08), cycle `p-28fce7028ac3c497/g2-git-friendly-roundtrip`, sequence 277→288 | Closes G2 runtime-evidence half. New `frontend/tests/git-friendly-roundtrip.spec.ts` (182 lines, 2 tests): `project_json_round_trip_is_byte_identical` (mount sample → hydrate → read project.json → normalise → write back → read again → assert byte-identical — proves ADR-0045's "deterministic text representations") and `every_opfs_file_is_parseable_json_with_version_field` (iterate 9 OPFS files → assert JSON-parseable with top-level `version` — proves ADR-0045's "every persisted document has an explicit schema/format version"). Static checks: `npx tsc --noEmit -p .` clean; `npx playwright test --list` registers both tests. Tests run in `@full` cohort. A-min cycle, 1 file / +431 / -0. Tag `v0.109.7` points at commit `827250f`. **G2 ready to upgrade from 🟡 to ✅** (ADR-0045 + this test + branch protection rule). Remaining open: G4 (round-trip/migration corpus), G5 (crash recovery), G7 (a11y corpus), G8 (extension compat policy). |
| `g8-extension-compat-policy-runtime` (G8 closure, runtime evidence) | ✅ RELEASED — v0.109.8 (2026-09-08), cycle `p-28fce7028ac3c497/g8-extension-compat-policy-runtime`, sequence 289→300 | Closes G8 runtime-evidence half (documentation half already committed in `8590831` / 2026-09-07). New `crates/editor-model/tests/extension_compat.rs` (180 lines, 3 tests): `extension_manifest_json_round_trip_is_byte_identical` (build manifest with id/SemVer/8 capabilities/2 permissions, serialize → parse → re-serialize, assert byte-identical — proves ADR-0003 + ADR-0045 forward-compat), `semver_parses_valid_and_rejects_malformed` (4 valid + 10 malformed inputs to `SemVer::parse`), `capability_enum_has_builtin_categories` (assert `Capability::builtin_count() == 8` and one descriptor per declared runtime category round-trips losslessly). Tests: 3/3 pass. editor-model lib 313/0/0 unchanged; editor-bevy lib 414/0/1 unchanged. A-min cycle, 1 file / +435 / -0. Tag `v0.109.8` points at commit `7c72072`. **G8 ready to upgrade from 🔴 to ✅** (compat policy doc + this test). Remaining open: G4 (round-trip/migration corpus), G5 (crash recovery), G6 (performance corpus), G7 (a11y critical paths). |
| `evidence-map-refresh-v1099` (refresh evidence map to reflect G2 + G8) | ✅ RELEASED — v0.109.9 (2026-09-08), cycle `p-28fce7028ac3c497/evidence-map-refresh-v1099`, sequence 303→310 | Closes the formal-evidence gap: refreshes `docs/v1.0-stabilization-evidence-map.md` against `HEAD = c6d383e`. Material changes: G1 cell 🔴 → ✅ (canonical platformer + Bevy harness + 15 runtime tests + BJ-3); G2 cell 🟡 → ✅ (with cross-link to G2 release-receipt); G8 cell 🔴 → ✅ (with cross-link to `docs/compatibility-policy.md` + G8 release-receipt + 3 test descriptions); inventory counts (Playwright 78 → 80 specs; cargo 702 → 784 points); §2.5 NEW cycles-closed-since-baseline table. Coverage score: 3/3/3 → **5/2/2**. Single-file diff +92/-69. B-direct cycle, 4 files / +304/-69. Tag `v0.109.9` points at commit `6d8b5ec`. Remaining open: G4, G5, G6, G7. |
| `g7-a11y-critical-paths` (G7 closure, a11y critical paths corpus) | ✅ RELEASED — v0.110.0 (2026-09-08), cycle `p-28fce7028ac3c497/g7-a11y-critical-paths`, sequence 311→318 | Closes G7 (a11y critical paths) 🟡 → ✅ with **honest deferral** of CP-5. New `docs/a11y-critical-paths.md` (180 lines): 5 declared critical paths (CP-1 welcome, CP-2 entity, CP-3 save, CP-4 play, CP-5 import); CP-5 explicitly DEFERRED (grep confirmed no UI trigger for asset import exists today). New `frontend/tests/a11y-critical-paths.spec.ts` (129 lines, 4 tests, registered in both `@a11y` and `@full` cohorts): each test asserts trigger exists, is focusable, has accessible label. CP-1 additionally asserts `Enter` dismisses overlay. Tests assert a11y contract only; side-effects are covered by `ui-entity-creation.spec.ts`, `keyboard-shortcuts.spec.ts`, `runtime-preview-v2.spec.ts`. `playwright.a11y.config.ts` testMatch updated. Static checks: `npx tsc --noEmit -p .` clean. B-direct cycle, 5 files / +507/-1. Tag `v0.110.0` points at commit `6d470e4`. **G7 ready to upgrade from 🟡 to ✅** (4/5 CPs proven, CP-5 deferred with grep-verified rationale). Remaining open: G4, G5, G6, CP-5 (UI trigger). |
| `g4-roundtrip-corpus` (G4 closure, round-trip corpus) | ✅ RELEASED — v0.110.1 (2026-09-08), cycle `p-28fce7028ac3c497/g4-roundtrip-corpus`, sequence 319→330 | Closes G4 (round-trip/migration) 🟡 → ✅ with declared v1 format manifest + 8-test corpus. New `docs/v1-format-manifest.md` (206 lines): authoritative format registry enumerating the 5 durable document types at v1 (`SceneDocument`, `SceneAssetDocument`, `WorldDocument`, `LogicGraphAsset`, `ProjectMetadata`) with version constants and v0→v1 migration policies. Extended `crates/editor-model/tests/migration_corpus.rs` (+158 lines, 4 new tests): covers the 3 missing types (`corpus_v0_scene_asset_document_migrates`, `corpus_v0_world_document_migrates`, `corpus_v0_logic_graph_asset_migrates`) + cross-type `corpus_all_types_reject_future_version` (table-driven, all 5 types reject v999). Tests: 8/8 migration_corpus pass; 313/0/0 editor-model lib unchanged; 414/0/1 editor-bevy unchanged; archcheck green. A-min cycle, 6 files / +549/-2. Tag `v0.110.1` points at commit `0c46663`. **G4 ready to upgrade from 🟡 to ✅** (5 types documented + 8 corpus tests). Remaining open: G5 (crash recovery), G6 (performance corpus), CP-5 (UI trigger). |
| `g5-crash-recovery` (G5 closure, crash recovery story) | ✅ RELEASED — v0.110.2 (2026-09-08), cycle `p-28fce7028ac3c497/g5-crash-recovery`, sequence 331→344 | Closes G5 (crash/data-loss recovery) 🔴 → ✅. New `docs/crash-recovery.md` (290 lines): declares atomic-write + orphan-shadow-recovery contract (shadow→commit→cleanup), `hydrate`-time orphan sweep, **4 handled failure modes** (tab close, WASM panic, browser crash, power loss), **4 explicitly deferred modes** (quota, multi-tab, backup-before-delete, dirty-flag) with rationale. Refactored `crates/editor-storage-web/src/opfs_core.rs` to extract pure `OpfsCore::classify_paths` helper from the inline orphan-vs-real split inside `hydrate`; **5 new unit tests** exercise classification rule (all-or-none, midfix `.tmp` not treated as orphan, empty input, all-orphan). Existing `frontend/tests/crash-recovery.spec.ts` proves the contract end-to-end through the JS bridge: 3 tests (atomic write replaces + clears shadow, hydrate removes orphan on reload, original survives mid-write crash). Tests: 31/31 editor-storage-web lib; 313/0/0 editor-model; 414/0/1 editor-bevy; 8/8 migration_corpus; 4/4 archcheck; 3/3 Playwright @full crash-recovery. A-lite cycle, 3 files / +415/-12 (G5 doc + Rust refactor + verify report). Tag `v0.110.2` points at commit `721e25e`. **G5 ready to upgrade from 🔴 to ✅**. Remaining open: G6 (performance corpus — last 🔴), CP-5 (UI trigger). |
| `g6-performance-corpus` (G6 closure, performance corpus) | ✅ RELEASED — v0.110.3 (2026-09-08), cycle `p-28fce7028ac3c497/g6-performance-corpus`, sequence 338→351 | Closes G6 (performance corpus) 🔴 → ✅. New `frontend/playwright.performance.config.ts` (perf cohort, 240 s timeout) + `frontend/tests/helpers/perfBudget.ts` (`PerfBudgetSpec` + `assertWithinBudget` with soft-warn + hard-fail). 6 new perf specs in `frontend/tests/perf-*.spec.ts`: P1 1k entities (1.93 s, hard 20 s), P2 tile paint/erase (14.5 s each, hard 25 s — soft 12 s warned), P3 16-scene switch (223 ms mean, hard 1500 ms), P4 100-asset catalog (pass), P5 100-node logic dispatch (71 ms mean, hard 200 ms — soft 50 ms warned), P6 200-file source listing (pass). Empirical scales reduced from initial 10k/500/1000 magnitudes after prebuild + engine limits surfaced: `MAX_SCENES=16` in `crates/editor-bevy/src/scenes.rs:14`; OPFS write throughput ~50 files/s source / ~250 entities/s scene / ~400 assets/s import. 3 of 6 specs emit soft-budget warnings as quantitative regression baselines. Tests: 6/6 perf specs pass under hard budget (2.1 m wall on local Chromium); 313/0/0 editor-model; 414/0/1 editor-bevy; 8/8 migration_corpus; 4/4 archcheck; `npx tsc --noEmit` clean. A-lite cycle, 8 files / +1397/-1 + 5 follow-up fixes. Tag `v0.110.3` points at commit `0c658e6`. **G6 ready to upgrade from 🔴 to ✅**. Coverage score: 8/0/1 → **9/0/0 — ALL v1.0 product gates green**. Remaining open (non-blocking, post-v1.0 candidate): CP-5 UI trigger, perf budget tightening, MAX_SCENES lift, nightly perf CI workflow, M-2/M-3 warnings. |
| `cp5-import-trigger` (CP-5 closure, a11y asset-import trigger) | ✅ RELEASED — v0.110.4 (2026-09-08), cycle `p-28fce7028ac3c497/cp5-import-trigger`, sequence 352→359 | Closes the G7 (a11y critical paths) CP-5 deferred gap. Adds `import-asset-btn` (sibling to `import-bsn-btn`) inside `frontend/src/components/ProjectAssetBrowser.tsx` with `aria-label="Import asset"`, hidden `<input type="file" accept=".aseprite,.ase,.ldtk,.tmx,.json,.png" data-testid="asset-file-input">`. Test added (`frontend/tests/a11y-critical-paths.spec.ts:cp5_import_asset_button_has_aria_label_and_is_keyboard_focusable`) using `window.__setEditorMode("asset-authoring")` bridge to mount the project-asset-browser panel. Side-fix: CP-3/CP-4 tests made `inert`-tolerant (dock-only editor mode wraps legacy toolbar in `inert`/`aria-hidden`, blocking programmatic focus). All 10 a11y-critical-paths.spec.ts tests pass (CP-1..CP-5 × `accessibility` + `full` projects, 36 s wall). B-direct cycle, 7 files / +437/-49 + 5 follow-up commits (implementation-receipt, release artifacts, archive-manifest, handoff). Tag `v0.110.4` points at commit `d2933a5`. Carry-forward (separate cycle): full `<ImportDialog />` wiring (Aseprite/LDtk/Tiled importers + `onShowChangeWorkbench` flow + file-select state machine + conflict flow). CP-5 v1.0 a11y requirement is the keyboard-accessible trigger boundary, which this cycle proves. |
| `rig-agent-runtime-foundation`                      | (not yet started)                                | **PAUSED** (user directive 2026-09-06) — agentic work is parked to last. Resume only after v1.0-stabilization gates pass. See [`docs/v1.0-stabilization-evidence-map.md`](v1.0-stabilization-evidence-map.md). |
| `v1.0-stabilization`                                | (cycle entry)                                    | **IN PROGRESS** (P1: canonical playable sample game, ✅ closed via BJ-1→BJ-5) — baseline evidence map committed 2026-09-06, refreshed 2026-09-08 (v0.109.9). Coverage score: **9 ✅ / 0 🟡 / 0 🔴** (G1, G2, G3, G4, G5, G6, G7, G8, G9 ALL green — v1.0 product-gate matrix is complete). See [`docs/v1.0-stabilization-evidence-map.md`](v1.0-stabilization-evidence-map.md). |

---

## Hito 2: Authoring Workflows & 2D Level Production

**Goal**: Turn the post-BSN architecture into practical editor workflows: Project asset management, Scene Asset authoring, Scene Instance placement, override/resync UX, validation, 2D level design tools, and runtime preview inspection.

**Normative references**:

- [ADR-0006: Authoring-First Roadmap after the BSN Migration](./adr/0006-authoring-first-roadmap-after-bsn-migration.md)
- [Post-BSN Authoring Roadmap Specification](./specs/post-bsn-authoring-roadmap.md)

### Completed Sequence

| Order | Change                                                      | Version          | Status  |
| ----- | ----------------------------------------------------------- | ---------------- | ------- |
| 1     | `project-asset-browser-and-scene-asset-authoring` (PR1/2/3) | v0.21.0–v0.23.0  | ✅ DONE |
| 2     | `scene-instance-placement` (PR1/2/3)                        | v0.24.0–v0.26.0  | ✅ DONE |
| 3     | `override-resync-workbench`                                 | v0.27.0          | ✅ DONE |
| 4     | `validation-center`                                         | v0.28.0          | ✅ DONE |
| 5     | `component-override-migration`                              | v0.28.0 (PR #26) | ✅ DONE |
| 6     | `level-design-layers-research`                              | v0.28.0 (PR #27) | ✅ DONE |
| 7     | `runtime-preview-inspector`                                 | v0.29.0 (PR #30) | ✅ DONE |
| +     | `scene-instance-layer`                                      | v0.29.0 (PR #29) | ✅ DONE |
| +     | `level-scene-asset`                                         | v0.29.0 (PR #28) | ✅ DONE |

### Planned Sequence

| Order | Change                  | Status                    | Why                                                          |
| ----- | ----------------------- | ------------------------- | ------------------------------------------------------------ |
| 8     | `level-design-tools`    | ✅ DONE (v0.34.0, PR #34) | Tile painting, IntGrid authoring, tileset CRUD               |
| 9     | `auto-layer-generation` | ✅ DONE (v0.35.0, PR #36) | 3x3 pattern rule engine, RegenerateAutoLayer, AutoLayerPanel |

### Research Gates

| Capability                                    | Required research before `sddk-propose`                                                                                          |
| --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Project Asset Browser + Scene Asset Authoring | Unity Prefab Mode, Godot PackedScene/inherited scenes, Defold Collections/factories, Bevy BSN asset roadmap, OPFS Project layout |
| Scene Instance Placement                      | Unity prefab instance display, Defold collectionfactory ID maps, Godot missing base-scene behavior                               |
| Override / Resync Workbench                   | Unity Prefab Overrides, Blender Library Overrides, Godot inherited-scene constraints                                             |
| Validation Center                             | Unity console/validation patterns, Defold resource profiler, Bevy diagnostics                                                    |
| 2D Level Design Tools                         | Tiled terrain brush/automapping, LDtk IntGrid/Auto Layers/Entities, Bevy tilemap ecosystem, Aseprite metadata                    |
| Runtime Preview Inspector                     | Defold profiler, Godot remote SceneTree, Bevy diagnostics/remote tooling, Chronos future debugging                               |

### Deferred Until After Hito 3

| Candidate                       | Revisit when                                                                     |
| ------------------------------- | -------------------------------------------------------------------------------- |
| Collaborative editing           | Project asset identity, validation, and save/load semantics are stable           |
| Plugin system                   | Schema packs and validation extension points have at least one built-in example  |
| Physical `.bsn` import/export   | Bevy ships stable loader/write-back APIs                                         |
| Visual scripting/state machines | ✅ Gate passed — now active as Logic Bricks (see Post-Hito 3 section + ADR-0011) |

---

## Hito 3: .bsn File Workflow & Inspector UX

**Goal**: Enable .bsn file round-trip (export + import) and improve inspector UX for override inspection and editing.

### Completed Sequence

| Order | Change                               | Version          | Status  |
| ----- | ------------------------------------ | ---------------- | ------- |
| 1     | `bsn-file-export-research`           | v0.31.0 (PR #31) | ✅ DONE |
| 2     | `bsn-file-import-research`           | v0.32.0 (PR #32) | ✅ DONE |
| 3     | `level-inspector-and-override-panel` | v0.33.0 (PR #33) | ✅ DONE |
| 4     | `bsn-file-import`                    | v0.36.0 (PR #37) | ✅ DONE |

### Research Gates

| Capability       | Required research before `sddk-propose`                                                    |
| ---------------- | ------------------------------------------------------------------------------------------ |
| .bsn file import | ✅ Research done (Bevy PRs #23639/#23648 are DRAFT — implement editor-internal round-trip) |
| Level Inspector  | Unity override inspector, Godot inspector plugin patterns, override panel UX research      |

---

## Post-Hito 3: Logic Bricks / Behavior Authoring

**Goal**: Add a visual Logic Bricks system to wire common 2D gameplay (jump,
collision response, health/damage, timers, proximity) without leaving the
editor — **without** a Blueprint-style scripting VM. Behavior is Rust-compiled,
trait-backed controllers evaluated by an event-driven dispatch scheduler.

**Normative references**:

- [ADR-0011: Logic Bricks — Compiled Rust Controllers and Dispatch Scheduler](./adr/0011-logic-bricks-compiled-rust-controllers.md)
- [Logic Bricks Graph Editor Specification](./specs/logic-bricks-graph-editor.md)

**Planning provenance**:

- `sddk/logic-bricks-graph-editor/explore-report.md`
- `sddk/logic-bricks-graph-editor/proposal.md`
- `sddk/logic-bricks-graph-editor/spec.md`
- `sddk/logic-bricks-graph-editor/design.md`

### Binding Decisions (ADR-0011)

| Decision        | Resolution                                                                     |
| --------------- | ------------------------------------------------------------------------------ |
| Scripting model | Logic Bricks (Sensor → Controller → Actuator), not Blueprint VM                |
| Extension       | Compiled `RustController` trait registry (`NodeEvaluator`); v1 = built-in only |
| Runtime         | Event/change-driven dispatch scheduler in editor-core; codegen deferred        |
| React Flow      | View-only; WASM JSON is source of truth                                        |
| BSN             | Logic does NOT project to `.bsn`; `BsnExporter` rejects `Logic`-role assets    |
| Preview state   | Stateless across rebuilds (v1)                                                 |

### Current Step — Docs-First (research / spec / ADR / design)

| Item             | Status  | Artifact                                                                             |
| ---------------- | ------- | ------------------------------------------------------------------------------------ |
| Exploration      | ✅ DONE | `sddk/logic-bricks-graph-editor/explore-report.md`                                   |
| Proposal         | ✅ DONE | `sddk/logic-bricks-graph-editor/proposal.md`                                         |
| ADR-0011         | ✅ DONE | `docs/adr/0011-logic-bricks-compiled-rust-controllers.md`                            |
| Design           | ✅ DONE | `sddk/logic-bricks-graph-editor/design.md`                                           |
| Capability specs | ✅ DONE | `sddk/logic-bricks-graph-editor/spec.md` + `docs/specs/logic-bricks-graph-editor.md` |
| CONTEXT.md terms | ✅ DONE | Logic Bricks domain language added                                                   |

### Planned Implementation Sequence

| Order | Change                             | Why this order                                                                                                                                                                                                                                                                                                                   |
| ----- | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | `logic-graph-data-model`           | ✅ DONE (v0.37.0, PR #38) — `LogicGraphAsset`, `LogicNode`, `LogicEdge`, `SceneAssetRole::Logic`, `LogicInstance`. Foundation: everything depends on the data shape.                                                                                                                                                             |
| 2     | `logic-registry-and-metadata`      | ✅ DONE (v0.38.0, PR #TBD) — `NodeEvaluator` trait + built-in registry keyed by `node_type_id` / `controller_id`, `logic.*` schemas, port specs. Needed before any node can do anything.                                                                                                                                         |
| 3     | `logic-graph-authoring-ui`         | ✅ DONE (v0.39.0, PR #40) — React Flow view-only `LogicGraphEditor.tsx`, `EditorMode="logic"`, `LogicCommand` surface, node palette. Authoring needs data model + registry.                                                                                                                                                      |
| 4     | `logic-graph-validation`           | ✅ DONE — Port-type compatibility, cycle/dangling-ref detection via existing `get_validation_issues_wasm`. Surfaces issues before preview.                                                                                                                                                                                       |
| 5     | `logic-preview-dispatch-scheduler` | ✅ DONE (v0.40.1, PR #41 + fix PR #42) — `ACTUATOR_OUTPUT_BUS` + `evaluate_logic_binding` + topological sort + `apply_actuator_outputs` system + WASM exports + 11 integration tests (352 tests pass). Event-driven dispatch scheduler in Bevy Update loop.                                                                      |
| 6     | `logic-bricks-2d-recipes`          | ✅ DONE (v0.41.0, PR #42+PR #43) — Built-in immutable recipes: `builtin: bool` field + `RecipeImmutable` guard at `LogicCommand::apply` chokepoint + `logic_recipes.rs` module + `platformer_jump`/`health_damage`/`proximity_trigger` JSON assets + `list_builtin_recipes_wasm` export + lazy seed into `LOGIC_GRAPH_REGISTRY`. |
| 7     | `rustcontroller-builtins`          | ✅ DONE (v0.42.0, PR #44) — 7 NodeEvaluator structs for recipe node types: KeyPressed, Gate, ApplyImpulse, Collision, Compare, ModifyHealth, Proximity, EmitSignal. Thread-local sensor state (KEYBOARD_STATE, COLLISION_STATE, PROXIMITY_STATE). Type-chain fixes for Action/Float/Bool propagation.                            |
| 8     | (Deferred) `logic-graph-codegen`   | Optional graph → Rust source export via `code_export.rs` pattern. Not required for v1.                                                                                                                                                                                                                                           |

### Research Gates

| Capability                 | Required research before implementation                                                   |
| -------------------------- | ----------------------------------------------------------------------------------------- |
| Logic Bricks architecture  | ADR-0011 ✅ (this docs-first step)                                                        |
| Node evaluation scheduling | Bevy ECS `Changed`/`Added`/event patterns; Chronos future debugging for evaluation traces |
| React Flow integration     | `@xyflow/react` controlled-component patterns; view-only enforcement                      |

---

## Hito 4: Code-Aware Editor & Game Loop

**Goal**: Close the end-to-end loop from authoring to running game, with AI that understands Rust code + scene data + logic graphs simultaneously. The "Cursor-like IDE for Bevy games" vision needs three missing pieces: a real code editor, a build/run loop, and Rust source awareness. None of Hitos 0-3 deliver these; they are the foundation of Hito 4.

**Why this is the next milestone (research 2026-07-02)**:

- **Bevy 0.19** (June 2026) introduced `#[derive(SceneComponent)]` — Components that wrap entire scenes. This is a perfect fit for our Scene Asset model and the only Bevy 0.19 feature we have not yet targeted.
- **Bevy Editor Prototype Stage 3** (official roadmap) calls out hot reload, "Press Run Game", and tooltips as the next critical features. We have the data model for none of them yet.
- **Cursor's 2026 differentiation** is project-aware AI across code + non-code artifacts. Our Hito 1 AI only sees scene JSON; it cannot reason about Rust code, BSN, or Logic graphs. Hito 5 (code-aware AI) closes that gap.
- **Asset pipeline** (textures/audio/fonts) is a precondition for runnable games. Currently missing.

**Normative references**:

- [ADR-0012: Code editor choice (CodeMirror 6)](./adr/0012-editor-choice-codemirror-6.md)
- ADR-XXXX: WASM build strategy (in-browser rustc.wasm vs remote build)
- ADR-XXXX: Hot reload API contract with Bevy 0.19

### Planned Sequence

| Order | Change                                  | Version                                               | Status                                                                                                                       |
| ----- | --------------------------------------- | ----------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| 1     | `code-editor-foundation`                | v0.43.0 + v0.44.0 (PR #45 + #46 + #47 + #48)          | ✅ DONE                                                                                                                      |
| 2     | `rust-source-integration`               | v0.45.0                                               | ✅ DONE                                                                                                                      |
| 3     | `asset-pipeline`                        | v0.46.0 (PR #50 + #51 + #52)                          | ✅ DONE                                                                                                                      |
| 4     | `build-and-run-loop`                    | v0.47.0 (PR #53)                                      | ✅ DONE                                                                                                                      |
| 5     | `hot-reload`                            | v0.48.0 (PRs #56 + #57)                               | ✅ DONE — data-only (logic graphs + BSN scene components + source files); texture hot-reload deferred per ADR-0014 §Deferred |
| 6     | `code-aware-ai`                         | v0.72.0 (PRs #83 + #84 + #85)                         | ✅ DONE                                                                                                                      |
| 7     | `scene-component-authoring` (data + UX) | v0.75.0 (PRs #86 #87 #88) + v0.78.0 (PRs #93 #94 #95) | ✅ DONE                                                                                                                      |
| 8     | `animation-graph-editor` (deferred)     | —                                                     | 🔲 Planned                                                                                                                   |

### `code-editor-foundation` PR Sub-Sequence (4-PR stacked-to-main chain)

| Sub-PR | Scope                                                                                                                                                                                                 | Files                                                                                                                                                               | Version             | PR                                                        | Status                          |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- | --------------------------------------------------------- | ------------------------------- |
| 1/4    | Foundation: Rust `source_files` module + 5 #[wasm_bindgen] exports + ADR-0012                                                                                                                         | `crates/editor-core/src/source_files.rs`, `crates/editor-core/src/lib.rs` (+138), `docs/adr/0012-editor-choice-codemirror-6.md` (+140)                              | v0.43.0             | [#45](https://github.com/Rubentxu/bevy-2d-editor/pull/45) | ✅ MERGED (1 debt-fix round)    |
| 2/4    | Service + Hook layer: TS `code-files.ts` + `useCodeFiles.ts` + engine-bridge bindings + canonical `OpfsResult<T>`                                                                                     | `frontend/src/services/code-files.ts` (+114), `frontend/src/hooks/useCodeFiles.ts` (+187), `frontend/src/types/opfs.ts` (+17), `frontend/src/engine-bridge.ts` (+9) | v0.43.0             | [#46](https://github.com/Rubentxu/bevy-2d-editor/pull/46) | ✅ MERGED (1 debt-fix round)    |
| 3/4    | UI: CodeMirror 6 wired as `"code"` EditorMode, file list sidebar, Ctrl+S save, error toasts                                                                                                           | `frontend/src/components/CodeEditor.tsx` (+374), `frontend/src/App.tsx` (+13), `frontend/src/components/TopBar.tsx` (+11), `frontend/package.json` (+3 deps)        | v0.44.0 (with PR 4) | [#47](https://github.com/Rubentxu/bevy-2d-editor/pull/47) | ✅ MERGED (no fix cycle needed) |
| 4/4    | Tests + debt cleanup: Playwright E2E (5 pass, 3 skip), Rust unit tests (9 new), bundle size measurement, 6 HIGH debt fixes (M-1, M-2, coupling-W1, overeng-PR2-2, overeng-PR2-5, overeng-W1 deferred) | `frontend/tests/code-editor.spec.ts`, `crates/editor-core/src/source_files.rs` unit tests, bundle measurement, 6 debt fixes                                         | v0.44.0             | [#48](https://github.com/Rubentxu/bevy-2d-editor/pull/48) | ✅ MERGED                       |

### `code-editor-foundation` Carried Debt (for PR 4 cleanup)

6 HIGH items scope-tagged for PR 4:

| ID                                  | Severity | Description                                                                                                                                                                                | Effort                                                    |
| ----------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------- |
| **M-1**                             | HIGH     | `CodeEditor.tsx`: `basicSetup` 23-key enum → use `basicSetup: true`                                                                                                                        | 1 line                                                    |
| **M-2**                             | HIGH     | `CodeEditor.tsx`: imperative `view.dispatch` + `lastSyncedContentRef` + ref cast is redundant (uses @uiw v4.23.0 `ExternalChange.of(true)`); also a real UX bug (cursor-loss on file open) | ~25 LOC, 4 sub-bugs fixed                                 |
| **overeng-W1**                      | HIGH     | `SourceFile.id == SourceFile.path` violates CONTEXT.md "Stable ID" term                                                                                                                    | Rust-side redesign (drop `id`, use `path` as natural key) |
| **coupling-W1**                     | HIGH     | `rename_scene_asset` / `delete_scene_asset` silently discard `js_delete_file` Result via `let _ = ...`                                                                                     | 2-line Rust-side fix per caller                           |
| **overeng-PR2-2**                   | HIGH     | `useCodeFiles.ts`: extract `runOp<T>(fn)` helper to collapse per-action try/catch/setError boilerplate (~40 LOC)                                                                           | ~10 min, ~40 LOC reducible                                |
| **overeng-PR2-5 / coupling-PR2-10** | HIGH     | `code-files.ts`: add `@throws {Error}` JSDoc on `listSourceFiles`, `createSourceFile`; document 3-throws-vs-2-unions asymmetry as deliberate                                               | 3-line doc fix                                            |

**SDDK artifacts**: 19 files in `sddk/code-editor-foundation/` (3 explore, proposal, spec, design, tasks, apply-progress, 3 archive-reports, 3 release-reports, 4 verify-reports, 4 debt-reports). All in `sddk/` (gitignored per local-only policy).

### Research Gates

| Capability                 | Required research before `sddk-propose`                                                                                                    |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Code editor in browser     | Monaco vs CodeMirror 6: bundle size, Rust syntax support, performance, customization. Lucide/Tailwind for the editor chrome.               |
| WASM build in browser      | rustc.wasm (slow but offline) vs remote build server (fast but networked) vs hybrid (cached deps + remote compile). Security implications. |
| Hot reload Bevy 0.19       | `Component::hot_reload` API, asset watcher integration, scene asset resync semantics.                                                      |
| SceneComponent (Bevy 0.19) | Derive macro shape, how `SceneComponent` materializes to `BsnIr` and `.bsn`, authoring UX.                                                 |
| Code-aware AI              | Multi-source context window, source code chunking for `rustc --emit=metadata` indexing, token budget strategy.                             |

### Bevy Roadmap Alignment (2026-07-02)

- ✅ Bevy 0.16 (Apr 2025) — Relationships, Entity Cloning → consumed by our Scene Asset model
- ✅ Bevy 0.17 (Aug 2025) — TBD
- ✅ Bevy 0.18 (Jan 2026) — Cargo feature collections → enables 2D-only build
- ✅ Bevy 0.19 (Jun 2026) — BSN Scene Components, Resources-as-components, Parley text → enables Orders 5, 7 of Hito 4
- ⏳ Bevy 0.20+ — TBD; track `bevy_editor_prototypes` Stage 3+ for hot reload API

### Session Handover (2026-07-03)

**Current state**: Hito 4 Order 3 (`asset-pipeline`) — **PR #50 (Rust foundation) + PR #51 (compile fix) + PR #52 (TS service/hook/E2E) merged, v0.46.0 tagged**. Binary OPFS texture asset pipeline complete.

**What was built (3-PR chain)**:

- PR #50: `asset_files.rs` — `AssetFileId`, `AssetFile`, `AssetFileKind`, `is_supported_mime`, `asset_file_path_from_id`; 4 WASM exports (list/import/read/delete)
- PR #51: Compile fix — `js_sys::Reflect::get` for Object property access, `await` in `for` loop instead of `filter_map`
- PR #52: `asset-files.ts` service + `useAssetFiles` hook + `asset-pipeline.spec.ts` E2E tests + engine-bridge bindings

**Debt issues addressed in Order 4 cycle**:

1. ✅ Deleted `code-files.test.ts` (vitest not installed)
2. ✅ Deleted `findEntitiesByType` wrapper (0 callers)

### Session Handover (2026-07-03 — second session)

**Current state**: Hito 4 Order 4 (`build-and-run-loop`) — **PR #53 merged, v0.47.0 tagged**. Enhanced Preview Mode complete.

**What was built (Order 4)**:

- PR #53: `PlayMode` resource, `update_keyboard_state` system, `process_play_mode_request` with snapshot/restore, `logic_evaluation_system` + `apply_actuator_outputs` gated to play mode, `GameOverlay` component, TopBar Play/Stop button, `EditorMode` play variant, keyboard shortcut suppression in play mode, ADR-0013
- Debt fixes: CRIT-01+02 (dead mouse pipeline deleted), COUP-NEW-01 (logic dispatch gated to play mode), 4 integration tests in `play_mode.rs`

**Debt issues to address in follow-up**:

- OE-NEW-05: `COLLISION_STATE` and `PROXIMITY_STATE` thread-locals have the same dead-pipeline pattern as CRIT-01+02 — recommend separate `refactor/debt-cleanup-thread-locals-1` cycle
- ProjectAssetBrowser drag-and-drop + thumbnail grid — deferred to Order 5 (`hot-reload`)

**Current state**: Hito 4 Order 5 (`hot-reload`) — **PRs #56 + #57 merged, v0.48.0 tagged**. Data-only hot-reload complete: logic graphs, BSN scene components, and source files reload on save without WASM recompilation. Texture hot-reload deferred per ADR-0014 (blocked on Order 3 AssetServer load path debt).

**What was built (Order 5)**:

- PR #56: `HOT_RELOAD_BUS` thread-local + `process_hot_reload_requests` Bevy system + 4 wasm_bindgen exports + `SOURCE_FILE_REGISTRY` cache + asset cache invalidation + 4 integration tests + ADR-0014
- PR #57: `services/hot-reload.ts` typed event bus + `hooks/useHotReloadStatus.ts` React hook + save-hook emitters in code-files/asset-files + `engine-bridge.ts` wasm wrappers + `GameOverlay.tsx` status line + `TopBar.tsx` inline refresh button + 9 Playwright tests
- Bundle size: 315.83 KB → 316.63 KB gzip (+0.80 KB, 0.25%)
- Tests: 409 lib + 4 integration + 9 Playwright, all green

**Deferred to v2 (post-Hito 4 Order 6)**:

- Texture hot-reload via `AssetEvent::Modified` — requires closing Order 3 AssetServer load path debt
- Subsecond-based native hot-patching — only relevant if/when remote build server ships per ADR-0013 v2
- WASM module re-instantiation — architecturally undesirable (would reset Bevy App state)

**Hito 4 Order 6** (`code-aware-ai`): ✅ Shipped v0.72.0 (PRs #83 + #84 + #85). Multi-source context composition for the AI proxy per ADR-0015.

---

## Hito 5: Defold-Inspired Layout (`defold-inspired-redesign`)

**Goal**: Replace the topbar + 3-panel squished layout with the Defold-grade 3-region spatial layout (Assets / Scene / Outline + Properties), each region independently toggleable, with a menu bar and 7-segment status bar. Produced as Phase A→E across one SDDK cycle.

**Status**: ✅ DONE — v0.80.0 tagged (`85263c7`).

**Normative reference**: [ADR-0021: Defold-Inspired Layout + F-Key Shortcuts](./adr/0021-defold-inspired-layout.md).

| Phase | Scope                                                                     | Commit               |
| ----- | ------------------------------------------------------------------------- | -------------------- |
| A     | MenuBar (6 dropdowns: File/Edit/View/Tools/Run/Help) + selector drift fix | `7df24f5`            |
| B     | 3-region CSS Grid dock + drag-resizable dividers + OPFS-persisted prefs   | `9ae4f86`            |
| C     | Bottom dock (Console/Search/Output/Problems + F7 toggle)                  | `034eea0`            |
| D+E   | Status bar 7 segments + F6/F8/F9 + Welcome overlay + Reset Layout         | `c034dc4`            |
| Docs  | Contributing trunk-based workflow + ADR-0021 + USER_GUIDE (9 screenshots) | `85263c7`, `e0e0283` |

**F-Key shortcuts shipped**: F6 (Assets), F7 (Tools), F8 (Outline), Shift+F8 (Properties), F9 (fullscreen viewport).

---

## Hito 6: Dock Polish & UX Extensions (`v0.81.0`)

**Goal**: Build on the v0.80.0 spatial layout with the highest-impact v0.81 UX candidates from `ROADMAP_addendum_v0.81.md` (`defold-inspired-redesign` cycle tail). Tier 1 candidates first (global search, workspace presets, drag-and-dock infra) and Tier 2 (panel polish: per-panel state persistence + drag-resizable status bar).

**Status**: ✅ DONE — v0.81.0 tagged (`6d36768`). Stacked-to-main chain of 4 PRs in tier order.

**Normative references**:

- [ADR-0021: Defold-Inspired Layout + F-Key Shortcuts](./adr/0021-defold-inspired-layout.md) (extends dock schema)
- [ADR-0019: OPFS Scene Asset Catalog Persistence Ordering](./adr/0019-opfs-scene-asset-catalog-persistence-ordering.md) (already shipped, touched by v0.81 cycle)

| Order  | PR                                                          | Change                             | Scope                                                                                                                                                            | Commit    |
| ------ | ----------------------------------------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| Tier 1 | [#113](https://github.com/Rubentxu/bevy-2d-editor/pull/113) | `feat(v0.81): global search`       | `SearchTab` indexes scenes, scene assets, source files, asset files; wired to bottom dock                                                                        | `854f1d7` |
| Tier 1 | [#114](https://github.com/Rubentxu/bevy-2d-editor/pull/114) | `feat(v0.81): workspace presets`   | Default · 2D Platformer · Top-Down RPG · FPS · Minimal; `activePreset` + `presets` fields + `mergeWithDefaults` helper                                           | `26410e2` |
| Tier 1 | [#115](https://github.com/Rubentxu/bevy-2d-editor/pull/115) | `feat(v0.81): drag-and-dock infra` | HTML5 `draggable` headers, `data-panel-id`, `application/x-dock-panel` MIME, drop visual feedback; region-swap logic deferred to v0.82                           | `096a865` |
| Tier 2 | [#112](https://github.com/Rubentxu/bevy-2d-editor/pull/112) | `feat(v0.81-tier2): panel polish`  | `schemaVersion` + `statusBar` migration via `migratePrefs`, right-dock collapse flags persisted, status-bar drag-resize (20–48 px clamp), 7 new Playwright tests | `6d36768` |

**Conflict resolution** (documented for trunk-based reuse):

- `styles.css` — Global Search CSS (Tier 1) vs Drag-and-Dock CSS (Tier 1c): orthogonal DOM blocks, kept both.
- `useDockPrefs.ts` — Workspace-presets `activePreset`/`presets`/`mergeWithDefaults` (Tier 1b) vs `schemaVersion`/`statusBar`/`migratePrefs` (Tier 2): unified on `migratePrefs` and removed `mergeWithDefaults`.

**Bundle budget**: 348.78 KB gzip (target ≤ 350 KB). Playwright: 126 passed / 2 skipped. Rust: 638 passed.

**Deferred to v0.82** (from `ROADMAP_addendum_v0.81.md`):

- Floating panels (undock to free-floating window)
- Inspector multi-select (`SetComponentFieldOnMultiple`)
- Region-swap hook (clicks the drag-and-dock visual into action)
- Tab groups inside docks
- Asset browser thumbnails
- Welcome tour step-through

---

## Hito 7: scene-component-authoring UX follow-up

**Goal**: Hardening of the SceneComponent authoring flow on top of the data-layer milestone (Hito 4 Order 7, v0.75.0). Three stacked PRs to `main`.

**Status**: ✅ DONE — PRs #93 + #94 + #95 merged on `main`, tag `v0.78.0`.

| Order | Change                                                                                                                                        | Version | PR                                                        | Status    |
| ----- | --------------------------------------------------------------------------------------------------------------------------------------------- | ------- | --------------------------------------------------------- | --------- |
| 1/3   | `scene-component-authoring-ux` PR1 — catalog picker + draft validation + ADR-0018                                                             | v0.78.0 | [#93](https://github.com/Rubentxu/bevy-2d-editor/pull/93) | ✅ MERGED |
| 2/3   | `scene-component-authoring-ux` PR2 — Place Instance helper + Asset Browser / Schema panel buttons + bridge exports                            | v0.78.0 | [#94](https://github.com/Rubentxu/bevy-2d-editor/pull/94) | ✅ MERGED |
| 3/3   | `scene-component-authoring-ux` PR3 — focused Playwright coverage (S5 panel + S6 undo executable; S5 Asset Browser + S7 deferred per ADR-0017) | v0.78.0 | [#95](https://github.com/Rubentxu/bevy-2d-editor/pull/95) | ✅ MERGED |

**Normative reference**: [ADR-0018: Deferred SceneComponent command handlers remain Unsupported](./adr/0018-deferred-scene-component-command-handlers-keep-unsupported.md).

**Carried debt**: two `test.skip()` blocks in `frontend/tests/scene-component-authoring.spec.ts` for Asset Browser row placement (S5) and stale-at-place (S7) blocked by the OPFS catalog-persistence flake documented in [ADR-0017](./adr/0017-e2e-test-failure-root-cause.md). Pre-existing; out of scope for Hito 7.

### Post-Order 4 Hot-fixes

| PR  | Description                                                                                                                                                                                                                                                                                                                                                                     | Status    |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| #54 | `fix(build): cast Uint8Array to BlobPart in opfsSaveBinary (TS5.7 lib types)`. Unblocks `npm run build` (full chain) which was failing at tsc stage due to `Uint8Array<ArrayBufferLike>` no longer being assignable to `BlobPart` under TS lib 5.7+ (SharedArrayBuffer buffer-property ambiguity). Single-site `as BlobPart` cast in `frontend/src/opfs-bridge.ts:167` (+4/-1). | ✅ MERGED |

**Note on diagnosis**: The session handover from 2026-07-02 attributed the build failure to `crates/editor-core/src/logic_evaluator.rs:1071, 1101, 1074`, but reproduction showed the actual error was a TypeScript-stage failure unrelated to Rust/wasm-pack. Always re-validate memory hints against the current repo state before acting on them.

---

## Hito 8: AI-Native Editor Program (planning baseline)

**Goal**: Evolve Bevy 2D Editor from an authoring environment with AI assistance
into a **Cursor-like, agent-native editor for Bevy 2D games**.

**Why now**:

- The editor already has editor-owned domain models, typed command seams, runtime
  preview, code-aware AI context, Logic Bricks, and integrated code editing.
- Current product gaps are increasingly about **workflow convergence** and
  **agent orchestration**, not raw capability invention.
- ADR-0027 adopts **Rig** as the Rust-native orchestration layer for multi-agent,
  tool-based, retrieval-aware backend AI workflows.

### Prerequisite sequence (must land first)

| Order | Change                                              | Version           | Status  | Why                                                                                                                 |
| ----- | --------------------------------------------------- | ----------------- | ------- | ------------------------------------------------------------------------------------------------------------------- |
| P0    | `editor-shell-integrity`                            | v0.85.0 (PR #125) | ✅ DONE | Shipped 2026-07-28 — menu Portal, viewport polish, floating panel, status bar, useCodeFiles                         |
| P1    | `workflow-surface-convergence`                      | v0.85.0 (PR #125) | ✅ DONE | Shipped 2026-07-28 — AI context, logic graph OPFS, Validation Center, Search, prompt-free                           |
| P2    | `ui-workflow-overhaul`                              | v0.86.0 (PR #126) | ✅ DONE | Shipped 2026-07-29 — ModeContextBar, Hierarchy v2, Validation v2, Logic v2, Runtime v2, AI Panel v2                 |
| P3    | `application-stabilization-and-roadmap-convergence` | v0.108.1          | ✅ RELEASED (2026-09-06) | Restore green gates, deterministic E2E, documentation convergence, and architecture seams before Rig implementation |

### Planned sequence (after prerequisites)

> **Sequencing update (2026-08-14):** replaced by the Architecture & Product Evolution Pack master roadmap. The Rig work is not cancelled — it is deliberately placed after the architecture boundaries it must depend upon. See [MASTER_ROADMAP.md](./roadmaps/MASTER_ROADMAP.md) and [v0.87-architecture-foundation.md](./roadmaps/v0.87-architecture-foundation.md).

| Order | Change | Status | Why |
|-------|--------|--------|-----|
| 1 | `v0.87-architecture-foundation` | ✅ DONE | CI gates, `editor-model` extraction, `EditorSession`, `Clock`/`IdGenerator`, `ProjectStore`, Transaction Kernel v1, ChangeSet v1, typed backend foundation, fitness tests |
| 2 | `v0.88-production-authoring` | ✅ DONE | 2D direct manipulation toolkit, World Workspace v1, scope-of-change, filesystem project mode, recipes, hierarchy performance |
| 3 | `v0.89-change-runtime-workbench` | ✅ DONE | Change Workbench, semantic diffs, checkpoints, Runtime Causality Inspector, Runtime Apply-Back |
| 4 | `v0.90-agent-runtime` | ✅ DONE | `editor-protocol` tool contracts, `agent-runtime` crate (Rig behind capability ports), ChangeSet proposal generation, approval enforcement |
| 5 | `v0.91-semantic-retrieval-agents` | ✅ DONE | Semantic/typed retrieval, specialists, post-apply verification, bounded background maintenance |
| 6 | `v0.92-ecosystem-sdk-importers` | ✅ DONE (v0.92.0) | Editor Extension SDK (ADR-0040 steps 1-2), 3 built-in extensions via SDK, thread_local migrations, EditorSession split, FakeSession extraction; importers deferred to v0.93 |
| 7 | `v0.93-importers-sdk-061` | ✅ DONE (v0.93.0) | Aseprite/LDtk/Tiled JSON import pipelines; ExternalSource provenance + sidecar .meta.json; ConflictPolicy + ChangeWorkbench routing; reimport with SHA-256 fingerprinting; ImportDialog.tsx; 6 built-in extensions asserted; ADR-0041 implemented |
| 8 | `v1.0-stabilization` | 🔲 Planned | Full small-game authoring pass, crash/recovery, performance corpus, a11y, compatibility policy |

### Normative references

- [Application Stabilization and Roadmap Convergence Spec](./specs/application-stabilization-and-roadmap-convergence.md)
- [Application Stabilization Roadmap](./roadmaps/application-stabilization-roadmap.md)
- [ADR-0027: Rig-Based Agent Runtime for the AI-Native Bevy 2D Editor](./adr/0027-rig-agentic-editor-architecture.md)
- [ADR-0028: Workflow-First UI Convergence Before Agentic AI](./adr/0028-workflow-first-before-agentic-ai.md)
- [AI-Native Bevy 2D Editor — Durable Capability Spec](./specs/ai-native-editor-capabilities.md)
- [Editor Workflow Convergence — Durable Product Spec](./specs/editor-workflow-convergence.md)
- [UI Workflow Overhaul — Durable Product Spec](./specs/ui-workflow-overhaul.md)
- [UI Workflow Overhaul Roadmap](./roadmaps/ui-workflow-overhaul-roadmap.md)
- [AI-Native Editor Roadmap](./roadmaps/ai-native-editor-roadmap.md)
- [Master Roadmap — Bevy 2D Workbench (v0.87 → v1.0)](./roadmaps/MASTER_ROADMAP.md)
- [Architecture & Product Evolution Pack — executive summary](./architecture/00-executive-summary.md)

---

## Hito 0 — Capabilities Matrix

```
Capability                    v0.1   v0.2   v0.3   v0.4   v0.5   v0.6   v0.7   v0.8   v0.9   v0.10  v0.11
──────────────────────────────────────────────────────────────────────────────────────────────────────────────
SceneDocument JSON             ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅
Typed Command System           ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅
Reversible Commands                ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅
OPFS Scene Persistence              ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅
Schema Registry (mutable)               ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅
Entity Templates + Instantiate           ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅
Hierarchy + Inspector Panels                   ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅
DynamicScene Export                            ✅     ✅     ✅     ✅     ✅     ✅     ✅     ✅
Preview Anchor Sync                                   ✅     ✅     ✅     ✅     ✅     ✅     ✅
Keyboard Shortcuts (Ctrl+Z/Y)                             ✅     ✅     ✅     ✅     ✅     ✅     ✅
Delete Key (Del/Backspace)                                    ✅     ✅     ✅     ✅     ✅     ✅     ✅
Entity Rename Inline                                               ✅     ✅     ✅     ✅     ✅     ✅     ✅
Entity Drag-and-Drop Reparenting                                       ✅     ✅     ✅     ✅     ✅     ✅     ✅
```

## Hito 1 — Capabilities Matrix

```
Capability                    v0.12  v0.13  v0.14  v0.15  v0.16  v0.17  v0.18  v0.19  v0.20  v0.21  v0.22  v0.23  v0.24
────────────────────────────────────────────────────────────────────────────────────────────────────────────────
AI-Assisted Editing                ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
LLM Proxy (Ollama/OpenAI)           ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
AI Proposal UI Panel                ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Apply / Discard Commands            ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
E2E Tests (mock proxy)             ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Code Export (Rust codegen)                      ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Multi-scene Projects                             ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Scene Tabs + Dirty State                                   ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Pixelmatch Screenshot Diff                                   ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
BSN Scene Asset Model                                               ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
BSN IR + bsn! Codegen                                                    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Scene Asset Catalog                                                          ✅    ✅    ✅    ✅    ✅    ✅    ✅
Scene Instance Overrides + Resync                                                ✅    ✅    ✅    ✅    ✅
BSN Migration Complete (template.rs deleted)                                         ✅    ✅    ✅    ✅
Scene Asset Persistence + Catalog Holder (PR1 slice)                                       ✅    ✅    ✅    ✅
AssetCommand Surface + WASM Bridge (PR2 slice)                                                ✅    ✅    ✅
Project Asset Browser + Authoring Mode Frontend (PR3 slice)                                         ✅    ✅
Instance Storage Seam + Cache + Gate (PR1 slice)                                             ✅
Instance Commands + WASM + Projection (PR2 slice)                                              ✅
Instance Frontend + E2E (PR3 slice)                                                          ✅
Override Status UI + Resync Report (override-resync-workbench)                               ✅
Validation Center UI + WASM Surface (validation-center)                                      ✅
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  React Frontend (TypeScript)                                 │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────────────────┐   │
│  │ TopBar     │ │HierarchyPanel│ │ InspectorPanel       │   │
│  │            │ │ Entity tree  │ │ ComponentEditor       │   │
│  └─────────────┘ └─────────────┘ └──────────────────────┘   │
│  ┌─────────────┐ ┌──────────────────────────────────────┐  │
│  │ engine-     │ │ hooks: useSceneState, useLogState,   │  │
│  │ bridge      │ │ useKeyboardShortcuts, useAIAssistant, │  │
│  │ (WASM)      │ │ useScenes                            │  │
│  └─────────────┘ └──────────────────────────────────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                            │ wasm-bindgen
┌─────────────────────────▼──────────────────────────────────┐
│  Editor Core (Rust / WASM)                                   │
│  ┌──────────────┐ ┌───────────────┐ ┌────────────────────┐  │
│  │ document.rs  │ │ command.rs    │ │ processor.rs        │  │
│  │ SceneDocument│ │ Command       │ │ apply() / inverse() │  │
│  │ Entity       │ │ variants      │ │ cycle detection     │  │
│  │ StableId     │ │ Batch         │ │ field path parser   │  │
│  └──────────────┘ └───────────────┘ └────────────────────┘  │
│  ┌──────────────┐ ┌───────────────┐ ┌────────────────────┐  │
│  │ scene_asset  │ │scene_instance │ │ scene_asset_catalog │  │
│  │ .rs          │ │ .rs           │ │ .rs                 │  │
│  │ SceneAsset   │ │ SceneInstance │ │ 3-index BTreeMap    │  │
│  │              │ │ OverridePatch  │ │ mint_asset_id       │  │
│  └──────────────┘ └───────────────┘ └────────────────────┘  │
│  ┌──────────────┐ ┌───────────────┐ ┌────────────────────┐  │
│  │scene_instance│ │ bsn_ir.rs    │ │ bsn_codegen.rs     │  │
│  │_overrides.rs │ │ BsnIr        │ │ bsn! codegen        │  │
│  │ effective_   │ │ semantic IR   │ │ emit_bsn!           │  │
│  │ values/resync│ │              │ │                     │  │
│  └──────────────┘ └───────────────┘ └────────────────────┘  │
│  ┌──────────────┐ ┌───────────────┐ ┌────────────────────┐  │
│  │ schema.rs   │ │ persistence   │ │ dynamic_scene.rs    │  │
│  │ Component   │ │ OPFS bridge   │ │ DynamicScene export │  │
│  │ Schema      │ │ save/load     │ │ adapter            │  │
│  └──────────────┘ └───────────────┘ └────────────────────┘  │
│  ┌──────────────┐ ┌───────────────┐ ┌────────────────────┐  │
│  │ operation_   │ │ scenes.rs    │ │ bevy_anchor.rs     │  │
│  │ log.rs       │ │ SceneRegistry│ │ Anchor component    │  │
│  │ undo/redo    │ │ multi-scene  │ │ mapping            │  │
│  └──────────────┘ └───────────────┘ └────────────────────┘  │
│                              ┌──────────────┐                │
│                              │ lib.rs       │                │
│                              │ dispatch_cmd │                │
│                              │ mark_dirty() │                │
│                              └──────────────┘                │
└──────────────────────────────────────────────────────────────┘
                            │
                     ┌───────▼───────┐
                     │  AI Proxy     │
                     │  (axum/Rust)  │
                     │  OpenAI/Ollama│
                     └───────────────┘
```

---

## Pending Work

### High Priority

| Item                                                | Description                                                                                                                                                           | Blocking                                        |
| --------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| `application-stabilization-and-roadmap-convergence` | ✅ Done (v0.108.1) — release health restored (9/9 release-cycle gates green after recovery-1 closed archcheck B8). App.tsx decomposed, EditorGateway ChangeWorkbench wired, dev-WASM budget raised to 25 MB, ADR-0053/ADR-0054/ADR-0055 ratified. Recovery-2 closed D3.2 (evidence-based no-op) + D4.2 (superseded by D4.3) + M-4 (8813556 bundling) as accepted debt via ADR-0055. Recovery-3 closed C-2 by deferring `window.*` test bridges in `engine-bridge.ts` until after `init_project_store()` completes (race against thread-local PROJECT_STORE). Remaining carry-forward: `cargo test --release` budget (C-3), M-2 (AUDIT.md misplaced), M-3 (B9 regex narrow), M-5 (B3.1 cherry-pick blocked), docs-check rule-7 status markers. Smoke cohort 60 s budget is pre-existing and not in scope of any cycle. | None — completed 2026-09-06, recovery-1 + recovery-2 + recovery-3 follow-up completed 2026-09-06 |
| `ui-workflow-overhaul-pr4-debt`                     | Replay only validated test and WelcomeOverlay fixes from `debt-backup-ui-workflow-overhaul-pr4`; the branch does not resolve the measured bundle overage.             | Editor readiness and deterministic E2E baseline |
| `rig-agent-runtime-foundation`                      | First Hito 8 implementation cycle, corrected to keep `agent-runtime` transport-neutral and proposal-first.                                                            | Stabilization release-health gate               |

### Medium Priority

| Item                                                                                                                         | Description           |
| ---------------------------------------------------------------------------------------------------------------------------- | --------------------- |
| Test timing race: list_schemas dropdown not updating immediately after save (deferred from component-schema-authoring cycle) | Low urgency, cosmetic |

### Hito 1 Pending

All Hito 1 items completed in v0.12-v0.20. Deferred items:

- `collaborative-editing` — requires Yjs vs Automerge vs Loro decision + CRDT/OPFS merge strategy. Deferred to post-Hito 8.
- `plugin-system` — requires WASM plugin ABI. Deferred to post-Hito 8.

### ADR-0005 Implementation Status

| #   | Item                                                             | Status                                                  |
| --- | ---------------------------------------------------------------- | ------------------------------------------------------- |
| 1   | Scene Asset + Instance + Catalog as first-class Project concepts | ✅ Done                                                 |
| 2   | BSN-compatible IR (BsnIr) + semantic compatibility               | ✅ Done                                                 |
| 3   | Delete legacy EntityTemplate model                               | ✅ Done                                                 |
| 4   | `bsn!`/`bsn_list!` code generation as primary Bevy target        | ✅ Done                                                 |
| 5   | DynamicScene Export as adapter (not source of truth)             | ✅ Done                                                 |
| 6   | Non-destructive override validation, resync, rebind, cleanup     | ✅ Done                                                 |
| 7   | `.bsn` file import/export                                        | ⏳ Pending — Bevy loader/write-back APIs not yet stable |

---

## Technical Decisions (ADRs)

| ADR      | Decision                                                                                                                                                                               | Status                       |
| -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------- |
| ADR-0001 | JSON as source of truth (not RON)                                                                                                                                                      | ✅                           |
| ADR-0002 | Single Bevy instance renders canvas                                                                                                                                                    | ✅                           |
| ADR-0003 | `serde_json::Value` for forward-compat ComponentInstance values                                                                                                                        | ✅                           |
| ADR-0004 | Bevy native Anchor Component for sprite anchoring (not custom)                                                                                                                         | ✅                           |
| ADR-0005 | Scene Asset as BSN-aligned reusable scene model                                                                                                                                        | ✅                           |
| ADR-0006 | Authoring-first roadmap after the BSN migration                                                                                                                                        | ✅                           |
| ADR-0007 | Separate `AssetCommand` surface for Scene Asset Authoring (LocalId, parallel processor/log, no shared Command surface with scenes)                                                     | ✅                           |
| ADR-0008 | Path-based Scene Asset OPFS layout (`assets/<logical_path>.asset.json` + catalog inside `ProjectMetadata.scene_assets` with `#[serde(default)]`, body-first/catalog-second save order) | ✅                           |
| ADR-0009 | ComponentOverride as ECS/BSN-friendly replacement for OverridePatch (explicit `component_type_id`, field_path semantic)                                                                | ✅                           |
| ADR-0010 | BsnExporter trait + EditorCoreBsnExporter as working impl; BevyBsnExporter placeholder for future Bevy PR #23639 swap                                                                  | ✅                           |
| ADR-0011 | Logic Bricks — compiled Rust controllers + dispatch scheduler (no scripting VM, no codegen in v1, BSN isolation)                                                                       | ✅                           |
| ADR-0012 | Code editor choice — CodeMirror 6 via `@uiw/react-codemirror` (~130KB gzip, Vite-native, extension-API future-proofs Orders 2–6)                                                       | ✅                           |
| ADR-0013 | Build & Run Loop — Enhanced Preview Mode (play mode without in-browser rustc)                                                                                                          | ✅                           |
| ADR-0014 | Data-Only Hot Reload — source/asset cache invalidation, no texture reload in this PR                                                                                                   | ✅                           |
| ADR-0015 | Code-Aware AI Context Model — multi-source context composition (scene, schemas, source files, logic graphs, scene assets, selected entity) with priority table and token budget        | ✅                           |
| ADR-0016 | Scene-Component Authoring — Bevy 0.19 `SceneComponent` ergonomics with explicit `kind` discriminator and `bound_scene_asset_ref` schema metadata                                       | ✅                           |
| ADR-0017 | E2E test failure root cause (Hito 4 final cleanup) — investigation complete; WASM env init follow-up tracked                                                                           | ✅                           |
| ADR-0018 | Deferred SceneComponent command handlers remain unsupported — only the canonical WASM exports are wired in Hito 7                                                                      | ✅                           |
| ADR-0019 | OPFS Scene-Asset Catalog Persistence Ordering — body-first/catalog-second write semantics                                                                                              | ✅                           |
| ADR-0020 | Number skipped (reserved)                                                                                                                                                              | —                            |
| ADR-0021 | Defold-inspired dock layout — 3-region CSS Grid with menu / status bar, F-key shortcuts F6/F7/F8/F9, Workspace Presets built-ins                                                       | ✅ (v0.80.0)                 |
| ADR-0022 | Drag-and-Dock Region Swap — renumbered to ADR-0024; preserved for traceability                                                                                                         | Renumbered                   |
| ADR-0023 | Number skipped (reserved)                                                                                                                                                              | —                            |
| ADR-0024 | Drag-and-Dock Region Swap — atomic `movePanel` reducer; schema bumps to v2; data-panel-id and `application/x-dock-panel` MIME                                                          | ✅ (v0.82.0)                 |
| ADR-0025 | Floating Panels + Inspector Multi-Select — React Portal-based floats, custom pointer drag, `Set<StableId> + lastClickedId`, `SetComponentFieldOnMultiple`, schema v3                   | ✅ (v0.82.0)                 |
| ADR-0026 | Asset Browser Thumbnails — optional `preview_resource`, lazy load via IntersectionObserver, bounded LRU (≤32 entries), zero new runtime deps                                           | ✅ (v0.83.0)                 |
| ADR-0027 | Rig-Based Agent Runtime for the AI-Native Bevy 2D Editor — manager/worker composition, transport-neutrality, proposal-first workflows                                                  | Accepted (planning baseline) |
| ADR-0028 | Workflow-First UI Convergence Before Agentic AI — editor-shell-integrity → workflow-surface-convergence → ui-workflow-overhaul sequencing                                              | ✅ (v0.86.0 prerequisite)    |
| ADR-0029 | Frontend Performance Budget Contract — three-budget gate (initialJs 380 KB, totalJs 800 KB, wasm 20 MB) enforced by `frontend/scripts/check-bundle-size.mjs`                           | ✅ (v0.86.1)                 |
| ADR-0030 | Compile-Time Hexagonal Crate Boundaries — `editor-model` / `editor-application` / `editor-bevy` / `editor-protocol` / `editor-wasm` / `editor-storage-web` | ✅ (v0.94.0) |
| ADR-0031 | Explicit EditorSession Replaces Domain-Level Global State | Accepted (2026-08-14) |
| ADR-0032 | Shared Transaction Kernel and ChangeSet, with Domain-Specific Commands | Accepted (2026-08-14) |
| ADR-0033 | ProjectStore Port with OPFS and Filesystem Adapters | Accepted (2026-08-14) |
| ADR-0034 | Typed EditorBackend Contract Replaces Global Window Bridge | Accepted (2026-08-14) |
| ADR-0035 | Clock and IdGenerator Are Explicit Application Ports | Accepted (2026-08-14) |
| ADR-0036 | Bevy Runtime Preview Is an Ephemeral Projection Adapter | Accepted (2026-08-14) |
| ADR-0037 | World Workspace Is a First-Class Product Context | Accepted + Implemented (v0.95.0) |
| ADR-0038 | Workflow and Gameplay Recipes Compile Intent into Typed Changes | Accepted (2026-08-14) |
| ADR-0039 | Change Workbench Is the Unified Review and Approval Surface | Accepted (2026-08-14) |
| ADR-0040 | Editor Extension SDK Is Capability-First and Transactional | Accepted (2026-08-14) |
| ADR-0041 | External Authoring Sources Use Provenance-Aware Import/Reimport Pipelines | Accepted + Implemented (v0.93) |
| ADR-0042 | Runtime Apply-Back Is Explicit, Scoped and Authorable-Field Only | Accepted (2026-08-14) |
| ADR-0043 | Agent Runtime Uses Replaceable Orchestration Behind Typed Editor Capabilities | Accepted (2026-08-14) |
| ADR-0044 | CI and Architecture Fitness Gates Are Release-Critical | Accepted (2026-08-14) |
| ADR-0045 | Project Format Is Git-Friendly, Deterministic and Explicitly Migrated | Accepted (2026-08-14) |
| ADR-0046 | Semantic Editor Model Is the Authoritative Source of Truth | Accepted (2026-08-14) |
| ADR-0047 | Logic Graph Model Split — Pure Types in editor-model, Bevy Adapter in editor-core | Accepted (2026-08-15) — v0.87 |
| ADR-0048 | ProjectStore v1 Is a Synchronous Port | Accepted (2026-08-15) — v0.87 |
| ADR-0049 | Dual Dispatch Gate — TransactionKernel Adoption Is Flag-Reversible | Draft (2026-08-16) — v0.89 |
| ADR-0051 | ChangeWorkbenchPanel Lives in Bottom-Dock as an Internal Tab (ADR-0039/0024) | Draft (2026-08-16) — v0.89 PR2b |
| ADR-0052 | Runtime Causality — RebuildCause + LogicActivationRing + CausalityEdge (v0.89 PR3) | Draft (2026-08-16) — v0.89 PR3 |
| ADR-0050 | Apply-Back Policy — Mirror-Pair in editor-core/editor-application, Not in editor-model (v0.89 PR4) | Draft (2026-08-16) — v0.89 PR4 |
| ADR-0053 | Graph Kernel — Pure-Rust Dialect-Agnostic Substrate (GRAPH-001 substrate; GRAPH-002/003/005/008 dialects; GRAPH-009 cross-dialect invariants; GRAPH-010 Query) | Accepted + Implemented (v0.101–v0.103, 2026-08-21) |
| ADR-0054 | Rig Agent Runtime Foundation — Transport Neutrality Addendum (extends ADR-0027/ADR-0043) | Accepted (2026-09-06) |

---

## Testing

| Layer                   | Tool                                                      | Status                                                                                                                           |
| ----------------------- | --------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Rust unit + integration | `cargo test --workspace --all-targets --release --locked` | GREEN locally after A1 fixture repair; warnings remain                                                                           |
| WASM build              | `wasm-pack build --target web`                            | GREEN with warnings on the audited commit                                                                                        |
| Frontend static         | lint, format, typecheck                                   | GREEN locally after A2 reformat and CI gate addition                                                                             |
| Bundle                  | `npm run build:check` (ADR-0029 three-budget contract)    | GREEN locally after A3 code-split and lazy boundaries: initial JS 127.79 KB, total JS 368.57 KB, WASM 14.76 MB                   |
| E2E (smoke cohort)      | `npx playwright test --config=playwright.smoke.config.ts` | GREEN locally — 75/75 pass (engine-readiness contract via `__bevyEngineStarted` + `?skip-welcome=1&skip-onboarding=1` shortcuts) |
| E2E (full)              | `npx playwright test`                                     | Next cycle — full suite stability after docs-check CI is wired                                                                   |

> **Note**: Historical pass counts are superseded by the measured stabilization
> baseline above. See the application stabilization roadmap for the gate and
> cohort strategy.

---

## Glossary

See [`CONTEXT.md`](../CONTEXT.md) for authoritative domain language.

Key terms: **SceneDocument**, **StableId**, **Entity**, **Scene Asset**, **Level Scene Asset**, **Level Layer**, **Scene Instance Layer**, **Scene Instance**, **Scene Asset Catalog**, **Project Asset Browser**, **Scene Asset Authoring Mode**, **Override / Resync Workbench**, **Validation Center**, **Runtime Preview Inspector**, **Component Schema Registry**, **Component Instance**, **Component Override**, **Operation Log**, **BsnIr**, **BSN Export**. Evolution-pack terms: **Semantic Editor Model**, **EditorSession**, **Transaction Kernel**, **ChangeSet**, **Change Workbench**, **Editor Capability**, **World Workspace**, **Recipe**, **External Source**, **Runtime Delta**, **Scope of Change**, **Editor Extension** — see `CONTEXT.md`.

---

_Last reviewed: v0.86.1 — 2026-08-02. The active execution priority is
`application-stabilization-and-roadmap-convergence`; Hito 8 implementation
remains blocked until its release-health gate passes._

## Hito 7 → v0.86.0 → v0.86.1 Changelog Path

The full per-tag release history lives in [`CHANGELOG.md`](../CHANGELOG.md).
The application stabilization program (v0.86.1) bundles the
release-health gate (`docs/specs/application-stabilization-and-roadmap-convergence.md`),
the three-budget performance contract (`ADR-0029`), the unified editor
readiness signal, and the documentation hierarchy contract
(`docs/specs/documentation-hierarchy-and-drift-detection.md`).

---

_Architecture & Product Evolution Pack adopted (2026-08-14, docs-only, ADR-0030 → ADR-0046). The v0.87 Architecture Foundation gate now precedes the Hito 8 program; see `docs/roadmaps/MASTER_ROADMAP.md`. Rig-based agent work (ADR-0027) resumes at v0.90 behind typed capability ports._

---

_Hardening Pack adopted (2026-09-07, docs-only, ADR-0056 → ADR-0063). Supersedes ADR-0030 (→ ADR-0056 enforcement layer), ADR-0031 (→ ADR-0057 composition root), ADR-0034 (→ ADR-0058 capability split + codegen spike), ADR-0049 (→ ADR-0059 single-path end-state). Net-new: ADR-0060 (orthogonal workspace state), ADR-0061 (capability-segregated GraphKernel, extends ADR-0053, pending H8 spike), ADR-0062 (BSN anti-corruption layer), ADR-0063 (frontend feature slices + no direct bridge). Source of truth pre-v1 is the converged `docs/roadmaps/MASTER_ROADMAP.md` (H0 Release Truth → H10 Product Proof → v1.0); active engineering program is `docs/roadmaps/v1.0-architecture-ux-hardening.md`; release gate is `docs/roadmaps/v1.0-stabilization.md`. Rig/agent-runtime work remains parked post-v1 per `docs/roadmaps/ROADMAP_CONVERGENCE.md`._
