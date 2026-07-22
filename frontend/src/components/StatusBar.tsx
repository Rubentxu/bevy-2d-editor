import { useEffect, useState } from "react";
import { useCanvasViewport } from "../hooks/useCanvasViewport";
import { useLogState } from "../hooks/useLogState";
import { useSceneState } from "../hooks/useSceneState";
import { useScenes } from "../hooks/useScenes";

interface PreviewMetrics {
  fps?: number;
  frame_time_ms?: number;
  entity_count?: number;
  instance_count?: number;
}

function parseMetrics(value: unknown): PreviewMetrics {
  if (typeof value === "string") {
    try {
      return JSON.parse(value) as PreviewMetrics;
    } catch {
      return {};
    }
  }
  return value && typeof value === "object" ? (value as PreviewMetrics) : {};
}

export default function StatusBar() {
  const { worldPos } = useCanvasViewport();
  const { scene } = useSceneState();
  const logState = useLogState();
  const { scenes, currentId } = useScenes();
  const [metrics, setMetrics] = useState<PreviewMetrics>({});

  useEffect(() => {
    const update = () => {
      try {
        const getter = (window as any).get_preview_metrics_wasm;
        if (typeof getter === "function") {
          setMetrics(parseMetrics(getter()));
        }
      } catch {
        setMetrics({});
      }
    };
    update();
    const interval = window.setInterval(update, 500);
    return () => window.clearInterval(interval);
  }, []);

  const sceneName =
    scenes.find((item) => item.id === currentId)?.name ??
    scene?.name ??
    "No scene";
  const entityCount = scene?.entities.length ?? 0;
  const instanceCount =
    metrics.instance_count ??
    scene?.entities.filter((entity) => entity.id.startsWith("inst_")).length ??
    0;
  const fps = Number.isFinite(metrics.fps) ? Math.round(metrics.fps!) : "--";
  const position = worldPos
    ? `(${worldPos.x.toFixed(1)}, ${worldPos.y.toFixed(1)})`
    : "(—, —)";

  return (
    <div className="status-bar" data-testid="status-bar">
      <div className="status-bar-region status-bar-left">
        <span data-testid="status-world-position">{position}</span>
        <span>
          {entityCount} {entityCount === 1 ? "entity" : "entities"}
        </span>
        <span>
          {instanceCount} {instanceCount === 1 ? "instance" : "instances"}
        </span>
      </div>
      <div className="status-bar-region status-bar-center">
        <span>{sceneName}</span>
        {/* Phase 5 — consistent save-state indicators:
            ● = dirty (unsaved changes in the log)
            ○ = saved (log is empty)
            ⟳ = saving (a save op is currently in flight) */}
        <span
          className="status-bar-dirty"
          data-testid="status-bar-dirty"
          data-state={logState.size > 0 ? "dirty" : "saved"}
          title={
            logState.size > 0 ? `Dirty (${logState.size} ops pending)` : "Saved"
          }
        >
          {logState.size > 0 ? "●" : "○"}
        </span>
      </div>
      <div
        className="status-bar-region status-bar-right"
        data-testid="status-fps"
      >
        FPS {fps}
      </div>
    </div>
  );
}
