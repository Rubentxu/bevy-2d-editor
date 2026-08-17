# Changelog

All notable changes to Bevy 2D Editor are documented here. The project follows semantic version tags; detailed milestone history is available in [docs/ROADMAP.md](docs/ROADMAP.md).

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
