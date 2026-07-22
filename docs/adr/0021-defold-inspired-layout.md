# ADR-0021: Defold-Inspired Layout + F-Key Shortcuts

## Status

Accepted (2026-07-22) — Hito 5 / `defold-inspired-redesign` cycle (v0.80.0)

## Context

After Phase 1-5 of the `ux-overhaul` cycle (commits cdff2e5..9303244, v0.78.0), the editor had:
- All functional capabilities wired (command palette, cheatsheet, themes, toasts, drag-drop, onboarding).
- A token-based design system with OKLCH colors, fonts, and motion.
- Accessibility (axe 0 critical/serious), bundle under budget (337 KB gzip).

But the **spatial layout** was still a problem. A user opening the app saw a topbar with 12 buttons, then a 3-panel squished layout (Hierarchy | Inspector | canvas). Assets were hidden inside modal panels that hijacked the right side one at a time. There was no project navigator visible by default.

Defold's editor layout is the gold standard for browser-runnable 2D editors:
1. **Three regions always visible**: Assets (left), Scene + Tools (center), Outline + Properties (right).
2. **Each region independently toggleable** (F6/F7/F8).
3. **Menu bar (File / Edit / View / Tools / Help)** instead of buttons.
4. **Status bar with rich info** (cursor position, zoom, fps, build status, version).

## Decision

Adopt the Defold 3-region spatial layout as the canonical editor structure. The new layout is:

```
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│ 🎮 MyGame │ File  Edit  View  Tools  Run  Help                  [☀] [🔍] [▶ Play]           │ ← MenuBar (replaces TopBar)
├──────────┬───────────────────────────────────────────────┬─────────────────────────────┤
│  Assets  │  SceneTabs [Level1] [Level2]                 │   Outline                    │
│  📁      │  ┌─────────────────────────────────────────┐  │   🔍 Search                  │
│   Scenes │  │                                         │  │   ▾ 🌳 World                │ ← drag-resizable
│   Asset…│  │              Scene Viewport             │  │     ▸ 🦸 Hero               │   (CSS vars
│   Logic  │  │                                         │  ├─────────────────────────────┤   persisted to OPFS)
│   Code   │  │                                         │  │   Properties                 │
│   Tile… │  │                                         │  │   🔍 Search                  │
│   Files │  │                                         │  │   ▾ Transform2D     ●        │
│          │  │                       ⊕  + − ⌂ ⛶        │  │                             │
├──────────┼───────────────────────────────────────────────┤                             │
│          │ 📋 Console  🔍 Search  📤 Output  ⚠ 3        │                             │ ← F7 toggles
│          │ [12:34] Starting Bevy engine...              │                             │
├──────────┴───────────────────────────────────────────────┴─────────────────────────────┤
│ (120, 80)│3 entities│MyGame│Level1●dirty│100%│60 fps│✓Built v0.80.0               │ ← 7-segment status bar
└──────────────────────────────────────────────────────────────────────────────────────────────┘
   F6=toggle Assets   F7=toggle Tools   F8=toggle Outline   Shift+F8=toggle Properties   F9=fullscreen
```

### Architecture

- **CSS Grid host** (`DockLayout.tsx`): 3 columns × 4 rows. Source of truth for sizes is `--dock-left-w`, `--dock-right-w`, `--dock-bottom-h` CSS custom properties.
- **Drag-resizable dividers** (`DockDivider.tsx`): pointer events + RAF-throttled updates + double-click reset.
- **OPFS persistence** (`useDockPrefs.ts`): `dock-prefs.json` with `{left: {width, visible}, right: {width, topHeight, outlineVisible, propertiesVisible}, bottom: {height, visible}}`. 500ms debounced save.
- **Synchronous bootstrap** in `main.tsx`: read prefs before first paint to avoid layout flash.

### F-Key Shortcuts (Defold convention)

| Shortcut | Action |
|---|---|
| **F6** | Toggle Assets dock (left) |
| **F7** | Toggle Tools dock (bottom) |
| **F8** | Toggle Outline (top half of right dock) |
| **Shift+F8** | Toggle Properties (bottom half) |
| **F9** | Fullscreen viewport (hide all docks except center; keep menu + status) |

### Status Bar (7 segments)

| Segment | Position | Click Action |
|---|---|---|
| Mouse world pos | `120.5, 80.0` | Opens zoom menu |
| Selection info | `3 entities · 1 instance` | Opens hierarchy search |
| Project path | `/projects/MyGame` | Opens recent projects |
| Scene + dirty | `Level1 ●dirty` | Opens scene tabs menu |
| Zoom | `100%` | Opens zoom menu |
| FPS | `60 fps · 16.6 ms` | Opens perf graph |
| Build | `✓ Built · v0.80.0` | Opens build menu |

### Welcome Overlay

First-visit overlay with 5 workflow cards. Persisted in OPFS as `welcome-dismissed.json`. Has `?skip-welcome=1` URL opt-out for tests.

## Consequences

### Positive

- **Spatial stability**: the layout of Assets/Scene/Outline/Properties/Tools/Status doesn't change as the user works. Panels toggle in/out, never modal-jack.
- **Familiarity**: anyone who has used Defold/Unity/Construct/VS Code recognizes the layout within 5 seconds.
- **Discoverability**: every panel has a visible header (title + collapse + close ×). No hidden modals.
- **Resizability**: each vertical divider is draggable. Widths persist across reloads.
- **Professional menu bar**: File/Edit/View/Tools/Run/Help dropdowns with shortcut hints. Replaces 12-button toolbar.
- **Status bar density**: 7 clickable segments with rich info.

### Negative

- **CSS Grid + drag-resize is non-trivial**: 4px hit targets, RAF throttling, OPFS debouncing. Mitigated by extracting `useDockResize` + `useDockPrefs` hooks.
- **Bundle delta**: +0.39 KB gzip JS (337.4 KB → 337.79 KB). Still well under the 400 KB budget.
- **Test suite drift**: 2 capabilities-smoke tests had to be updated (Code/Logic selectors moved from toolbar buttons to menu items).
- **Welcome overlay pointer-intercept**: a fullscreen backdrop on first visit intercepted pointer events in 9 of 10 non-welcome specs. Mitigated by `?skip-welcome=1` query-string opt-out honored in 9 specs; dedicated `ux-welcome.spec.ts` covers the overlay lifecycle.

### Neutral

- **Dock state in OPFS**: persisted per-browser (not synced across devices). Defold also stores layout per-installation.
- **Welcome overlay can't be dismissed via menu**: it's only shown on first visit. Users can re-trigger via Help > Welcome Tour menu item (TODO v0.81).

## Alternatives Considered

### Alt 1: Floating panels (Defold-style)

Allow panels to be undocked and float over the editor as separate windows. **Rejected** for v0.80 because:
- Adds ~30 KB of bundle (window management library).
- Complexity for drag-between-regions detection.
- Better as a v0.81 follow-up once the dock system is stable.

### Alt 2: Workspace presets (2D Platformer / FPS / Top-Down)

Predefined layout configurations that users can switch between. **Rejected** for v0.80 because:
- The dock prefs already let users save their own layout.
- Workspace presets are a v0.81+ feature (requires UI for managing presets).

### Alt 3: Tab groups inside docks (Chrome-style)

Multiple panels share a single dock region via tabs. **Rejected** for v0.80 because:
- The current one-panel-per-region is sufficient for the editor's 5 main surfaces (Assets/Outline/Properties/Console/Search).
- Tab groups would complicate the drag-resize model.

### Alt 4: VS Code Activity Bar (icons on a thin left rail)

Replicate VS Code's icon rail for switching between Explorer/Search/Source Control/Panel. **Rejected** for v0.80 because:
- Defold's 3-region layout is more discoverable for first-time users.
- The Activity Bar would be a v0.81+ addition (or alternative).

## Implementation Notes

### CSS Grid math

```css
.dock-layout {
  display: grid;
  grid-template-columns: var(--dock-left-w, 280px) 1fr var(--dock-right-w, 320px);
  grid-template-rows: var(--menu-h, 40px) 1fr var(--dock-bottom-h, 240px) var(--status-h, 24px);
  grid-template-areas:
    "menu menu menu"
    "left center right"
    "left bottom right"
    "status status status";
  height: 100vh;
}

[data-fullscreen="true"] .dock-layout {
  grid-template-rows: var(--menu-h, 40px) 1fr var(--status-h, 24px);
  grid-template-areas:
    "menu menu menu"
    "left center right"
    "status status status";
}
```

### Drag-resize

```typescript
// In useDockResize
const setLeftWidth = useCallback((w: number) => {
  const clamped = Math.max(200, Math.min(500, w));
  document.documentElement.style.setProperty('--dock-left-w', `${clamped}px`);
  setDockPrefs((p) => ({ ...p, left: { ...p.left, width: clamped } }));
  schedulePersist();  // 500ms debounced save to OPFS
}, []);
```

### Bootstrap (avoid first-paint flash)

```typescript
// In main.tsx (synchronous)
const dockPrefsJson = opfsLoadFileSync('dock-prefs.json');  // try sync read
if (dockPrefsJson) {
  const prefs = JSON.parse(dockPrefsJson);
  document.documentElement.style.setProperty('--dock-left-w', `${prefs.left.width}px`);
  // ... etc
}
```

## Testing

### Test coverage added

- `ux-menubar.spec.ts` (3 tests): 6 menu headers · File dropdown · Escape close
- `ux-dock.spec.ts` (9 tests): 3-region layout · widths · drag divider · 7 status segments · zoom dropdown · F6/F9/Reset Layout · dock tabs
- `ux-welcome.spec.ts` (3 tests): first-visit · Skip · Take the tour

### Total

- **Baseline**: 40 tests, 337.4 KB gzip JS
- **Final**: 55 tests (+15), 337.79 KB gzip JS (+0.39 KB)
- All Rust tests still green (637).
- axe-core 0 critical/serious violations.

## Out of scope (defer)

- Drag-and-drop docking between regions
- Workspace presets (FPS / 2D Platformer / Top-Down)
- Floating panels
- Multi-window editor
- Tab groups inside docks
- Layout export/import

## References

- SDDK artifacts: `sddk/defold-inspired-redesign/{proposal.md, spec.md, design.md, tasks.md}`
- Defold editor overview: <https://defold.com/manuals/editor/>
- Defold keyboard shortcuts: <https://forum.defold.com/t/toggle-surrounding-editor-panes/49104>
- Related ADRs: 0005 (Scene Asset model), 0019 (OPFS catalog persistence)
- Commits: 7df24f5 (Phase A), 9ae4f86 (Phase B), 034eea0 (Phase C), c034dc4 (Phases D+E)
