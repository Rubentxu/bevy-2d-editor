# ROADMAP Addendum — v0.81 Candidates (Historical)

> **Historical — superseded by `docs/ROADMAP_addendum_v0.86.md`.** All
> Tier 1 / Tier 2 candidates were shipped in v0.81.0, v0.82.0, and
> v0.83.0; deferred items #7 (Tab groups) and #9 (Welcome tour) remain
> open and are tracked in the active v0.86 addendum. This file is
> preserved for traceability.

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

| Tier   | Item                           | Status                                                      | PR                                                          | Merge commit |
| ------ | ------------------------------ | ----------------------------------------------------------- | ----------------------------------------------------------- | ------------ |
| Tier 1 | #3 Global search               | ✅ Shipped in v0.81.0                                       | [#113](https://github.com/Rubentxu/bevy-2d-editor/pull/113) | `854f1d7`    |
| Tier 1 | #2 Workspace presets           | ✅ Shipped in v0.81.0                                       | [#114](https://github.com/Rubentxu/bevy-2d-editor/pull/114) | `26410e2`    |
| Tier 1 | #1 Drag-and-dock infra         | ✅ Shipped in v0.81.0 (region-swap in v0.82)                | [#115](https://github.com/Rubentxu/bevy-2d-editor/pull/115) | `096a865`    |
| Tier 2 | #5 Per-panel state persistence | ✅ Shipped in v0.81.0 (collapse flags + OPFS schemaVersion) | [#112](https://github.com/Rubentxu/bevy-2d-editor/pull/112) | `6d36768`    |
| Tier 2 | #10 Drag-to-resize status bar  | ✅ Shipped in v0.81.0 (clamp 20–48 px)                      | [#112](https://github.com/Rubentxu/bevy-2d-editor/pull/112) | `6d36768`    |
| Tier 2 | #4 Floating panels             | ✅ Shipped in v0.82.0 (ADR-0025)                            | [#117](https://github.com/Rubentxu/bevy-2d-editor/pull/117) | `abde2cb`    |
| Tier 2 | #6 Inspector multi-select      | ✅ Shipped in v0.82.0 (ADR-0025)                            | [#118](https://github.com/Rubentxu/bevy-2d-editor/pull/118) | `364cc32`    |
| Tier 3 | #7 Tab groups                  | 🔲 Deferred                                                 | —                                                           | —            |
| Tier 3 | #8 Asset browser thumbnails    | ✅ Shipped in v0.83.0 (ADR-0026)                            | [#119](https://github.com/Rubentxu/bevy-2d-editor/pull/119) | `9da7683`    |
| Tier 3 | #9 Welcome tour step-through   | 🔲 Deferred                                                 | —                                                           | —            |

**Tag**: `v0.81.0` anchored on `6d36768` (merge commit of PR #112).
**Bundle**: 348.78 KB gzip (target ≤ 350 KB).
**Tests**: Playwright 126 passed / 2 skipped, Rust 638 passed.

### Schema migration impact (PR #112)

PR #112 added `schemaVersion` and `statusBar` to `DockPrefs`. The original
Tier 1 PRs (#113, #114, #115) had independently evolved `useDockPrefs.ts`;
during rebase the two schema-evolution paths were unified onto a single
`migratePrefs` helper that also normalises `activePreset` and `presets`.
The legacy `mergeWithDefaults` helper was removed (zero callers). All
v0.80 OPFS `dock-prefs.json` files upgrade in-place on next load.

## v0.82 candidates (carry-over)

| Priority | Item                                     | Effort    | Why this order                                                                                                 |
| -------- | ---------------------------------------- | --------- | -------------------------------------------------------------------------------------------------------------- |
| 1        | Drag-and-dock region-swap (completes #1) | 1 week    | v0.81 already shipped the HTML5 draggable primitives + drop visual; the swap hook is the missing runtime piece |
| 2        | Floating panels (#4)                     | 1 week    | Closes the React Portal + z-index work; enables the undock-to-window UX                                        |
| 3        | Inspector multi-select (#6)              | 1 week    | Requires `SetComponentFieldOnMultiple` command extension; useful for bulk property changes                     |
| 4        | Tab groups inside docks (#7)             | 1–2 weeks | Risky — may break the spatial stability principle; needs a UX spike                                            |
| 5        | Asset browser thumbnails (#8)            | 3 days    | Reads bytes via existing `read_asset_file_bytes`; small scope                                                  |
| 6        | Welcome tour step-through (#9)           | 1 week    | Onboarding for first-time users; uses `react-joyride` or build minimal                                         |

Estimated total: 5-7 weeks for one dev, 3-5 weeks for two devs in parallel.
