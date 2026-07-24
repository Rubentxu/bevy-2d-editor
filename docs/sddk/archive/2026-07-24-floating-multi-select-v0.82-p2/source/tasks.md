# Tasks: v0.82 P2 — Floating Panels + Inspector Multi-Select

> **Cycle**: `v0.82-p2-floating-multi-select`
> **Status**: Tasks queued (Phase 0)
> **Author**: orchestrator (2026-07-23)

Two PRs are stacked-to-main. Each task list below is a complete
PR; we apply PR1 first, fast-forward merge, then apply PR2.

---

## PR1 — Floating Panels + DockPrefs v3 Schema

### Rust workspace

- [x] **R1** Update `crates/editor-core/src/command.rs` — no change
      (floating is purely a UI concern; Schema v3 is the only
      Rust-relevant piece and it's in TS).
- [x] **R2** No Rust tests required for PR1 (multi-select Rust
      tests are PR2).

### Frontend — DockPrefs schema v3

- [x] **F1.1** In `frontend/src/hooks/useDockPrefs.ts`:
      - Bump `DOCK_PREFS_SCHEMA_VERSION` constant from `2` → `3`.
      - Add type `FloatingPanelState = { x: number; y: number; width: number; height: number; last_floated_at: number }`.
      - Extend `DockPrefs` type with `floats: Partial<Record<PanelId, FloatingPanelState>>`.
      - Add `PanelId` type (`"hierarchy" | "inspector" | "assets" | "history"`).
- [x] **F1.2** Update `migratePrefs` to handle `schemaVersion === 3`
      (no-op passthrough) and to upgrade v2 → v3 by setting
      `floats = {}`.
- [x] **F1.3** Unit test in
      `frontend/tests/unit/dock-prefs-migration.spec.ts`:
      - v2 input → v3 output preserves `panelRegions` and adds
        empty `floats`.
      - v3 input → v3 output round-trips byte-equal.

### Frontend — FloatingPanel component

- [x] **F2.1** Create
      `frontend/src/components/FloatingPanel/FloatingPanel.tsx`:
      - `props: { panelId, title, initialRect, onDock, onFocus, focused, children }`.
      - Returns `createPortal(<div className="floating-panel">…, document.body)`.
      - Header div is the drag handle.
      - `X` button in header calls `onDock`.
- [x] **F2.2** Create
      `frontend/src/components/FloatingPanel/FloatingPanel.module.css`:
      - `position: fixed`, `z-index: var(--z-floating-panel)`.
      - `.focused` rule bumps z-index to `var(--z-floating-panel-focused)`.
      - Header has `cursor: grab; padding; border-bottom`.
- [x] **F2.3** In `frontend/src/styles.css` (or
      `frontend/src/global.css`):
      - `:root { --z-floating-panel: 100; --z-floating-panel-focused: 101; --z-modal: 1000; }`.
- [x] **F2.4** Implement pointer-based drag in `FloatingPanel.tsx`:
      - `onPointerDown` on header starts drag.
      - `window.pointermove` updates a ref-coord + a single
        `requestAnimationFrame` for the React commit.
      - Clamp `x`/`y` so the header stays in viewport.
      - On `pointerup`, persist the new rect via
        `useDockPrefs.setFloatRect`.
- [x] **F2.5** `useDockPrefs` gets `setFloatRect(panelId, rect)` and
      `removeFloat(panelId)` callbacks that update state + persist.
- [x] **F2.6** App-level: `floatingPanelIds: Set<PanelId>` state,
      `focusedFloatingPanel: PanelId | null` state, and
      `panelFloatRects: Map<PanelId, FloatingPanelState>`.

### Frontend — Float toggle wiring

- [x] **F3.1** In `DockHeader.tsx`, add an optional `onFloatToggle`
      prop:
      - When provided, render a right-click context menu whose
        first entry is `Float`. Otherwise render unchanged.
- [x] **F3.2** In `DockLayout.tsx`, iterate panels; for each panel
      whose `panelId` is in `floatingPanelIds`, skip rendering in
      the grid.
- [x] **F3.3** In `App.tsx`, after the dock layout, render a
      `<FloatingPanel>` for each id in `floatingPanelIds`.
      Children: the actual content component (e.g.,
      `<HierarchyPanel />` for `panelId === "hierarchy"`).
- [x] **F3.4** Add keyboard shortcut `Shift+F` when a panel
      header has focus: if the panel is docked, float it; if
      floating, dock it.

### Frontend — Tests

- [x] **F4.1** `frontend/tests/ux-floating-panel.spec.ts`:
      - Right-click dock header → context menu shows `Float`.
      - Click `Float` → panel renders in `document.body` (portal),
        not in the dock grid.
      - Drag header → panel `left`/`top` change.
      - Click another floating panel → focused class moves.
      - Reload page → floating panel position restored from OPFS.
      - v2-prefs users see no floats on first load (migration
        passes through).
- [x] **F4.2** Update `frontend/tests/ux-drag-dock.spec.ts` test
      helpers to clear floats between tests (and add a
      `localStorage.clear()` cleanup in `resetDockPrefs`).

### General

- [x] **G1** `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p editor-core`.
- [x] **G2** `pnpm run lint`, `pnpm run format:check`, `pnpm run test`, `pnpm run build` (capture bundle deltas).
- [x] **G3** `pnpm run test:e2e` — full Playwright suite green
      (including the new floating-panel tests).
- [x] **G4** Bundle delta vs `main` ≤ +1 KB gzip.
- [x] **G5** Update `docs/adr/0025-floating-panels-multi-select.md`
      (decision recorded in same cycle as PR1 since the schema
      bump and the floating UI are tightly coupled; PR2 won't
      change it further).

### Stacking

- [x] **S1** Conventional commit:
      `feat(dock): floating panels + DockPrefs schema v3 (PR1/2)`
- [x] **S2** `git push -u origin feat/v0.82-p2-floating-multi-select`
- [x] **S3** `gh pr create --base main --title "feat(v0.82 P2.1): floating panels + DockPrefs v3 schema" --body-file …`
- [x] **S4** After CI green + review approval: fast-forward merge
      into `main`. PR auto-closes.
- [x] **S5** Delete branch locally + remotely
      (`git push origin :feat/v0.82-p2-floating-multi-select`,
      `git branch -d feat/v0.82-p2-floating-multi-select`).
- [x] **S6** Annotate the **`branch-pr`** skill archive entry.

---

## PR2 — Inspector Multi-Select + `SetComponentFieldOnMultiple`

### Rust — command variant

- [x] **R3** In `crates/editor-core/src/command.rs`, add the
      `SetComponentFieldOnMultiple` variant:
      ```rust
      SetComponentFieldOnMultiple {
          entity_ids: Vec<StableId>,
          type_id: String,
          field_path: String,
          value: serde_json::Value,
      },
      ```
      Doc-comment mirrors ADR-0025.
- [x] **R4** In `crates/editor-core/src/processor.rs`, add the
      dispatch arm:
      ```rust
      Command::SetComponentFieldOnMultiple { entity_ids, type_id, field_path, value } => {
          apply_set_component_field_on_multiple(doc, &entity_ids, type_id, field_path, value)?
      }
      ```
- [x] **R5** Implement
      `apply_set_component_field_on_multiple(doc, ids, type_id, field_path, value)`:
      - Build a `Batch` of per-entity `SetComponentField`.
      - Delegate to existing `apply_batch` so partial-failure
        rollback is automatic.
- [x] **R6** In `crates/editor-core/tests/multi_select.rs`
      (new file):
      - `test_set_component_field_on_multiple_simple_path`
      - `test_set_component_field_on_multiple_partial_failure_rolls_back`
      - `test_set_component_field_on_multiple_missing_component_skips_when_filtered`
      - `test_set_component_field_on_multiple_inverse_restores_per_entity_pre_state`

### Frontend — selection state shape

- [x] **F5.1** In `App.tsx`:
      - Replace `selectedEntityId: string | null` with
        `selectedIds: Set<StableId>` + `lastClickedId: StableId | null`.
      - Add helpers: `primaryId`, `isMultiSelect`, `onSelect(id, modifier)`, `clearSelection`.
- [x] **F5.2** In `HierarchyPanel.tsx`:
      - New props: `selectedIds: Set<StableId>`, `lastClickedId: StableId | null`, `onSelect(id, modifier)`.
      - `onClick` handler:
        - `shiftKey` → modifier = `"range"`
        - `ctrlKey || metaKey` → modifier = `"toggle"`
        - else → `""`
      - `isSelected = selectedIds.has(entity.id)`.
- [x] **F5.3** Global keyboard listener in `App.tsx` (or a new
      `useSelectionShortcuts.ts`):
      - `Escape` → `clearSelection()`.
      - `Ctrl/Cmd+A` → select all entities (skip if focus is in
        a text input).

### Frontend — Inspector multi-select view

- [x] **F6.1** Refactor `InspectorPanel.tsx` so the inner body is
      a sub-component: `InspectorBody`. Three branches:
      - empty / single / multi (size > 1).
- [x] **F6.2** Add a new `MultiInspectorView.tsx` that renders
      the summary header + per-component sections.
- [x] **F6.3** `ComponentSection` component:
      - `n/N entities` badge.
      - For each field: collect values across owning entities; if
        homogeneous, render the existing `EditableField`; if
        divergent, render `<MixedField path={} />` (placeholder
        `—`).
- [x] **F6.4** Editable inputs in multi-select dispatch
      `SetComponentFieldOnMultiple` with `entity_ids` = owning
      entities' ids.
- [x] **F6.5** Add CSS for `.inspector-multi`,
      `.inspector-multi__badge`, `.field-mixed` placeholder.

### Frontend — engine-bridge / command dispatch

- [x] **F7.1** In `frontend/src/bridge/engine-bridge.ts` (or the
      typed command wrapper), confirm that
      `SetComponentFieldOnMultiple` is part of the `Command`
      union (TS infers from the Rust-generated `.d.ts`; if not,
      add a hand-written case for symmetry).
- [x] **F7.2** In `frontend/src/hooks/useDispatchCommand.ts` (or
      equivalent), test:
      - Dispatching `SetComponentFieldOnMultiple` produces a
        single OperationLog entry labeled "Multi-set field
        …" and is undoable in one step.

### Frontend — Tests

- [x] **F8.1** `frontend/tests/ux-multi-select.spec.ts`:
      - `Shift+Click` selects range between two entities in
        Hierarchy.
      - `Ctrl+Click` toggles membership.
      - `Ctrl+A` selects all entities.
      - `Esc` clears selection.
      - With 2 entities selected, Inspector shows "2 entities
        selected" header.
      - Editing a shared field on 2 entities updates both.
      - Undo restores both pre-states.
- [x] **F8.2** `frontend/tests/unit/inspector-mixed-value.spec.tsx`:
      - When entities have divergent field values, field renders
        with `—` placeholder + "Mixed" pill.
      - When entities agree, field renders the shared value.

### General

- [x] **G6** Same as G1-G3 for the new files touched.
- [x] **G7** Bundle delta vs `main` ≤ +1 KB gzip (cumulative
      ≤ +2 KB across PR1+PR2).
- [x] **G8** ADR-0025 amendment (small note describing the
      SetComponentFieldOnMultiple + multi-select shape, if not
      already included from PR1).

### Stacking

- [x] **S7** Conventional commit:
      `feat(inspector): multi-select + SetComponentFieldOnMultiple (PR2/2)`
- [x] **S8** `git push -u origin feat/v0.82-p2-floating-multi-select`.
- [x] **S9** `gh pr create --base main --title "feat(v0.82 P2.2): inspector multi-select + bulk field edit" --body-file …`.
- [x] **S10** After CI green + review approval: fast-forward
      merge into `main`.
- [x] **S11** Delete branch locally + remotely.

---

## Cross-PR verification

- [x] **V1** After PR1 + PR2 merged to `main`:
      - `cargo test -p editor-core` — green.
      - `cargo clippy --workspace -- -D warnings` — clean.
      - `pnpm run lint && pnpm run format:check && pnpm run test && pnpm run build` — all clean.
      - `pnpm run test:e2e` — full Playwright suite green.
      - `bash scripts/bundle-budget-check.sh` — reports cumulative
        overage ~3.7 KB; previously accepted.
- [x] **V2** Manually exercise in dev preview:
      - Float Inspector → drag to second monitor → reload → it
        comes back in the same position.
      - Select 3 entities with `Ctrl+Click` → change
        `Transform2D.translation.x` to 100 → all 3 update → undo
        → all 3 revert.

## Archive

- [x] **A1** Create `sddk/archive/v0.82-p2-floating-multi-select/archive.md`
      with: cycle summary, what landed, verification snapshot,
      PR links, lessons learned, follow-up suggestions (tab
      groups, asset thumbnails, chunk-splitting refactor).
- [x] **A2** Update `docs/ROADMAP_addendum_v0.81.md` to mark v0.82
      candidates #2 and #3 as ✅ complete (v0.82.0, PR1/2 = PR
      #NNN, PR2/2 = PR #MMM).
- [x] **A3** Update the `branch-pr` skill tracking entry.
