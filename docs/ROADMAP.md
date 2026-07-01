# Bevy 2D Editor — Project Roadmap

## Hito 0: Scene Editor Foundation

**Goal**: Browser-based 2D scene editor with entity hierarchy, inspector, undo/redo, and Bevy preview rendering.

### Completed Milestones

| Milestone | Version | Status | Key Deliverables |
|-----------|---------|--------|------------------|
| scene-document | v0.1.0 | ✅ | `SceneDocument` JSON, `StableId`, `Entity`, `ComponentInstance`, `ComponentSchemaRegistry` |
| command-system | v0.1.0 | ✅ | Typed `Command` enum, `processor.rs` with reversibility, `dispatch_command` WASM entry |
| opfs-persistence | v0.2.0 | ✅ | `save_scene`/`load_scene` to OPFS, `project.json` atomic load |
| schema-registry-persistence | v0.3.0 | ✅ | Mutable user schemas, `register_schema_from_json`, builtin + user combined registry |
| entity-template-persistence + **instantiate** | v0.4.0 | ✅ | `EntityTemplate` tree, `instantiate()` with fresh ID minting, OPFS save/load, inverse = Batch of DeleteEntity |
| ui-panels | v0.5.0 | ✅ | `HierarchyPanel` + `InspectorPanel` React components, `useSceneState`, `useLogState` hooks |
| dynamic-scene-export | v0.6.0 | ✅ | `export_dynamic_scene_wasm` → Bevy `DynamicScene`, component mapping from editor schemas to Bevy native components |
| preview-anchor-sync | v0.7.0 | ✅ | Preview world honors `editor.Sprite2D.values.anchor` via Bevy 0.19 Anchor Component |
| keyboard-shortcuts | v0.8.0 | ✅ | `useKeyboardShortcuts` hook, Ctrl+Z/Y + Cmd+Z/Y, input guard, Playwright screenshot diff E2E |
| delete-key | v0.9.0 | ✅ | Delete/Backspace removes selected entity, input guard, `handleDeleteEntity` in App, 3 Playwright E2E tests |
| entity-rename-inline | v0.10.0 | ✅ | Double-click name in hierarchy → inline input, Enter/blur commits via RenameEntity, Escape cancels, empty/unchanged no-op |
| entity-drag-drop | v0.11.0 | ✅ | HTML5 DnD reparenting in HierarchyPanel, `ReparentEntity` via `window.dispatch_command`, root-drop zone, self-drop guard, cycle safety via backend |
---

## Hito 1: AI-Assisted Editing

**Goal**: LLM-powered scene editing via a Rust HTTP proxy that routes to OpenAI, with a React UI panel for proposing, reviewing, and dispatching scene-edit commands.

### Completed Milestones

| Milestone | Version | Status | Key Deliverables |
|-----------|---------|--------|------------------|
| ai-assisted-editing | v0.12.0 | ✅ | Rust axum proxy (Ollama + OpenAI), `crates/ai-proxy`, WASM bridge `get_combined_schemas_json`, `AIAssistantPanel` + `ProposalCard` React components, `useAIAssistant` hook, mock LLM proxy fixture, 6 Playwright E2E tests |
| code-export | v0.14.0 | ✅ | `crates/editor-core/src/code_export.rs` (590 LOC): pure-string codegen, `rust_type_for_field`, `emit_header/user_structs/plugin_shell/spawn_scene`, snapshot tests |
| multi-scene | v0.15.0 | ✅ | `SceneRegistry`, scene switching with dirty-state tracking, `SceneTabs` UI, `UnsavedChangesDialog`, E2E tests, WASM bindings |
| pixelmatch quantitative diff | — | ✅ | `frontend/tests/pixelmatchHelper.ts`, upgraded screenshot tests to per-pixel quantitative output with explicit % metrics |
| scene-asset-document (BSN spike) | v0.16.0 | ✅ | Rust types for `SceneAssetDocument`, `SceneInstance`, `BsnIr` per ADR-0005; aligned with Bevy 0.19 `bsn!` semantics; 10/10 spec scenarios covered |
| code-export-bsn | v0.17.0 | ✅ | `bsn_codegen.rs` emits `bsn!`/`bsn_list!` source from `BsnIr`; parallels existing `Commands::spawn` codegen; 7 integration tests |
| scene-asset-catalog | v0.18.0 | ✅ | `SceneAssetCatalog` metadata index: three `BTreeMap` indices, 11 public methods, `CatalogError`/`CatalogWarning`, `mint_asset_id`; 12 integration tests; wasm32 build green |
| scene-instance-overrides | v0.19.0 | ✅ | `scene_instance_overrides.rs`: non-destructive override lifecycle + asset-version resync; 7 public functions (`effective_values`, `resync`, `mint_id_map`, `reconcile_id_map`, `validate_overrides`, `classify_overrides`, `try_rebind`); field-path segment-0 = full `type_id`; 11 integration tests; `StableId` gets `Ord` derive for `BTreeSet` usage |
| remove-template-rs | v0.20.0 | ✅ | Deletion of legacy `EntityTemplate` per ADR-0005 §Implementation Direction step 3. `crates/editor-core/src/template.rs` (507 LOC) and all 9 callers removed; net -892 LOC; 206 existing tests still compile on wasm. Completes BSN migration roadmap (Fases 0–4). |
| project-asset-browser-and-scene-asset-authoring (PR1 slice) | v0.21.0 | ✅ (partial) | Scene Asset persistence + catalog holder foundation. Path-based OPFS layout (`assets/<logical_path>.asset.json`), `ProjectMetadata.scene_assets` with `#[serde(default)]`, `SCENE_ASSET_CATALOG` / `SCENE_ASSET_DOC` / `SCENE_ASSET_CATALOG_WARNINGS` thread-locals, typed `CatalogWarning` for orphaned entries (S16), 9/9 PR1 spec scenarios compliant (S4–S8, S16–S19). PR #16 (docs/plan) and PR #17 (code) merged; tag `v0.21.0`. ADR-0007/ADR-0008/ADR README and SDDK artifacts (`docs/sddk/project-asset-browser-and-scene-asset-authoring/`) added. |
| project-asset-browser-and-scene-asset-authoring (PR2 slice) | v0.22.0 | ✅ (partial) | AssetCommand surface + WASM bridge. Separate `AssetCommand` enum (AddEntity, RemoveEntity, RenameEntity, SetComponentValue) per ADR-0007, `AssetOperationLog` (undo/redo) scoped to scene assets, `AssetProcessor` with `set_field_path_vec` helper, thread-local `ASSET_OPERATION_LOG`, WASM CRUD bridge (dispatch_asset_command, create/rename/duplicate/delete/list_scene_assets, open/close/get_asset_document/get_scene_asset_catalog, save_scene_asset body-first/catalog-second). 16/16 PR2 tasks complete; 23/23 spec scenarios covered (S10, S13, S14, S15 PR2 + PR1 regression). PR #18 merged; tag `v0.22.0`.
| project-asset-browser-and-scene-asset-authoring (PR3 slice) | v0.23.0 | ✅ | Project Asset Browser + Scene Asset Authoring Mode frontend. React components (ProjectAssetBrowser, AssetAuthoringView, AssetUnsavedChangesDialog), hooks (useSceneAssets), services (scene-assets.ts), App.tsx editorMode state, TopBar mode-aware toolbar, Playwright E2E tests (14 scenarios + EC1-EC6). C-1 (engine-bridge TypeError) and C-4 (canvas unmount) fixed in correction commit f85333b. Follow-up issue #19 tracks C-NEW (SystemTime::now panic on wasm32). PR #20 merged; tag `v0.23.0`. **Capability 1 (Project Asset Browser + Scene Asset Authoring) CLOSED**. |
| scene-instance-placement (PR1 slice) | v0.24.0 | ✅ (partial) | Storage seam + cache + gate. `SceneDocument.instances: BTreeMap<StableId, SceneInstance>` with `#[serde(default)]`, `ASSET_BODY_CACHE` skeleton (resolve, warm, invalidate, clear), `instance_projection.rs` with `root_local_ids` gate (single-root enforcement). PR #21 merged; tag `v0.24.0`. |
| scene-instance-placement (PR2 slice) | v0.25.0 | ✅ | Commands + WASM + projection. `PlaceSceneInstance`/`RemoveSceneInstance` commands, `instance_projection.rs` with `place_instance`/`remove_instance`/`root_local_ids`, WASM bridge (`dispatch_scene_instance_command`, `place_scene_instance`, `remove_scene_instance`), warm_asset_body_cache integration. PR #22 merged; tag `v0.25.0`. |
| scene-instance-placement (PR3 slice) | v0.26.0 | ✅ | Frontend + E2E. HierarchyPanel/InspectorPanel/ProjectAssetBrowser UI updates, `useSceneAssets` hook, `scene-assets.ts` service, `engine-bridge` methods, 14 Playwright E2E tests (13 blocked by OPFS headless, S21 terminology passed). PR #23 merged; tag `v0.26.0`. **Hito 2 Order 2 CLOSED**. |
| override-resync-workbench | v0.27.0 | ✅ | Override status surfacing UI. 4 new WASM functions (validate_overrides_wasm, effective_values_wasm, try_rebind_wasm, get_resync_reports), RESYNC_REPORTS thread-local, HierarchyPanel colored override dot, InspectorPanel override summary + collapsible issues list. PR #24 merged; tag `v0.27.0`. **Hito 2 Order 3 CLOSED**. |
| validation-center | v0.28.0 | ✅ | Unified ValidationIssue types + get_validation_issues_wasm (catalog + export warnings), ValidationCenter panel (severity grouping, empty state, refresh), TopBar toggle button. PR #25 merged; tag `v0.28.0`. **Hito 2 Order 4 CLOSED**. |

### Active Work

| Change | Branch | Status |
|--------|--------|--------|
| — | — | — |

---

## Hito 2: Authoring Workflows & 2D Level Production

**Goal**: Turn the post-BSN architecture into practical editor workflows: Project asset management, Scene Asset authoring, Scene Instance placement, override/resync UX, validation, 2D level design tools, and runtime preview inspection.

**Normative references**:

- [ADR-0006: Authoring-First Roadmap after the BSN Migration](./adr/0006-authoring-first-roadmap-after-bsn-migration.md)
- [Post-BSN Authoring Roadmap Specification](./specs/post-bsn-authoring-roadmap.md)

### Completed Sequence

| Order | Change | Version | Status |
|-------|--------|---------|--------|
| 1 | `project-asset-browser-and-scene-asset-authoring` (PR1/2/3) | v0.21.0–v0.23.0 | ✅ DONE |
| 2 | `scene-instance-placement` (PR1/2/3) | v0.24.0–v0.26.0 | ✅ DONE |
| 3 | `override-resync-workbench` | v0.27.0 | ✅ DONE |
| 4 | `validation-center` | v0.28.0 | ✅ DONE |
| 5 | `component-override-migration` | v0.28.0 (PR #26) | ✅ DONE |
| 6 | `level-design-layers-research` | v0.28.0 (PR #27) | ✅ DONE |
| 7 | `runtime-preview-inspector` | v0.29.0 (PR #30) | ✅ DONE |
| + | `scene-instance-layer` | v0.29.0 (PR #29) | ✅ DONE |
| + | `level-scene-asset` | v0.29.0 (PR #28) | ✅ DONE |

### Planned Sequence

| Order | Change | Status | Why |
|-------|--------|--------|-----|
| 8 | `level-design-tools` | ✅ DONE (v0.34.0, PR #34) | Tile painting, IntGrid authoring, tileset CRUD |
| 9 | `auto-layer-generation` | ✅ DONE (v0.35.0, PR #36) | 3x3 pattern rule engine, RegenerateAutoLayer, AutoLayerPanel |

### Research Gates

| Capability | Required research before `sddk-propose` |
|------------|------------------------------------------|
| Project Asset Browser + Scene Asset Authoring | Unity Prefab Mode, Godot PackedScene/inherited scenes, Defold Collections/factories, Bevy BSN asset roadmap, OPFS Project layout |
| Scene Instance Placement | Unity prefab instance display, Defold collectionfactory ID maps, Godot missing base-scene behavior |
| Override / Resync Workbench | Unity Prefab Overrides, Blender Library Overrides, Godot inherited-scene constraints |
| Validation Center | Unity console/validation patterns, Defold resource profiler, Bevy diagnostics |
| 2D Level Design Tools | Tiled terrain brush/automapping, LDtk IntGrid/Auto Layers/Entities, Bevy tilemap ecosystem, Aseprite metadata |
| Runtime Preview Inspector | Defold profiler, Godot remote SceneTree, Bevy diagnostics/remote tooling, Chronos future debugging |

### Deferred Until After Hito 3

| Candidate | Revisit when |
|-----------|--------------|
| Collaborative editing | Project asset identity, validation, and save/load semantics are stable |
| Plugin system | Schema packs and validation extension points have at least one built-in example |
| Physical `.bsn` import/export | Bevy ships stable loader/write-back APIs |
| Visual scripting/state machines | ✅ Gate passed — now active as Logic Bricks (see Post-Hito 3 section + ADR-0011) |

---

## Hito 3: .bsn File Workflow & Inspector UX

**Goal**: Enable .bsn file round-trip (export + import) and improve inspector UX for override inspection and editing.

### Completed Sequence

| Order | Change | Version | Status |
|-------|--------|---------|--------|
| 1 | `bsn-file-export-research` | v0.31.0 (PR #31) | ✅ DONE |
| 2 | `bsn-file-import-research` | v0.32.0 (PR #32) | ✅ DONE |
| 3 | `level-inspector-and-override-panel` | v0.33.0 (PR #33) | ✅ DONE |
| 4 | `bsn-file-import` | v0.36.0 (PR #37) | ✅ DONE |

### Research Gates

| Capability | Required research before `sddk-propose` |
|------------|------------------------------------------|
| .bsn file import | ✅ Research done (Bevy PRs #23639/#23648 are DRAFT — implement editor-internal round-trip) |
| Level Inspector | Unity override inspector, Godot inspector plugin patterns, override panel UX research |

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

| Decision | Resolution |
|----------|------------|
| Scripting model | Logic Bricks (Sensor → Controller → Actuator), not Blueprint VM |
| Extension | Compiled `RustController` trait registry (`NodeEvaluator`); v1 = built-in only |
| Runtime | Event/change-driven dispatch scheduler in editor-core; codegen deferred |
| React Flow | View-only; WASM JSON is source of truth |
| BSN | Logic does NOT project to `.bsn`; `BsnExporter` rejects `Logic`-role assets |
| Preview state | Stateless across rebuilds (v1) |

### Current Step — Docs-First (research / spec / ADR / design)

| Item | Status | Artifact |
|------|--------|----------|
| Exploration | ✅ DONE | `sddk/logic-bricks-graph-editor/explore-report.md` |
| Proposal | ✅ DONE | `sddk/logic-bricks-graph-editor/proposal.md` |
| ADR-0011 | ✅ DONE | `docs/adr/0011-logic-bricks-compiled-rust-controllers.md` |
| Design | ✅ DONE | `sddk/logic-bricks-graph-editor/design.md` |
| Capability specs | ✅ DONE | `sddk/logic-bricks-graph-editor/spec.md` + `docs/specs/logic-bricks-graph-editor.md` |
| CONTEXT.md terms | ✅ DONE | Logic Bricks domain language added |

### Planned Implementation Sequence

| Order | Change | Why this order |
|-------|--------|----------------|
| 1 | `logic-graph-data-model` | ✅ DONE (v0.37.0, PR #38) — `LogicGraphAsset`, `LogicNode`, `LogicEdge`, `SceneAssetRole::Logic`, `LogicInstance`. Foundation: everything depends on the data shape. |
| 2 | `logic-registry-and-metadata` | `NodeEvaluator` trait + built-in registry keyed by `node_type_id` / `controller_id`, `logic.*` schemas, port specs. Needed before any node can do anything. |
| 3 | `logic-graph-authoring-ui` | React Flow view-only `LogicGraphEditor.tsx`, `EditorMode="logic"`, `LogicCommand` surface, node palette. Authoring needs data model + registry. |
| 4 | `logic-graph-validation` | Port-type compatibility, cycle/dangling-ref detection via existing `get_validation_issues_wasm`. Surfaces issues before preview. |
| 5 | `logic-preview-dispatch-scheduler` | `project_logic_instances()`, `LogicSchedule` system set, fix `spawn_preview_entity` skip-arm (lib.rs:1666), event-driven graph evaluation with a cheap Update gate. |
| 6 | `logic-bricks-2d-recipes` | Curated built-in `LogicGraphAsset` recipes (`platformer_jump`, `health_damage`, `proximity_trigger`) reused via `LogicInstance`. |
| 7 | `rustcontroller-builtins` | Compiled built-in `RustController` impls for non-graph extension points. |
| 8 | (Deferred) `logic-graph-codegen` | Optional graph → Rust source export via `code_export.rs` pattern. Not required for v1. |

### Research Gates

| Capability | Required research before implementation |
|------------|------------------------------------------|
| Logic Bricks architecture | ADR-0011 ✅ (this docs-first step) |
| Node evaluation scheduling | Bevy ECS `Changed`/`Added`/event patterns; Chronos future debugging for evaluation traces |
| React Flow integration | `@xyflow/react` controlled-component patterns; view-only enforcement |

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

| Item | Description | Blocking |
|------|-------------|----------|
| `project-asset-browser-and-scene-asset-authoring` | First Hito 2 implementation cycle. Adds Project Asset Browser and isolated Scene Asset authoring mode. | Requires Hito 2 explore to refine OPFS layout and authoring UX |
| `scene-instance-placement` | Place Scene Assets into SceneDocuments as Scene Instances, preserving `id_map` and asset provenance. | Depends on Project Asset Browser + Scene Asset Authoring |
| `override-resync-workbench` | UI for active/orphaned/stale/conflict overrides, resync report, apply/revert/reset. | Depends on Scene Instance Placement |

### Medium Priority

| Item | Description |
|------|-------------|
| Test timing race: list_schemas dropdown not updating immediately after save (deferred from component-schema-authoring cycle) | Low urgency, cosmetic |

### Hito 1 Pending

| Item | Description | Est. LOC | Path |
|------|-------------|-----------|------|
| ~~AI-assisted editing~~ | ✅ Completed in v0.12.0 | — | — |
| ~~Code export~~ | ✅ Completed in v0.14.0 | — | — |
| ~~Multi-scene projects~~ | ✅ Completed in v0.15.0 | — | — |
| ~~pixelmatch quantitative diff~~ | ✅ Completed | — | — |
| ~~BSN Scene Asset model~~ | ✅ Completed v0.16.0 | — | — |
| ~~bsn! codegen~~ | ✅ Completed v0.17.0 | — | — |
| ~~Scene Asset Catalog~~ | ✅ Completed v0.18.0 | — | — |
| ~~Scene Instance Overrides + Resync~~ | ✅ Completed v0.19.0 | — | — |
| ~~BSN Migration (template.rs deleted)~~ | ✅ Completed v0.20.0 | — | — |
| ~~`project-asset-browser-and-scene-asset-authoring` PR1 (persistence + catalog holder)~~ | ✅ Completed in v0.21.0 | ~733 (code+tests) | A-full |
| ~~`project-asset-browser-and-scene-asset-authoring` PR2 (AssetCommand surface + WASM bridge)~~ | ✅ Completed in v0.22.0 | ~1871 (code+tests) | A-full |
| ~~`project-asset-browser-and-scene-asset-authoring` PR3 (PAB + AAM frontend)~~ | ✅ Completed in v0.23.0 | ~790 (code+tests) | A-lite |
| **`scene-instance-placement` PR1 (storage seam + cache + gate)** | ✅ Completed in v0.24.0 | ~240 (backend only) | A-lite (partial) |
| **Collaborative editing** | Deferred until after Hito 2. CRDT-based multi-user editing still requires decisions: Yjs vs Automerge vs Loro, transport, awareness state, OPFS+CRDT merge strategy, conflict UX | 3000–5000 | A-full |
| **Plugin system** | Deferred until after Hito 2. WASM plugin ABI should follow schema packs + validation extension points, not precede them | 2000–3000 | A-full |

### ADR-0005 Implementation Status

| # | Item | Status |
|---|------|--------|
| 1 | Scene Asset + Instance + Catalog as first-class Project concepts | ✅ Done |
| 2 | BSN-compatible IR (BsnIr) + semantic compatibility | ✅ Done |
| 3 | Delete legacy EntityTemplate model | ✅ Done |
| 4 | `bsn!`/`bsn_list!` code generation as primary Bevy target | ✅ Done |
| 5 | DynamicScene Export as adapter (not source of truth) | ✅ Done |
| 6 | Non-destructive override validation, resync, rebind, cleanup | ✅ Done |
| 7 | `.bsn` file import/export | ⏳ Pending — Bevy loader/write-back APIs not yet stable |

---

## Technical Decisions (ADRs)

| ADR | Decision | Status |
|-----|----------|--------|
| ADR-0001 | JSON as source of truth (not RON) | ✅ |
| ADR-0002 | Single Bevy instance renders canvas | ✅ |
| ADR-0003 | `serde_json::Value` for forward-compat ComponentInstance values | ✅ |
| ADR-0004 | Bevy native Anchor Component for sprite anchoring (not custom) | ✅ |
| ADR-0005 | Scene Asset as BSN-aligned reusable scene model | ✅ |
| ADR-0006 | Authoring-first roadmap after the BSN migration | ✅ |
| ADR-0007 | Separate `AssetCommand` surface for Scene Asset Authoring (LocalId, parallel processor/log, no shared Command surface with scenes) | ✅ |
| ADR-0008 | Path-based Scene Asset OPFS layout (`assets/<logical_path>.asset.json` + catalog inside `ProjectMetadata.scene_assets` with `#[serde(default)]`, body-first/catalog-second save order) | ✅ |
| ADR-0009 | ComponentOverride as ECS/BSN-friendly replacement for OverridePatch (explicit `component_type_id`, field_path semantic) | ✅ |
| ADR-0010 | BsnExporter trait + EditorCoreBsnExporter as working impl; BevyBsnExporter placeholder for future Bevy PR #23639 swap | ✅ |
| ADR-0011 | Logic Bricks — compiled Rust controllers + dispatch scheduler (no scripting VM, no codegen in v1, BSN isolation) | ✅ |

---

## Testing

| Layer | Tool | Status |
|-------|------|--------|
| Rust unit | `cargo test` (harness bypasses libudev) | ✅ ~112 tests |
| WASM build | `wasm-pack build --target web` | ✅ |
| E2E | Playwright 1.49 (`npx playwright test`) | ✅ 27+ tests |

> **Note**: Native `cargo test` fails on systems without libudev (e.g., Fedora containers). Use the harness at `/tmp/opencode/scene-doc-verify/` or run tests via `wasm-pack` + Playwright.

---

## Glossary

See [`CONTEXT.md`](../CONTEXT.md) for authoritative domain language.

Key terms: **SceneDocument**, **StableId**, **Entity**, **Scene Asset**, **Level Scene Asset**, **Level Layer**, **Scene Instance Layer**, **Scene Instance**, **Scene Asset Catalog**, **Project Asset Browser**, **Scene Asset Authoring Mode**, **Override / Resync Workbench**, **Validation Center**, **Runtime Preview Inspector**, **Component Schema Registry**, **Component Instance**, **Component Override**, **Operation Log**, **BsnIr**, **BSN Export**.

---

*Last updated: v0.36.0 — 2026-07-01 (Hito 3 Order 4 bsn-file-import done)*
