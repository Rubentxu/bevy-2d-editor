# Explore Report: v0.82 P2 — Floating Panels + Inspector Multi-Select

> **Cycle**: `v0.82-p2-floating-multi-select`
> **Target version**: v0.82.0 (PR#2 of 2; PR#1 = drag-dock region-swap)
> **Author**: orchestrator (2026-07-23)
> **Branch**: `feat/v0.82-p2-floating-multi-select`
> **Status**: Exploration complete — ready for `propose`

## Context

v0.82 P1 (`drag-dock-region-swap`, ADR-0024) shipped on `main` as commit
`9b076e1` (PR #116, fast-forward merged 2026-07-23). The
`docs/ROADMAP_addendum_v0.81.md` lists 5 deferred-to-v0.82 items;
the user picked **#2 Floating Panels** (#4 in the addendum) and
**#3 Inspector multi-select** (#6 in the addendum) for the next
cycle. Both target the dock-and-panel subsystem already shipped in
v0.80.0–v0.82.0 and the selection subsystem used by
`HierarchyPanel` → `InspectorPanel`.

## Context Quality

- **Level**: C1 — full durable knowledge present in repo (current
  code, ADRs, recent commit history, the v0.82 P1 artifacts).
- **Evidence Present**:
  - `docs/ROADMAP_addendum_v0.81.md` (v0.82 candidates section)
  - `docs/adr/0021-defold-inspired-layout.md` (spatial layout context)
  - `docs/adr/0024-drag-dock-swap.md` (immediate predecessor — same
    subsystem)
  - `frontend/src/components/Dock/{DockLayout,DockHeader,BottomDock,LeftDock,RightDock}.tsx`
    (current dock subsystem)
  - `frontend/src/components/{InspectorPanel,HierarchyPanel}.tsx`
    (single-selection wiring)
  - `frontend/src/App.tsx:77` — `const [selectedEntityId, setSelectedEntityId] = useState<string | null>(null);`
  - `crates/editor-core/src/command.rs:48` — `SetComponentField { entity_id, type_id, field_path, value }`
  - `crates/editor-core/src/processor.rs:830` — processor pattern for `SetComponentField`
  - `sddk/ux-overhaul/` (existing proposal/spec for a broader UX cycle;
    out of scope here — distinct cycle)
- **Missing Context**:
  - No in-repo ADR for floating panels (Decision 3 above requires one)
  - No `SetComponentFieldOnMultiple` command yet (Decision 2 below
    creates it)
- **Recommended Effort**: confirmed Option A from the user question
  (v0.82 P2 strict, ~2-3 weeks, 2 PRs stacked-to-main).

## Current State Audit

### Selection subsystem

```ts
// frontend/src/App.tsx:77
const [selectedEntityId, setSelectedEntityId] = useState<string | null>(null);
```

```ts
// frontend/src/components/HierarchyPanel.tsx:7-8
selectedId: string | null;
onSelect: (id: string | null) => void;
```

```ts
// frontend/src/components/InspectorPanel.tsx:25, 113, 124
selectedId: string | null;
// ...
const entity = scene?.entities.find((e) => e.id === selectedId) ?? null;
```

- **Single-select only.** `selectedId` is `string | null`, no set
  semantics. Selection lives in `App.tsx` and is drilled through to
  `HierarchyPanel` (which renders the active row) and `InspectorPanel`
  (which inspects the entity).
- **No multi-select wiring.** `onSelect(entity.id)` is called on plain
  click; shift/ctrl modifiers are ignored.
- **Drag-drop reparent works on `selectedId`** (single only).
- **InspectorPanel treats empty selection** with the "Select an entity
  to inspect" empty state.

### Command subsystem

```rust
// crates/editor-core/src/command.rs:48
SetComponentField {
    entity_id: StableId,
    type_id: String,
    field_path: String,
    value: serde_json::Value,
},
```

- **Single-entity only.** `entity_id: StableId` (not a set). Processor
  pattern: `match cmd { Command::SetComponentField { .. } => /* mutate */ }`.
- **No `SetComponentFieldOnMultiple` command exists.**
- **OperationLog supports `Batch`** (line 73 of `command.rs`) — useful
  for multi-select atomic undo.

### Dock subsystem (carried over from v0.80.0–v0.82.0)

- `DockLayout.tsx` renders 4 region containers: left, center, right,
  bottom. Each has a `data-region` + `data-drop-allowed` attribute.
  Center is protected (no DnD handlers, `data-drop-allowed="false"`).
- `DockHeader.tsx` per-region header (draggable + Move menu from
  v0.82 P1).
- `BottomDock.tsx` has its own header + tab strip + Move menu.
- The layout is CSS-Grid based; switching to React Portal for floating
  is an orthogonal concern — we can render the same content into a
  different root.
- Z-index management is currently **not centralized** (the CSS file
  has scattered `z-index: 1/2/10` values).

### Existing UX groundwork (NOT used by this cycle but worth noting)

The `sddk/ux-overhaul/` proposal covers command palette (`Ctrl+K`),
cheat sheet (`?`), onboarding tour, design system, themes. **Out of
scope** for v0.82 P2 — distinct cycle. But: the existing
`frontend/src/components/CommandPalette.tsx` (238 LOC) and
`CheatSheet.tsx` (97 LOC) already exist, and the user has picked
floating panels + multi-select for this cycle. We do not regress or
redesign those components.

## Open Questions Resolved (per SDDK proposal phase)

1. **Floating panel container**: React Portal to `document.body`,
   absolute-positioned `position: fixed` div with the dock content
   as children. No new shell — reuse the existing `LeftDock` /
   `RightDock` / `BottomDock` components.
2. **Multi-select storage shape**: `Set<StableId>` in App state.
   Keep `selectedEntityId` as the "primary" for Inspector default
   case; expose `selectedEntityIds: Set<StableId>` for the multi-aware
   components. Add `lastClickedId` to support shift-range select.
3. **Multi-select command**: add `SetComponentFieldOnMultiple` to
   `Command` enum. Inverse is itself (per-entity field captures
   pre-state). Per-entity failures abort the batch and roll back.
4. **Inspector multi-select rendering**: when 2+ entities selected,
   show common components with mixed-value indicator ("—"). Hide
   entity-specific panels (no header / no Name field) and surface a
   "2 entities selected" summary.
5. **Z-index scale**: define a semantic scale
   `z-dock-region: 1; z-dock-drop-indicator: 10; z-floating-panel: 100;
   z-floating-panel-focused: 101; z-modal: 1000` so future overlays
   can compose predictably. Done in Phase 0.
6. **Keyboard shortcuts**: `Shift+Click` adds to selection;
   `Ctrl/Cmd+Click` toggles; `Esc` clears selection; `Ctrl+A`
   selects all entities in scene.

## Architectural Decisions

### Decision 1 — Floating panels use React Portal + same components

A "floating" dock panel is the same `LeftDock` / `RightDock` /
`BottomDock` content rendered into a `createPortal(div, document.body)`
wrapper. The wrapper handles:
- `position: fixed` with x/y/width/height from a new `FloatingPanel`
  state shape
- drag handle (the dock header becomes the drag handle when floating)
- focus order (highest z-index panel receives keyboard focus on click)
- close button (`X` in the header — adds `onFloatToggle` to the
  existing `DockHeader`)

The docked layout (CSS Grid) stays untouched — floating is an
*alternative layout* for a panel that toggles out of the grid into a
free-floating window.

### Decision 2 — `SetComponentFieldOnMultiple` command

```rust
Command::SetComponentFieldOnMultiple {
    entity_ids: Vec<StableId>,   // sorted, no duplicates
    type_id: String,
    field_path: String,
    value: serde_json::Value,
}
```

- **Processor behavior**: iterate `entity_ids`, find entity with the
  `type_id` component, write `field_path`. If any entity is missing
  the component or the field path doesn't exist, return
  `CommandError::FieldNotFound` (and roll back prior writes via the
  captured pre-state on a `Batch`-style unwinding — OR delegate the
  whole thing to a `Batch` of `SetComponentField` commands so the
  existing processor/inverse machinery handles partial failures).
- **Decision**: delegate to `Batch`. The processor gets a thin wrapper
  that builds a `Batch { label: "Multi-set field 'foo.bar'", commands:
  vec![SetComponentField { entity_id, type_id, field_path, value }; N] }`
  and forwards to the existing `apply_batch` path. This avoids
  duplicating inverse-generation logic.
- **Inverse**: the existing `Batch` inverse works (per-command inverse
  captured at apply time). Undo restores per-entity pre-state.

### Decision 3 — Selection state shape

```ts
// New App.tsx state
const [selectedIds, setSelectedIds] = useState<Set<StableId>>(new Set());
const [lastClickedId, setLastClickedId] = useState<StableId | null>(null);

// Derived
const primaryId = selectedIds.size === 1 ? [...selectedIds][0] : null;
const isMultiSelect = selectedIds.size > 1;
```

- Single-click: replace selection with `{ id }`.
- `Shift+Click`: range-select from `lastClickedId` to clicked id in the
  hierarchy order.
- `Ctrl/Cmd+Click`: toggle membership.
- `Esc`: clear selection.
- `Ctrl/Cmd+A`: select all entities in the scene.

`HierarchyPanel` receives both `selectedIds` (set) and `onSelect(id,
modifier)`. The click handler in `HierarchyPanel` builds the new set
according to the modifier and reports it via `onSelect(id, modifier)`.

### Decision 4 — Inspector multi-select view

When `selectedIds.size > 1`:
- Hide entity header (name, id) — entities are heterogeneous.
- For each component type present in ≥1 selected entity:
  - Show a section titled `<ComponentType>` with a badge
    `n/N entities have this component` (n = with-component count,
    N = total selected).
  - For each field:
    - If all entities with the component share the value: show the
      value (editable).
    - If values diverge: show `—` placeholder and an inline icon
      "Mixed".
- The "edit" action issues `SetComponentFieldOnMultiple` with the
  updated value applied to all entities in the section.

### Decision 5 — Floating panel z-index + drag

- All floating panels render at `z-index: 100`. The most-recently-
  clicked/focused panel bumps to `z-index: 101` via a single shared
  `focusedFloatingPanelId` state in `App.tsx`. Clicking a panel
  header promotes it.
- Drag is implemented in `FloatingPanel.tsx` with `pointermove` +
  `pointerup` handlers attached to the drag handle (the dock header).
  The transform is `position: fixed; left; top` updates — no React
  re-render per pixel (use `useRef` for coords, `requestAnimationFrame`
  to commit).
- Width/height come from a new `FloatingPanelState` slice
  `{ id: PanelId, x, y, w, h }` persisted to OPFS under the
  `panelRegions` / `floats` key (schema v3 bump — see ADR-0025).

### Decision 6 — Schema bump to v3

v0.82 P1 already shipped `schemaVersion: 2`. Adding
`floats: Record<PanelId, FloatingPanelState>` requires a v2 → v3
migration. The migration:
- Adds the `floats` key with defaults (no panel floating by default;
  panels start docked).
- Stale panelRegions from v2 are kept verbatim.
- The migration is additive — no destructive change.

`migratePrefs` (in `useDockPrefs.ts`) gets a new branch for v2 → v3.

## Risks

- **Z-index ordering regressions**: existing CSS has scattered z-index
  values. The new semantic scale requires auditing existing rules.
  Mitigation: Phase 0 introduces the CSS variables; existing rules
  keep their values.
- **Floating drag interferes with grid resize**: a floating panel's
  drag handle is the dock header; the dock header was already
  `draggable=true` for the v0.82 P1 region swap. Need to disable the
  HTML5 DnD when floating (`draggable={!isFloating}`).
- **Multi-select on Inspector with divergent component schemas**: if
  one entity has `editor.Sprite2D` and another doesn't, the section
  shows "1/2 entities have this component". Editing applies only to
  the entities that have the component (the processor skips
  entities missing the type_id).
- **Bundle size**: floating logic is small (~50 LOC for the
  `FloatingPanel` component + drag handler), multi-select is
  similarly small (~80 LOC). Bundle +1 KB at most; we're already over
  budget so this needs to be called out (chunk-splitting refactor
  still deferred).

## Out of Scope (explicit)

- Tab groups inside docks (#7 in addendum — deferred, risky)
- Asset browser thumbnails (#8 — separate cycle)
- Welcome tour step-through (#9 — separate cycle)
- Drag-resize of the floating panel's *contents* (only the panel
  outer rect is draggable; inner DockDivider-driven resize stays
  applicable if the user wants to resize the panel internally)
- Multi-window editor (separate browser windows)
- Collaborative editing

## Workload Forecast

- **Phase 0** (already in this report): scaffold SDDK + ADR-0025 +
  design + tasks + spec (~1 day)
- **Phase 1 — Floating panels**: `FloatingPanel.tsx` + portal +
  drag-handle + z-index scale + App wiring + CSS. ~250 LOC, 1 PR
  (~1 week)
- **Phase 2 — Multi-select command**: `SetComponentFieldOnMultiple`
  (delegates to `Batch`) + Rust tests + WASM export + frontend
  selection-shape refactor (`Set<StableId>` + `lastClickedId`). ~200
  LOC, 1 PR (~3 days)
- **Phase 3 — Inspector multi-select view**: inspector rendering for
  N>1, mixed-value indicators, "1/N entities have component" badges.
  ~300 LOC, included in Phase 2 PR (~3 days)
- **Phase 4 — Schema v3 + OPFS persistence for floating panels**:
  `migratePrefs` v2→v3 + `floats` schema + persistence hooks in
  `useDockPrefs`. ~120 LOC, included in Phase 1 PR
- **Phase 5 — Playwright + bundle verification**: 6-8 new tests
  (drop on float-toggle, multi-select keyboard parity, inspector
  mixed-value, schema migration). ~150 LOC test code, included in
  Phase 1/2 PRs
- **Total**: ~1000 LOC across 2 stacked PRs over ~2-3 weeks

## Stacking Strategy

Two PRs, stacked-to-main:

- **PR1 — Floating panels** (Phases 1+4+5 part): v3 schema bump,
  `FloatingPanel` component + portal + drag, OPFS persistence, App
  composition, Playwright tests.
- **PR2 — Multi-select** (Phases 2+3+5 part): Rust command + tests,
  frontend selection shape refactor, Inspector mixed-value view,
  Playwright tests.

Order: PR1 first because it touches `useDockPrefs` schema — easier
to merge PR2 (which doesn't touch schema) on top of a stable v3
prefs loader. PR2 includes a `migratePrefs` v3 → v3 pass-through
(no-op) test to confirm.

## Pre-existing Baseline Debt (carried, not addressed here)

- v0.82 P1 already pushed the bundle 2.7 KB over the 350 KB budget;
  v0.82 P2 will add ~1 KB. Cumulative overage ~3.7 KB.
- ADR-0017 §Issue 2 — Bevy 0.19 query conflict, UNRESOLVED (not on
  this cycle's path).
- 6 pre-existing format violations in `menus.ts`, `StatusBar.tsx`,
  `WelcomeOverlay.tsx`, and the wasm-generated `editor_core*.d.ts`
  files (out of scope, tracked separately).
- `cargo clippy --workspace --lib --bins -- -D warnings` reports
  110+ pre-existing errors in `editor-core` (mostly
  `unused_must_use` on `Result` from `HashMap::insert`, accumulated
  through Hito 1-7 cycles). PR1 contributes zero new clippy
  warnings (no `.rs` files touched); PR2 must keep its delta at
  zero. Clippy cleanup is a separate dedicated cycle.

## Artifacts Produced in This Cycle

- `sddk/active/v0.82-p2-floating-multi-select/explore-report.md` (this file)
- `sddk/active/v0.82-p2-floating-multi-select/proposal.md`
- `sddk/active/v0.82-p2-floating-multi-select/spec/spec.md`
- `sddk/active/v0.82-p2-floating-multi-select/design.md`
- `sddk/active/v0.82-p2-floating-multi-select/tasks.md`
- `docs/adr/0025-floating-panels-multi-select.md` (added during apply)
