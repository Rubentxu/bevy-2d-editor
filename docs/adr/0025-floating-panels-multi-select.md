# ADR-0025: Floating Panels + Inspector Multi-Select — React Portal, DockPrefs v3, Bulk Field Apply

## Status

Accepted (2026-07-23) — Hito 7 / `v0.82-p2-floating-multi-select` cycle
(v0.82.0, PR #NNN + PR #MMM stacked-to-main)

## Context

v0.82 P1 (ADR-0024) shipped drag-dock region swap as commit `9b076e1`
on `main` — panels can be moved between left/right/bottom cells of the
CSS-Grid layout, but they remain pinned to *some* cell. Two real
workflows are still unsupported:

1. **Two-monitor authoring** — a senior engineer (P-A "Ariadna")
   wants to undock the Inspector to a second monitor. With the current
   CSS-Grid layout the Inspector is fixed inside the grid; there is no
   way to "lift" it out.

2. **Bulk field edit** — an indie dev (P-B "Marco") selecting six
   enemy sprites needs to set `Transform2D.translation.y = 0` on all
   six at once. Today the workflow is six click-edit cycles. Same root
   cause: the selection state in `App.tsx:77` is
   `selectedEntityId: string | null` — only the last clicked entity is
   addressable.

`docs/ROADMAP_addendum_v0.81.md` lines 113-118 list both items as
v0.82 deferred candidates. The user picked **Option A** (strict
v0.82 P2, 2-3 weeks, 2 PRs) on 2026-07-23, choosing these two items
and explicitly excluding the broader `sddk/ux-overhaul/` cycle that
covers onboarding tour step-through, asset browser thumbnails, and
tab groups inside docks (those remain deferred).

Two sibling subsystems are involved:

- **Dock subsystem** (`useDockPrefs`, `DockLayout`, dock headers,
  `LeftDock`/`RightDock`/`BottomDock`) — already evolved through
  ADR-0021 (defold layout) → ADR-0024 (region swap).
- **Selection subsystem** (`App.tsx:77`, `HierarchyPanel.tsx:7-8`,
  `InspectorPanel.tsx:25`) — has been single-select since v0.78.0.

This ADR fixes the architectural surface for both:

1. **Floating panel container**: React Portal to `document.body` vs
   same-CSS-Grid reparenting vs separate `<iframe>`.
2. **Drag implementation**: HTML5 DnD vs custom `pointermove` vs
   `react-draggable` library.
3. **Selection shape**: `Set<StableId>` vs `string[]` vs two parallel
   booleans (`primaryId`, `isMultiSelect`).
4. **Multi-edit command surface**: new `SetComponentFieldOnMultiple`
   variant vs reusing the existing `Batch { commands: Vec<Command> }`
   wrapper as the only dispatch path.
5. **Schema bump**: `DockPrefs` is at schemaVersion 2 — does the new
   `floats` field require v3, or can it be added compatibly at v2?

## Decision

### 1. Floating panels use React Portal to `document.body`

A floating panel is the existing `LeftDock` / `RightDock` /
`BottomDock` content rendered into
`createPortal(<div>, document.body)`. The wrapper handles:

- `position: fixed; left; top; width; height` from a new
  `FloatingPanelState` slice
- drag handle (the dock header) for `pointermove`-based drag
- focus promotion to z-index 101 on header click
- `X` button (or right-click `Dock`) to snap back into the grid

The docked CSS-Grid layout (`<DockLayout>`) is **left untouched** —
a floating panel is removed from the grid entirely. When a panel id
is in `floatingPanelIds` (Set held in `App.tsx`), the dock grid
renders nothing for that cell, and the `FloatingPanel` portal takes
over.

### 2. Custom pointer-based drag, no library

The drag handle is the panel header. The implementation:

- `onPointerDown` on the header starts drag (sets `dragging` state,
  captures start position).
- `window.pointermove` updates a `useRef`-held coordinate plus a
  pending `requestAnimationFrame` (≤1 React commit per frame).
- `window.pointerup` ends drag, clamps `x`/`y` so the header stays
  in viewport, persists the new rect via
  `useDockPrefs.setFloatRect`.

We reject `react-draggable` because it would add ~15 KB gzip
(currently over the 350 KB bundle budget by ~2.7 KB — see
ADR-0024 §Consequences). A custom 20-LOC drag is the cheapest path.

### 3. Selection state is `Set<StableId>` + `lastClickedId`

```ts
const [selectedIds, setSelectedIds] = useState<Set<StableId>>(new Set());
const [lastClickedId, setLastClickedId] = useState<StableId | null>(null);

const primaryId = selectedIds.size === 1 ? [...selectedIds][0] : null;
const isMultiSelect = selectedIds.size > 1;
```

The `Set` gives O(1) toggle for `Ctrl/Cmd+Click` and O(1)
membership for the per-row "is this selected?" check. We accept a
single extra `useMemo(() => [...selectedIds], [selectedIds])`
conversion for any consumer that needs an array.

Modifiers per click on a Hierarchy row:

- **Plain click**: replace `selectedIds` with `{ id }`,
  `lastClickedId = id`.
- **`Shift+Click`**: range-select — every entity between
  `lastClickedId` and the clicked id in `scene.entities` iteration
  order.
- **`Ctrl/Cmd+Click`**: toggle membership.

Global keyboard:

- `Escape` clears selection (skipped when focus is in a text input).
- `Ctrl/Cmd+A` selects all entities in the scene (skipped when
  focus is in a text input).

### 4. `SetComponentFieldOnMultiple` command, delegating to `Batch`

```rust
Command::SetComponentFieldOnMultiple {
    /// Sorted, de-duplicated target entities.
    entity_ids: Vec<StableId>,
    type_id: String,
    field_path: String,
    value: serde_json::Value,
},
```

Processor behaviour: build a `Batch { label: "Multi-set field …",
commands: vec![SetComponentField { entity_id, type_id, field_path,
value }; N] }` and delegate to the **existing `apply_batch`** path.
This gives us:

- Per-entity inverse generation for free (`apply_batch` captures
  pre-state per sub-command)
- Partial-failure rollback for free (an inner
  `CommandError::FieldNotFound` rolls back all prior writes)
- A single OperationLog entry with label `Multi-set field …` — one
  undo step

Edge cases handled:

- Empty `entity_ids` → `CommandError::InvalidArgument("empty
  entity_ids")`
- Duplicate ids → de-duplicated at apply time
- Entity missing the component → we **filter at the frontend** so
  only entities that own `type_id` are dispatched; the inner
  SetComponentField never sees a missing-component scenario

### 5. `DockPrefs` schema bumps to v3

The new `floats: Partial<Record<PanelId, FloatingPanelState>>` field
lives at the root. Per ADR-0017 we record each versioned change in
`migratePrefs` and persist back as v3 on first load. The
v2 → v3 migration is additive (sets `floats = {}` default), so old
v2 prefs round-trip cleanly.

`FloatingPanelState` carries `{ x, y, width, height,
last_floated_at }`. `PanelId` is the union
`"hierarchy" | "inspector" | "assets" | "history"`.

### 6. Z-index scale

```css
:root {
  --z-floating-panel: 100;
  --z-floating-panel-focused: 101;
  --z-modal: 1000;
}
```

Existing scoped z-index values in `DockLayout.module.css` are
retained (a follow-up cycle will normalize them to the scale). The
new variables reserve headroom for future overlays.

## Stack and ordering

Two stacked-to-main PRs:

- **PR1** ships floats + DockPrefs v3 + tests
  (`crates/editor-core/src/command.rs` is untouched in this PR;
  the schema bump is purely TypeScript).
- **PR2** ships `SetComponentFieldOnMultiple` + selection shape +
  Inspector mixed-value + tests.

PR1 lands first because it touches the schema-loading path; PR2
stacks cleanly on a stable v3 loader with no need for a
`migratePrefs` v3 → v3 pass-through test.

## Consequences

Positive:

- **Two-monitor authoring unlocked** — Inspector can be lifted to a
  second monitor and persist its position across reloads.
- **Bulk field edit atomic and undoable** — six entity edits
  collapse to one OperationLog entry, one `Ctrl+Z` undoes them all,
  per-entity pre-state preserved even when values diverge.
- **Selection parity with industry editors** — `Shift+Click` /
  `Ctrl+Click` / `Esc` / `Ctrl+A` are the standard gesture set
  every senior user already knows.
- **No new runtime dependencies** — `react-dom`'s
  `createPortal` is already transitive; zero bundle dep additions.
- **Schema migration is additive** — old v2 prefs upgrade
  losslessly; the `useDockPrefs` v0.82 P1 localStorage write-through
  (mirroring `panelRegions`) carries over to also mirror `floats`.

Negative:

- **Bundle size overage grows** — measured PR1 delta: total JS
  gzip went from **352.70 KB** (post v0.82 P1, ADR-0024 baseline) →
  **354.80 KB** post-PR1 (≈ +2.10 KB delta, slightly higher than
  the +1 KB estimate due to the actual `FloatingPanel` component
  + drag handler being larger than the design estimate). PR2 is
  estimated to add ~1 KB (multi-select state shape refactor +
  Inspector mixed-value markers). Cumulative overage by the end of
  PR2 will be approximately **+3.1 KB** above the 350 KB target.
  The chunk-splitting refactor needed to claw this back remains
  deferred to a follow-up cycle (carried from ADR-0024).
- **HTML5 DnD interference on float** — the dock header is also
  the drag handle for region swap (v0.82 P1) and now also for
  float drag (this ADR). Mitigation: when a panel is floating,
  set `draggable={false}` on the header so only the portal's
  pointer-based drag is active. When docked, keep `draggable=true`
  for region swap.
- **Multi-edit divergence UI** — Inspector must surface "1/N
  entities have this component" and "Mixed" markers so the user
  knows what they're editing. We accepted this UI surface as the
  cost of divergence transparency.
- **`Set<StableId>` re-renders** — every `setSelectedIds(new
  Set(…))` invalidates any consumer that depends on the set
  identity. Children must use `useMemo` on derived arrays.
  Acceptable at our scale (< 1000 typical entities) but a future
  virtualization might need to revisit.
- **v0.82 P2 has zero Rust command tests for floats** — floats are
  entirely a UI concern; the schema migration test
  (`dock-prefs-migration.spec.ts`) is the only contract. We accept
  this because floats have no Rust paths and persist no
  scene-relevant state.

## Rollout

1. **PR1** — merged into `main`; `DockPrefs.schemaVersion` = 3
   in OPFS; no user action needed (migration is automatic on first
   load).
2. **PR2** — merged into `main`; `SetComponentFieldOnMultiple`
   appears in `Command` enum and is callable from the frontend.

The localStorage write-through key (carried from ADR-0024) is
extended to mirror `floats` alongside `panelRegions` — same key
`bevy-2d-editor:dock-panel-regions`, same write/read flow.

## Alternatives considered

- **Floating via separate `<iframe>` (cross-window panel)**: rejected
  — adds complexity (postMessage protocol, two OPFS contexts to
  reconcile) for a workflow a single fullscreen browser can already
  accommodate via OS-level window management.
- **`react-draggable` library**: rejected — see §Decision 2.
- **`string[]` for selection**: rejected — O(n) toggle on
  `Ctrl+Click` and O(n) `isSelected` checks; the `Set` shape is
  cleaner and matches the design precedent in
  [ADR-0022 (renumbered)](./0022-drag-and-dock-region-swap-renumbered.md)'s
  selection patterns.
- **`Batch` only (no `SetComponentFieldOnMultiple`)**: rejected —
  the frontend dispatch path stays explicit about intent, no Batch
  construction logic at call sites. The Rust cost is the new
  variant (≈10 LOC) plus processor arm (≈15 LOC) plus 4 unit tests.
- **Schema bump stayed at v2 (backward-compatible field)**:
  rejected — contradicts ADR-0017 (root-level key changes warrant
  a version bump) and complicates the `migratePrefs` logic.

## References

- v0.82 P1 immediate predecessor: `docs/adr/0024-drag-dock-swap.md`
- v0.82 P1 cycle archive: `sddk/archive/v0.82-p1-drag-dock-region-swap/archive.md`
- Roadmap addendum: `docs/ROADMAP_addendum_v0.81.md` (v0.82 #2 +
  #3)
- Spec: `sddk/active/v0.82-p2-floating-multi-select/spec/spec.md`
- Design: `sddk/active/v0.82-p2-floating-multi-select/design.md`
- Tasks: `sddk/active/v0.82-p2-floating-multi-select/tasks.md`
- Single-selection lineage: ADR-0017 (selection state),
  [ADR-0022 (renumbered)](./0022-drag-and-dock-region-swap-renumbered.md)
  (selection patterns)
- Dock schema lineage: ADR-0021, ADR-0024
