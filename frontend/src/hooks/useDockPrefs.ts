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
const DEBOUNCE_MS = 500;

/**
 * Bump this whenever a new top-level key is added to `DockPrefs` so old prefs
 * files can be migrated gracefully (see `migratePrefs` below). v0.81 ships
 * v1; v0.82 will introduce v2 if it adds panel themes or floating-panel
 * positions.
 */
export const SCHEMA_VERSION = 1;

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
  presets: {},
};

/**
 * Migrate a parsed `dock-prefs.json` payload into the current schema.
 *
 * - Missing keys → fill with defaults.
 * - Old schemaVersion → log a one-time warning so developers can spot drift.
 *
 * Always returns a valid `DockPrefs` (never throws). Callers don't need to
 * know about versioning — `load()` calls this internally.
 */
export function migratePrefs(parsed: unknown): DockPrefs {
  const obj = (parsed ?? {}) as Record<string, unknown>;
  const v =
    typeof obj.schemaVersion === "number" ? obj.schemaVersion : 0;
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
      : DEFAULT_DOCK_PREFS.activePreset ?? null;
  const presets =
    typeof obj.presets === "object" && obj.presets !== null
      ? (obj.presets as Record<string, UserPresetRecord>)
      : {};
  return {
    schemaVersion: SCHEMA_VERSION,
    left,
    right,
    bottom,
    statusBar,
    activePreset,
    presets,
  };
}

export function useDockPrefs() {
  const timerRef = useRef<number | null>(null);

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

  const load = useCallback(async (): Promise<DockPrefs | null> => {
    try {
      const result = await opfsLoadFile(DOCK_PREFS_PATH);
      if (!result.ok || !result.value) return null;
      const parsed = JSON.parse(result.value);
      // Migrate to the current schema (fills missing keys, logs version
      // mismatch warnings, and normalises activePreset/presets from
      // workspace-presets).
      return migratePrefs(parsed);
    } catch (e) {
      console.warn("[useDockPrefs] load failed:", e);
      return null;
    }
  }, []);

  const save = useCallback(async (prefs: DockPrefs): Promise<void> => {
    try {
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
   * strategy in tasks.md §B.4).
   */
  const scheduleSave = useCallback(
    (prefs: DockPrefs, ms: number = DEBOUNCE_MS): void => {
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
    (
      prefs: DockPrefs,
      name: string,
    ): { next: DockPrefs; id: string } => {
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
        activePreset: prefs.activePreset === id ? "default" : prefs.activePreset,
      };
    },
    [],
  );

  return {
    load,
    save,
    scheduleSave,
    applyBootstrap,
    applyPreset,
    saveCurrentAsPreset,
    deleteUserPreset,
  };
}
