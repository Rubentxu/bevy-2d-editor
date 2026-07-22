# ROADMAP Addendum — v0.81 Candidates

> Generated 2026-07-22 from `defold-inspired-redesign` cycle completion.

## Context

After v0.80.0 ships the Defold-inspired layout, the natural next iteration extends the dock system without breaking the spatial stability we've established. Candidates are prioritized by impact/effort and listed below.

## Candidates

### Tier 1 (high impact, medium effort — 1-2 weeks each)

#### 1. Drag-and-drop docking between regions
**What**: Drag a panel from one dock region to another (e.g., Properties from right → bottom). Currently regions are fixed at creation time.
**Why**: Unifies Defold's tab groups + Unity's drag-dock. Users can build their own layout.
**Effort**: 1 week (the dividers and OPFS prefs infrastructure is already there).
**Risks**: Visual feedback during drag (need a `DockPlaceholder` preview element), accessibility (keyboard-only equivalent).

#### 2. Workspace presets (FPS / 2D Platformer / Top-Down)
**What**: Predefined dock layouts that users can switch between. Each preset saves: widths, visibility, last-opened tabs, scroll positions.
**Why**: Matches Defold's workspace presets. Game developers have radically different layouts per genre.
**Effort**: 1 week.
**Storage**: Extend `dock-prefs.json` with a `workspace` field and a list of named presets.

#### 3. Global search (Phase C stub upgrade)
**What**: Wire `useGlobalSearch` to actually index scenes, assets, source files, logic graphs, and commands. Open with `Ctrl+P` (palette already exists) or `Ctrl+Shift+F`.
**Why**: The current SearchTab is a placeholder. With 50+ scenes / 100+ assets, search becomes essential.
**Effort**: 1-2 weeks. Requires WASM-side index for source-file content (or lazy load via OPFS).

### Tier 2 (medium impact, low-medium effort)

#### 4. Floating panels (undock to free-floating window)
**What**: Right-click panel header → "Float" → panel becomes an absolute-positioned draggable window over the editor.
**Why**: Matches VS Code's "Move to new window" and Defold's undocked panels.
**Effort**: 1 week (React Portal + `position: fixed` + drag handle).
**Risks**: Z-index management when multiple panels float.

#### 5. Per-panel state persistence
**What**: Remember scroll position, expanded sections, selected asset in left dock, etc.
**Why**: Coming back to the editor after a reload feels familiar.
**Effort**: 3-5 days. Extends `dock-prefs.json`.

#### 6. Inspector multi-select
**What**: Select 2+ entities, inspector shows common components with mixed values (e.g., "—" or "Mixed" when fields differ).
**Why**: Standard editor feature. Useful for bulk property changes.
**Effort**: 1 week. Requires Command API extension (`SetComponentFieldOnMultiple`).

### Tier 3 (low-medium impact)

#### 7. Tab groups inside docks
**What**: Multiple panels share a region via tabs (e.g., right dock with tabs for Outline / Properties / Console).
**Why**: Users with small screens can stack panels.
**Effort**: 1-2 weeks. Risky — may break the spatial stability principle.

#### 8. Asset browser thumbnails
**What**: Drag-imported textures show previews in the AssetNavigator tree.
**Why**: Visual feedback for non-coders.
**Effort**: 3 days. Need to read asset bytes via existing `read_asset_file_bytes` and render.

#### 9. Welcome tour overlay step-through
**What**: "Take the tour" button in Welcome now triggers a multi-step walkthrough (highlight each dock + explain).
**Why**: Onboarding for first-time users.
**Effort**: 1 week. Uses `react-joyride` or build minimal.

#### 10. Drag-to-resize status bar height
**What**: Bottom dock height is currently fixed at 240px; make it drag-resizable like other docks.
**Why**: Consistency.
**Effort**: 1 day (reuses DockDivider).

## Out of scope (defer to v0.82+)

- Multi-window editor (separate browser windows showing different scenes)
- Collaborative editing (multiple cursors, OT/CRDT)
- Mobile/touch UI
- Voice/scriptable editor extensions

## Recommended next cycle

Start with **#3 (Global search)** since the Search tab is already in the bottom dock and users will hit it first. Then **#2 (Workspace presets)** for power users. **#1 (Drag-and-drop docking)** last because it requires the most visual polish.

## v0.81 shipped status

| Tier | Item | Status | PR |
| --- | --- | --- | --- |
| Tier 1 | #3 Global search | ✅ Shipped in v0.81.0 | PR1 |
| Tier 1 | #2 Workspace presets | ✅ Shipped in v0.81.0 | PR2 |
| Tier 1 | #1 Drag-and-dock infra | ✅ Shipped in v0.81.0 (region-swap in v0.82) | PR3 |
| Tier 2 | #5 Per-panel state persistence | ✅ Shipped in v0.81.0 (collapse flags + OPFS schemaVersion) | PR4 |
| Tier 2 | #10 Drag-to-resize status bar | ✅ Shipped in v0.81.0 (clamp 20–48 px) | PR4 |
| Tier 2 | #4 Floating panels | 🔲 Deferred to v0.82 | — |
| Tier 2 | #6 Inspector multi-select | 🔲 Deferred to v0.82 | — |
| Tier 3 | #7 Tab groups | 🔲 Deferred | — |
| Tier 3 | #8 Asset browser thumbnails | 🔲 Deferred | — |
| Tier 3 | #9 Welcome tour step-through | 🔲 Deferred | — |


Estimated total: 5-7 weeks for one dev, 3-5 weeks for two devs in parallel.
