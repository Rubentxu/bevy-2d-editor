/**
 * useFullscreenBody — mirrors the `fullscreen.enabled` flag onto the
 * document body via a `data-fullscreen` attribute so CSS can react to
 * fullscreen transitions.
 */
import { useEffect } from "react";

export function useFullscreenBody(enabled: boolean): void {
  useEffect(() => {
    if (enabled) {
      document.body.dataset.fullscreen = "true";
    } else {
      delete document.body.dataset.fullscreen;
    }
  }, [enabled]);
}
