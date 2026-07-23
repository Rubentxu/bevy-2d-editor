/**
 * useDockPrefs — load/save dock layout preferences to OPFS.
 *
 * Phase B (Defold-inspired redesign): persists dock widths/heights/visibility
 * to `dock-prefs.json` in OPFS. The hook exposes a tiny, dependency-free
 * service surface: `load()` reads (returns null if missing or OPFS
 * unavailable), `save(prefs)` writes JSON, `scheduleSave(prefs, ms)`
 * debounces.
 *
 * v0.81 Tier 1b (Workspace Presets) extends the persisted envelope with
 * `activePreset` (the id of the currently-applied preset, if any) and
 * `presets` (a map of user-saved preset id → preset record). The pure
 * helpers `applyPreset`, `saveCurrentAsPreset` and `deleteUserPreset`
 * transform the prefs without holding any state of their own — the caller
 * (e.g. `useDockResize`) owns React state and re-issues `save()` on its
 * normal debounce path.
 *
 * v0.82 P1 (drag-and-dock region swap, ADR-0024) extends the envelope with
 * `panelRegions` and bumps `schemaVersion` to `2`. The shape is fixed by
 * the P1 spec: every canonical panel id (`assets`, `outline`, `properties`,
 * `bottom`) maps to exactly one of `left`, `right`, `bottom`. The center
 * region is protected at runtime — see DockLayout's `data-drop-allowed`
 * — but `migratePrefs` discards any `"center"` value it finds. The pure
 * reducer `movePanel(prefs, panelId, target)` implements atomic swap; the
 * companion `flushSave()` helper cancels any pending debounce and writes
 * the latest prefs synchronously so a rapid reload never races the
 * 500 ms debounce.
 *
 * v0.82 P2 (floating panels, ADR-0025) adds `floats` — a per-panel
 * `FloatingPanelState` (`{ x, y, width, height, last_floated_at }`) for
 * panels that have been lifted out of the CSS-Grid layout into a
 * free-positioned portal overlay. `schemaVersion` bumps from `2` to `3`;
 * `migratePrefs` performs a lossless v2 → v3 migration by filling
 * `floats = {}` for users upgrading from v0.82 P1.
 *
 * Persistence is best-effort: errors are logged but never thrown so a
 * broken OPFS layer cannot crash the editor.
 */

import { useCallback, useRef } from "react";
import { opfsLoadFile, opfsSaveFile } from "../opfs-bridge";
import {
  applyPresetToDockPrefs,
  buildUserPresetRecord,
  derivePresetId,
  type UserPresetRecord,
} from "../data/workspacePresets";

const DOCK_PREFS_PATH = "dock-prefs.json";
/**
 * localStorage key holding the synchronous write-through cache of the
 * critical subset of `DockPrefs` that must survive a rapid reload even
 * when the OPFS async write hasn't flushed yet (ADR-0024 §Consequences,
 * extended in ADR-0025 to also cover the new `floats` slice). The payload
 * is intentionally small — just the `panelRegions` map plus the `floats`
 * map — to keep localStorage usage minimal and avoid duplicating the
 * larger workspace state held in OPFS.
 */
const DOCK_PREFS_LS_KEY = "bevy-2d-editor:dock-panel-regions";
const DEBOUNCE_MS = 500;

/**
 * Bump this whenever a new top-level key is added to `DockPrefs` so old prefs
 * files can be migrated gracefully (see `migratePrefs` below). v0.81 ships
 * v1; v0.82 P1 bumps to v2 (ADR-0024) to add `panelRegions`; v0.82 P2
 * bumps to v3 (ADR-0025) to add `floats`.
 */
export const SCHEMA_VERSION = 3;

/**
 * Persisted state for a single floating panel — the panel's last-known
 * `position: fixed` rect plus the timestamp at which it was most recently
 * floated. The frontend (`FloatingPanel.tsx`, ADR-0025) writes new
 * values here on every drag-drop / dock-toggle interaction; the loader
 * picks them up on next mount.
 */
export interface FloatingPanelState {
  x: number;
  y: number;
  width: number;
  height: number;
  /** Epoch millis when the panel was last floated (or its float rect was
   * last updated). Used by the frontend to surface a "last floated"
   * indicator in the panel header; does not gate any logic. */
  last_floated_at: number;
}

/**
 * Canonical panel identifiers carried in `DockPrefs.panelRegions` and over
 * the drag MIME. They are deliberately bare (no regional prefix like
 * `left-assets`) so the same id routes through every layer of the dock
 * subsystem — React state, the movePanel reducer, the keyboard Move →
 * menu, the dataTransfer payload, and the migration logic. The DOM
 * continues to expose the legacy `data-panel-id` selectors (`left-assets`,
 * `right-outline`, etc.) so the v0.81 Tier 1c E2E tests keep their
 * existing wiring without churn.
 */
export type PanelId = "assets" | "outline" | "properties" | "bottom";

/**
 * Dockable regions. `center` is intentionally absent — it hosts the scene
 * viewport and stays protected at runtime (the layout renders no DnD
 * handlers on the center container and `migratePrefs` discards any legacy
 * `"center"` value it encounters).
 */
export type DockableRegion = "left" | "right" | "bottom";

/**
 * Default `panelRegions` arrangement. Mirrors the v0.81 fixed layout:
 * `assets` left; `outline` + `properties` right; `bottom` bottom. The
 * reducer's atomic-swap preserves the invariant that every PanelId has
 * exactly one region mapping.
 */
export const DEFAULT_PANEL_REGIONS: Record<PanelId, DockableRegion> = {
  assets: "left",
  outline: "right",
  properties: "right",
  bottom: "bottom",
};

const VALID_REGIONS: ReadonlySet<DockableRegion> = new Set([
  "left",
  "right",
  "bottom",
]);

export interface DockPrefs {
  schemaVersion: number;
  statusBar: { height: number };
  /** id of the preset last applied; null after a manual edit. */
  activePreset?: string | null;
  left: { width: number; visible: boolean };
  right: {
    width: number;
    visible: boolean;
    outlineVisible: boolean;
    outlineCollapsed: boolean;
    propertiesVisible: boolean;
    propertiesCollapsed: boolean;
    topHeight: number;
  };
  bottom: { height: number; visible: boolean };
  /**
   * Panel-to-region assignment (ADR-0024). Every key in the v2 default set
   * is guaranteed to map to a valid `DockableRegion` after a migration
   * pass — readers do not need to tolerate missing or invalid entries.
   */
  panelRegions: Record<PanelId, DockableRegion>;
  /**
   * Per-panel floating state (ADR-0025). A panel id appears here only
   * while that panel is *floating* — i.e. lifted out of the CSS-Grid
   * layout into a `createPortal(…, document.body)` overlay. The
   * presence of an entry corresponds to "this panel is currently
   * floating" in `App.tsx`'s `floatingPanelIds` set; the entry's
   * payload holds the rect to restore on reload.
   *
   * Migrated from v2 to v3 by filling `floats = {}` (no panels float
   * by default for users upgrading from v0.82 P1).
   */
  floats: Partial<Record<PanelId, FloatingPanelState>>;
  /** User-defined presets keyed by id. Built-ins are not stored here. */
  presets?: Record<string, UserPresetRecord>;
}

export const DEFAULT_DOCK_PREFS: DockPrefs = {
  schemaVersion: SCHEMA_VERSION,
  statusBar: { height: 24 },
  activePreset: "default",
  left: { width: 280, visible: true },
  right: {
    width: 320,
    visible: true,
    outlineVisible: true,
    outlineCollapsed: false,
    propertiesVisible: true,
    propertiesCollapsed: false,
    topHeight: 60,
  },
  bottom: { height: 240, visible: true },
  panelRegions: { ...DEFAULT_PANEL_REGIONS },
  floats: {},
  presets: {},
};

/**
 * Pure reducer implementing the atomic-swap rule from ADR-0024 §Decision 1.
 *
 * - `panelId` is unknown → return `prefs` unchanged.
 * - Source already in `target` → no-op (idempotent same-region drop).
 * - `target` is empty (no panel maps to it) → just re-home `panelId` there.
 * - `target` already holds a panel (`other`) → exchange `panelId` and
 *   `other` in one immutable update so React sees a single state
 *   transition and the OPFS save writes once.
 *
 * Always clears `activePreset` to surface the manual-customization state in
 * the workspace-preset menu (ADR-0024 §Decision 4).
 */
export function movePanel(
  prefs: DockPrefs,
  panelId: PanelId,
  target: DockableRegion,
): DockPrefs {
  if (!Object.prototype.hasOwnProperty.call(prefs.panelRegions, panelId)) {
    return prefs;
  }
  const current = prefs.panelRegions[panelId];
  if (current === target) return prefs;

  // Find which panel (if any) currently occupies the target region. If a
  // collision occurs we swap into it; if the target is empty we just re-home
  // `panelId` and leave the source region empty (with its other panels
  // untouched).
  const occupant =
    (Object.entries(prefs.panelRegions) as [PanelId, DockableRegion][]).find(
      ([, region]) => region === target,
    )?.[0] ?? null;

  if (occupant === null) {
    // Empty destination — move the source panel there without affecting
    // the regions of the other panels.
    return {
      ...prefs,
      panelRegions: { ...prefs.panelRegions, [panelId]: target },
      activePreset: null,
    };
  }

  if (occupant === panelId) return prefs;

  // Collision — atomic swap.
  return {
    ...prefs,
    panelRegions: {
      ...prefs.panelRegions,
      [panelId]: target,
      [occupant]: current,
    },
    activePreset: null,
  };
}

/**
 * Migrate a parsed `dock-prefs.json` payload into the current schema.
 *
 * - Missing keys → fill with defaults.
 * - Old schemaVersion → log a one-time warning so developers can spot drift.
 * - Invalid `panelRegions` entries (`"center"`, unknown ids, mismatched
 *   types) → discard and fall back to defaults per ADR-0024 §Consequences.
 * - Missing / invalid `floats` entries → drop (a panel whose entry is
 *   malformed is treated as docked — only well-formed entries restore a
 *   floating overlay). Per ADR-0025 §Decision 5, this branch runs for
 *   users upgrading from v0.82 P1 (no `floats` key on disk) and any
 *   malformed entries written by older experimental builds.
 *
 * Always returns a valid `DockPrefs` (never throws). Callers don't need to
 * know about versioning — `load()` calls this internally.
 */
export function migratePrefs(parsed: unknown): DockPrefs {
  const obj = (parsed ?? {}) as Record<string, unknown>;
  const v = typeof obj.schemaVersion === "number" ? obj.schemaVersion : 0;
  if (v !== SCHEMA_VERSION) {
    console.warn(
      `[useDockPrefs] migrating prefs from v${v} → v${SCHEMA_VERSION}`,
    );
  }
  const left = {
    ...DEFAULT_DOCK_PREFS.left,
    ...((obj.left as object | undefined) ?? {}),
  };
  const right = {
    ...DEFAULT_DOCK_PREFS.right,
    ...((obj.right as object | undefined) ?? {}),
  };
  const bottom = {
    ...DEFAULT_DOCK_PREFS.bottom,
    ...((obj.bottom as object | undefined) ?? {}),
  };
  const statusBar = {
    ...DEFAULT_DOCK_PREFS.statusBar,
    ...((obj.statusBar as object | undefined) ?? {}),
  };
  const activePreset =
    typeof obj.activePreset === "string"
      ? (obj.activePreset as string)
      : (DEFAULT_DOCK_PREFS.activePreset ?? null);
  const presets =
    typeof obj.presets === "object" && obj.presets !== null
      ? (obj.presets as Record<string, UserPresetRecord>)
      : {};

  // Build `panelRegions` defensively: every required id must map to one of
  // the three `DockableRegion` values. Defaults catch anything missing.
  const rawRegions =
    typeof obj.panelRegions === "object" && obj.panelRegions !== null
      ? (obj.panelRegions as Record<string, unknown>)
      : {};
  const panelRegions: Record<PanelId, DockableRegion> = {
    ...DEFAULT_PANEL_REGIONS,
  };
  for (const id of Object.keys(DEFAULT_PANEL_REGIONS) as PanelId[]) {
    const candidate = rawRegions[id];
    if (
      typeof candidate === "string" &&
      VALID_REGIONS.has(candidate as DockableRegion)
    ) {
      panelRegions[id] = candidate as DockableRegion;
    }
    // Invalid candidate → keep the default already in `panelRegions[id]`.
  }

  // Build `floats` defensively per ADR-0025 §Decision 5. v2 files have
  // no `floats` key, so we start empty; v3 files contribute any persisted
  // entries, but every entry must have positive numeric x/y/width/height
  // and a numeric `last_floated_at` — malformed entries are dropped.
  const floats: Partial<Record<PanelId, FloatingPanelState>> = {};
  const rawFloats =
    typeof obj.floats === "object" && obj.floats !== null
      ? (obj.floats as Record<string, unknown>)
      : {};
  for (const id of Object.keys(rawFloats) as PanelId[]) {
    const candidate = rawFloats[id];
    if (!candidate || typeof candidate !== "object") continue;
    const c = candidate as Record<string, unknown>;
    if (
      typeof c.x === "number" &&
      typeof c.y === "number" &&
      typeof c.width === "number" &&
      typeof c.height === "number" &&
      typeof c.last_floated_at === "number" &&
      // Width/height must be positive so the panel can be rendered.
      c.width > 0 &&
      c.height > 0
    ) {
      floats[id] = {
        x: c.x,
        y: c.y,
        width: c.width,
        height: c.height,
        last_floated_at: c.last_floated_at,
      };
    }
    // Malformed entry → skip; the panel stays docked.
  }

  return {
    schemaVersion: SCHEMA_VERSION,
    left,
    right,
    bottom,
    statusBar,
    activePreset,
    panelRegions,
    floats,
    presets,
  };
}

export function useDockPrefs() {
  const timerRef = useRef<number | null>(null);
  // Latest prefs captured so `flushSave()` can write the most recent state
  // synchronously. The caller (`useDockResize`) keeps a ref of its own too;
  // this one is the bridge from the persistence service to beforeunload.
  const latestRef = useRef<DockPrefs | null>(null);

  /**
   * Synchronous bootstrap (Phase E, §E.5): try to read `dock-prefs.json`
   * during the initial bundle load so we can apply the CSS custom properties
   * (`--dock-left-w` / `--dock-right-w` / `--dock-bottom-h` / `--status-h`)
   * BEFORE the first paint. OPFS is normally async, so this best-effort
   * relies on a tiny race: we try to resolve the OPFS directory handle
   * synchronously via the `navigator.storage.getDirectory()` →
   * `getFileHandle()` chain wrapped in a non-awaited Promise. If it doesn't
   * resolve in time the defaults stand and `useDockResize` will apply the
   * persisted values once the async OPFS call completes.
   *
   * NOTE: this is a *best effort* that mostly helps when OPFS is cached
   * from a previous session; the React state in `useDockResize` remains the
   * source of truth.
   */
  const applyBootstrap = useCallback((prefs: DockPrefs) => {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.style.setProperty("--dock-left-w", `${prefs.left.width}px`);
    root.style.setProperty("--dock-right-w", `${prefs.right.width}px`);
    root.style.setProperty("--dock-bottom-h", `${prefs.bottom.height}px`);
    root.style.setProperty("--status-h", `${prefs.statusBar.height}px`);
  }, []);

  /**
   * Read the synchronous localStorage fallback holding the most recent
   * `panelRegions` + `floats` snapshot. Used to recover from the
   * rapid-reload race where the OPFS async write hasn't completed
   * before the page tears down (ADR-0024 §Consequences, extended in
   * ADR-0025 to also cover `floats`). Returns null when localStorage is
   * unavailable or no snapshot exists yet.
   */
  const loadFromLocalStorage = useCallback((): {
    panelRegions?: DockPrefs["panelRegions"];
    floats?: DockPrefs["floats"];
  } | null => {
    try {
      if (typeof localStorage === "undefined") return null;
      const raw = localStorage.getItem(DOCK_PREFS_LS_KEY);
      if (!raw) return null;
      const parsed = JSON.parse(raw);
      if (!parsed || typeof parsed !== "object") return null;
      const out: {
        panelRegions?: DockPrefs["panelRegions"];
        floats?: DockPrefs["floats"];
      } = {};
      if (parsed.panelRegions && typeof parsed.panelRegions === "object") {
        out.panelRegions = parsed.panelRegions as DockPrefs["panelRegions"];
      }
      if (parsed.floats && typeof parsed.floats === "object") {
        // Validate each entry — drop malformed entries defensively.
        const floats: Partial<Record<PanelId, FloatingPanelState>> = {};
        for (const id of Object.keys(parsed.floats)) {
          const c = (parsed.floats as Record<string, unknown>)[id];
          if (
            c &&
            typeof c === "object" &&
            typeof (c as { width?: unknown }).width === "number" &&
            typeof (c as { height?: unknown }).height === "number" &&
            (c as { width: number }).width > 0 &&
            (c as { height: number }).height > 0
          ) {
            const cc = c as FloatingPanelState;
            floats[id as PanelId] = {
              x: cc.x,
              y: cc.y,
              width: cc.width,
              height: cc.height,
              last_floated_at: cc.last_floated_at,
            };
          }
        }
        out.floats = floats;
      }
      return out.panelRegions || out.floats ? out : null;
    } catch {
      return null;
    }
  }, []);

  const load = useCallback(async (): Promise<DockPrefs | null> => {
    try {
      const result = await opfsLoadFile(DOCK_PREFS_PATH);
      if (!result.ok || !result.value) {
        // OPFS is empty (first-run or recently cleared). Fall back to the
        // localStorage write-through cache so a previous swap or float
        // survives a rapid reload even when the OPFS file wasn't yet
        // flushed.
        const fallback = loadFromLocalStorage();
        if (fallback) {
          return migratePrefs({
            ...DEFAULT_DOCK_PREFS,
            ...fallback,
          });
        }
        return null;
      }
      const parsed = JSON.parse(result.value);
      // Migrate to the current schema (fills missing keys, logs version
      // mismatch warnings, and normalises activePreset/presets from
      // workspace-presets).
      const migrated = migratePrefs(parsed);
      // Layer the localStorage snapshot on top to win any race where
      // the OPFS write hadn't yet completed when the page reloaded.
      const lsFallback = loadFromLocalStorage();
      if (lsFallback?.panelRegions || lsFallback?.floats) {
        return {
          ...migrated,
          ...(lsFallback.panelRegions
            ? { panelRegions: lsFallback.panelRegions }
            : {}),
          ...(lsFallback.floats ? { floats: lsFallback.floats } : {}),
        };
      }
      return migrated;
    } catch (e) {
      console.warn("[useDockPrefs] load failed:", e);
      return null;
    }
  }, [loadFromLocalStorage]);

  const save = useCallback(async (prefs: DockPrefs): Promise<void> => {
    try {
      // Synchronous localStorage write-through for the small subset of
      // state that must survive a rapid reload (ADR-0024 §Consequences —
      // rapid-reload race). The OPFS write is async and can race the
      // page tear-down; localStorage is synchronous and reliable. On
      // next mount, `load()` falls back to the localStorage snapshot
      // when OPFS hasn't yet flushed.
      //
      // v0.82 P2 (ADR-0025) extends the payload to also mirror the
      // `floats` slice — a floating panel's position is critical to
      // surviving a reload (otherwise the user has to re-float).
      try {
        if (typeof localStorage !== "undefined") {
          localStorage.setItem(
            DOCK_PREFS_LS_KEY,
            JSON.stringify({
              panelRegions: prefs.panelRegions,
              floats: prefs.floats,
            }),
          );
        }
      } catch {
        /* localStorage quota or disabled — non-fatal */
      }
      // Always stamp the current schemaVersion on write so a future reader
      // can detect out-of-date files.
      const stamped: DockPrefs = { ...prefs, schemaVersion: SCHEMA_VERSION };
      await opfsSaveFile(DOCK_PREFS_PATH, JSON.stringify(stamped));
    } catch (e) {
      console.warn("[useDockPrefs] save failed:", e);
    }
  }, []);

  /**
   * Schedule a debounced save. Multiple rapid calls within `ms` collapse
   * into a single write of the most-recent prefs (per OPFS save throttling
   * strategy in tasks.md §B.4). The latest prefs are stashed in
   * `latestRef` so `flushSave()` can complete a pending write immediately.
   */
  const scheduleSave = useCallback(
    (prefs: DockPrefs, ms: number = DEBOUNCE_MS): void => {
      latestRef.current = prefs;
      if (timerRef.current !== null) {
        window.clearTimeout(timerRef.current);
      }
      timerRef.current = window.setTimeout(() => {
        timerRef.current = null;
        void save(prefs);
      }, ms);
    },
    [save],
  );

  /**
   * Cancel any pending debounce and write the latest staged prefs
   * immediately. Used by `useDockResize`'s `beforeunload` listener to
   * guarantee that a rapid reload survives the 500 ms debounce (ADR-0024
   * §Consequences — rapid-reload race). Returns synchronously after the
   * fire-and-forget save is dispatched; an `await`-less reload
   * (`window.location.reload()`) still drains because the OPFS write starts
   * before tear-down.
   */
  const flushSave = useCallback((): void => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    if (latestRef.current !== null) {
      // The synchronous localStorage write inside `save()` happens before
      // the OPFS await chain, so even if the page tears down before the
      // OPFS write completes, the swap state is preserved for the next
      // mount via `loadFromLocalStorage` (ADR-0024 §Consequences).
      void save(latestRef.current);
    }
  }, [save]);

  /**
   * Apply a preset (built-in or user) to the prefs envelope. Pure: returns
   * the same `prefs` reference when no preset matches the id. The caller is
   * responsible for re-rendering with the returned value and triggering a
   * save.
   */
  const applyPreset = useCallback(
    (prefs: DockPrefs, presetId: string): DockPrefs =>
      applyPresetToDockPrefs(prefs, presetId).next,
    [],
  );

  /**
   * Persist the current dock layout under `name` as a new user preset. The
   * returned id is the slug derived from the name; passes it back so the
   * caller can surface a toast like "Saved workspace 'my-layout' (my-layout)".
   */
  const saveCurrentAsPreset = useCallback(
    (prefs: DockPrefs, name: string): { next: DockPrefs; id: string } => {
      const id = derivePresetId(name) || `preset-${Date.now()}`;
      const record = buildUserPresetRecord(
        prefs,
        `Saved at ${new Date().toLocaleTimeString()}`,
      );
      return {
        id,
        next: {
          ...prefs,
          activePreset: id,
          presets: { ...(prefs.presets ?? {}), [id]: record },
        },
      };
    },
    [],
  );

  /**
   * Remove a user-defined preset. Returns the prefs unchanged when `id`
   * is a built-in (built-ins cannot be deleted from the editor). If the
   * caller was on the deleted preset we fall back to "default" so the
   * menu highlights a valid option.
   */
  const deleteUserPreset = useCallback(
    (prefs: DockPrefs, id: string): DockPrefs => {
      if (id === "default") return prefs; // canonical "default" is implicit
      const { [id]: _removed, ...rest } = prefs.presets ?? {};
      return {
        ...prefs,
        presets: rest,
        activePreset:
          prefs.activePreset === id ? "default" : prefs.activePreset,
      };
    },
    [],
  );

  /**
   * Update the floating rect for a single panel. Pure helper — returns a
   * new `DockPrefs` snapshot; the caller (`App.tsx`) is responsible for
   * re-rendering and re-issuing `save()`. v0.82 P2 (ADR-0025).
   */
  const setFloatRect = useCallback(
    (
      prefs: DockPrefs,
      panelId: PanelId,
      rect: FloatingPanelState,
    ): DockPrefs => ({
      ...prefs,
      floats: { ...prefs.floats, [panelId]: rect },
    }),
    [],
  );

  /**
   * Remove a panel from the floats map (i.e. dock it back into the grid).
   * Pure helper — returns the prefs unchanged when the panel is not in the
   * floats map. v0.82 P2 (ADR-0025).
   */
  const removeFloat = useCallback(
    (prefs: DockPrefs, panelId: PanelId): DockPrefs => {
      if (!(panelId in prefs.floats)) return prefs;
      const next = { ...prefs.floats };
      delete next[panelId];
      return { ...prefs, floats: next };
    },
    [],
  );

  return {
    load,
    save,
    scheduleSave,
    flushSave,
    applyBootstrap,
    applyPreset,
    saveCurrentAsPreset,
    deleteUserPreset,
    setFloatRect,
    removeFloat,
  };
}
