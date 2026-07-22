/**
 * useDockPrefs — load/save dock layout preferences to OPFS.
 *
 * Phase B (Defold-inspired redesign): persists dock widths/heights/visibility
 * to `dock-prefs.json` in OPFS. The hook exposes a tiny, dependency-free
 * surface: `load()` reads (returns null if missing or OPFS unavailable),
 * `save(prefs)` writes JSON, `scheduleSave(prefs, ms)` debounces.
 *
 * Persistence is best-effort: errors are logged but never thrown so a
 * broken OPFS layer cannot crash the editor.
 */

import { useCallback, useRef } from "react";
import { opfsLoadFile, opfsSaveFile } from "../opfs-bridge";

const DOCK_PREFS_PATH = "dock-prefs.json";
const DEBOUNCE_MS = 500;

export interface DockPrefs {
  left: { width: number; visible: boolean };
  right: {
    width: number;
    visible: boolean;
    outlineVisible: boolean;
    propertiesVisible: boolean;
    topHeight: number;
  };
  bottom: { height: number; visible: boolean };
}

export const DEFAULT_DOCK_PREFS: DockPrefs = {
  left: { width: 280, visible: true },
  right: {
    width: 320,
    visible: true,
    outlineVisible: true,
    propertiesVisible: true,
    topHeight: 60,
  },
  bottom: { height: 240, visible: true },
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

  const load = useCallback(async (): Promise<DockPrefs | null> => {
    try {
      const result = await opfsLoadFile(DOCK_PREFS_PATH);
      if (!result.ok || !result.value) return null;
      const parsed = JSON.parse(result.value) as Partial<DockPrefs>;
      // Merge with defaults so newly added keys still have a value.
      return {
        left: { ...DEFAULT_DOCK_PREFS.left, ...parsed.left },
        right: { ...DEFAULT_DOCK_PREFS.right, ...parsed.right },
        bottom: { ...DEFAULT_DOCK_PREFS.bottom, ...parsed.bottom },
      };
    } catch (e) {
      console.warn("[useDockPrefs] load failed:", e);
      return null;
    }
  }, []);

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

  return { load, save, scheduleSave, applyBootstrap };
}
