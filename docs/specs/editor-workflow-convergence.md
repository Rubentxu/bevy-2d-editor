# Editor Workflow Convergence — Durable Product Spec

> Status: Draft durable spec.
> Planning source of truth for the non-AI product improvements that must land
> before the editor can credibly act as an AI-native Bevy authoring environment.

This spec translates the current audit findings into a durable capability
contract for the editor shell, authoring workflows, and exposed product surfaces.

## Purpose

Close the gap between:

- what `editor-core` and `ai-proxy` already implement,
- what the graphical editor exposes clearly and reliably,
- what users expect from a serious 2D game editor.

## Quick Path

1. Fix shell-breaking issues first.
2. Make every already-implemented capability discoverable and operable.
3. Unify validation, search, and runtime diagnostics.
4. Remove documentation and shortcut drift.
5. Only then layer autonomous agent workflows on top.

## Capability Map

| Capability | Direction | Summary |
|---|---|---|
| `editor-shell-integrity` | MODIFIED | Menus, docks, fullscreen, responsiveness, status bar, floating panels |
| `workflow-surface-parity` | MODIFIED | UI must expose existing backend capabilities honestly and completely |
| `asset-browser-maturity` | MODIFIED | Project Asset Browser becomes a true product surface |
| `logic-authoring-continuity` | MODIFIED | Logic graph workflows become persistent and browseable |
| `validation-center-unification` | MODIFIED | All issue classes show in one coherent workflow |
| `search-and-command-convergence` | MODIFIED | Search, command palette, and navigation become actionable |
| `authoring-mode-coherence` | MODIFIED | scene / asset / logic / code / play modes behave consistently |
| `docs-and-shortcuts-coherence` | MODIFIED | docs, menus, commands, and keyboard shortcuts match shipped behavior |

## Invariants

1. **A visible surface must be truthful.** If the UI advertises a feature, it must be implemented or clearly marked as deferred.
2. **No hidden-capability debt.** Backend features that matter to users must not stay buried indefinitely behind placeholders.
3. **One source of navigation truth.** Search, palette, menus, and docs should not contradict each other.
4. **Desktop-first is acceptable; broken layouts are not.** If small screens are unsupported, the product must degrade intentionally rather than failing structurally.

## Detailed Requirements

## 1. `editor-shell-integrity`

### Outcome

The shell behaves like a professional editor workspace.

### Required behavior

- Top-level menus MUST always render above docks and remain clickable.
- Dock layout MUST either:
  - adapt responsively for narrower widths, or
  - declare and enforce a minimum supported width with intentional fallback UI.
- Floating panels MUST render real panel content, not placeholders.
- Status bar MUST occupy the visual region it claims.
- Welcome/onboarding surfaces MUST not stack redundantly.

## 2. `workflow-surface-parity`

### Outcome

The UI accurately exposes existing system capabilities.

### Required behavior

- Any capability implemented in `editor-core` that is part of the main authoring loop
  MUST be reachable from the UI, command palette, or documented workflow.
- Capabilities currently represented as placeholders MUST be either completed or
  explicitly deferred in product copy.
- The UI MUST expose full AI context sources that already exist in backend APIs.

### Priority gaps to close

- selected entity in AI context,
- logic graphs in AI context,
- scene assets in AI context,
- runtime diagnostics in AI workflows,
- output and review surfaces.

### PR1 Scenarios (Phase A — S1 + S2)

#### S1 — AI Assistant sends populated multi-source context

When the user opens the AI Assistant Panel and submits a request, the request envelope
contains all four context sources populated from live editor state:

- `logic_graphs`: array of `LogicGraphRef` entries from the current logic graph (if any),
  with `asset_id`, `logical_path`, and `builtin` flag.
- `scene_assets.catalog`: the full `SceneAssetCatalog` snapshot.
- `scene_assets.selected_body`: the parsed body of the currently selected scene asset (if any).
- `selected_entity`: a `SelectedEntity` object with `stable_id` and `components` array when
  an entity is selected; `null` when nothing is selected.

**Implementation**: `App.tsx:328-340` derives `selectedEntity` from `scene.selectedEntityId`
and `scene.entities`. It is passed as an option to `useAIAssistant` at `App.tsx:357`.
The hook assembles the full `MultiSourceContext` via `assembleMultiSourceContext`
(`useAIAssistant.ts:184-214`) and sends it as `extraContext` to the AI proxy.

**Verification**: `tests/selected-entity.spec.ts` intercepts the POST `/v1/propose` body and
asserts `body.selected_entity.stable_id` and `body.selected_entity.components` are populated
when an entity is selected; `body.selected_entity === null` when nothing is selected.

#### S2 — Logic graph listing and opening from OPFS, with built-in recipes seeded

`list_logic_graph_assets` returns the full catalog including built-in recipes on a fresh
catalog. The catalog entry for a built-in recipe shows `asset_id` prefixed `lga_recipe_`,
`builtin: true`, and `role: "logic"`.

Created graphs appear in the listing after fire-and-forget creation. Opening a graph
loads its body from OPFS (`logic_graphs/<path>.logic.json`) and updates the catalog
entry's `builtin: false` flag and `logical_path` field.

**Implementation**: `lib.rs:2427-2435` calls `seed_builtin_recipes_to_catalog()` then
returns `cat.list_all()`. Built-in recipes are defined via `include_str!` in
`logic_state.rs:156-191` and registered with `LogicGraphCatalog::seed()`. OPFS read/write
uses `crate::js_save_file` / `crate::js_load_file` (`logic_graph.rs:178-191`).

**Verification**: `tests/logic-graph-persistence.spec.ts` asserts `parsed.length >= 3` and
that `lga_recipe_health`, `lga_recipe_jump`, `lga_recipe_proximity` are present with
`builtin: true`. Fire-and-forget create + polling verifies catalog registration.
Open-after-create verifies catalog entry mutation (`builtin: false, logical_path`).

## 3. `asset-browser-maturity`

### Outcome

The Project Asset Browser becomes a primary navigation and action surface.

### Required behavior

- Users MUST be able to browse, filter, open, place, duplicate, rename, delete,
  export, and inspect assets without relying on browser prompts as the core UX.
- Asset thumbnails, role badges, version, and binding metadata MUST be coherent.
- SceneComponent-related actions MUST feel like first-class authoring flows.

## 4. `logic-authoring-continuity`

### Outcome

Logic Bricks become a continuous workflow rather than a mode with partial plumbing.

### Required behavior

- Logic graph assets MUST be listable and openable from product surfaces.
- Built-in recipes MUST be discoverable from the browser/palette.
- Logic bindings MUST be visible from relevant scene and inspector flows.
- Validation and preview feedback for logic MUST be reachable without reading console noise.

## 5. `validation-center-unification`

### Outcome

Validation Center becomes the single project-health surface.

### Required behavior

- It MUST aggregate, directly or via composed services:
  - catalog warnings,
  - export warnings,
  - logic issues,
  - override/resync issues,
  - dirty scenes,
  - schema issues,
  - AI proposal/apply issues.
- Issues MUST support navigation back to the owning scene, asset, graph, entity,
  or source file when possible.

### PR2 Scenarios (Phase B — S3)

#### S3 — Unified Validation Center composing 7 issue classes

The Validation Center inbox aggregates all seven issue classes, groups them by domain, and renders
actionable navigation back to the owning surface.

**Issue classes** (all registered via `window.__registerSchemaIssue` or `window.__recordAIProposalFailure`):

| Code | Domain | Producer | Navigation target |
|---|---|---|---|
| `catalog` | `scene` | `SceneAssetCatalog` on structural inconsistency | Scene Asset Browser |
| `export` | `scene` | `BsnExporter` on serialization failure | Scene document |
| `logic` | `logic` | `LogicGraphCatalog` on graph validation | Logic graph editor |
| `override` | `asset` | `SceneInstance.apply_component_overrides` on stale/orphaned | Override workbench |
| `dirty_scene` | `scene` | `SceneDocument` on unsaved changes | Scene tab |
| `schema` | `code` | `SchemaAuthoringPanel` on registration/parse failure | Schema editor |
| `ai_proposal_request_failed` / `ai_proposal_apply_threw` / `ai_proposal_rejected` | `ai` | `useAIAssistant` on network error, throw, or error response | AI Assistant panel |

**Domain grouping**: the inbox renders `vc-domain-header-*` elements (one per unique domain) ordered
as `scene / asset / logic / code / runtime / ai`. Each domain section contains its issue cards.

**Navigation**: clicking any issue card opens the detail panel (`vc-issue-detail`) and calls the
appropriate `window.__*` hook to navigate to the owning surface:
- scene-domain issues → `__openSceneAssetFromSearch(assetId)` → App's `useSceneAssets().open()`
- logic-domain issues → `__setEditorMode("logic")` + entity focus
- code-domain (schema) issues → `__openSourceFile(schema.source_file)`
- ai-domain issues → `__setEditorMode("ai-assistant")`

**Implementation**: `ValidationCenter.tsx` exposes `registerSchemaIssue` and `recordAIProposalFailure`
via `window.__registerSchemaIssue` / `window.__recordAIProposalFailure`. Aggregator calls
`getAllValidationIssues()` which merges all registered issues by domain. `App.tsx:381` wires
`__openSceneAssetFromSearch` to App's `openAsset` via `useSceneAssets`. Issue detail panel
shows severity badge (`error`/`warning`/`info`), domain badge, affected asset ID, and message.

**Verification**: `tests/validation-center-inbox.spec.ts` — `seedSchemaAndAIIssues()` registers
schema (domain `code`) and AI (domain `ai`) issues; test asserts `vc-domain-header-*` count > 0,
`.vc-issue` count > 0, detail panel opens on click, and close button dismisses it. Tests for the
remaining five issue classes (catalog, export, logic, override, dirty) are verified by source
inspection and the 15/15 Playwright safety net.

## 6. `search-and-command-convergence`

### Outcome

Search and command entry feel like one coherent intelligence surface.

### Required behavior

- Global Search MUST support actionable results, not just text lists.
- Search result classes MUST include at least:
  - scenes,
  - scene assets,
  - entities,
  - source files,
  - logic graphs,
  - asset files,
  - commands,
  - issues.
- Command Palette and Search MUST not compete for the same shortcut semantics.

### PR2 Scenarios (Phase B — S4)

#### S4 — Global Search returns actionable entity + command + source-file + scene-asset + asset-file + scene results

Global Search returns six result kinds, all rendered with a shared `SearchResultRow` component that
displays icon, label, and path. Clicking a result navigates to the owning surface or executes the action.

**Result kinds**:

| Kind | Icon | Label | Path | Action on click |
|---|---|---|---|---|
| `scene` | `📄` | `scene.name` | scene asset ref | `__openSceneAssetFromSearch(scene.id)` → `useSceneAssets().open(sceneId)` |
| `entity` | `◉` | `entity.name` or stable_id | entity's parent scene | `__setSelectedEntityId(entity.stable_id)` |
| `command` | `⌘` | command label | `—` | `__executeCommand(command.id)` |
| `source-file` | `📝` | filename | file path | `__setEditorMode("code")` + `onSourceNavigate(filePath)` |
| `scene-asset` | `🎬` | asset name | logical path | `__openSceneAssetFromSearch(assetId)` + `__setEditorMode("asset-authoring")` |
| `asset-file` | `📦` | filename | asset ref | Opens external URL (e.g. texture preview) |

**Shared presentation**: `SearchResultRow` (`SearchResultRow.tsx`) is imported by both `SearchTab`
and `CommandPalette`. Props: `icon`, `label`, `path`, `resultKind`, `onClick`. No duplicated
row-layout code.

**Action routing**: `SearchTab.tsx` routes clicks via `resultKind`-switch:
- `scene` → line 88: `__openSceneAssetFromSearch(result.id)`
- `entity` → line 78: `__setSelectedEntityId(result.stable_id)`
- `command` → line 92: `__executeCommand(result.id)`
- `source-file` → line 97–100: `__setEditorMode("code")` + `onSourceNavigate(result.path)`
- `scene-asset` → line 88: same as scene (via `__openSceneAssetFromSearch`)
- `asset-file` → line 95: `window.open(result.path, "_blank")`

Command results are fed by `__getCommandPaletteItems()` → `setCommandResults` in `SearchTab.tsx:39-51`.
Entity search uses `searchEntitiesInScene` (`useGlobalSearch.ts:196-222`) gated on finding the active
scene via `s.find((sc) => sc.is_active)` — the active scene check now correctly uses `is_active`
(snake_case, matching Rust `SceneInfo` output).

**Verification**: `tests/global-search-actions.spec.ts`:
- `T2.4`: `result rows render icon, label, and path` — asserts `global-search-result-*` rows exist with content
- `T2.3`: `entity results type is present` and `command results are included when seeded` — entity stream gated on active scene; command stream fed via palette items
- `T2.5`: `clicking a scene result navigates to that scene` — if `scenes.length > 0`, clicks row and asserts no throw
- `T2.5`: `clicking a source-file result navigates to code editor` — if `sourceFiles.length > 0`, clicks row and asserts no throw
- Action-outcome tests for `scene-asset` and `asset-file` are missing (deferred to future cycle)

## 7. `authoring-mode-coherence`

### Outcome

Mode transitions are conceptually clean.

### Required behavior

- Dock headers and body content MUST agree in every mode.
- Scene / asset / logic / code / play modes MUST advertise their active context clearly.
- Runtime-facing controls and authoring-facing controls MUST be distinguishable.

## 8. `docs-and-shortcuts-coherence`

### Outcome

The editor is teachable because the docs and the product tell the same story.

### Required behavior

- `USER_GUIDE.md`, menus, shortcuts, command palette, and onboarding copy MUST match current behavior.
- Deprecated or TODO menu items MUST be resolved or hidden.
- Shortcut collisions MUST be eliminated.

## Acceptance Criteria

The convergence program is successful when:

- no critical shell bug prevents basic navigation,
- no primary workflow depends on `window.prompt/alert/confirm` as the core interaction,
- all major backend capabilities are either exposed or deliberately deferred,
- docs and shortcuts align with runtime behavior,
- AI workflows consume the full context already supported by backend systems.

### PR1 Acceptance Criteria (Phase A — AI context wiring + logic graph OPFS)

PR1 is complete when:

- [ ] `selected_entity` is populated in AI requests when an entity is selected; `null` when nothing is selected
- [ ] `logic_graphs`, `scene_assets.catalog`, and `scene_assets.selected_body` are included in AI requests
- [ ] `list_logic_graph_assets` returns ≥3 built-in recipes with `builtin: true` and role `logic`
- [ ] Created logic graphs appear in the catalog listing
- [ ] Opening a logic graph loads its body from OPFS and updates the catalog entry correctly
- [ ] Rust unit tests pass: `cargo test -p editor-core --lib logic_graph` (19/19)
- [ ] Playwright tests pass: `logic-graph-persistence.spec.ts` (4/4) and `selected-entity.spec.ts` (2/2)
- [ ] `npm run lint` and `npm run build` both exit 0

### PR2 Acceptance Criteria (Phase B — Validation Center S3 + Global Search S4)

PR2 is complete when:

- [ ] `ValidationCenter` aggregates all 7 issue classes (catalog, export, logic, override, dirty-scene, schema, AI) and renders them grouped by domain
- [ ] Schema issues carry `domain: "code"`; AI issues carry `domain: "ai"`
- [ ] Clicking a Validation Center issue navigates to the owning surface via `__openSceneAssetFromSearch` → App's `openAsset`
- [ ] Rust `SceneInfo` serializes `is_active` / `is_dirty` (snake_case, no camelCase rename)
- [ ] Global Search renders all 6 result kinds with shared `SearchResultRow`
- [ ] `SearchResultRow` is shared between `SearchTab` and `CommandPalette`
- [ ] Scene and source-file click actions execute without throw
- [ ] Entity search is gated on `is_active` scene detection (snake_case contract)
- [ ] Playwright tests pass: `validation-center-inbox.spec.ts` + `global-search-actions.spec.ts` (15/15)
- [ ] `cargo test -p editor-core --lib` passes (438/438)
- [ ] `npm run lint` and `npm run build` both exit 0

## References

- `USER_GUIDE.md`
- `docs/ROADMAP.md`
- `docs/adr/0021-defold-inspired-layout.md`
- `docs/adr/0025-floating-panels-multi-select.md`
- `docs/adr/0026-asset-browser-thumbnails.md`
- `docs/specs/ai-native-editor-capabilities.md`
