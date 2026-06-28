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

### Active Work

| Change | Branch | Status |
|--------|--------|--------|
| — | — | No active work |

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
Capability                    v0.12  v0.13  v0.14  v0.15
───────────────────────────────────────────────────────────
AI-Assisted Editing                ✅    ✅    ✅    ✅
LLM Proxy (Ollama/OpenAI)           ✅    ✅    ✅    ✅
AI Proposal UI Panel                ✅    ✅    ✅    ✅
Apply / Discard Commands            ✅    ✅    ✅    ✅
E2E Tests (mock proxy)             ✅    ✅    ✅    ✅
Code Export (Rust codegen)                      ✅    ✅
Multi-scene Projects                             ✅    ✅
Scene Tabs + Dirty State                                   ✅
Pixelmatch Screenshot Diff                                   ✅
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  React Frontend (TypeScript)                                 │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────────────────┐   │
│  │TopBar      │ │HierarchyPanel│ │InspectorPanel        │   │
│  │TopBar      │ │Entity tree   │ │ComponentEditor       │   │
│  └─────────────┘ └─────────────┘ └──────────────────────┘   │
│  ┌─────────────┐ ┌──────────────────────────────────────┐  │
│  │engine-bridge│ │hooks: useSceneState, useLogState,     │  │
│  │(WASM bridge)│ │useKeyboardShortcuts, useAIAssistant   │  │
│  └─────────────┘ └──────────────────────────────────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                           │ wasm-bindgen
┌─────────────────────────▼──────────────────────────────────┐
│  Editor Core (Rust / WASM)                                   │
│  ┌──────────────┐ ┌───────────────┐ ┌────────────────────┐  │
│  │ document.rs  │ │ command.rs    │ │ processor.rs        │  │
│  │ SceneDocument│ │ 9 Command     │ │ apply() / inverse() │  │
│  │ Entity      │ │ variants      │ │ cycle detection     │  │
│  │ StableId    │ │ Batch         │ │ field path parser   │  │
│  └──────────────┘ └───────────────┘ └────────────────────┘  │
│  ┌──────────────┐ ┌───────────────┐ ┌────────────────────┐  │
│  │ operation_   │ │ template.rs   │ │ schema.rs           │  │
│  │ log.rs      │ │ EntityTemplate│ │ ComponentSchema     │  │
│  │ undo/redo   │ │ instantiate() │ │ combined_registry() │  │
│  │ LogEntry    │ │ mint_stable_id│ │ validate()          │  │
│  └──────────────┘ └───────────────┘ └────────────────────┘  │
│  ┌──────────────┐ ┌───────────────┐ ┌────────────────────┐  │
│  │ persistence  │ │ dynamic_scene │ │ bevy_anchor.rs     │  │
│  │ OPFS JS     │ │ .rs           │ │ anchor_str_to_bevy_ │  │
│  │ bridge      │ │ DynamicScene   │ │ anchor() helper    │  │
│  └──────────────┘ └───────────────┘ └────────────────────┘  │
│                              ┌──────────────┐                │
│                              │ lib.rs       │                │
│                              │ dispatch_cmd │                │
│                              │ mark_dirty() │                │
│                              └──────────────┘                │
└──────────────────────────────────────────────────────────────┘
                           │
                           │ LinearBus (64KiB shared memory)
┌─────────────────────────▼──────────────────────────────────┐
│  Bevy Preview World (Rust / Native)                          │
│  ┌──────────────┐ ┌───────────────┐ ┌────────────────────┐  │
│  │ SceneEntity  │ │ Sprite2D      │ │ rebuild_preview_    │  │
│  │ marker       │ │ Anchor (0.19) │ │ world system        │  │
│  └──────────────┘ └───────────────┘ └────────────────────┘  │
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
| — | — | — |

### Medium Priority

| Item | Description |
|------|-------------|
| — | — |

### Hito 1 Pending

| Item | Description |
|------|-------------|
| ~~AI-assisted editing~~ | ✅ Completed in v0.12.0 |
| ~~Code export~~ | ✅ Completed in v0.14.0 |
| ~~Multi-scene projects~~ | ✅ Completed in v0.15.0 |
| ~~pixelmatch quantitative diff~~ | ✅ Completed |
| **Collaborative editing** | CRDT-based multi-user editing |
| **Plugin system** | Extensible component schema registry with runtime loading |

---

## Technical Decisions (ADRs)

| ADR | Decision | Status |
|-----|----------|--------|
| ADR-0001 | JSON as source of truth (not RON) | ✅ |
| ADR-0002 | Single Bevy instance renders canvas | ✅ |
| ADR-0003 | `serde_json::Value` for forward-compat ComponentInstance values | ✅ |
| ADR-0004 | Bevy native Anchor Component for sprite anchoring (not custom) | ✅ |
| ADR-0005 | Scene Asset as BSN-aligned reusable scene model | ✅ |

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

Key terms: **SceneDocument**, **StableId**, **Entity**, **Entity Template**, **Component Schema Registry**, **Component Instance**, **Operation Log**, **LinearBus**.

---

*Last updated: v0.18.0 — 2026-06-28*
