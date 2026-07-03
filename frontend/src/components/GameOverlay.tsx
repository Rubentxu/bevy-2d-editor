import { useEffect, useState, useCallback } from "react";
import { PreviewMetrics, getPreviewMetrics } from "../services/scene-assets";

interface GameOverlayProps {
  onStop: () => void;
}

/**
 * GameOverlay — renders above the canvas during play mode.
 * Displays live FPS and frame time from the Bevy preview, plus a Stop button.
 * Container uses pointer-events:none so mouse input reaches the canvas;
 * only the Stop button needs pointer-events:auto.
 */
export default function GameOverlay({ onStop }: GameOverlayProps) {
  const [metrics, setMetrics] = useState<PreviewMetrics | null>(null);

  const refresh = useCallback(async () => {
    try {
      const m = await getPreviewMetrics();
      setMetrics(m);
    } catch {
      // silently ignore — overlay stays at last known values
    }
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, 500);
    return () => clearInterval(id);
  }, [refresh]);

  return (
    <div
      className="game-overlay"
      data-testid="game-overlay"
      style={{ pointerEvents: "none" }}
    >
      {metrics && (
        <div className="game-overlay-metrics" data-testid="game-overlay-metrics">
          <span data-testid="game-overlay-fps">{metrics.fps.toFixed(1)} FPS</span>
          <span data-testid="game-overlay-frame-ms">
            {metrics.frame_time_ms.toFixed(2)} ms
          </span>
        </div>
      )}
      <button
        onClick={onStop}
        data-testid="game-overlay-stop-btn"
        style={{ pointerEvents: "auto" }}
        className="game-overlay-stop-btn"
      >
        ⏹ Stop
      </button>
    </div>
  );
}
