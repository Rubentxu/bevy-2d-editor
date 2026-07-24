# Proposal: v0.82 P2 — Floating Panels + Inspector Multi-Select

> **Cycle ID**: `v0.82-p2-floating-multi-select`
> **Status**: Proposed
> **Author**: orchestrator (2026-07-23)
> **Target version**: v0.82.0 (PR#2 of 2 stacked-to-main)
> **Related ADRs**: ADR-0017, ADR-0021, ADR-0024

## Why

The v0.82 P1 cycle just shipped drag-and-drop region swap (ADR-0024,
PR #116 merged 2026-07-23). User picked **Option A** — a strict v0.82 P2
covering the two items from `docs/ROADMAP_addendum_v0.81.md` lines
113-118 that align with the *same* dock/selection subsystem P1 just
evolved: **Floating Panels (#2 in v0.82 list / #4 in addendum)** and
**Inspector Multi-Select (#3 in v0.82 list / #6 in addendum)**.

### Persona pain (Why now)

- **P-A "Ariadna"** (senior, Unity/Godot background): wants to **undock
  the Inspector** to a second monitor while keeping the Hierarchy
  docked on the primary. Today's CSS-Grid layout pins every panel to a
  grid cell.
- **P-B "Marco"** (indie): **selecting 6 enemy entities** to bulk-set
  their `Transform2D.translation.y` is currently a click-loop: select
  one, edit, select next, edit, ×6. Will not adopt the editor.
- **P-C "Sasha"** (designer): wants to **compare two sprite entities**
  side-by-side in the inspector without losing the Canvas view. The
  Canvas is currently pinned in the center and the Inspector is
  fixed on the right; overlapping viewports requires either alt-tab
  or browser DevTools.

### Scope boundary (in / out)

**In scope** (this cycle, 2-3 weeks):

1. **Floating panel**: any docked panel can be toggled to a
   free-floating window overlay via a `Float` action in the panel
   header context menu. The floating panel:
   - has a drag-handle (the existing dock header bar) that moves the
     panel
   - has a focus-on-click that promotes its z-index above sibling
     floating panels
   - has a `Dock` action that snaps it back into its previous
     grid cell
   - persists position/size to OPFS (schema v3)
   - coexists with the docked `DockLayout` (no dock-region rewrite)
2. **Inspector multi-select**: HierarchyPanel supports
   `Shift+Click` (range), `Ctrl/Cmd+Click` (toggle), `Ctrl/Cmd+A`
   (all), `Esc` (clear). The Inspector view, when 2+ entities are
   selected, renders shared components with mixed-value indicators
   (`—`) and a "1/N entities have this component" badge. Edits issue
   `SetComponentFieldOnMultiple` and apply atomically to all selected
   entities that own the component.

**Out of scope** (deferred):

- Tab groups inside docks (addendum #7) — distinct UX problem
- Asset browser thumbnails (addendum #8) — separate cycle
- Onboarding tour step-through (addendum #9) — separate cycle
- Multi-window editor (separate browser windows) — not a near-term
  goal
- Drag-resize of the floating panel's *contents* (only the outer
  rect is draggable)
- Collaborative multi-user editing

## What

### Feature 1 — Floating Panels

A new `FloatingPanel` React component wraps the existing
`LeftDock` / `RightDock` / `BottomDock` content. The component:

- Renders into a `createPortal(div, document.body)` so it escapes
  the CSS-Grid layout
- Takes `panelId`, `x`, `y`, `width`, `height`, `onClose` props
- Implements pointer-based drag on the header (no HTML5 DnD —
  `pointermove` + `pointerup` with `useRef` for coords and a single
  `requestAnimationFrame` per move for React commits)
- Promotes to `z-index: 101` on header click; demotes back to
  `z-index: 100` on focus loss
- Persists `{ panelId: FloatingPanelState }` to OPFS through
  `useDockPrefs` (schema v3 — see ADR-0025)

The dock headers (`DockHeader.tsx`) get a new `onFloatToggle`
callback that the user can invoke via:

- Right-click on the header → context menu (`Float`, `Dock`)
- Keyboard shortcut `Shift+F` when a panel header is focused

The CSS introduces a semantic z-index scale:

```
--z-floating-panel: 100;
--z-floating-panel-focused: 101;
```

The existing scoped z-index values in `DockLayout.module.css` are
retained (and a follow-up cycle can normalize them to the scale).

### Feature 2 — Inspector Multi-Select

A new `Command` variant:

```rust
Command::SetComponentFieldOnMultiple {
    entity_ids: Vec<StableId>,   // sorted, de-duplicated
    type_id: String,
    field_path: String,
    value: serde_json::Value,
},
```

whose processor delegates to a `Batch { label: "Multi-set field …",
commands: vec![SetComponentField { … }; N] }`. The existing
`apply_batch` machinery handles partial failures (returns the
`CommandError::FieldNotFound` for the failing entity and rolls back
all prior writes).

Frontend selection state in `App.tsx` migrates from
`selectedEntityId: string | null` to:

```ts
const [selectedIds, setSelectedIds] = useState<Set<StableId>>(new Set());
const [lastClickedId, setLastClickedId] = useState<StableId | null>(null);
```

with `primaryId = selectedIds.size === 1 ? [...selectedIds][0] : null`
as the single-select fallback for any legacy single-id components.

`HierarchyPanel` click handler:

- `Shift+Click`: extend selection to include every entity between
  `lastClickedId` and the clicked entity in `scene.entities` iteration order.
- `Ctrl/Cmd+Click`: toggle membership of the clicked id in
  `selectedIds`.
- Plain click: replace `selectedIds` with `{ id }`, set
  `lastClickedId = id`.

`InspectorPanel` rendering for `selectedIds.size > 1`:

- Hide entity header (name, id — heterogeneous)
- For each component type present in ≥1 selected entity:
  - Section title: `<TypeName>` with a `n/N entities` badge
  - For each field:
    - If all entities with the component share the value: show value
    - If values diverge: show `—` placeholder + inline "Mixed" icon
  - Editable inputs dispatch `SetComponentFieldOnMultiple` on commit
- Edit on an input applies to entities that own the component;
  processor skips entities missing it.

## How (high-level)

### Stacking strategy

Two PRs, stacked-to-main, each independently mergeable:

- **PR1** — floating panels + DockPrefs v3 schema
  - ~250 LOC (component, portal, drag, z-index, App wiring, CSS)
  - ~120 LOC (schema v3 + `migratePrefs` v2→v3 + OPFS persistence)
  - ~100 LOC (Playwright tests)
- **PR2** — multi-select + inspector mixed-value
  - ~80 LOC Rust (command variant + tests)
  - ~200 LOC frontend (selection-shape refactor + UI)
  - ~150 LOC Playwright tests

PR1 lands first because it touches the schema-loading path; PR2's
focus on selection/inspector is orthogonal and stacks cleanly.

### Architectural fit

- **Frontend stack**: React 19 + TypeScript + Vite 6. No new deps.
  `react-dom`'s `createPortal` is already a transitive dep.
- **Backend stack**: Bevy 0.19 + WASM bindings. The new command
  variant is a small additive change to `command.rs` with a
  delegated processor that reuses `apply_batch`.
- **Persistence**: v3 schema loaded via `useDockPrefs` → `opfs_load`.
  Migration v2→v3 is additive (sets `floats = {}` default).
- **CSS**: One new module file `FloatingPanel.module.css`; the
  existing `DockLayout.module.css` is left untouched.

### Tradeoffs considered

- **Drag library vs custom** — picked custom (20 LOC) over
  `react-draggable` (would add ~15 KB; we are 2.7 KB over the bundle
  budget already).
- **`Set<StableId>` vs array** — picked Set for O(1) toggle; React
  still re-creates the set on each change so children must use a
  derived `selectedIdsArr = useMemo(() => [...selectedIds], [selectedIds])`.
- **`SetComponentFieldOnMultiple` vs `Batch` only** — picked the
  new variant for symmetry with `SetComponentField` and to keep the
  frontend dispatch path explicit (no need to build a Batch from N
  inputs at the call site; the command expresses the intent
  directly).
- **Schema bump v2→v3 vs backward-compatible** — picked bump because
  the new field is at the root of the DockPrefs object (not nested)
  and the loader already supports versioned upgrades from ADR-0017.

## Acceptance criteria

PR1 is mergeable when:

1. Right-click on any dock header → context menu shows `Float`
2. Selecting `Float` lifts the panel into a draggable overlay
3. Drag handle (the panel header) moves the panel at ≥30 fps
4. Clicking another floating panel promotes it visually
5. Reloading the page restores floating panel positions from OPFS
6. Schema v2 prefs (no `floats` key) upgrade to v3 without loss
7. All existing v0.82 P1 Playwright tests still pass
8. `cargo test -p editor-core` and frontend tests pass
9. ESLint, Prettier, bundle budget check pass
10. ADR-0025 is committed and reviewed

PR2 is mergeable when:

1. `Shift+Click` selects a range of entities in the Hierarchy
2. `Ctrl/Cmd+Click` toggles membership
3. `Ctrl/Cmd+A` selects all entities in the scene
4. `Esc` clears selection
5. With 2+ entities selected, the Inspector shows shared components
   with `—` for mixed values
6. Editing a shared field on multiple entities applies the value to
   all that own the component, as one atomic command
7. Undo restores per-entity pre-state even when entities had
   divergent values
8. Rust unit tests cover partial-failure rollback
9. Frontend tests cover all five selection interactions
10. PR1 is already on `main` (stack order preserved)

## Verification snapshot (target)

- `cargo test -p editor-core` — all green
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- `pnpm run lint` — clean
- `pnpm run format:check` — clean
- `pnpm run build` — bundle delta ≤ +1 KB gzip
- `pnpm run test:e2e` — all new + existing Playwright tests green
- `bash scripts/bundle-budget-check.sh` — reports current overage;
  we accept it (see ADR-0024 §Consequences; chunk-splitting deferred)

## Risks

- Z-index conflicts with future modals — semantic scale
  `--z-modal: 1000` reserves headroom
- Drag-handle interference with HTML5 DnD — float header disables
  `draggable=true`
- Multi-select edit divergence — per-component "1/N have it" badge
  surfaces the divergence so users see before they edit
- v3 schema bump requires a migration test that ships in PR1

## Open questions

None — all decisions resolved in the explore report. ADR-0025 will
document them with full status / context / decision / consequences
sections.
