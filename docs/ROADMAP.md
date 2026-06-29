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
| project-asset-browser-and-scene-asset-authoring (PR2 slice) | v0.22.0 | ✅ (partial) | AssetCommand surface + WASM bridge. Separate `AssetCommand` enum (AddEntity, RemoveEntity, RenameEntity, SetComponentValue) per ADR-0007, `AssetOperationLog` (undo/redo) scoped to scene assets, `AssetProcessor` with `set_field_path_vec` helper, thread-local `ASSET_OPERATION_LOG`, WASM CRUD bridge (dispatch_asset_command, create/rename/duplicate/delete/list_scene_assets, open/close/get_asset_document/get_scene_asset_catalog, save_scene_asset body-first/catalog-second). 16/16 PR2 tasks complete; 23/23 spec scenarios covered (S10, S13, S14, S15 PR2 + PR1 regression). PR #18 merged; tag `v0.22.0`. PR3 (frontend Project Asset Browser + Authoring Mode) remains pending for full Capability 1 closure. |

### Active Work

| Change | Branch | Status |
|--------|--------|--------|
| `project-asset-browser-and-scene-asset-authoring` PR3 (frontend Project Asset Browser + Asset Authoring Mode + dirty-guard + Playwright E2E) | next | Pending — backend surface (PR1 + PR2) complete, ready for frontend integration |

---

## Hito 2: Authoring Workflows & 2D Level Production

**Goal**: Turn the post-BSN architecture into practical editor workflows: Project asset management, Scene Asset authoring, Scene Instance placement, override/resync UX, validation, 2D level design tools, and runtime preview inspection.

**Normative references**:

- [ADR-0006: Authoring-First Roadmap after the BSN Migration](./adr/0006-authoring-first-roadmap-after-bsn-migration.md)
- [Post-BSN Authoring Roadmap Specification](./specs/post-bsn-authoring-roadmap.md)

### Planned Sequence

| Order | Change | Status | Why |
|-------|--------|--------|-----|
| 1 | `project-asset-browser-and-scene-asset-authoring` (PR1 ✅ v0.21.0; PR2 ✅ v0.22.0; PR3 pending) | In Progress | Exposes existing `SceneAssetDocument` + `SceneAssetCatalog` as usable Project workflows |
| 2 | `scene-instance-placement` | Planned | Lets users place Scene Assets in SceneDocuments without deep cloning |
| 3 | `override-resync-workbench` | Planned | Makes `OverridePatch` status and resync reports visible/actionable |
| 4 | `validation-center` | Planned | Centralizes broken refs, schema issues, export warnings, override conflicts, dirty scenes, and invalid AI proposals |
| 5 | `level-design-layers-research` | Planned research | Defines tile/object/IntGrid/auto-layer semantics before committing to a tilemap model |
| 6 | `runtime-preview-inspector` | Planned | Shows runtime preview provenance, metrics, and editor-to-preview mapping |

### Research Gates

| Capability | Required research before `sddk-propose` |
|------------|------------------------------------------|
| Project Asset Browser + Scene Asset Authoring | Unity Prefab Mode, Godot PackedScene/inherited scenes, Defold Collections/factories, Bevy BSN asset roadmap, OPFS Project layout |
| Scene Instance Placement | Unity prefab instance display, Defold collectionfactory ID maps, Godot missing base-scene behavior |
| Override / Resync Workbench | Unity Prefab Overrides, Blender Library Overrides, Godot inherited-scene constraints |
| Validation Center | Unity console/validation patterns, Defold resource profiler, Bevy diagnostics |
| 2D Level Design Tools | Tiled terrain brush/automapping, LDtk IntGrid/Auto Layers/Entities, Bevy tilemap ecosystem, Aseprite metadata |
| Runtime Preview Inspector | Defold profiler, Godot remote SceneTree, Bevy diagnostics/remote tooling, Chronos future debugging |

### Deferred Until After Hito 2

| Candidate | Revisit when |
|-----------|--------------|
| Collaborative editing | Project asset identity, validation, and save/load semantics are stable |
| Plugin system | Schema packs and validation extension points have at least one built-in example |
| Physical `.bsn` import/export | Bevy ships stable loader/write-back APIs |
| Visual scripting/state machines | Scene Asset workflows and runtime preview inspection are mature |

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
Capability                    v0.12  v0.13  v0.14  v0.15  v0.16  v0.17  v0.18  v0.19  v0.20  v0.21  v0.22
────────────────────────────────────────────────────────────────────────────────────────────────────
AI-Assisted Editing                ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
LLM Proxy (Ollama/OpenAI)           ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
AI Proposal UI Panel                ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Apply / Discard Commands            ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
E2E Tests (mock proxy)             ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Code Export (Rust codegen)                      ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Multi-scene Projects                             ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Scene Tabs + Dirty State                                   ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
Pixelmatch Screenshot Diff                                   ✅    ✅    ✅    ✅    ✅    ✅    ✅    ✅
BSN Scene Asset Model                                               ✅    ✅    ✅    ✅    ✅    ✅
BSN IR + bsn! Codegen                                                    ✅    ✅    ✅    ✅    ✅    ✅
Scene Asset Catalog                                                          ✅    ✅    ✅    ✅    ✅
Scene Instance Overrides + Resync                                                ✅    ✅    ✅    ✅
BSN Migration Complete (template.rs deleted)                                         ✅    ✅    ✅
Scene Asset Persistence + Catalog Holder (PR1 slice)                                       ✅    ✅
AssetCommand Surface + WASM Bridge (PR2 slice)                                                ✅
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
| **`project-asset-browser-and-scene-asset-authoring` PR2 (AssetCommand surface + WASM bridge)** | ✅ Completed in v0.22.0 | ~1871 (code+tests) | A-full |
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

Key terms: **SceneDocument**, **StableId**, **Entity**, **Scene Asset**, **Scene Instance**, **Scene Asset Catalog**, **Project Asset Browser**, **Scene Asset Authoring Mode**, **Override / Resync Workbench**, **Validation Center**, **Runtime Preview Inspector**, **Component Schema Registry**, **Component Instance**, **Operation Log**, **BsnIr**, **OverridePatch**.

---

*Last updated: v0.22.0 — 2026-06-29 (PR2 slice of `project-asset-browser-and-scene-asset-authoring` landed; PR3 frontend remains pending for full Hito 2 Capability 1 closure)*
