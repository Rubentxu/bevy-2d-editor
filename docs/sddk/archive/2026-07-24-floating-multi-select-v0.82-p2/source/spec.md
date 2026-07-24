# Spec: v0.82 P2 — Floating Panels + Inspector Multi-Select

> **Cycle**: `v0.82-p2-floating-multi-select`
> **Status**: Specified (Phase 0)
> **Author**: orchestrator (2026-07-23)

This is the behavior contract for v0.82 P2. Two stacked PRs:
**PR1** ships features F1-F5 (floating panels). **PR2** ships
features F6-F11 (multi-select + inspector mixed-value).

## S1. Floating panel surface (PR1)

A floating panel is a previously-docked panel lifted into a
free-positioned, free-sized overlay above the dock layout.

### F1. Float toggle
- Right-clicking a dock header shows a context menu whose first
  entry is `Float` (current entries: `Move`, `Close`).
- Selecting `Float` causes the panel to detach from the CSS-Grid
  layout and render into `document.body` via `createPortal`.
- The panel retains its current width and approximate height.
- The grid cell it vacated becomes empty space (no placeholder).

### F2. Float-restore toggle
- A floating panel header has a `Dock` action (e.g., `X` button or
  right-click menu entry) that re-attaches the panel to its prior
  grid cell.
- On restore, the floating overlay is unmounted and the dock
  container renders the panel content again at its prior position.

### F3. Drag-to-move
- The floating panel header is the drag handle.
- On `pointerdown` over the header, the panel becomes "dragging";
  subsequent `pointermove` events translate to `left`/`top`
  updates.
- Movement uses `useRef` coordinates + a single
  `requestAnimationFrame` for React commits (≥30 fps target).
- Movement respects the viewport: the panel cannot be dragged so
  far that its header is offscreen.

### F4. Focus stacking
- Multiple floating panels coexist. The most-recently-clicked
  panel renders on top (z-index 101); others sit at z-index 100.
- Clicking a floating panel header promotes it; clicking elsewhere
  demotes it back to z-index 100 unless it remains the focus
  owner.
- A click outside any floating panel demotes the previously
  focused panel back to 100.

### F5. Persistence (OPFS schema v3)
- `{ panelId: FloatingPanelState }` is written to OPFS under the
  same `dock-prefs.json` path, in a new top-level key `floats`.
- `FloatingPanelState = { x: number, y: number, width: number,
  height: number, last_floated_at: number }`.
- `useDockPrefs` reads `floats` from the prefs object on load and
  restores the relevant floating overlays.
- v2 → v3 migration populates `floats = {}` on load for users
  upgrading from v0.82 P1.

## S2. Multi-select (PR2)

### F6. Selection state shape
- `App.tsx` holds `selectedIds: Set<StableId>` (a JS Set) and
  `lastClickedId: StableId | null`.
- Components receive the derived props they need:
  - `HierarchyPanel` receives `selectedIds` (set) and
    `lastClickedId` (for shift-range)
  - `InspectorPanel` receives `selectedIds` and a derived
    `primaryId` (single-id fallback for the empty/single case)
  - `DockLayout` and other components unchanged for now

### F7. Click interaction modifiers
- Plain click on a Hierarchy row: replace `selectedIds` with
  `{ id }`, set `lastClickedId = id`.
- `Shift+Click`: range-select — extend `selectedIds` to include
  every entity between `lastClickedId` and the clicked id in
  `scene.entities` iteration order. The direction
  (top→bottom or bottom→top) follows the click, not the order.
- `Ctrl/Cmd+Click`: toggle membership of the clicked id in
  `selectedIds`.
- `Esc`: clear `selectedIds` and `lastClickedId`.

### F8. Select-all shortcut
- `Ctrl/Cmd+A` while focus is in the Hierarchy panel (or no
  text-input is focused) selects every entity in the scene
  (`selectedIds = new Set(scene.entities.map(e => e.id))`).
- If focus is in a text input or a context menu, the shortcut
  passes through to the underlying default behavior (select all in
  input).

### F9. Visual selection indicator
- Each Hierarchy row whose id is in `selectedIds` gets the
  existing `selected` CSS class.
- Range selection (Shift+Click) animates the addition of `selected`
  to all rows in the range (browser-native highlight, no
  animation requirement).

### F10. Inspector multi-select view
- `selectedIds.size === 0`: render existing empty state ("Select
  an entity to inspect").
- `selectedIds.size === 1`: render existing single-entity
  inspector (entity header, components).
- `selectedIds.size > 1`: render the **multi-inspector view**:
  - No entity header (entities are heterogeneous).
  - A summary line: `<N> entities selected · <K> components in
    common`.
  - For each component type present in ≥1 selected entity:
    - Section title `<TypeName>` + badge `<n>/<N> entities have
      this component` (where n = selected entities that own the
      component).
    - Each field renders:
      - If all entities that own the component agree on the value:
        show the value (editable input).
      - If values diverge across selected entities that own the
        component: show a `—` placeholder + a small "Mixed" pill
        on the right side of the field.
    - Editable inputs dispatch
      `SetComponentFieldOnMultiple` on commit (Enter or blur,
      whichever the existing single-select inspector uses).

### F11. Multi-edit apply + undo
- `SetComponentFieldOnMultiple` builds a `Batch` of N
  `SetComponentField` commands internally (N = number of entities
  owning the component).
- The processor applies the Batch; existing `apply_batch`
  machinery collects inverse commands per sub-apply and rolls back
  on partial failure (`CommandError::FieldNotFound` if the
  field_path doesn't exist on any of the targeted entities).
- Undo via `Ctrl+Z` (or `toolbar.undo`) restores per-entity
  pre-state, even when entities had divergent values.
- A single undo entry appears in the OperationLog with label
  `Multi-set field <type>.<path>`.

## NFR

- **Performance**: floating-panel drag sustains ≥30 fps on a
  mid-range laptop (Intel i5 / M1). Multi-select UI does not
  re-fetch the scene from the engine — selection is a frontend
  derivation over the cached `scene` from OPFS / engine bridge.
- **Accessibility**: floating panels are focus-trapped with
  keyboard; `Esc` returns focus to the dock layout. Multi-select
  rows expose `aria-selected={true}` for screen readers.
- **Persistence**: floats survive reload. OPFS persists schema
  v3 with migration v2→v3 no-op transform for old data.
- **Bundle**: PR1 +1 KB gzip; PR2 +1 KB gzip.
- **No new runtime dependencies**.

## Out-of-scope (definitively)

- Tab groups inside docks (addendum #7)
- Asset browser thumbnails (addendum #8)
- Onboarding tour step-through (addendum #9)
- Multi-window editor
- Drag-resize of the panel *contents*
- Collaborative editing

## Verification matrix

| Layer | Test | PR |
|------|------|----|
| Rust unit | `command_v2_to_v3_passes_through` | PR1 |
| Rust unit | `set_component_field_on_multiple_simple_path` | PR2 |
| Rust unit | `set_component_field_on_multiple_partial_failure_rolls_back` | PR2 |
| Rust unit | `set_component_field_on_multiple_missing_component_skips` | PR2 |
| Frontend | `FloatingPanel renders to document.body via portal` | PR1 |
| Frontend | `FloatingPanel drag updates left/top ≥30 fps` | PR1 |
| Frontend | `FloatingPanel focus promotes z-index to 101` | PR1 |
| Frontend | `DockPrefs v2 → v3 migration preserves panelRegions` | PR1 |
| Frontend | `HierarchyPanel Shift+Click selects range` | PR2 |
| Frontend | `HierarchyPanel Ctrl+Click toggles membership` | PR2 |
| Frontend | `HierarchyPanel Ctrl+A selects all` | PR2 |
| Frontend | `HierarchyPanel Esc clears selection` | PR2 |
| Frontend | `Inspector shows Mixed marker on divergent field` | PR2 |
| Frontend | `Inspector edit dispatches SetComponentFieldOnMultiple` | PR2 |
| E2E Playwright | `Right-click dock header → Float → panel renders in portal` | PR1 |
| E2E Playwright | `Float → reload → float position survives` | PR1 |
| E2E Playwright | `Select 2 entities → edit Transform2D.y → both update` | PR2 |
| E2E Playwright | `Multi-edit undo restores per-entity pre-state` | PR2 |
| Lint / Format | ESLint + Prettier clean | PR1, PR2 |
| Bundle | Δ ≤ +1 KB gzip per PR | PR1, PR2 |
