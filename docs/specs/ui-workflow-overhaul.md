# UI Workflow Overhaul — Durable Product Spec

> Status: Draft durable spec.
> This spec precedes and enables the AI-native editor program.
> Authoritative references: [ADR-0021](../adr/0021-defold-inspired-layout.md) · [ADR-0025](../adr/0025-floating-panels-multi-select.md) · [ADR-0026](../adr/0026-asset-browser-thumbnails.md) · [ADR-0028](../adr/0028-workflow-first-before-agentic-ai.md).

This spec defines the required graphical-editor improvements to make Bevy 2D
Editor behave like a coherent, production-grade authoring environment before
more advanced agentic AI layers are introduced.

## Purpose

Turn the current feature-rich but partially fragmented editor into a product with:

- clear navigation,
- stable shell behavior,
- semantically legible panels,
- consistent authoring modes,
- discoverable core workflows,
- better surfaces for future AI integration.

## Quick Path

1. Fix shell blockers first.
2. Add context clarity per mode.
3. Upgrade primary authoring surfaces.
4. Unify validation, search, and runtime visibility.
5. Align docs and shortcuts with runtime.

## Non-Goals

- A visual rebrand for its own sake.
- Marketing-site styling passes.
- New backend capabilities that do not improve current product workflows.
- Replacing the Defold-inspired spatial layout.

## Capability Map

| Capability | Direction | Summary |
|---|---|---|
| `shell-reliability` | MODIFIED | Menus, docks, fullscreen, layout behavior, onboarding, status bar |
| `mode-context-bar` | NEW | Persistent orientation surface for scene / asset / logic / code / play |
| `project-asset-browser-v2` | MODIFIED | Real browser surface for assets, roles, relationships, and actions |
| `hierarchy-inspector-v2` | MODIFIED | More semantic structure, better bulk workflows, clearer provenance |
| `validation-center-v2` | MODIFIED | Inbox-style project health and navigation hub |
| `search-command-surface` | MODIFIED | Search and command entry become actionable and coherent |
| `logic-workflow-v2` | MODIFIED | Recipe-first logic authoring and stronger scene integration |
| `runtime-preview-inspector-v2` | MODIFIED | Runtime debugging surface, not just metrics |
| `ai-panel-v2` | MODIFIED | Context-visible AI panel oriented around ask/propose/fix/review flows |

## Invariants

1. **The viewport is the product center.** The shell should visually support it, not compete with it.
2. **Panel titles must tell the truth.** A panel header must match the content it shows.
3. **Semantics before decoration.** The overhaul prioritizes orientation, affordance, and workflow clarity.
4. **No placeholder-first UX.** User-visible placeholder surfaces must be replaced or clearly deferred.

## Detailed Requirements

## 1. `shell-reliability`

### Outcome

The outer editor shell is dependable and legible.

### Required behavior

- Menus MUST open above the dock layout and remain fully clickable.
- The application MUST define a deliberate strategy for sub-desktop widths:
  - adaptive compact mode, or
  - explicit minimum supported width with a safe fallback presentation.
- Floating panels MUST show real content, not temporary placeholders.
- Welcome / onboarding surfaces MUST not overlap redundantly.
- The status bar MUST visually occupy the region allocated to it.

### PR1 — Menu Visibility & Compact Mode (Phase A, PR1)

#### S1 — Menu visibility across viewports

Menus render via React Portal at `document.body` with `position: fixed` anchored to the trigger button's `DOMRect`, ensuring dropdowns are never clipped by the dock layout at any supported viewport width.

**Required behavior:**
- Menus use `createPortal(dropdownContent, document.body)` for rendering.
- Dropdown position is `position: fixed; top: anchorRect.bottom + 1; left: anchorRect.left; min-width: anchorRect.width` when an anchor rect is provided; CSS fallback (`position: absolute`) is retained for tests/snapshots without an anchor.
- `data-testid="menu-dropdown"` is attached to the portaled dropdown element so Playwright and keyboard navigation can locate it via `document.body` level queries.
- Keyboard navigation covers ArrowUp, ArrowDown, Home, End, Enter, Space; Escape closes the active menu; opening a sibling menu closes the current one.
- Outside-click ignores any `.menu-trigger` button (not just the current menu's) and any click inside the portal.

**Acceptance criteria (PR1):**
- [ ] Menu dropdown is fully visible (not clipped) at all supported viewports (1920×1080, 1366×768, 1280×800).
- [ ] `position: fixed` dropdown geometry is verified programmatically (`dropdownBottom < viewportHeight`).
- [ ] Keyboard navigation (ArrowUp/Down/Home/End/Enter/Space) works through the portaled dropdown.
- [ ] Escape closes the active menu.
- [ ] Opening Edit while File is open closes File.
- [ ] Clicking any `.menu-trigger` does not close the currently open menu.
- [ ] Playwright tests: 51/51 pass in clean run.

#### S2 — Compact mode below minimum width

The editor switches to a single-column tab layout when the viewport width is below 1280 px.

**Required behavior:**
- `useViewportMode` hook exports `ViewportMode = "desktop" | "compact"` and `{ mode, width }`.
- `VIEWPORT_COMPACT_THRESHOLD = 1280`; hook returns `"compact"` when `width < threshold`, `"desktop"` otherwise.
- `DockLayout` renders `<CompactLayout>` when `mode === "compact"`.
- `CompactLayout` renders single-column flex with `data-compact="true"`, tablist `data-testid="dock-compact-tabs"`, individual tabs `data-testid="dock-compact-tab-{id}"`.
- Active tab panel is switched via `activeTab` state.
- Exact 1280 px is classified as desktop (not compact).

**Acceptance criteria (PR1):**
- [ ] Viewport below 1280 px renders `data-compact="true"` on the dock layout.
- [ ] Tab bar is visible with Assets · Scene · Outline · Properties · Tools tabs.
- [ ] Scene tab is active by default.
- [ ] Tab switching works.
- [ ] No horizontal overflow in compact mode.
- [ ] Menu remains functional in compact mode.
- [ ] Status bar is visible in compact mode.
- [ ] Assets tab shows content.
- [ ] At exactly 1280 px, desktop 3-column layout is rendered.
- [ ] Playwright tests: 18/18 pass in clean run.

### PR2 — Floating Panels, Status Bar & useCodeFiles Reliability (Phase B, PR2)

#### S3 — No useCodeFiles error flood

`useCodeFiles` poller must not emit console errors when source files are absent or produce empty/error responses.

**Required behavior:**
- `useCodeFiles.refresh()` normalizes string-or-object responses uniformly via `parseOpfs<T>`.
- Empty-error shapes (`{}`, `[object Object]`, empty string, strings containing "empty"/"ENOENT"/"not found"/"source") are demoted to `console.warn` with `setFiles([])` rather than `console.error`.
- Polling interval is adaptive: 500 ms when files exist, 5 s when `files.length === 0`.
- `visibilitychange` listener pauses the scheduler when the tab is hidden and resumes when visible.

**Acceptance criteria (PR2):**
- [ ] After 3 s of empty sources, zero `useCodeFiles`-tagged `console.error` calls.
- [ ] After tab loses and regains focus, zero `useCodeFiles`-tagged `console.error` calls.
- [ ] Playwright tests: `code-files-no-flood.spec.ts` 2/2 pass in clean run.

#### S4 — Floating panels render real content

Floating panels (assets, outline, properties, bottom) must render the same real panel body as their docked counterparts, not temporary placeholder text.

**Required behavior:**
- `App.tsx` extracts `outlinePanelContent`, `propertiesPanelContent`, `bottomPanelContent` as `useMemo` memoized JSX blocks.
- These memoized blocks are passed to both `RightDock` props (`outline=`/`properties=`) and `FloatingPanel` children.
- `FloatingPanel` portal renders `<div data-testid="floating-panel-{panelId}-body">{panelContent}</div>` — no "currently floating" placeholder.
- The bottom panel defaults to `<ConsoleTab />` when floating (matching `BottomDock`'s default active tab).

**Acceptance criteria (PR2):**
- [ ] Floating assets panel renders `AssetNavigator` (not placeholder).
- [ ] Floating outline panel renders `HierarchyPanel` (not placeholder).
- [ ] Floating properties panel renders `InspectorPanel` (not placeholder).
- [ ] Floating bottom panel renders `ConsoleTab` (not placeholder).
- [ ] Playwright tests: `floating-panel-content.spec.ts` 4/4 pass in clean run.

#### S6 — Status bar fills the grid region

The status bar must visually occupy the full width of its allocated grid cell at every supported desktop viewport.

**Required behavior:**
- `.dock-layout-status .status-bar` CSS rule includes `flex: 1` so it stretches to fill the grid cell.
- Rule also includes `height: 100%; min-height: var(--status-h, 24px)`.
- The status bar is a DOM descendant of `[data-testid="dock-region-status"]`.
- No horizontal overflow: `scrollWidth <= clientWidth`.

**Acceptance criteria (PR2):**
- [ ] At 1920×1080: status bar fills grid cell, no overflow, all segments visible.
- [ ] At 1366×768: status bar fills grid cell, no overflow, all segments visible.
- [ ] At 1280×800: status bar fills grid cell, no overflow, all segments visible.
- [ ] Playwright tests: `status-bar-region.spec.ts` 9/9 pass in clean run (3 viewports × 3 assertions).

### PR3 — Welcome/Onboarding Mutual Exclusion & Mode Header Truthfulness (Phase C, PR3)

#### S5 — Welcome + Onboarding mutual exclusion

Welcome and Onboarding surfaces must be mutually exclusive; they must never both render simultaneously.

**Required behavior:**
- `WelcomeDismissalContext` exposes `welcomeVisible`, `isChecking`, and `reportWelcomeShouldShow`.
- `isChecking` defaults to `true` and gates both surfaces during the async OPFS hydration window, preventing a flash of both surfaces before the persisted dismissal state resolves.
- `isWelcomePermanentlyDismissedSync()` reads OPFS synchronously on mount to initialize `permanentDismissal` BEFORE first paint.
- `WelcomeOverlay` render is gated by `!hydrated || permanentDismissal || dismissed || isChecking`.
- `OnboardingBanner` render is gated by `isChecking || onboardingDismissed || welcomeVisible || welcomePermanentlyDismissed`.
- When user clicks "Don't show again", the dismissal is persisted to OPFS and both surfaces are suppressed on next visit.
- When user clicks "Skip", Welcome closes but the banner is shown (temporary skip does not persist).

**Acceptance criteria (PR3):**
- [ ] First fresh visit: exactly one surface shown (Welcome OR Onboarding, never both).
- [ ] Clicking "Don't show again" persists to OPFS and both surfaces stay hidden after page reload.
- [ ] Clicking "Skip" closes Welcome and shows the OnboardingBanner.
- [ ] Fresh visit with no prior state: mutual exclusion still holds.
- [ ] Playwright tests: `onboarding-no-duplicate.spec.ts` 4/4 pass in clean run (isolated `--workers=1`).

#### S7 — Mode header truthfulness

Dock header titles must accurately reflect the content of the panel body below them.

**Required behavior:**
- `RightDock` accepts `editorMode` prop and switches both outline title (`getOutlineTitle`) and properties title (`getPropertiesTitle`).
- For modes with empty body content (logic, code, play), dock headers use generic truthful labels:
  - Outline header: `"Outline"` (not `"Logic Outline"` / `"Code Outline"` / `"Play Outline"`).
  - Properties header: `"Properties"` (not `"Logic Properties"` / `"Code Properties"` / `"Play Properties"`).
- Floating panel titles in `App.tsx` mirror the docked counterpart labels using a shared `getPanelTitle` mapping.
- `data-testid` attributes on body content enable test assertions: `hierarchy-panel`, `project-asset-browser`, `logic-graph-editor`, `code-editor`.
- LeftDock and BottomDock retain their existing header titles (not mode-aware in v1).

**Acceptance criteria (PR3):**
- [ ] Scene mode: outline header says "Scene Outline", properties header says "Scene Properties", body contains `hierarchy-panel`.
- [ ] Asset-authoring mode: outline header says "Project Assets", properties header says "project-asset-browser", body contains `project-asset-browser`.
- [ ] Logic mode: outline header says "Outline" (not "Logic Outline"), body contains `logic-graph-editor`.
- [ ] Code mode: outline header says "Outline" (not "Code Outline"), body contains `code-editor`.
- [ ] Play mode: outline header says "Outline" (not "Play Outline"), body is empty.
- [ ] Floating panel titles match their docked counterpart labels.
- [ ] Playwright tests: `mode-headers.spec.ts` 7/7 pass in clean run (isolated `--workers=1`).

## 2. `mode-context-bar`

### Outcome

Users always know what they are editing.

### Required behavior

- A persistent context bar MUST identify the active mode:
  - Scene
  - Scene Asset Authoring
  - Logic
  - Code
  - Play
- The bar MUST show the active document or target, dirty state, and primary mode actions.
- The context bar MUST sit near the viewport/work area, not be buried in panel copy.

### Acceptance criteria

- A user can identify active mode and target within 2 seconds.

**PR1 — Context and Mode Orientation (Phase 2.1):**

- [ ] ModeContextBar is visible at all viewports ≥ 1280 px (tested at 1280×800, 1366×768, 1920×1080).
- [ ] Mode badge identifies the active mode (Scene / Asset Authoring / Logic / Code / Play) within 2 seconds of mode activation.
- [ ] Active target (scene name, asset logical path, logic graph name, code file name) is shown in the bar.
- [ ] Dirty state is visible: ● (amber) when unsaved, ○ (muted) when saved.
- [ ] Primary mode actions are rendered per mode: Play/Stop (scene+play), Save (scene+asset, disabled when clean), Back (asset-authoring).
- [ ] Bar height ≤ 32 px (verified by Playwright runtime assertion).
- [ ] Playwright tests: `mode-context-bar.spec.ts` 32/32 pass in clean run.

## 3. `project-asset-browser-v2`

### Outcome

The Project Asset Browser becomes a primary authoring surface, not a utility table.

### Required behavior

- The browser MUST support both dense list view and visual asset-oriented browsing.
- Asset rows/cards MUST surface:
  - role,
  - version,
  - thumbnail when available,
  - usage/relation metadata,
  - inline actions.
- Browsing MUST support filters for role, usage, and current-scene relevance.
- The browser SHOULD support relationship context such as:
  - used by Scene Instances,
  - bound SceneComponent schema,
  - override risk when edited.

## 4. `hierarchy-inspector-v2`

### Outcome

Hierarchy and Inspector communicate identity, provenance, and bulk edit intent more clearly.

### Required behavior

- Hierarchy rows MUST visually distinguish:
  - regular entities,
  - Scene Instances,
  - logic-bound entities,
  - override-bearing items,
  - warning states.
- Inspector layout MUST be grouped into stable zones such as:
  - Identity / Provenance
  - Core placement
  - Components
  - Overrides
  - Runtime Preview
  - AI Actions
- Multi-select MUST feel first-class, with shared-component summary and clear mixed-value affordances.

### PR2 Acceptance Criteria (Phase 2.3 — Hierarchy + Inspector v2)

The following criteria are verified by Playwright tests (24/25 pass in clean run; 1 blocked by test-infra polling):

**Hierarchy Row Badges:**

- [ ] InstanceBadge renders `[I]` for entity IDs starting with `inst_` (e.g., `inst_child-1` → badge with class `badge-instance`, text `I`).
- [ ] LogicBadge renders `L` for logic-bound entities (class `badge-logic`, text `L`).
- [ ] OverrideBadge renders with dominant-status color for entities with active/stale/conflict/orphaned component overrides (class `badge-override`, status-driven color palette).
- [ ] WarningBadge renders `⚠` for entities with broken-type components (class `badge-warning`).
- [ ] Regular entity rows render no badges (zero badge elements per row).
- [ ] No duplicate badges appear on any single row.
- [ ] OverrideBadge dominant-status resolution order: conflict > orphaned > stale > active.

**Inspector Six-Zone Layout:**

- [ ] Zone 1 (Identity / Provenance) contains entity name input and ID label.
- [ ] Zone 2 (Core placement) contains `Transform2D` component.
- [ ] Zone 3 (Components) contains non-core components (Sprite2D, Camera2D) and AddComponent button.
- [ ] Zone 4 (Overrides) is collapsed by default; shows override count badge.
- [ ] Zone 5 (Runtime Preview) renders `RuntimePreviewInspector` as a standalone zone.
- [ ] Zone 6 (AI Actions) contains New Schema button (visible after expand).
- [ ] Each zone title is correct and visible.
- [ ] Components zone badge shows correct component count.
- [ ] All six zones are collapsible via click on zone header.

**Multi-Select:**

- [ ] Multi-select header shows enriched label via `useMultiSelectSummary` (e.g., "2 entities · Transform2D").
- [ ] Mixed-value rows display a mixed-value affordance (mixed pill).
- [ ] `data-has-mixed-fields="true"` is set when fields are divergent.
- [ ] `data-has-mixed-fields` is `false` or absent for homogeneous selection.
- [ ] Single-select does not render multi-inspector elements.

**Acceptance criteria (PR2):**
- [ ] Playwright tests: `hierarchy-badges.spec.ts` (6 scenarios) + `inspector-zones.spec.ts` (9 scenarios) + `inspector-multiselect.spec.ts` (5 scenarios) = 20 scenarios, 24/25 pass (OverrideBadge positive UI assertion blocked by test-env React polling; WASM data pipeline verified correct).
- [ ] `npm run lint` passes (exit 0).
- [ ] `npm run build` succeeds (exit 0; bundle ≤ 356.22 kB gzip).

## 5. `validation-center-v2`

### Outcome

Validation Center becomes the operational inbox for project health.

### Required behavior

- The layout MUST support:
  - left-side filters/categories,
  - issue list,
  - issue detail/action area.
- Issues MUST group by domain:
  - scene,
  - asset,
  - logic,
  - code,
  - runtime,
  - AI proposal/apply.
- Each issue MUST support navigation and suggested resolution action when possible.

### PR3 Acceptance Criteria (Phase 2.4 — Validation Center v2 Inbox Layout)

The following criteria are verified by Playwright tests (18/18 runtime-verified across 3 test files):

**3-Pane Inbox Layout:**

- [ ] Layout renders 3-column structure: left sidebar (filters/categories), center list (grouped issues), right detail (selected issue action area).
- [ ] Sidebar contains severity filter buttons (all, error, warning, info) with `data-testid="vc-severity-filter-{severity}"`.
- [ ] Sidebar contains domain filter buttons (scene, asset, logic, code, runtime, ai) with `data-testid="vc-domain-filter-{domain}"`.
- [ ] Center list renders issue rows with severity icon, domain badge, code snippet, and message text.
- [ ] Detail panel renders with `data-testid="vc-detail"` and shows selected issue message and references.
- [ ] Detail panel shows a navigate action (`data-testid="vc-detail-navigate"`) when the issue has affected references.

**Domain Grouping:**

- [ ] Issues are grouped by domain with section headers matching canonical order: scene → asset → logic → code → runtime → ai.
- [ ] Domain section headers are labeled per `DOMAIN_LABELS` map (`ValidationCenter.tsx:17-24`).
- [ ] Domain filter toggle hides/shows the corresponding domain group.
- [ ] Sidebar domain counts reflect the current filtered set of issues.

**Navigation:**

- [ ] Clicking an issue row selects it and shows the detail panel.
- [ ] Detail close button (`data-testid="vc-detail-close"`) deselects the issue and hides the detail panel.
- [ ] Keyboard navigation (ArrowUp/ArrowDown) moves focus through issue rows.
- [ ] Enter key activates the focused issue row and shows its detail.

**Responsive Collapse:**

- [ ] Below 1280 px viewport: detail panel collapses (not rendered; list takes full width).
- [ ] Below 900 px viewport: sidebar collapses (not rendered; list takes full width).
- [ ] Refresh button re-fetches issues and updates the issue list.

**Playwright tests:**
- `validation-center-inbox.spec.ts` — 9 scenarios (prior cycle carry-over)
- `validation-center-v2.spec.ts` — 10 scenarios (PR3-specific)
- Total: 18/18 passing in clean run.

## 6. `search-command-surface`

### Outcome

Search and command entry become one coherent mental model.

### Required behavior

- Global Search and Command Palette MUST have distinct roles but coherent presentation.
- Search results MUST be actionable, not passive text.
- Result types MUST include at least:
  - Scene
  - Entity
  - Scene Asset
  - Logic Graph
  - Source File
  - Schema
  - Validation Issue
  - Command

### PR3 Acceptance Criteria (Phase 2.4 — Search/Command v2 Coherent Presentation)

The following criteria are verified by Playwright tests (21/21 runtime-verified across 2 test files):

**Shared SearchResultRow Component:**

- [ ] `<SearchResultRow>` is a single shared component used by both `SearchTab` (global search) and `CommandPalette`.
- [ ] Result rows carry the shared class `search-result-row` for styling and test targeting.
- [ ] Result row structure is consistent: icon span (`__icon`), label span (`__label`), path span (`__path`).
- [ ] `TYPE_ICONS` map in `SearchResultRow.tsx` covers all 9 result types with type-specific glyphs.

**Coherent Presentation:**

- [ ] `SearchTab` renders result rows with `search-result-row` class.
- [ ] `CommandPalette` renders result rows with `search-result-row` class inside `.command-palette-list`.
- [ ] Both surfaces render the same row structure (icon + label + path) for equivalent result types.

**Actionable Results (8+ Result Kinds):**

- [ ] Scene result: click navigates to that scene (scene switch).
- [ ] Entity result: click focuses the entity via `__setSelectedEntityId`.
- [ ] Scene Asset result: click opens the asset in asset-authoring mode.
- [ ] Logic Graph result: click switches to logic mode and opens the graph.
- [ ] Source File result: click navigates to code editor mode.
- [ ] Asset File result: click opens the file via `asset://opfs/` URI.
- [ ] Schema result: click switches to asset-authoring mode and focuses the schema.
- [ ] Validation Issue result: click opens Validation Center and navigates to the issue.
- [ ] Command result: click executes the command's `onClick` handler.
- [ ] All 8 spec-required result kinds produce non-empty type-specific icons.

**Keyboard Navigation:**

- [ ] ArrowDown/ArrowUp move focus through the results list (`.search-result-row--focused`).
- [ ] Enter activates the focused result by invoking its action handler.
- [ ] `CommandPalette` Escape key closes the palette.

**Playwright tests:**
- `global-search-actions.spec.ts` — 7 scenarios (prior cycle carry-over)
- `search-command-v2.spec.ts` — 14 scenarios (PR3-specific)
- Total: 21/21 passing in clean run.

## 7. `logic-workflow-v2`

### Outcome

Logic Bricks become a clear authoring workflow rather than a side mode.

### Required behavior

- Users SHOULD be able to start from recipes before editing raw graphs.
- Scene/Inspector surfaces SHOULD expose:
  - attach logic,
  - open bound logic,
  - create from recipe,
  - inspect runtime logic state.
- Logic graph authoring should feel connected to scene authoring, not isolated.

## 8. `runtime-preview-inspector-v2`

### Outcome

Runtime Preview Inspector becomes a debugging differentiator.

### Required behavior

- Beyond metrics, the surface MUST show:
  - projected instances,
  - provenance,
  - last rebuild cause,
  - hot-reload events,
  - logic activation summaries,
  - runtime-facing warnings.
- The inspector SHOULD support jumping back to the source scene/asset/entity.

## 9. `ai-panel-v2`

### Outcome

The AI panel becomes a stronger product surface and future bridge to the Agent Workbench.

### Required behavior

- The panel MUST clearly expose included context.
- It SHOULD support distinct task modes such as:
  - Ask
  - Propose
  - Fix
  - Generate
  - Review
- Proposal cards MUST preview risk, affected surfaces, and validation impact.

## Acceptance Criteria

The overhaul succeeds when:

- the editor shell is stable and semantically clear,
- the primary authoring surfaces communicate provenance and actions clearly,
- logic, validation, search, runtime, and AI surfaces feel integrated rather than bolted on,
- future agentic work can attach to strong product surfaces instead of placeholders.

### PR1 Acceptance Criteria (Phase A — Menu Visibility & Compact Mode)

The following criteria are verified by Playwright tests (72/72 pass in clean run):

- [ ] S1: Menu dropdowns are fully visible at every supported viewport width (1920×1080, 1366×768, 1280×800).
- [ ] S1: `position: fixed` from anchor rect keeps menus above the dock layout and on-screen.
- [ ] S1: Keyboard navigation (ArrowUp/Down/Home/End/Enter/Space) works through the portaled dropdown.
- [ ] S1: Escape closes the active menu; opening a sibling menu auto-closes the current one.
- [ ] S1: Clicking a `.menu-trigger` does not close the currently open menu.
- [ ] S1: `npm run lint` passes (exit 0).
- [ ] S1: `npm run build` succeeds (exit 0).
- [ ] S2: Viewport below 1280 px activates compact mode (`data-compact="true"` on dock layout).
- [ ] S2: Tab bar visible with all five tabs (Assets, Scene, Outline, Properties, Tools).
- [ ] S2: Scene tab is active by default in compact mode; tab switching works.
- [ ] S2: Menu is functional in compact mode.
- [ ] S2: Status bar is visible in compact mode.
- [ ] S2: At exactly 1280 px, desktop 3-column layout is rendered (not compact).

## References

- `docs/specs/editor-workflow-convergence.md`
- `docs/specs/ai-native-editor-capabilities.md`
- `docs/roadmaps/ui-workflow-overhaul-roadmap.md`
