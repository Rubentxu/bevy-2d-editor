/**
 * useViewportMode — reports whether the current viewport qualifies as
 * "desktop" or "compact" based on a 1280 px threshold.
 *
 * The threshold is the minimum supported width for the full 3-column
 * dock layout. Below this the editor switches to compact mode (single
 * column with tabs for panels).
 *
 * Maintenance note: revisit the threshold when a mobile / responsive
 * strategy is formally adopted.
 */

import { useState, useEffect } from "react";

export type ViewportMode = "desktop" | "compact";

export interface UseViewportModeResult {
  mode: ViewportMode;
  width: number;
}

/** Minimum viewport width for desktop layout (px). */
export const VIEWPORT_COMPACT_THRESHOLD = 1280;

function getViewportWidth(): number {
  if (typeof window === "undefined") return VIEWPORT_COMPACT_THRESHOLD + 1;
  return window.innerWidth;
}



/**
 * Returns `{ mode: "desktop" | "compact", width: number }`.
 *
 * `mode` transitions whenever the window is resized across the 1280 px
 * boundary. `width` is the live `window.innerWidth`.
 */
export function useViewportMode(): UseViewportModeResult {
  const [width, setWidth] = useState<number>(getViewportWidth);

  useEffect(() => {
    let rafId: number | null = null;

    const handleResize = () => {
      // Debounce to one update per animation frame.
      if (rafId !== null) return;
      rafId = requestAnimationFrame(() => {
        rafId = null;
        setWidth(window.innerWidth);
      });
    };

    window.addEventListener("resize", handleResize, { passive: true });
    return () => {
      window.removeEventListener("resize", handleResize);
      if (rafId !== null) cancelAnimationFrame(rafId);
    };
  }, []);

  const mode: ViewportMode =
    width < VIEWPORT_COMPACT_THRESHOLD ? "compact" : "desktop";

  return { mode, width };
}
