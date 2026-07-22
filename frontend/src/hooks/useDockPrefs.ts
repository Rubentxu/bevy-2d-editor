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

export interface DockPrefs {
  /** id of the preset last applied; null after a manual edit. */
  activePreset?: string | null;
  left: { width: number; visible: boolean };
  right: {
    width: number;
    visible: boolean;
    outlineVisible: boolean;
    propertiesVisible: boolean;
    topHeight: number;
  };
  bottom: { height: number; visible: boolean };
  /** User-defined presets keyed by id. Built-ins are not stored here. */
  presets?: Record<string, UserPresetRecord>;
}

export const DEFAULT_DOCK_PREFS: DockPrefs = {
  activePreset: "default",
  left: { width: 280, visible: true },
  right: {
    width: 320,
    visible: true,
    outlineVisible: true,
    propertiesVisible: true,
    topHeight: 60,
  },
  bottom: { height: 240, visible: true },
  presets: {},
};

export function useDockPrefs() {
  const timerRef = useRef<number | null>(null);

  /**
   * Synchronous bootstrap (Phase E, §E.5): try to read `dock-prefs.json`
   * during the initial bundle load so we can apply the CSS custom properties
   * (`--dock-left-w` / `--dock-right-w` / `--dock-bottom-h`) BEFORE the first
   * paint. OPFS is normally async, so this best-effort relies on a tiny
   * race: we try to resolve the OPFS directory handle synchronously via the
   * `navigator.storage.getDirectory()` → `getFileHandle()` chain wrapped in
   * a non-awaited Promise. If it doesn't resolve in time the defaults stand
   * and `useDockResize` will apply the persisted values once the async OPFS
   * call completes.
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
  }, []);

  /**
   * Merge a partial JSON envelope onto the defaults. Missing branches fall
   * back to `DEFAULT_DOCK_PREFS` so the rest of the editor never crashes on
   * a half-migrated file. Also normalises `presets` to a (possibly empty)
   * object so callers can read it without optional chaining.
   */
  const mergeWithDefaults = useCallback(
    (parsed: Partial<DockPrefs>): DockPrefs => ({
      activePreset: parsed.activePreset ?? DEFAULT_DOCK_PREFS.activePreset,
      left: { ...DEFAULT_DOCK_PREFS.left, ...parsed.left },
      right: { ...DEFAULT_DOCK_PREFS.right, ...parsed.right },
      bottom: { ...DEFAULT_DOCK_PREFS.bottom, ...parsed.bottom },
      presets: { ...(parsed.presets ?? {}) },
    }),
    [],
  );

  const load = useCallback(async (): Promise<DockPrefs | null> => {
    try {
      const result = await opfsLoadFile(DOCK_PREFS_PATH);
      if (!result.ok || !result.value) return null;
      const parsed = JSON.parse(result.value) as Partial<DockPrefs>;
      return mergeWithDefaults(parsed);
    } catch (e) {
      console.warn("[useDockPrefs] load failed:", e);
      return null;
    }
  }, [mergeWithDefaults]);

  const save = useCallback(async (prefs: DockPrefs): Promise<void> => {
    try {
      await opfsSaveFile(DOCK_PREFS_PATH, JSON.stringify(prefs));
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
