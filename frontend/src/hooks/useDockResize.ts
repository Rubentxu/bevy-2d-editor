/**
 * useDockResize — manages dock layout state and CSS custom properties.
 *
 * Phase B (Defold-inspired redesign): owns the DockPrefs state for the 3-region
 * dock layout (Assets left / Viewport center / Outline+Properties right).
 * Exposes setLeftWidth/setRightWidth/setBottomHeight setters that update both
 * React state and the CSS variables `--dock-left-w` / `--dock-right-w` /
 * `--dock-bottom-h` consumed by DockLayout's CSS Grid template.
 *
 * The hook also debounces a save to OPFS (via useDockPrefs) so frequent drags
 * don't write on every pixel.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import {
  useDockPrefs,
  DEFAULT_DOCK_PREFS,
  type DockPrefs,
} from "./useDockPrefs";
import { BUILTIN_PRESETS } from "../data/workspacePresets";

const CSS_VAR_LEFT = "--dock-left-w";
const CSS_VAR_RIGHT = "--dock-right-w";
const CSS_VAR_BOTTOM = "--dock-bottom-h";
const CSS_VAR_STATUS = "--status-h";
const CSS_VAR_RIGHT_TOP = "--dock-right-top-h";

const MIN_LEFT = 160;
const MAX_LEFT = 600;
const MIN_RIGHT = 200;
const MAX_RIGHT = 600;
const MIN_BOTTOM = 100;
const MAX_BOTTOM = 480;
const MIN_RIGHT_TOP = 30;
const MAX_RIGHT_TOP = 80;
const MIN_STATUS = 20;
const MAX_STATUS = 48;

function clamp(n: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, n));
}

function applyCssVar(name: string, value: string): void {
  if (typeof document === "undefined") return;
  document.documentElement.style.setProperty(name, value);
}

export function useDockResize() {
  const {
    load,
    scheduleSave,
    applyPreset: applyPresetPrefs,
    saveCurrentAsPreset: saveCurrentAsPresetPrefs,
    deleteUserPreset: deleteUserPresetPrefs,
  } = useDockPrefs();
  const [prefs, setPrefs] = useState<DockPrefs>(DEFAULT_DOCK_PREFS);
  // Track whether the initial load has completed so we don't fight the default.
  const [hydrated, setHydrated] = useState(false);
  // Latest prefs in a ref so the debounced save can read the most recent values.
  const prefsRef = useRef<DockPrefs>(DEFAULT_DOCK_PREFS);
  prefsRef.current = prefs;

  // Push initial CSS vars on mount and whenever prefs change.
  useEffect(() => {
    applyCssVar(CSS_VAR_LEFT, `${prefs.left.width}px`);
  }, [prefs.left.width]);
  useEffect(() => {
    applyCssVar(CSS_VAR_RIGHT, `${prefs.right.width}px`);
  }, [prefs.right.width]);
  useEffect(() => {
    applyCssVar(CSS_VAR_BOTTOM, `${prefs.bottom.height}px`);
    applyCssVar(CSS_VAR_STATUS, `${prefs.statusBar.height}px`);
    applyCssVar(
      CSS_VAR_RIGHT_TOP,
      `${Math.round((prefs.right.topHeight / 100) * 100)}%`,
    );
  }, [prefs.bottom.height, prefs.statusBar.height, prefs.right.topHeight]);

  // Hydrate from OPFS on mount.
  useEffect(() => {
    let cancelled = false;
    void load().then((loaded) => {
      if (cancelled) return;
      if (loaded) {
        setPrefs(loaded);
      }
      setHydrated(true);
    });
    return () => {
      cancelled = true;
    };
  }, [load]);

  // Debounced save after hydration so we don't overwrite saved prefs with defaults.
  useEffect(() => {
    if (!hydrated) return;
    scheduleSave(prefs);
  }, [prefs, hydrated, scheduleSave]);

  const setLeftWidth = useCallback((w: number) => {
    setPrefs((prev) => ({
      ...prev,
      left: { ...prev.left, width: clamp(w, MIN_LEFT, MAX_LEFT) },
    }));
  }, []);

  const setRightWidth = useCallback((w: number) => {
    setPrefs((prev) => ({
      ...prev,
      right: { ...prev.right, width: clamp(w, MIN_RIGHT, MAX_RIGHT) },
    }));
  }, []);

  const setBottomHeight = useCallback((h: number) => {
    setPrefs((prev) => ({
      ...prev,
      bottom: { ...prev.bottom, height: clamp(h, MIN_BOTTOM, MAX_BOTTOM) },
    }));
  }, []);

  const setStatusBarHeight = useCallback((h: number) => {
    setPrefs((prev) => ({
      ...prev,
      statusBar: { height: clamp(h, MIN_STATUS, MAX_STATUS) },
    }));
  }, []);

  const setRightTopHeight = useCallback((pct: number) => {
    setPrefs((prev) => ({
      ...prev,
      right: {
        ...prev.right,
        topHeight: clamp(pct, MIN_RIGHT_TOP, MAX_RIGHT_TOP),
      },
    }));
  }, []);

  const toggleLeft = useCallback(() => {
    setPrefs((prev) => ({
      ...prev,
      left: { ...prev.left, visible: !prev.left.visible },
    }));
  }, []);

  const toggleRight = useCallback(() => {
    setPrefs((prev) => {
      // When showing the right dock, also re-enable both sub-sections so the
      // toggle behaves as "show all of the right panel" rather than "show
      // a half-collapsed shell".
      if (!prev.right.visible) {
        return {
          ...prev,
          right: {
            ...prev.right,
            visible: true,
            outlineVisible: true,
            propertiesVisible: true,
          },
        };
      }
      return { ...prev, right: { ...prev.right, visible: false } };
    });
  }, []);

  const toggleBottom = useCallback(() => {
    setPrefs((prev) => ({
      ...prev,
      bottom: { ...prev.bottom, visible: !prev.bottom.visible },
    }));
  }, []);

  const toggleOutline = useCallback(() => {
    setPrefs((prev) => ({
      ...prev,
      right: { ...prev.right, outlineVisible: !prev.right.outlineVisible },
    }));
  }, []);

  const toggleProperties = useCallback(() => {
    setPrefs((prev) => ({
      ...prev,
      right: {
        ...prev.right,
        propertiesVisible: !prev.right.propertiesVisible,
      },
    }));
  }, []);

  const toggleOutlineCollapsed = useCallback(() => {
    setPrefs((prev) => ({
      ...prev,
      right: {
        ...prev.right,
        outlineCollapsed: !prev.right.outlineCollapsed,
      },
    }));
  }, []);

  const togglePropertiesCollapsed = useCallback(() => {
    setPrefs((prev) => ({
      ...prev,
      right: {
        ...prev.right,
        propertiesCollapsed: !prev.right.propertiesCollapsed,
      },
    }));
  }, []);

  const reset = useCallback(() => {
    setPrefs(DEFAULT_DOCK_PREFS);
  }, []);

  // ── Workspace presets (v0.81 Tier 1b) ──────────────────────────────────
  // These three methods forward to the useDockPrefs service layer. They
  // mutate the local React state so the dock reflects the new layout
  // immediately; the existing `scheduleSave` effect will persist to OPFS
  // on its normal 500ms debounce.
  const applyPreset = useCallback(
    (presetId: string) => {
      setPrefs((prev) => applyPresetPrefs(prev, presetId));
    },
    [applyPresetPrefs],
  );

  const saveCurrentAsPreset = useCallback(
    (name: string) => {
      // Read the latest prefs from the ref so we capture the user's most
      // recent tweaks (those that haven't been debounced into state yet).
      let createdId = "";
      setPrefs((prev) => {
        const { next, id } = saveCurrentAsPresetPrefs(prev, name);
        createdId = id;
        return next;
      });
      return createdId;
    },
    [saveCurrentAsPresetPrefs],
  );

  const deleteUserPreset = useCallback(
    (presetId: string) => {
      setPrefs((prev) => deleteUserPresetPrefs(prev, presetId));
    },
    [deleteUserPresetPrefs],
  );

  return {
    prefs,
    hydrated,
    setLeftWidth,
    setRightWidth,
    setBottomHeight,
    setStatusBarHeight,
    setRightTopHeight,
    toggleLeft,
    toggleRight,
    toggleBottom,
    toggleOutline,
    toggleProperties,
    toggleOutlineCollapsed,
    togglePropertiesCollapsed,
    reset,
    applyPreset,
    saveCurrentAsPreset,
    deleteUserPreset,
    builtinPresets: BUILTIN_PRESETS,
  };
}
