import { useEffect, useState, useCallback } from "react";
import { PreviewMetrics, getPreviewMetrics } from "../services/scene-assets";
import { useHotReloadStatus } from "../hooks/useHotReloadStatus";

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
  const [isRefreshing, setIsRefreshing] = useState(false);
  const { lastReloadedAt, inFlightSaves, refresh: refreshHotReload } = useHotReloadStatus();

  const refreshMetrics = useCallback(async () => {
    try {
      const m = await getPreviewMetrics();
      setMetrics(m);
    } catch {
      // silently ignore — overlay stays at last known values
    }
  }, []);

  const handleHotReloadRefresh = useCallback(async () => {
    setIsRefreshing(true);
    try {
      await refreshHotReload();
    } finally {
      setIsRefreshing(false);
    }
  }, [refreshHotReload]);

  useEffect(() => {
    refreshMetrics();
    const id = setInterval(refreshMetrics, 500);
    return () => clearInterval(id);
  }, [refreshMetrics]);

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
          {lastReloadedAt != null && (
            <span data-testid="hot-reload-status">
              Auto-reloaded at {lastReloadedAt.toLocaleTimeString()}
            </span>
          )}
        </div>
      )}
      <button
        onClick={handleHotReloadRefresh}
        data-testid="hot-reload-refresh-btn"
        disabled={inFlightSaves > 0 || isRefreshing}
        style={{ pointerEvents: "auto" }}
        className={`hot-reload-refresh-btn ${isRefreshing ? "spinning" : ""}`}
      >
        ↻
      </button>
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
