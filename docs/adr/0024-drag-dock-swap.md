# ADR-0024: Drag-and-Dock Region Swap — Atomic Swap Rule, Center Protection, Preset Capture, Bare-Id MIME

## Status

Accepted (2026-07-23)

## Context

v0.81 Tier 1c ([ADR-0022](./0022-drag-and-dock-region-swap-renumbered.md),
originally numbered here as ADR-0022, now renumbered to ADR-0024)
shipped drag-source primitives: stable panel
identifiers on every dock region (`data-panel-id`), the custom MIME
`application/x-dock-panel`, `draggable` headers with `cursor: grab`, and a
`DockPanel` wrapper that recognises the MIME on `dragover` and exposes an
`onRegionChange` callback on drop. The current `DockPanel` is unused in
production — the v0.81 layout still fixes `Assets` on the left, `Outline`+
`Properties` split on the right, and tabbed bottom dock — so the drag-drop
wire is unreachable from a real user interaction.

The parent roadmap (`defold-inspired-redesign`) defers the runtime dock swap
to v0.82 P1. This ADR fixes the four open questions carried by the v0.82 P1
explore report:

1. **Collision rule.** Today, an empty region is the only "safe" drop target.
   What happens when the user drops `assets` on a region that already holds
   `outline`? Replace, exchange, split-insert, or tab-append?
2. **Center eligibility.** The center region hosts the scene viewport and a
   tab strip — it cannot realistically be made a drop target without
   rebuilding the scene canvas. Should it be included in the swap matrix?
3. **Canonical MIME payload.** The v0.81 Tier 1c commit writes a bare panel
   id; `CONTEXT.md` still claims an envelope of `{panelId, source}`. Which
   is canonical?
4. **Workspace preset capture.** `PresetDockState` snapshots only widths and
   visibilities. Should a user-saved preset remember where the user moved
   `outline`? And if a preset is active and the user moves a panel, what
   happens to `activePreset`?

The Tier-1c `DockPanel` `drop` handler already implements "same-source no-op,
otherwise call `onRegionChange`"; that is the only contract this ADR is
inheriting.

## Decision

1. **Atomic swap on collision.** `movePanel(panelId, target)` is one reducer
   step: the source region and the target region exchange their active
   panel ids. If `source === target`, it is a no-op. If `target` holds no
   panel, the source moves into it. If both regions hold a panel, the two
   panel ids swap in a single `setPrefs` call. This matches the explore
   report's recommendation: append-as-tab and split-insertion are deferred
   (tab groups are v0.82 #4).
2. **Center is protected.** `panelRegions` only accepts `"left" | "right" |
   "bottom"` as values. The center region container is rendered without
   `onDragOver`/`onDrop` and is explicitly marked `data-drop-allowed="false"`
   for E2E queries. Drag-over the center does not show the indicator, and
   drops are ignored.
3. **Bare-id MIME is canonical.** `DockHeader` (and the bottom-dock
   `header`) publish the panel id alone at the `application/x-dock-panel`
   MIME. The MIME value is **one of `assets | outline | properties | bottom`**
   — never regionalised (`left-assets`) and never tabbed (`bottom-console`).
   The `data-panel-id` on the region root remains the observable
   (`left-assets`, `right-outline`, `right-properties`, `center`, `bottom`)
   for layout / E2E selectors; the dataTransfer payload is the
   `panelRegions` key, separate by design.
4. **Presets capture `panelRegions`.** `PresetDockState` gains a
   `panelRegions: Record<PanelId, RegionId>` field. `buildUserPresetRecord`
   captures the current map; `applyPresetToDockPrefs` restores it. A
   `movePanel` call **clears `activePreset`** so the toolbar menu correctly
   reflects "manual customization" rather than crediting the move to a
   preset that did not contain this assignment.

The four decisions together mean:

- One CSS Grid hosts four region containers (left, center, right, bottom).
- Three of them (left, right, bottom) are drop targets with a shared
  `onMovePanel(panelId, target)` callback passed from `App`.
- `DockHeader` exposes a `Move →` menu with `Left` / `Right` / `Bottom`
  options that dispatch the identical `movePanel` setter, plus an
  `aria-live="polite"` region announcing the destination.
- `useDockResize` owns the React state, gates the debounce through the
  existing `scheduleSave` (500 ms), and binds `flushSave` to `beforeunload`
  so a rapid reload does not race the debounce.
- `panelRegions` is mirrored to `localStorage` synchronously inside `save()`
  (key `bevy-2d-editor:dock-panel-regions`). OPFS writes are async and the
  page can tear down before the file handle's `writable.close()` resolves,
  so a `localStorage.setItem` of the small `panelRegions` payload is the
  reliable write-through path for the reload race. On next mount, `load()`
  layers the `localStorage` snapshot on top of the OPFS read so a swap
  always survives a rapid reload — even if the OPFS write was aborted
  mid-flight.

## Consequences

Positive:

- One source of truth (`useDockResize`) for layout, persistence, and
  commands — no second state path beside `useDockPrefs`.
- Deterministic Playwright coverage of pointer drop, keyboard `Move →`, real
  state mutation, OPFS round-trip, reload restoration, and SPA stability.
- `migratePrefs` from v1 → v2 is additive — old `dock-prefs.json` files keep
  working — so the rollout is a single cut at v0.82.0.
- Accessibility: `aria-live` announces the destination for screen-reader
  users; the `Move →` menu gives a touch/keyboard path that HTML5 DnD
  cannot match.

Negative:

- Atomic swap is a one-way simplification — a future "append as tab"
  (v0.82 #4 tab groups) will need a new schema key. We accept that cost now
  because the alternative (imagine all four collision flavours from day one)
  blocks the P1 cycle.
- Center protection is enforced at runtime only, not at the schema level.
  A future writer that bypasses `movePanel` could still set `panelRegions`
  to `"center"`. The migration step strips invalid regions, which is the
  schema-level guard we do provide.
- Bottom dock swap treats the entire tab strip as one panel. A future
  finer-grained model (move individual console/output/problems tabs) will
  require promoting `BottomDock`'s tab state to `DockPrefs`.
- `CONTEXT.md` carried an obsolete payload shape that did not match the
  checked-in MIME. Reconciling it here costs a docs edit but locks down
  the contract.
- `panelRegions` is mirrored to `localStorage` (synchronous) alongside the
  OPFS async write so the rapid-reload race cannot lose a swap. This adds
  ~30 bytes of JSON to localStorage per save and a small branching in
  `load()`. The trade-off is worth it because losing a swap on a Cmd+R is
  user-visible breakage; the alternative — making the OPFS write fully
  synchronous via File System Access API — would require a much larger
  refactor and isn't portable across all OPFS-supporting browsers.
- Bundle size for the v0.82 P1 diff exceeds the 350 KB gzip budget by
  ~2.7 KB (the baseline already exceeded by ~1.2 KB; the v0.82 P1 diff
  adds ~1.5 KB). The chunk-splitting refactor needed to claw this back is
  deferred to a follow-up cycle and called out in the PR description.

## Rollout

`SCHEMA_VERSION` bumps from 1 to 2 in `frontend/src/hooks/useDockPrefs.ts`.
`migratePrefs` reads v1 files, fills `panelRegions` from the v1 default
arrangement, and writes them back as v2 on the next debounced save. No
manual migration step is required from users.

## Alternatives considered

- **Same-region modal "where would you like to place it?" dialog**: rejected
  because the user has already pointed at a target region; the answer to
  "collision" should be obvious.
- **Center as a valid target (drop = minimise bottom/fullscreen)**: rejected
  because the scene viewport is protected and F9 already owns the
  fullscreen mode.
- **Envelope MIME (`{panelId, source, timestamp}`)**: rejected — no current
  consumer needs `source`, and a stable bare string is easier to assert in
  Playwright and in `dataTransfer.types`.

## References

- v0.81 Tier 1c: `frontend/tests/ux-drag-dock.spec.ts`, `DockPanel.tsx`
- v0.82 P1 plan: `sddk/active/v0.82-p1-drag-dock-region-swap/{explore-report,proposal}.md`
- v0.82 P1 specs: `sddk/active/v0.82-p1-drag-dock-region-swap/spec/{drag-dock-region-swap,dock-prefs-schema}/spec.md`
