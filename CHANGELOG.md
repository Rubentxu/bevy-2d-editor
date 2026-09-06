# Changelog

All notable changes to Bevy 2D Editor are documented here. The project follows semantic version tags; detailed milestone history is available in [docs/ROADMAP.md](docs/ROADMAP.md).

## Unreleased

### v1.0-stabilization — P1 canonical sample game (closes G1)

Closes v1.0 product gate **G1** ("no canonical playable game"): a committed,
end-to-end authored sample game (`examples/platformer-minimal/`) the editor
can ingest, render, and export to `.bsn`. This is the **evidence** that the
IDE works in all its documented functionalities before AI/Rig features are
considered.

- **Authored sample** (committed as ground truth for v1.0 readiness):
  - `project.json` — version 0.1, name `platformer-minimal`, scene
    `main`, 2 custom schemas, 4 scene assets in the project catalog.
  - `schemas/game.PlayerController.schema.json` — typed schema with
    `speed` (`F32`) and `jump_force` (`F32`).
  - `schemas/game.EnemyPatrol.schema.json` — typed schema with `speed`
    (`F32`) and `patrol_range` (`F32`) constrained `Min: 0`.
  - `scene-assets/characters/player.actor.json` — `actor` role; entity
    Player carrying `editor.Name + editor.Sprite2D + editor.Transform2D
    + editor.Visible + game.PlayerController`.
  - `scene-assets/characters/enemy.actor.json` — `actor` role; entity
    Enemy carrying `editor.Name + editor.Sprite2D + editor.Transform2D +
    game.EnemyPatrol`.
  - `scene-assets/environment/ground.fragment.json` — `fragment` role;
    entity Ground carrying `editor.Name + editor.Sprite2D +
    editor.Transform2D` (with BottomCenter Anchor for floor placement).
  - `scene-assets/effects/pickup.actor.json` — `actor` role; entity
    Pickup carrying `editor.Name + editor.Sprite2D +
    editor.Transform2D`.
  - `logic-graphs/contact-death.logic.json` — `LogicGraphAsset` with 3
    nodes (sensor.contact_enter → controller.branch →
    actuator.destroy_entity) and 2 typed edges; `SceneAssetRole::Logic`
    is **not** exported to .bsn (per `BsnExporter::export_to_bsn_text`).
  - `scenes/main.scene.json` — `SceneDocument` placing 4 Scene Instances
    (player / enemy / ground / pickup) with `instance_components` and a
    per-instance Component Override patching the enemy's patrol speed.
- **Round-trip evidence (Rust integration tests)** in
  `crates/editor-bevy/tests/bsn_codegen_canonical_sample.rs`:
  - `bsn_round_trip_<player|enemy|ground|pickup>` deserialize each
    `.actor.json` / `.fragment.json` via `serde_json`, emit BSN via
    `emit_bsn_source_from_document`, and assert byte-equality against the
    committed `export/<name>.bsn` reference (`include_str!`). 4/4 PASS.
  - `regenerate_<name>_bsn` (ignored by default) overwrites the
    references for intentional updates.
- **Live editor evidence (Playwright @full)** in
  `frontend/tests/e2e-game-creation.spec.ts`:
  - Test 1 mounts the committed sample into OPFS via the JS
    `opfs_save_file` bridges, reloads the page so `init_project_store`
    re-hydrates from OPFS, calls `load_project`, then asserts the
    editor surfaces all 4 scene assets in the catalog, both custom
    schemas in the registry, and all 4 Scene Instances in the main
    scene's instance map (each referencing its `logical_path`).
  - Test 2 re-loads the project, then calls `export_asset_to_bsn_wasm`
    for each scene asset's catalog `asset_id` and asserts the emitted
    `.bsn` text opens with `bsn!{` and references the asset's
    `#ent_<name>_root` identifier (plus per-asset component checks:
    `PlayerController` for player, `Anchor` for ground).
  - Pattern: mount → reload → load_project → assert. Bypasses the
    `window.*` test bridges to write raw JSON so the test exercises
    the same persistence path that the editor uses.
- **Documentation** in `examples/platformer-minimal/README.md`: design
  rationale, the 12-feature evidence matrix mapping the sample to
  editor surfaces, the manual authoring recipe the editor UI itself
  would walk through, and the Playwright e2e pattern.

Coverage update: G1 moves from 🔴 to 🟢. The 9 v1.0 product gates
re-score to **4 ✅ / 3 🟡 / 2 🔴** (G1 added, G6/G8 still red —
pending P3 and P8 next).

Cycle context: this is `v1.0-stabilization` P1. The companion
`rig-agent-runtime-foundation` cycle remains **paused per user
directive 2026-09-06** until the v1.0 gates pass — Cursor-like AI
authoring is not built on top of unproven editor core.

### Recovery-3 — Playwright OPFS persistence race fix

Closes C-2 (deterministic Playwright smoke flake on `engine.spec.ts` `:526` and `:744`).

- **Root cause**: `frontend/src/engine-bridge.ts` was assigning the test bridge
  functions (`window.load_scene`, `save_scene`, `load_project`, `dispatch_command`,
  etc.) immediately after the WASM module loaded — BEFORE
  `await wasm.init_project_store()` could register the thread-local
  `PROJECT_STORE` slot (`crates/editor-model/src/ports.rs`). After
  `page.reload()`, React renders `AppHeader` (with `data-testid="topbar"`)
  unconditionally, so the topbar-visibility wait becomes a no-op readiness
  signal. The test pattern `waitForFunction(typeof load_scene === "function")`
  then resolves while `init_project_store()` is still mid-flight, and the
  caller hits `with_project_store().ok_or_else("project store not initialized")`.

- **Fix**: install only the OPFS bridge (`window.opfs_*`) before
  `init_project_store` (that function uses it to hydrate); defer every
  other `window.*` test bridge to AFTER `init_project_store` completes.
  New contract: once `initEngine()` resolves, every `window.*` bridge is
  callable.

- **Verified**: 20/20 `engine.spec.ts` tests pass; both previously-failing
  OPFS persistence tests (`:526`, `:744`) now pass deterministically
  (14.6 s combined). No regressions; full suite time unchanged at 1.2 m.

### Recovery-1 — Archcheck B8 closure (Clock trait dep injection)

Closes C-1 (the only cycle-introduced release-gate failure from `application-stabilization-and-roadmap-convergence`).

- Removed `now_millis()` / `now_nanos()` free functions from `crates/editor-model/src/time.rs`. Both called `js_sys::Date::now()` inside the pure `editor-model` crate, violating archcheck rule B8 (ADR-0030).
- Threaded `&dyn Clock` through `SceneAssetCatalog::update_version`, `current_unix_millis`, `random_hex_8`, plus `build_change_set_from_diff` and `build_new_sidecar` in `editor-application/src/reimport.rs`.
- Replaced production callsites in `editor-bevy` and `editor-wasm` with explicit `JsSysClock::new()` / `SysClock::new()` injection.
- Updated test callsites to pass `&FakeClock::new()`.
- 6 commits, all atomic per logical change.

### Recovery-2 — Work-unit discipline + D3.2 / D4.2 close-out

Closes M-4 (bundled commit `8813556`), D3.2, D4.2 via [ADR-0055](docs/adr/0055-work-unit-commit-discipline-trunk-based-split-policy.md):

- **M-4 closed as accepted debt.** Cycle commit `8813556` bundles 5 distinct work units (B1.1+B1.2, D2.2, D3.3, E.1, ADR-0054+ROADMAP). Trunk-based workflow forbids rebase of `main`; revert+reapply would break GitHub links and add 14 commits for zero functional value.
- **D3.2 closed as evidence-based no-op.** The verify-report claim of "9 direct mutation sites" is contradicted by `grep -rn "SCENE_DOC\|OPERATION_LOG\|DIRTY" frontend/src/`: only the WASM type declarations in `editor_application.d.ts` match. All scene state lives behind `frontend/src/scene-session/index.ts` (commit `2ce4a1d`).
- **D4.2 closed as superseded by D4.3.** D4.2 assumed `scene_facade.rs` lives in `editor-bevy/lib.rs`; it actually lives in `editor-wasm/src/scene_facade.rs` (commit `4b3e14d`). D4.3 (commit `50012b9`) accomplished the same goal with the correct crate.
- **Work-unit discipline becomes a cycle gate.** Future `sddk-apply` invocations require explicit commit-list; bundling >2 sub-tasks into one commit is a verifier BLOCKER.

## v0.108.1 — 2026-09-06 — Application stabilization and roadmap convergence

Patch rollup closing the application-stabilization-and-roadmap-convergence cycle
(`p-28fce7028ac3c497`). Headline changes:

- **App.tsx decomposed** from a 1972-line monolith into a 603-line composition root
  plus 6 headless hooks (`useSceneHandlers`, `useAppModeController`,
  `useAppCommandPalette`, `useAppShortcuts`, `useSearchBridges`, `useFullscreenBody`)
  and the `AppShell` presentation component (D2.3).
- **EditorGateway ChangeWorkbench routing** — all ChangeWorkbench calls now route
  through the `scene_*` WASM facade exported by `crates/editor-wasm/src/scene_facade.rs`
  (D4.3). Original `submit_*`/`get_*`/`approve_*`/`reject_*` exports remain for
  backward compat.
- **Dev-WASM bundle budget raised to 25 MB** with explicit release-vs-dev policy
  documented; production WASM (~7 MB gzip) is unaffected (ADR-0029 D3 amendment).
- **Cycle-introduced lint/format regressions fixed** (`prefer-const` on `_listeners`
  and `_operationLog`).
- **24 cycle commits since v0.108.0** (cycle SHA 50012b9): see `git log v0.108.0..v0.108.1`.

Documentation: CHANGELOG/ROADMAP backfilled to v0.108.0 (commit 558de79);
ADR-0053 (Graph Kernel — Pure Rust Dialects) ratified to Accepted + Implemented;
ADR-0054 (rig-agent-runtime-foundation transport-neutrality addendum) published.

Release-health aggregator: 8 of 9 release-cycle gates PASS at HEAD `50012b9`. One known
pre-existing failure carried forward: `tools/archcheck` B8 (`wasm_bindgen`/`js_sys`
import in `crates/editor-model/src/time.rs`). **Closed by recovery-1** (commit `0cb2605`)
— see Unreleased section above. All 9 cycle gates green after recovery-1.

Verification artifact: `release-receipt.md` (sha256 1d98637e9…).

## v0.108.0 — Logic Bricks cycle 2: event-driven scheduler (2026-08-20)

Delivers the second slice of Logic Bricks M2: converts the per-frame dispatcher into an event-driven scheduler gated by per-binding `dirty` + `binding_version` and edge-only `SensorEvent::DidFire`. Sensors fire once on transition; actuators run only when their inputs changed.

### New features

- **Per-binding `dirty: bool` + `binding_version: u64`** on `LogicBinding` (`crates/editor-bevy/src/logic_state.rs`) — gates recomputation; starts at version 1 on first apply
- **`SensorEvent` + `SensorStateCache`** for edge transitions (`crates/editor-bevy/src/logic_state.rs`): `SensorEvent::DidFire { sensor_id, ... }` emitted on rising/falling edge; `SensorStateCache` persists last value per sensor
- **Two-pass scheduler** (`crates/editor-bevy/src/logic_evaluator.rs`): `mark_bindings_dirty` (topological walk, bumps dirty when inputs changed) + `dispatch_dirty_bindings` (only evaluates dirty bindings)
- **`apply_*` functions bump `binding_version` + `dirty`** on every state mutation (e.g. `apply_actuator_outputs`, `apply_set_field`, `apply_bind`)
- **`apply_actuator_outputs_in_preview` gates on dirty** — no work when nothing changed since last frame

### Fixes

- **WASM32 observer API** — replaced `before()` chain with `add_observer` (`crates/editor-bevy/src/lib.rs`); rustfmt cleanups for the new system boundaries

### Stats

- 7 feature commits + 1 fix commit
- 8/8 spec requirements verified, 23/23 cycle-2 tests pass, 1213/1213 total workspace tests pass

### Architecture

- ADR-0011 (Logic Bricks) preserved: Sensor → Controller → Actuator flow, compiled Rust evaluators, no scripting VM, BSN isolation
- Per-binding versioning is the canonical invalidation key across the Bevy `Update` loop

## v0.107.0 — Logic Bricks cycle 1: Inspector binding UX + preview actuator application (2026-08-20)

Delivers the first end-to-end user-facing slice of Logic Bricks: bind a recipe to a Scene Instance from the Inspector and watch its actuators apply in preview. Closes the "press Space → player jumps" gap.

### New features

- **`BindLogicGraphToInstance` / `UnbindLogicGraphFromInstance` / `SetBindingFieldOverride` commands** (`crates/editor-bevy/src/logic_command.rs`) — full apply/inverse mirroring `LogicOperation` lifecycle
- **`apply_actuator_outputs_in_preview` system + `Velocity` component** (`crates/editor-bevy/src/preview_runtime.rs`) — applies compiled actuator outputs to Bevy entities during preview
- **`LogicBindingSection`** (`frontend/src/components/Inspector/LogicBindingSection.tsx`, 276 LOC): inspector UI for bind/unbind + field overrides
- **`LogicBadge`** on `HierarchyPanel` — visual indicator that an entity carries a `LogicBinding`
- **`useLogicGraph` hook** (`frontend/src/hooks/useLogicGraph.ts`, 76 LOC) + **`logic-graphs.ts` service** (92 LOC) for binding lifecycle
- **WASM bridge** wired through `EditorGateway.logic` namespace

### Fixes

- **`LogicBindingSection` was never imported in `InspectorPanel`** — wiring gap closed; `LogicBadge` now detects `editor.LogicBinding` entities
- **BsnExporter rejects `LogicRole` assets** — added 8 integration tests to lock the BSN isolation guard

### Stats

- 5 commits, 1536 insertions / 10 deletions across 12 files
- 4 new Rust tests + 5 new Playwright tests
- All previous tests pass unchanged

### Architecture

- `LogicOperation` enum added additively alongside existing `LogicCommand` — parallel processor / log, no shared surface with the document Command path (mirrors ADR-0007's split for Scene Assets)
- `editor.LogicBinding` ComponentInstance convention from ADR-0011 / `logic-graph-data-model` slice is now user-visible

## v0.103.0 — Graph Kernel M1: Query builder + GRAPH-010 (2026-08-20)

Delivers the GRAPH-010 Query Language for the Graph Kernel: a fluent, terminal-driven `Query<'a, D>` builder that replaces the per-cycle `detect_cycles` / `reachable_via_deps` helpers with a uniform predicate+terminal surface across all graph dialects.

### New features

- **`Query<'a, D> + QueryState + PredicateTable`** (`crates/editor-model/src/graph_kernel/query.rs`, 1016 LOC) — fluent builder with 4 terminals (find any, find all, count, exists)
- **Topological terminal** (`topological_sort_subset`) — kernel helper for partial topological sorts used by cycle-aware workflows
- **`with_edge_kind` predicate** — typed edge-kind filtering (now actually filters, see fix below)

### Fixes

- **`with_edge_kind` predicate was a stub** — accepted the predicate but never applied it to the edge set. Now filters edges by kind before traversal. Behavioural fix; tests added.

### Refactors

- `detect_cycles` + `reachable_via_deps` migrated to the Query builder (callers unchanged; logic moved to kernel primitives)

### Stats

- 4 commits, 1540 insertions / 249 deletions across 15 files
- 8 new Query builder tests

## v0.102.0 — Graph Kernel M1: GraphMut + per-dialect mutators + cross-dialect invariants (2026-08-20)

Delivers the mutable surface for the Graph Kernel: a `GraphMut` trait with explicit `GraphMutStrictness` and three per-dialect mutator implementations, plus the GRAPH-009 cross-dialect invariants mandated by ADR-0053.

### New features

- **`GraphMut` trait + `GraphMutStrictness` + 3 error variants** (`crates/editor-model/src/graph_kernel.rs`) — object-safe trait; strictness is a method, not a const (see fix below)
- **`LogicGraphDialectMut` — `CyclicNoSelfLoop` strictness** + 20 tests
- **`SceneAssetDialectMut` — `Dag` strictness + cycle check** + 12 tests
- **`WorldGraphDialectMut` — `Free` strictness** + 9 tests
- **GRAPH-009 ADR-0053 addendum + cross-dialect invariants** + 5 tests (e.g. `SelfLoop` rejected by all dialects; node-id uniqueness)

### Fixes

- **`GraphMut` object-safety** — `GraphMutStrictness` was originally a `const`, which broke `dyn GraphMut`. Moved to a method so the trait stays object-safe.

### Stats

- 6 commits, 2317 insertions / 257 deletions across 6 files
- 52 new tests across dialects and invariants

## v0.101.0 — Graph Kernel M1: pure-Rust substrate + ChangeSet wiring (2026-08-20)

Delivers the Graph Kernel M1 base (GRAPH-001 through GRAPH-008): a pure-Rust, dialect-agnostic substrate in `editor-model` (no Bevy / no WASM dependencies) plus three concrete dialect implementations and the `ChangeSetDialect` integration into the Transaction Kernel.

### New features

- **Graph kernel substrate** (`crates/editor-model/src/graph_kernel.rs`, 630 LOC) — pure-Rust, dialect-agnostic: nodes, edges, traversal, terminal walkers; ADR-0053 §Substrate
- **`SceneAssetDialect`** for `SceneAssetDocument` (GRAPH-002) — reads/writes a graph view over scene entities and `ComponentInstance` relationships
- **`ChangeSetDialect`** for `ChangeSet<O>` (GRAPH-003) — same substrate over the typed ChangeSet type
- **`WorldGraphDialect`** for `WorldDocument` (GRAPH-005) — world-level topology (levels, links, entrances) projected to a graph
- **`ChangeSetDialect` wired into `TransactionKernel`** (GRAPH-008) — ChangeSet graph now queryable through the kernel
- **ADR-0053** `Graph Kernel Pure-Rust Dialects` (`docs/adr/0053-graph-kernel-pure-rust-dialects.md`, 161 LOC)

### Test fixes

- `s9_hierarchy_via_relationships_only` aligned with post-S4 contract
- `set_logic_graph_for_test` aligned with `LOGIC_GRAPH_DOC`

### Stats

- 7 commits, 2341 insertions / 127 deletions across 13 files
- All existing tests pass unchanged

### Architecture

- `editor-model` remains bevy-free, wasm-free per ADR-0030 (compile-time hexagonal crate boundaries)
- Graph operations are sync, deterministic, and BTreeMap-backed (ADR-0045 deterministic ordering)

## v0.100.0 — Wave D1: EditorGateway seam + stability fixes (2026-08-19)

Delivers the Wave D1 unification of the frontend WASM access path: exposes the 22 bindings that were typed in the gateway but never actually exposed on `window` (World, ChangeWorkbench, Importers), introduces an injectable `createEditorGateway(bridge?)` for testing, and migrates 13 services + 11 components/hooks off direct `window.*` access onto the single gateway surface. Includes P3 Wave B stabilization fixes and the v0.99 S4 test-fixture follow-ups.

### New features

- **`createEditorGateway(bridge?)` injectable factory** + mock bridge for unit tests (`frontend/src/services/EditorGateway.ts`)
- **`getSceneAssetCatalog` shape normalization** — backend returns flat array; gateway normalizes to `{entries, warnings}`
- **22 WASM bindings re-exposed on `window`** (ChangeWorkbench 6, World 11, Importers 5) — fixes the "World Workspace broken in production" regression that started in v0.95
- **`WindowWithBridge` regenerated** — 128 bindings, real signatures, no false positives
- **WORLD_CATALOG initialization** in engine setup + rebuild from `project.worlds` on load (root cause of the v0.95 regression: `world_state.rs:96` panic)

### Refactors

- **13 services + 11 components/hooks migrated** from `window.*` to `EditorGateway` (`useSceneAssets`, `useLogicGraph`, HierarchyPanel, InspectorPanel, …)
- **Domain-by-domain migration**: world, change-workbench, importers, scene, schema, logic — each reversible independently

### Fixes

- **Asset cache sync** + scene-component list + scene swap current (`crates/editor-core`)
- **Recursive OPFS hydrate** via `opfsListTree` (`crates/editor-storage-web`)
- **WASM-safe time reads** — all `std::time::SystemTime` reads routed through wasm-safe helpers (no more `SystemTime::now` panic on wasm32 from issue #19)
- **Dropdown toggle + legacy toolbar inert + dirty-switch payload** (`crates/editor-frontend`)
- **Defensive override + catalog shapes** (P3 Wave B)
- **Unify duplicated `SCENE_DOC` / `OPERATION_LOG` thread-locals** (P3 Wave B)
- **B2 cohort tags** + B1 migration regression fixes (P3 Wave B)
- **extension_data field on all test fixtures** (v0.99 SDD-0046 S4 follow-up)

### Stats

- 14 commits, 2280 insertions / 1517 deletions across 130 files
- `world-workspace.spec.ts`: 9/9 pass (was 3/9)
- New `gateway-contract.spec.ts`: 9 unit tests with mock bridge
- Gate: `(window as any)` occurrences in `src/services` + `src/components` → 0 (outside `engine-bridge.ts` / `EditorGateway.ts`)
- Smoke cohort: 25/25 deterministic (3 runs)

### Architecture

- ADR-0034 (Typed EditorBackend Contract Replaces Global Window Bridge) partially satisfied: the gateway is now the single frontend entry point
- ADR-0027 (Rig-Based Agent Runtime) prerequisites advanced: `EditorGateway` is the seam any future agent tool will call through

## v0.99.0 — Semantic Editor Model Extension Bags (ADR-0046 S4) (2026-08-18)

Delivers SDD-0046 Slice 4: unknown JSON fields are now PRESERVED in a per-type extension bag instead of silently dropped (ADR-0046 rule 2 / SEM-3 satisfied).

### New features

- **`extension_data` extension bag** on 7 core types (`SceneDocument`, `SceneAssetDocument`, `WorldDocument`, `LogicGraphAsset`, `ProjectMetadata`, `Entity`, `SceneAssetEntity`): `#[serde(default, flatten)] BTreeMap<String, serde_json::Value>`.
  - Deserialize: unknown top-level keys land in the bag (known fields parse normally)
  - Serialize: bag entries inline; empty bag adds ZERO bytes
  - Deterministic BTreeMap ordering (ADR-0045); unknown-field order not preserved (documented)
  - Old documents parse with an empty bag — no version bump required
- **S2 D4 completion**: `SceneAssetEntity` unknown fields upgrade from drop → preserve
- **S3 migration interplay**: `migrate::scene_document` / `migrate::project_metadata` materialize the empty bag explicitly
- **Adapter contract updated**: `JsonProjectAdapter::Lossless` semantics documented — known fields byte-exact; unknown fields preserved (deterministic)

### Stats

- 2 commits, ~360 LOC (135 type/wiring + 225 tests)
- 12 new tests (11 extension_bags + 1 adapter round-trip)
- All existing round-trip tests pass unchanged (empty bag = zero noise)

### Architecture

- SEM-3 satisfied: unknown data never silently disappears
- ADR-0045 deterministic ordering via BTreeMap
- ADR-0030 upheld: editor-model bevy-free, wasm-free

## v0.98.0 — Semantic Editor Model Migration Framework (ADR-0046 S3) (2026-08-18)

Delivers SDD-0046 Slice 3: the SEM-5 migration framework. Every document type now declares a format version and upgrades through typed, pure migration functions — old documents migrate explicitly, future documents fail loudly instead of silently corrupting.

### New features

- **`MigrationError`** (`crates/editor-model/src/migration.rs`): `UnsupportedVersion` (future document in older editor) + `MigrationFailed` (step failure), thiserror-derived.
- **`parse_version_string`**: `"0.1" → 0`, `"99.0" → 99` (major component), non-numeric → error.
- **Typed `migrate::<type>` functions** for all 5 core document types (`SceneDocument`, `SceneAssetDocument`, `WorldDocument`, `LogicGraphAsset`, `ProjectMetadata`). Current version = no-op; V0→V1 materializes serde-defaulted fields explicitly; future versions → `UnsupportedVersion`.
- **Adapter wiring**: `JsonProjectAdapter::decode` runs migrations after deserialization. Migration errors surface as `AdapterError::Decode`.
- **Load-site wiring**: all 4 project.json load paths route through `migrate::project_metadata`. **Critical fix**: the `update_project_metadata`/`update_project_schemas` helpers previously read a future-version project.json, silently defaulted it, and wrote back an empty project — destroying the file. Migration errors now surface.

### Stats

- ~590 LOC across 6 files (2 commits)
- 18/18 spec scenarios (16 unit/corpus + adapter scenarios)
- 630 workspace tests pass / 1 pre-existing failure unchanged
- 16 new tests (9 unit + 4 corpus + 3 adapter)

### Architecture

- ADR-0046 rule 4 honored: typed structures, never string replacement
- ADR-0030 upheld: `editor-model` remains bevy-free, wasm-free (wasm32 build clean)
- SEM-5 satisfied: every document type declares a format version with a tested upgrade path

## v0.97.0 — Semantic Editor Model Impl Hardening (ADR-0046 S2) (2026-08-18)

Delivers SDD-0046 Slice 2: closes all 4 ponytail debts left by S1. The adapter contract is now structurally sound — no memory leaks, cross-thread registry, real Bevy dispatch for Scene, and the `Lossless` fidelity claim is fully honest.

### Improvements

- **Owned `SemanticModel`** (`crates/editor-model/src/adapter.rs`): the enum no longer borrows (`'a` lifetime removed); `decode()` returns fully owned values — eliminates the `Box::leak` per-decode memory leak from S1.
- **`OnceLock` registry**: `thread_local!` replaced with `std::sync::OnceLock`; `init_registry(Vec)` is single-shot (panics on double-call) and `all_adapters()` is cross-thread safe. The S1 `set_registry_fn` API was replaced.
- **Real Bevy dispatch** (`BevyRuntimeAdapter`): `Scene(SceneDocument)` now calls the actual `export_dynamic_scene` projection (via new `From<editor_model::SceneDocument>` + `SceneInstance`/`ComponentOverride` conversions). `SceneAsset`/`LogicGraph`/`World`/`ProjectMetadata` return `UnsupportedModel` honestly — no editor-bevy projection accepts those types directly.
- **`SceneAssetEntity` fidelity violator removed**: `#[serde(deny_unknown_fields)]` dropped; unknown JSON fields are tolerated (matching every other document type). The `JsonProjectAdapter::Lossless` claim is now provable without caveats.

### Stats

- 3 commits, ~460 LOC changed across 6 files
- 16/16 spec scenarios passing (1 documented deviation: SceneAsset/LogicGraph → UnsupportedModel)
- 476/477 workspace tests pass (1 pre-existing failure unchanged)
- New tests: 4 unit (adapter.rs) + 3 unit (adapter_impls) + 1 integration (adapter_contract) + 2 D4 regression (scene_asset.rs)

### Architecture

- ADR-0030 upheld: `editor-model` remains bevy-free, wasm-free (wasm32 build clean)
- ADR-0046 progresses: SEM-6 fidelity contracts fully honest
- `From` impls added in `editor-bevy/src/{document,scene_instance}.rs` — field-by-field, no silent drops

## v0.96.0 — Semantic Editor Model Adapter Contract (ADR-0046 S1) (2026-08-18)

Delivers SDD-0046 Slice 1: the `EditorAdapter` trait + `AdapterFidelity` enum + 3 retroactive impls that establish the adapter contract required by SEM-6. SEM-6 (Fidelity contracts) is satisfied for the first time.

### New features

- **`EditorAdapter` trait** (`crates/editor-model/src/adapter.rs`): object-safe trait with `name()`, `fidelity()`, `encode()`, `decode()`, `supports()` methods. `SemanticModel<'a>` is a borrowed sum-type enum over 5 variants (`Scene`, `SceneAsset`, `LogicGraph`, `World`, `ProjectMetadata`). wasm32-compatible — no Bevy/WASM deps.
- **`AdapterFidelity` runtime enum**: `Lossless` ("encode+decode round-trip exact; no data loss"), `SemanticLossless` ("encode+decode preserves semantics; formatting may differ"), `ExportOnlyLossy` ("encode only; decode is not supported").
- **`AdapterError`**: 5-variant error enum (`UnsupportedModel`, `UnsupportedRole`, `ExportOnly`, `DecodeError`, `EncodeError`) each carrying the adapter name; implements `std::error::Error` + `Display` via `thiserror`.
- **`all_adapters()` registry factory** + **`set_registry_fn`** seam: returns `Arc<[Box<dyn EditorAdapter + Send + Sync>]>` via a thread_local `RefCell` slot.
- **3 retroactive impls** in `crates/editor-bevy/src/adapter_impls/`:
  - `JsonProjectAdapter` — Lossless, wraps 6 JSON writer sites; round-trips `SceneDocument`, `SceneAssetDocument`, `LogicGraphAsset`, `WorldDocument`, `ProjectMetadata` byte-for-byte
  - `BsnExportAdapter` — SemanticLossless, wraps `EditorCoreBsnExporter` (newtype); rejects `LogicGraph` role
  - `BevyRuntimeAdapter` — ExportOnlyLossy, placeholder JSON serialization for 4 Bevy projection sites (`export_dynamic_scene`, `export_rust_source`, `project_instances`, `rebuild_preview_world`)
- **41 new tests**: 8 unit (`adapter.rs`) + 21 unit (`adapter_impls/`) + 12 integration (`adapter_contract.rs`)

### Architecture

- `crates/editor-model/src/lib.rs` adds `pub mod adapter;` — pure, wasm32-compatible
- `crates/editor-bevy/src/lib.rs` adds `pub mod adapter_impls;`
- `crates/editor-bevy/src/adapter_impls/mod.rs` exposes `all_adapters_init()` factory
- No existing writer sites modified (`bsn_export.rs`, `dynamic_scene.rs`, `instance_projection.rs`, `preview_runtime.rs` verified unchanged by `git diff main`)
- SEM-6 (Fidelity contracts) satisfied for the first time

### ADR status changes

- **ADR-0046**: Accepted + Implemented (SEM-6 partial, v0.96.0) — S1 implemented; S2–S6 deferred

### Warnings (tracked to S2)

- **`Box::leak` per `JsonProjectAdapter::decode`** (5 sites): `decode` returns `SemanticModel<'static>` via `Box::leak(Box::new(owned))`; long-lived WASM sessions accumulate unbounded growth. S2 redesigns `SemanticModel` to own its data.
- **Registry seam is `thread_local! + RefCell`**: `set_registry_fn` slot is per-thread; multi-thread usage would silently no-op. S2 moves to `OnceLock<Arc<AdapterSlice>>` for cross-thread safety.
- **`BevyRuntimeAdapter` is a JSON stub**: body produces JSON not real Bevy projection output; `ExportOnlyLossy` fidelity claim is honest but S2 will wire to actual `export_dynamic_scene` / `project_instances` calls.
- **`SceneAssetEntity deny_unknown_fields`** at `scene_asset.rs:76`: `Lossless` caveat documented in `JsonProjectAdapter` rustdoc; S2 removes the lint and documents the Lossless asymmetry.

## v0.95.0 — World Workspace (ADR-0037) (2026-08-18)

Delivers the World Workspace as a first-class product context (ADR-0037), grouping
placed Level Scene Assets into a navigable authoring surface with topology validation,
layout policies, neighbour/portal relationships, and an LDtk import bridge.

### New features

- **`WorldDocument`** family of types in `editor-model` (`crates/editor-model/src/world.rs`):
  `WorldId`, `LayoutPolicy` (Free/Grid/Horizontal/Vertical), `WorldLinkKind`,
  `EntranceRef`, `StreamingPolicy`, `WorldLevelRef`, `WorldLink`, `WorldDocument`,
  `WorldCatalogEntry` — ADR-0037 line 14 invariant: WorldDocument **refers-to**
  Level Scene Assets, never contains level content
- **`WorldSessionState`** on `EditorSession` (`editor-application/session.rs:473`) with
  create-on-write accessor; `EditorSessionPort` extended with `world_state_mut`
- **WorldApi trait** (`editor-protocol/capabilities.rs:55-145`): 10 methods including
  `get_workspace`, `add_level_to_world`, `connect_levels`, `place_level`,
  `set_layout_policy`, `find_unreachable`, `layout_world_proposal`,
  `set_streaming_policy`, `validate_topology`
- **`ValidationApi::get_topology_issues`** wired for world-level topology validation
  (orphan levels → Warning, reciprocal mismatch → Warning, missing neighbour → Warning,
  missing asset_ref → Error)
- **`WorldCommand` enum** (`editor-bevy/world_command.rs`): 9 variants
  (WorldCreate, WorldPlaceLevel, WorldRemoveLevel, WorldConnectLevels,
  WorldRemoveLink, WorldSetLayoutPolicy, WorldSetStreamingPolicy, WorldSave,
  WorldDelete) with mechanical inverses for undo/redo
- **`WorldDocumentApplier`** in `transaction_bridge.rs:305-391` implementing the
  TransactionKernel `Applier` trait for world mutations
- **11 WASM exports** in `editor-bevy/src/lib.rs`:
  `create_world_wasm`, `save_world_wasm`, `load_world_wasm`, `list_worlds_wasm`,
  `delete_world_wasm`, `validate_world_topology_wasm`, `place_level_in_world_wasm`,
  `connect_levels_wasm`, `remove_link_wasm`, `set_layout_policy_wasm`,
  `open_level_from_world_wasm`
- **Frontend `WorldApi` TypeScript types** (`EditorGateway.ts:96-216`): `WorldSummary`,
  `WorldLevelRef`, `WorldLink`, `EntranceRef`, `LayoutPolicyDto`, `TopologyIssue`,
  `WorldCatalogEntry`, `TopologyIssueCode`, `TopologySeverity`
- **`EditorGateway.world` namespace** (`EditorGateway.ts:547-722`): 11 methods delegating
  to `window.*_wasm` exports
- **`WorldWorkspace.tsx`** component (471 lines): canvas rendering of level squares,
  directional link arrows, layout-policy toolbar (Free/Grid/Horizontal/Vertical),
  minimap, drag-to-place, double-click-to-open
- **`useWorldWorkspace` hook** (200 lines): exposes world document, topology issues,
  selected level, drag state, viewport, and all editing operations
- **`create_room_chain` recipe** (`editor-bevy/world_recipes.rs:30+`): emits
  `WorldConnectLevels` commands for consecutive level pairs in a chain
- **Recipe registry** (`world_recipes_registry.rs`): `world.room_chain.v1` with
  `Capability::Commands`
- **LDtk world bridge** (`importer/ldtk.rs:740-899`): when LDtk world has ≥2 levels,
  emits `WorldCreate` + per-level `WorldPlaceLevel` + per-neighbour `WorldConnectLevels`
  + `create_room_chain` results + `WorldSave`; golden snapshot at
  `tests/fixtures/ldtk/expected_world.json`
- **`EditorMode`** extended with `"world"` entry; `ModeContextBar` adds world domain label

### Architecture

- World state lives on `EditorSession.world_states: BTreeMap<String, WorldSessionState>`
  (not a thread_local on the editing path — WASM↔Bevy seam thread_locals are separate)
- `WORLDS_DIR = "worlds"`; world documents stored as `<path>.world.json` in project OPFS
- `ProjectMetadata.worlds: Vec<WorldCatalogEntry>` + `active_world: Option<String>`
  both `#[serde(default)]` for backward compat
- `editor-storage-web` has no world-specific code (generic `ProjectStore` covers it)

### ADR status changes

- **ADR-0037**: **Implemented** (was: Accepted, 2026-08-14)

### Known limitations

- **`validation_center_tests::wasm_validation_cycle_in_active_graph`** test fails
  (pre-existing on `origin/main`, unrelated to this cycle)
- Pre-existing TypeScript errors in `editor_application.d.ts` (wasm-bindgen-generated)
- Menu "View → World Workspace" entry falls through to `todo()` — use ModeContextBar
  to enter World mode

## v0.94.0 — Hexagonal Crate Boundaries (ADR-0030) (2026-08-17)

Implements the full 6-crate architecture from ADR-0030 with compile-time hexagonal
boundaries. editor-model is now fully pure (no Bevy, no WASM bindings).

### Architecture changes

- **`editor-bevy`** (renamed from `editor-core`): Bevy adapter — scene authoring,
  logic graphs, WASM preview systems. No WASM glue at the root level.
- **`editor-protocol`** (NEW): Wire types and capability API trait definitions.
  Houses `Command`, `AssetCommand`, `LogicCommand`, `DispatchError`, and
  `SceneApi`, `SceneAssetApi`, `WorldApi`, `LogicApi`, `RuntimeApi`,
  `CodeApi`, `ValidationApi`, `ChangeApi` traits per ADR-0034.
  Pure — no Bevy, no wasm-bindgen.
- **`editor-wasm`** (NEW, cdylib): WASM cdylib extracted from editor-application.
  `[lib] name = "editor_application"` preserves frontend imports.
  The only crate with wasm_bindgen/web_sys/js_sys at the root level.
- **`editor-storage-web`** (NEW): OPFS adapter extracted from editor-application.
  `OpfsCore`, `RawStoreBridge`, WASM bridge in cfg-gated module.
- **`editor-application`**: Now pure application services. WASM glue removed.
  Depends on: editor-model, editor-protocol, editor-storage-web.
- **`editor-model`**: Unchanged. Pure semantic types.

### CI / quality gates

- **`archcheck`** (NEW CI workflow): Runs all B1–B9 assertions on every PR.
  B7: editor-protocol purity. B8: editor-model/application/protocol wasm-free.
  B9: editor-storage-web wasm imports cfg-gated.
- **D1 regression test**: `dispatch_kernel_regression` test suite (4 tests)
  proves `dispatch_command_via_kernel` routes through the ChangeWorkbench
  approval path.

### ADR status changes

- ADR-0030: **Implemented** (was: Accepted, partial)
- ADR-0044: **CI gate wired** (archcheck in CI)

## v0.93.0 — External Source Importers (SDK-061) (2026-08-17)

Delivers the Aseprite, LDtk, and Tiled import pipelines with provenance tracking,
reimport conflict handling, and the SDK-061 importer protocol. Completes the
v0.92 ecosystem roadmap item.

### New features

- **`ExternalSource` provenance types** (`editor-model`): `ExternalSource`,
  `ExternalSourceKind`, `SourceMapping`, `OwnershipRule`, `ProvenanceDiff`,
  `ConflictPolicy` — sidecar `.meta.json` stores path, SHA-256 fingerprint,
  and last-import time
- **`ImporterRegistryPort`** trait + `Arc<Mutex<dyn ImporterRegistry>>` as
  9th `EditorSession` sub-state; mirrors the `ExtensionRegistryPort` pattern
- **Aseprite import pipeline** (`builtin.aseprite`): parses Aseprite JSON v1/v2
  → `LevelLayer::Tile` per frame; PNG spritesheet → `AssetFile` at `resources/`;
  validated via `ImporterVersionRange`
- **LDtk import pipeline** (`builtin.ldtk`): parses LDtk JSON 1.0.0–1.5.0;
  one `SceneAssetDocument` per level at `levels/<world>/<level>`;
  `LevelLayer::IntGrid` + `LevelLayer::Auto` for semantic/auto layers;
  entity instances → `SceneInstance` with `Transform2D` + field components
- **Tiled import pipeline** (`builtin.tiled`): parses Tiled JSON v1.0–v1.10;
  TMX/XML rejected with `ImporterError::UnsupportedKind("xml")`;
  tile layers → `LevelLayer::Tile`; object layers → `SceneInstance`;
  embedded tilesets → `TilesetAsset`; templates → `Fragment` documents
- **Reimport pipeline** (`editor-application/reimport.rs`): SHA-256 fingerprint
  of source bytes; `ProvenanceDiff` computes added/removed/changed mappings;
  editor-owned changes routed to Change Workbench with `ApprovalPolicy::RequiresHuman`
- **`ConflictPolicy`** enum (`AutoApply`, `HumanReview`, `SkipOnConflict`) wired
  into reimport per `OwnershipRule`
- **`ImportDialog.tsx`** frontend component: kind selector, file picker,
  destination path, conflict summary
- **5 WASM exports**: `list_importers_wasm`, `importer_descriptor_wasm`,
  `import_external_source_wasm`, `reimport_external_source_wasm`,
  `list_imported_sources_wasm`
- **SDK versioning**: `ImporterVersion` + `ImporterVersionRange` mandatory on
  all descriptors; range-gating validator rejects out-of-range sources
- **6 built-in assertions**: FakeSession test gate verifies exactly 6 built-in
  extensions (3 logic + 3 importers)

### Architecture

- `crates/editor-model/src/int_grid.rs`: `IntGridCoord`, `IntGridCell`,
  `IntGridLayer`, `IntGridSchemaKind` — `LevelLayer::IntGrid` variant for
  LDtk semantic cells
- Architecture fitness gate: importer crates cannot import `EditorSession`
  directly; must use typed port traits only
- ADR-0041 updated to "Accepted + Implemented (v0.93)"

### Known limitations

- **`editor-model` WASM build** has a pre-existing `js_sys` unresolved import
  in `scene_asset_catalog.rs` (was present on `main` before this cycle;
  not introduced by v0.93)
- `wasm_validation_cycle_in_active_graph` test fails on `main` (pre-existing,
  unrelated to importers)

## v0.92.0 — Ecosystem & Architecture Hardening (2026-08-17)

Delivers the Editor Extension SDK (ADR-0040 steps 1 and 2), completes the
v0.91 deferred thread-local migrations, and ships two significant
refactorings (EditorSession split, FakeSession extraction).

### New features

- **Editor Extension SDK — internal registry** (`ExtensionManifest`,
  `Capability`, `Permission`/`PermissionArea`/`PermissionScope` type system;
  `ExtensionRegistryPort` trait + `ExtensionRegistry` impl; held as 8th
  `EditorSession` sub-state via `Arc<Mutex<dyn ExtensionRegistryPort>>`)
- **Apply-time permission re-check**: `transaction_kernel_check_plugin_permission`
  fires for `ChangeOrigin::Plugin` ChangeSets before the preflight loop;
  `extension:<id>` actor prefix is the single source of truth for
  extension-originated ChangeSets
- **WASM exports**: `register_extension_wasm`, `list_extensions_wasm`,
  `unregister_extension_wasm`, `submit_plugin_change_set_wasm` (routes through
  pending ChangeSet flow for ChangeWorkbench visibility)
- **Three built-in extensions via SDK**: `builtin.logic-bricks.controllers`
  (`Capability::Commands`), `builtin.logic-recipes` (`Capability::Recipes`),
  `builtin.scene-validator` (`Capability::Validators`) — all registered at
  `EditorSession::with_builtins()` composition time
- **`ValidationIssue` unified as canonical WASM-boundary type**:
  `LogicValidationIssue` retained as `#[deprecated]` specialization with
  adapter at boundary
- **Architecture fitness gate** (ADR-0044): extensions must not import
  `EditorSession` directly (enforced by archcheck)

### Also delivered (from v0.91 cycle amendment)

- **SCENE_DOC thread_local migration** (HIGH-1): scene_session re-entrancy
  fixed via `with_asset_doc_take()` / `restore_doc()` pattern —
  `apply_command` no longer holds `&mut SceneDocument` across re-entrant
  session locks
- **LOGIC_GRAPH_DOC thread_local migration** (HIGH-2): logic_state
  re-entrancy fixed via `take` / `restore` pattern mirroring scene_session
- **ASSET_OPERATION_LOG migration** (HIGH-3): `undo_asset` / `redo_asset`
  now use `with_asset_doc_take()` / `with_asset_log_take()` instead of nested
  `RefCell` borrows; re-entrancy safe
- **EditorSession god-class split** (MEDIUM-5): `session.rs` restructured
  into 3 sub-structs — `PreviewSessionState`, `ChangeSetsSessionState`,
  `RuntimeSessionState` — with domain methods; `EditorSession` reduced from
  16 to 11 fields; 14 delegate methods updated
- **FakeSession shared test harness** (MEDIUM-6): `FakeSession` struct with
  full `EditorSessionPort` impl extracted to
  `crates/editor-core/tests/support/mod.rs`; eliminates ~500 lines of
  duplicate struct+impl across 5 integration test files
- **StableId type consolidation** (ADR-0049): unified `editor_model::StableId`
  across `editor_core`; `document::StableId` conversion via `as_str()`
- **Poll loop wiring + dedup**: `get_change_set_summaries_wasm` fully wired
  to `EditorSession.recent_change_sets` via `EditorSessionPort`
- **`OperationLog` placeholder cleanup** (post-v0.92): `AssetSessionState::operation_log_bytes`
  and `LogicSessionState::operation_log_bytes` removed; the poll loop in
  `preview_runtime.rs` reads `OperationLog` thread_local directly and pushes
  `ChangeSetSummary` via `push_recent_change_set`, making the serialized-bytes
  path obsolete

### New features (from Unreleased)

- Frontend ESLint and Prettier production gates.
- GitHub Actions CI, tagged release packaging, Dependabot, and JavaScript bundle budget enforcement.
- User, contributor, security, and release documentation.
- Production checks now enforce a 350 KB gzip budget across built JavaScript assets.

### Known limitations

- Pre-existing test `validation_center_tests::wasm_validation_cycle_in_active_graph`
  (unrelated to this cycle; to be fixed separately)
- Cosmetic unused-variable warning in `extension.rs:179` (trivial fix)
- **FakeSession built-in manifest assertion** (`assert_eq!(list.len(), 3)`)
  not yet added as a CI gate — infra in place, test missing
- **Multi-thread concurrent register test** and **stale permission removal test**
  not yet written — infrastructure ready, tests are the remaining work

## v0.91.0 — Thread-Local Migration Completion (2026-08-16)

Closes the functional gaps from v0.90 and ships the post-v0.90 cleanup
work. The cycle is intentionally narrow (3 PRs vs the 6-PR v0.90): the
remaining v0.91 work is split out as a v0.92 follow-up.

### New features

- **`apply_back_eligible` derived from `ApplyBackPolicy`** (PR1): the
  hardcoded `true` in the 3 RuntimeDelta capture sites is replaced with a
  schema lookup. The 3 builtin schemas (including `editor.Transform2D`)
  have `apply_back = Never` per D4, so their deltas are correctly flagged
  ineligible. `ExplicitOnly` and `Tunable` schemas produce eligible
  deltas. Falls back to `true` (conservative) for unknown schemas.
- **`get_change_set_summaries_wasm` wired to session** (PR1): the v0.90
  stub returning `[]` now reads from `EditorSession.recent_change_sets`
  via the `EditorSessionPort::all_recent_change_sets` trait method.

### Changed

- `crates/editor-core/src/scene_asset_catalog.rs` moved to
  `crates/editor-model/src/scene_asset_catalog.rs` (PR2): the data type
  is editor-model canon per the v0.88 PR B architecture. Re-exports
  updated in both editor-core and editor-application.
- `SCENE_ASSET_CATALOG` and `SCENE_ASSET_CATALOG_WARNINGS` thread_locals
  removed (PR2): both are now fields on `EditorSession::asset_states["_active"]`
  (`catalog: Option<SceneAssetCatalog>` and `catalog_warnings: Vec<CatalogWarning>`).
  The `with_asset_catalog`, `with_asset_catalog_mut`,
  `get_asset_catalog_warnings`, `clear_asset_catalog_warnings` functions
  are reimplemented as session accessors.
- `dispatch_command`, `dispatch_asset_command`, `dispatch_logic_command`
  simplified to always route through `TransactionKernel` (PR5): the legacy
  pre-kernel `is_dispatch_via_kernel() == false` branch is removed.
  ADR-0032 established the kernel as the single dispatch path; the
  legacy fallback was never reachable in production since v0.89.
- 3 legacy dispatch functions deleted (PR5): `dispatch_command_legacy`,
  `dispatch_asset_command_legacy`, `dispatch_logic_command_legacy`.
  ~80 LOC of pre-v0.89 dead code removed.

### Known limitations (deferred to v0.92)

- **Pre-existing TypeScript errors** in `editor_application.d.ts`
  (17 errors, lines 807-1015). Pre-existing since v0.89 PR4; `vite build`
  exits 0 (transpileOnly). Out of scope for v0.91.

> All three thread_local migrations (SCENE_DOC, LOGIC_GRAPH_DOC, ASSET_OPERATION_LOG),
> the EditorSession split, and the FakeSession extraction were completed
> in v0.92.0 — see v0.92.0 "Also delivered" section.

## v0.90.0 — Thread-Local Liquidation & Apply-Back Wiring (2026-08-16)

Closes the v0.89 14-item debt-report backlog (PR2a cycle amendment deferred to v0.90). The cycle delivers the `EditorSessionPort` seam (10 accessors) for cross-crate session access, migrates 3 high-priority thread_locals to the session, and removes `ProcessorContext::from_globals()`. The full thread_local migration (18 thread_locals remain) is deferred to v0.91 — the seam is now in place to do the per-file migration mechanically.

### New features

- **`EditorSessionPort` trait** (`crates/editor-model/src/session_port.rs`): object-safe trait with 10 accessor methods (`tunable_baselines_mut`, `last_rebuild_cause_mut`, `pending_causality_edges_mut`, `runtime_delta_buffer_mut`, `scene_state_mut`, `asset_state_mut`, `logic_state_mut`, `preview_inspector_mut`, `source_files_mut`, `recent_change_sets_for`, `logic_activation_ring_mut`). `editor-application::EditorSession` is the canonical impl; `editor-core` Bevy systems access the session through `editor_model::ports::with_session_mut`.
- **`editor_model::ports::{register_editor_session, with_session_mut, with_session}`**: pattern parallel to the existing `with_project_store`. WASM `init_project_store` registers the session; Bevy systems read/write via the trait.
- **Sub-state types in `editor_model::session`**: `SceneSessionState`, `AssetSessionState`, `LogicSessionState`, `ChangeSetSummary`, `PreviewInspectorState`, `SourceFilesCache`. Replaces the v0.89 PR2a local structs in `editor-application::session`; the `EditorSession` maps now use the editor-model types.
- **`RuntimeDelta` moved to `editor_model`**: pure data type, no Bevy/WASM deps. `editor-application` re-exports it.
- **Per-field recursive diff** (`editor_core::preview_runtime::compute_runtime_deltas_internal`): compares baseline vs runtime values for each leaf key; pushes one `RuntimeDelta` per differing field. Closes the v0.89 OVERENG-2 functional gap (`ApplyBackPanel` will now show real data instead of always-empty state).
- **ApplyBackPolicy mirror-pair serde-equivalence test** (PR6, D3 spec §8): guards the ADR-0050 invariant that `editor_core::ApplyBackPolicy` and `editor_application::ApplyBackPolicy` serialize byte-equal for all 3 variants.

### Changed

- `EditorSession` sub-state types unified on `editor_model` types (removes the v0.89 PR2a local structs in `editor-application::session`).
- `get_rebuild_cause_wasm` reads ONLY from the session (the v0.89 dual-read fallback from the editor-core thread_local is gone).
- `capture_tunable_baselines_internal` writes ONLY to the session (the v0.89 dual-write to the `TUNABLE_BASELINES` thread_local is gone).
- `RUNTIME_DELTA_BUFFER_CAP` constant extracted from 4 hard-coded `64` literals (1 in `runtime_delta.rs`, 3 in `session.rs`).

### Fixed

- **Functional gap closed**: `ApplyBackPanel` will now show real `RuntimeDelta` records (was always empty in production because Bevy `process_play_mode_request(PlayModeExit)` wrote to a thread_local, never to the session). The diff is per-field recursive.
- **ADR-0052 unification**: `LAST_REBUILD_CAUSE` + `PENDING_CAUSALITY_EDGES` thread_locals removed from `editor-core::preview_inspector`. Both now live canonically in `EditorSession` via the trait. The `document::StableId` ↔ `editor_model::StableId` conversion is via `as_str()`.
- **`ProcessorContext::from_globals()` removed** (was deprecated in v0.89 T-02-03). Use `ProcessorContext::with_asset_body` or `ProcessorContext::empty`.
- **Pre-existing dead code removed** (3 functions): `build_path_index`, `suffix_match` (both from 2026-06-28, 4 cycles old), `_ensure_component_instance_linked` (from 2026-07-18).
- **No-trigger `ponytail:` marker rewritten** in `asset_command.rs:285` (was 3 cycles old, pointed to a non-scheduled Validation Center capability).

### Known limitations (deferred to v0.91)

- **18 `thread_local!` declarations remain in `editor-core`** (target was 5-6 per D2). The `EditorSessionPort` seam is complete and the per-file migration is now mechanical: for each thread_local, replace `XXX.with(|c| ...)` with `with_session_mut(|s| s.method_mut())` and delete the `thread_local!` declaration. The remaining thread_locals span: `scene_session` (SCENE_DOC), `asset_state` (SCENE_ASSET_CATALOG + WARNINGS), `asset_command` (ASSET_OPERATION_LOG), `logic_state` (LOGIC_GRAPH_DOC + LOGIC_OPERATION_LOG), `logic_command` (LOGIC_OPERATION_LOG), `preview_inspector` (PREVIEW_METRICS + MAPPING + PROVENANCE), `logic_evaluator` (global_node_registry), `logic_recipes` (BUILTIN_RECIPES), `schema` (COMPONENT_SCHEMA_REGISTRY), `source_files` (SOURCE_FILE_REGISTRY), `operation_log` (OPERATION_LOG), `actuator_bus` (ACTUATOR_OUTPUT_BUS), `hot_reload_state` (HOT_RELOAD_BUS), `preview_runtime` (DIRTY_FLAG).
- **`get_change_set_summaries_wasm` is a stop-gap** returning `[]`. Wiring to `EditorSession.recent_change_sets` (populated by the `OperationLog::recent_change_sets_for` poll loop) is deferred to v0.91.
- **Pre-existing TypeScript errors** in `editor_application.d.ts` (17 errors, lines 807-1015) caused by wasm-bindgen 0.2 + wasm-pack 0.14 generated JSDoc. `vite build` exits 0 (transpileOnly). Out of scope.
- **`OperationLog` type itself not yet in `editor-model`**: the `AssetSessionState::operation_log_bytes` and `LogicSessionState::operation_log_bytes` fields are `Vec<u8>` placeholders. The real `OperationLog` type still lives in `editor-core`; moving it is a v0.91 task.

## v0.89.0 — Change & Runtime Workbench (2026-08-16)

Closes the v0.88 deferred "TransactionKernel not yet wired into actual editor dispatch paths" and ships 3 epics: Change Workbench UI, Runtime Causality, Runtime Apply-Back. All 14 spec scenarios pass with covering runtime tests (PASS WITH WARNINGS); 8 tasks explicitly deferred to v0.90 per cycle amendment.

### New features

- **TransactionKernel adoption (D1, PR1, #145)**: `DISPATCH_VIA_KERNEL: AtomicBool` runtime flag + Cargo `dispatch-via-kernel` feature (default ON). `dispatch_command` / `dispatch_asset_command` / `dispatch_logic_command` route through `SceneTransactionKernel::apply_atomic` when the flag is set; v0.88 path stays as documented reference impl. `AssetCommandApplier` and `LogicCommandApplier` mirror `SceneCommandApplier` in `crates/editor-core/src/transaction_bridge.rs` (validate → apply → inverse → rollback). Byte-equality undo/redo test (spec §3) passes — kernel and legacy produce identical `OperationLog` entries.
- **EditorSession consolidation (D2, PR2a, #146 + #147)**: kernel types live in `editor-model` (eliminates `editor-core → editor-application` dep that v0.88 introduced). `EditorSession` owns 6 sub-state maps (`scene_states`, `asset_states`, `logic_states`, `validation_issues`, `recent_change_sets: VecDeque<ChangeSetSummary>`, `runtime_delta_buffer: VecDeque<RuntimeDelta>`) plus `pending_change_sets`. `recent_change_sets_for(scene_path)` query wired. `ProcessorContext::from_globals()` deprecated in favor of `&EditorSession` parameter.
- **ChangeWorkbench + Partial-Apply (D5+D8, PR2b, #149)**: `ChangeWorkbenchPanel` is a bottom-dock tab with `PanelId = "change-workbench"` and `useDockPrefs.SCHEMA_VERSION` bumped 3 → 4. `migratePrefs` defaults the new `panelRegions["change-workbench"] = "bottom"` for v3 fixtures. `SceneTransactionKernel::approve_selected(op_indices)` supports partial approval with revalidation per `docs/specs/change-workbench.md §Actions` — 2-of-5 + all-or-nothing-on-revalidation-failure tests pass.
- **ChangeWorkbench WASM relocation (PR2b-fix, #150 + #151)**: the 6 workbench exports (`submit_pending_change_set`, `get_pending_change_sets`, `approve_change_set`, `approve_selected_ops`, `reject_change_set`, `get_change_set_summaries`) moved to `editor-application::wasm` accessing `EditorSession::pending_change_sets_mut()` through `OnceLock<Arc<Mutex<EditorSession>>>` — replaces the v0.88-era thread_local + unsafe raw-pointer bridge, restoring ADR-0031. WASM cdylib target moves from `editor-core` to `editor-application`.
- **Runtime Causality (PR3, #152)**: `RebuildCause` 6-variant enum (`UserEdit{command_id}`, `HotReload{file_id}`, `PlayModeEnter`, `PlayModeExit`, `SceneSwitch{from,to}`, `AssetResync{asset_ref}`) recorded on every `rebuild_preview_world`. `LogicActivationEvent` ring buffer capped at 64 (FIFO evict). `CausalityEdge{Kind}` (5 variants: `Definition`, `Instance`, `Override`, `Logic`, `Source`) attached to `PreviewProvenance`. `RuntimeCausalityPanel` renders the rebuild cause + activation ring + provenance edges.
- **Runtime Apply-Back ThisInstance (D4+D6+D8, PR4, #153)**: `ApplyBackPolicy` (`Never` default, `ExplicitOnly`, `Tunable`) attached to `ComponentSchema.apply_back` with `#[serde(default)]` (legacy v0.88 fixtures deserialize to `Never` per ADR-0050). `RuntimeDelta` ring (cap 64) on `EditorSession.runtime_delta_buffer` populated on `PlayModeExit`. `create_apply_back_change_set_wasm` emits one `Command::SetComponentField` per selected delta (scope `ThisInstance` only). `ApplyBackPanel` submits the resulting `ChangeSet` to the workbench. ADR-0050 establishes a documented mirror-pair for `ApplyBackPolicy` / `ApplyBackScope` (canonical in `editor-application`, parallel in `editor-core`).
- **Architecture assertions extended (NFR-2, PR4, #153)**: `tools/archcheck` adds B5 (`ChangeWorkbenchPanel` only in `BottomDock`) and B6 (`ApplyBackPanel` no Bevy Entity references) — 8/8 assertions pass.
- **Doc-completeness gate (NFR-4)**: `#![deny(missing_docs)]` enforced on `editor-application` post-PR4.

### New ADRs

- **ADR-0049** — Dual Dispatch Gate (TransactionKernel adoption is flag-reversible).
- **ADR-0050** — Apply-Back Policy: mirror-pair in `editor-core` + `editor-application`, NOT in `editor-model` (cross-crate serde compatibility invariant).
- **ADR-0051** — `ChangeWorkbenchPanel` lives in bottom-dock as an internal tab (ADR-0039/0024).
- **ADR-0052** — Runtime Causality: `RebuildCause` + `LogicActivationRing` + `CausalityEdge` (with §Architectural Note documenting the transitional dual-write path for the rebuild cause).

### Changed

- **WASM binary target** moves from `editor-core` to `editor-application` (`crates/editor-application/Cargo.toml` gains `crate-type = ["cdylib", "rlib"]`; `editor-core` becomes rlib-only). `justfile` `editor_crate` updated.
- **EditorGateway** (frontend) adds 5 workbench methods + 3 causality methods + 3 apply-back methods.

### Fixed

- ADR-0031 violation reintroduced by PR #150 (thread_local+unsafe bridge in `editor-application::wasm`). Restored by PR #151 (`OnceLock<Arc<Mutex<EditorSession>>>` accessor + no thread_local + no unsafe).
- Functional bug in `get_rebuild_cause_wasm` (PR3 #152): the write path was `editor-core::preview_inspector::record_rebuild_cause` (thread_local), but the WASM export read from `EditorSession.last_rebuild_cause` (always None). Fixed in the post-merge follow-up to read from the thread_local first, session fallback.
- `Cargo.lock` regenerated to include the v0.89 sub-state types and the new `editor-application` WASM target.

### Known limitations (deferred to v0.90)

- Deep `thread_local!` migration in `editor-core` (T-02-02, T-02-03, T-02-05): 14 thread_locals still in editor-core; the v0.88 → v0.89 cycle shipped 8 deferred tasks per the cycle amendment.
- `EditorSession::runtime_delta_buffer` is the documented write path but Bevy's `process_play_mode_request` writes to `TUNABLE_BASELINES` thread_local and does not yet compute `RuntimeDelta`. `ApplyBackPanel` will show empty state in production until the Bevy→RuntimeDelta pipeline is wired.
- `get_change_set_summaries` is a stub returning `[]`; to be sourced from `EditorSession::recent_change_sets` after the `OPERATION_LOG` thread_local migration.
- Pre-existing TypeScript errors in `editor_application.d.ts` (wasm-bindgen 0.2 + wasm-pack 0.14 generated JSDoc) cause `tsc --noEmit` to fail. `vite build` exits 0 (transpileOnly).
- ADR-0050 mirror-pair invariant (apply-back policy in 2 crates) is documented but not yet enforced by an automated serde-equivalence test.

## v0.88.0 — Architecture Debt (2026-08-15)

Liquidates the tracked debt from v0.87 (4 verify WARNINGs, deferred ADR decisions) and lands the application-layer composition infrastructure for the v0.88 production-authoring epics.

### New features

- **`crates/editor-application::session::EditorSession`** (ADR-0031): explicit application-level owner of mutable editing state — composes `Arc<dyn ProjectStore>` + `Arc<dyn Clock>`, owns active-document selection, explicit per-document `HistoryScope`s (survive deselection), and named caches with generation-based invalidation. Session isolation is unit-tested; the WASM composition root holds exactly one.
- **`crates/editor-application::transaction`** (ADR-0032): `TransactionKernel` + `ChangeSet<O>` — dry-run preflight simulation, atomic apply with inverse-based rollback, approval gate (`RequiresHuman`), effects/diff summaries, and `ApplyReceipt`. No universal command enum: domains plug in via the generic `Applier` trait. `crates/editor-core::transaction_bridge::SceneCommandApplier` bridges scene commands onto the kernel with atomicity/rollback/approval integration tests.
- **Real `OpfsProjectStore`** (ADR-0033/0048): mirror + write-through flush over the proven `window.opfs_*` JS bridge — sync `ProjectStore` semantics per ADR-0048 with durability-preserving `flush()`. Eager `hydrate()` at WASM startup; contract tests shared with `InMemoryProjectStore`. The 7 legacy `js_*` wrappers in `editor-core` now delegate to the store (signatures and call sites unchanged); writes still resolve only after the OPFS write is durable.

### Changed

- **`Timestamp`** is now a newtype (`pub struct Timestamp(pub u64)`) with transparent serde (persisted JSON unchanged), `Display` (keeps `mint_asset_id` format byte-identical) and `From<u64>` (WARNING-4).
- **LocalId collapse completed** (T-02-14): `editor-core::document::LocalId` duplicate struct replaced by a re-export of `editor_model::ids::LocalId`; exactly one canonical definition remains.
- **`tools/archcheck` expanded 2 → 6 assertions** (NFR-4): editor-model purity, editor-application root purity, dependency direction, LocalId uniqueness; new `--list` mode (T-01-04).
- `editor-core` now depends on `editor-application` (ports/adapters wiring).

### Fixed

- `#![deny(missing_docs)]` on `editor-model` with full pub-item documentation (WARNING-1).
- Zero `unwrap`/`expect`/`panic` in `editor-application` non-test code — lock poisoning maps to `StoreError::LockPoisoned` (WARNING-2).
- Doc gate `RUSTDOCFLAGS="-D warnings" cargo doc` in CI for `editor-model` + `editor-application` (NFR-1).
- Latent defect: `cargo test -p editor-application` failed to compile in isolation (tokio dev-dependency missing `macros`, masked by workspace feature unification with `ai-proxy`'s `tokio=full`); tokio removed entirely — contract tests are sync.

### Known limitations

- `editor-core` still owns 14+ `thread_local!` stores; full `EditorSession` adoption is gradual (later cycles). `ProcessorContext::from_globals()` still exists (ADR-0031 rule pending).
- OPFS `hydrate()` is eager — binary assets load at startup (sync-access-handles are the future fix).
- `TransactionKernel` is not yet wired into actual editor dispatch paths (bridge + tests only; adoption lands with the v0.89 Change Workbench).

## v0.87.0 — Architecture Foundation (2026-08-15)

### Breaking changes

- **`crates/editor-core` ↔ `crates/editor-model`**: 9 pure modules moved out of `editor-core` into a new `editor-model` crate (no Bevy, no WASM). Use `editor_model::document::Document`, `editor_model::scene_asset::SceneAssetDocument`, etc. Legacy `editor_core::document::Document` paths still re-export.

### New features

- **`crates/editor-application`**: new application crate with sync `ProjectStore` port per ADR-0048. `InMemoryProjectStore` (test canonical) + `OpfsProjectStore` wasm32 stub (full wiring in v0.88).
- **`crates/editor-model::time::Clock` trait**: per ADR-0035. `JsSysClock` (production, `js_sys::Date::now()` on wasm32) + `FakeClock` (tests).
- **`mint_asset_id` refactor**: takes `&dyn Clock` parameter; byte-pinned regression test.
- **`tools/archcheck`**: new architecture-fitness tool, runs in CI as `Architecture fitness` job.

### CI changes

- New job: `Architecture fitness` (runs `tools/archcheck`).
- New job: `editor-model purity (no Bevy/WASM)` (grep gate + wasm32 build).
- `docs/branch-protection.md` documents the 5 required GitHub checks.

### ADRs ratified this cycle

- ADR-0047 — Logic Graph Model Split (pure types in editor-model, LogicBinding adapter in editor-core)
- ADR-0048 — ProjectStore v1 is synchronous port

### Deprecations

- `editor_core::document::LocalId` and `editor_core::scene_asset::LocalId` are deprecated. Use `editor_model::ids::LocalId`.

### Known issues (v0.88 debt)

- WARNING-1: `editor-model` missing `#![deny(missing_docs)]` (NFR-1)
- WARNING-2: `editor-application` has 6 `.unwrap()` in non-test code paths (NFR-2)
- WARNING-3: `tools/archcheck` has 2 active assertions vs NFR-4 ≥6
- WARNING-4: `Timestamp` is `pub type Timestamp = u64`, not newtype struct (spec §5)
- `OpfsProjectStore` is a `unimplemented!()` stub; OPFS migration deferred to v0.88.

### Migration notes

- Replace `editor_core::document::Document` with `editor_model::document::Document` (legacy re-export still works).
- Pass `&JsSysClock::new()` (or your own `Clock` impl) to `mint_asset_id`.
- `ProjectStore` trait is sync; if you need async, the migration is deferred to v0.88.

### Stats

- 4 PRs merged: #135, #136, #137, #138
- 1 fix PR: #139
- 5 commits on main since 3dd0aad (v0.86.1 stabilization)
- 38 implementation tasks across 4 PRs
- 29 spec scenarios, 25 COMPLIANT + 4 PASS-WITH-WARNING
- 684 tests pass with `--locked`
- Architecture-fitness entropy: CoN -26%, CoM -42%, DQS 0.35 → 0.50

## v0.78.0 - 2026-07-21

### Added

- Scene Component catalog picker and draft validation.
- Place Instance helpers in the Asset Browser and Schema panel.
- Focused Playwright coverage for placement and undo workflows.

### Changed

- Scene Component authoring UX now reuses supported direct WASM exports.

### Fixed

- Improved executable coverage for the Hito 7 authoring flow.

### Removed

- None.

## v0.79.0 - 2026-07-22

### Added

- Hito 2 Order 8 — level-design tools (tile painting, IntGrid authoring, tileset CRUD).

## v0.80.0 - 2026-07-23

### Added

- Hito 5 — Defold-inspired 3-region dock layout with menu portal, viewport polish,
  status bar, useCodeFiles reliability, and the Workspace Preset hooks.

## v0.81.0 - 2026-07-25

### Added

- Hito 6 Tier 1 — global search (PR #113), workspace presets (PR #114),
  drag-and-dock infra (PR #115), and panel polish + status-bar drag-resize (PR #112).

## v0.82.0 - 2026-07-26

### Added

- v0.82 P1 — drag-and-dock region swap (ADR-0024, PR #117).
- v0.82 P2 — floating panels and inspector multi-select (ADR-0025, PR #118).

## v0.83.0 - 2026-07-27

### Added

- Asset browser thumbnails (ADR-0026, PR #119) — optional `preview_resource`,
  IntersectionObserver lazy loading, bounded LRU (≤32 entries).

## v0.84.0 - 2026-07-28

### Fixed

- `fix/code-aware-ai-debt` (PR #120) — security filter wiring, UTF-8 panic guard,
  frontend bundle divergence, dead UI toggle, weak test, six doc-drift corrections.

## v0.85.0 - 2026-07-29

### Added

- `editor-shell-integrity` (Hito 8 prerequisite) and `workflow-surface-convergence`
  via PR #125.

## v0.86.0 - 2026-07-30

### Added

- `ui-workflow-overhaul` (PR #126) — ModeContextBar, Hierarchy v2, Validation v2,
  Logic v2, Runtime v2, AI Panel v2.

## v0.86.1 - 2026-08-02

### Added

- Application stabilization release-health gate
  (`docs/specs/application-stabilization-and-roadmap-convergence.md`).
- Frontend performance budget contract (`ADR-0029`): three budgets
  (initialJs 380 KB, totalJs 800 KB, wasm 20 MB) enforced by
  `frontend/scripts/check-bundle-size.mjs`.
- Unified editor readiness signal: `window.__bevyEngineStarted`
  published only after `start_engine` returns without throwing.
- Documentation hierarchy and drift detection contract
  (`docs/specs/documentation-hierarchy-and-drift-detection.md`)
  plus `tools/docs-check/` automation.

### Changed

- `engine-bridge.ts` no longer uses dynamic imports of `opfs-bridge` or
  `services/hot-reload`; both are statically imported.
- `LogicGraphEditor` and `CodeEditor` are lazy chunks behind
  `React.lazy` + `Suspense` so they no longer contribute to the
  initial JS budget.
- `WelcomeOverlay` now respects `?skip-welcome=1` from the first render
  via a synchronous `useState` guard.

## v0.77.1

### Added

- Follow-up end-to-end coverage for Hito 5 workflows.

### Changed

- None.

### Fixed

- Hardened Hito 5 browser test behavior.

### Removed

- None.

## v0.77.0

### Added

- Code-aware AI workflow capabilities and project context integration.

### Changed

- AI context expanded beyond scene JSON to project artifacts.

### Fixed

- None.

### Removed

- None.

## v0.76.0

### Added

- Scene Component authoring data-layer support.

### Changed

- Editor workflows aligned with Bevy Scene Component concepts.

### Fixed

- None.

### Removed

- None.

## v0.75.0

### Added

- Initial Scene Component authoring data and command foundations.

### Changed

- Scene authoring now models Scene Component relationships explicitly.

### Fixed

- None.

### Removed

- None.

## Earlier releases

### Added

- Hitos 0–4 delivered scene editing, schemas, OPFS persistence, Scene Assets and instances, BSN workflows, level-design tools, Logic Bricks, a Rust source editor, asset pipeline, play mode, and data hot reload.

### Changed

- The architecture migrated from legacy entity templates to BSN-aligned reusable Scene Assets.

### Fixed

- Release-specific fixes are recorded in [docs/ROADMAP.md](docs/ROADMAP.md) and the Git history.

### Removed

- The legacy `EntityTemplate` model was removed in v0.20.0.
