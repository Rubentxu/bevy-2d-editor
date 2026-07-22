/**
 * useFullscreen — toggle the `data-fullscreen` attribute on <body>.
 *
 * Phase E (Defold-inspired redesign): F9 is the "fullscreen viewport" shortcut.
 * We don't use the browser's fullscreen API here — we just hide every non-center
 * dock so the canvas fills the workspace. The MenuBar + StatusBar stay visible
 * so the user can still navigate and see scene state. State is local to the
 * document attribute (no need to round-trip through React state because the
 * CSS rule is attribute-driven and there's nothing else consuming it).
 */

import { useCallback, useEffect, useState } from "react";

const ATTR = "fullscreen";
const VALUE = "true";

function readFullscreen(): boolean {
  if (typeof document === "undefined") return false;
  return document.body.dataset.fullscreen === VALUE;
}

export function useFullscreen() {
  const [enabled, setEnabled] = useState<boolean>(readFullscreen());

  // Sync from the DOM on mount in case another component flipped the flag.
  useEffect(() => {
    setEnabled(readFullscreen());
  }, []);

  const setFullscreen = useCallback((next: boolean) => {
    if (typeof document === "undefined") return;
    if (next) {
      document.body.dataset.fullscreen = VALUE;
    } else {
      delete document.body.dataset.fullscreen;
    }
    setEnabled(next);
  }, []);

  const toggle = useCallback(() => {
    setFullscreen(!readFullscreen());
  }, [setFullscreen]);

  return { enabled, setFullscreen, toggle };
}
