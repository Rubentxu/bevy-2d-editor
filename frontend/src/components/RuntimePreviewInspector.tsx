import { useEffect, useState, useCallback, useRef } from "react";
import {
  PreviewMetrics,
  PreviewMappingEntry,
  PreviewProvenance,
  getPreviewMetrics,
  getPreviewMapping,
  getPreviewProvenance,
} from "../services/scene-assets";
import { useHotReloadStatus } from "../hooks/useHotReloadStatus";
import type { HotReloadEvent } from "../services/hot-reload";
import { useLogicActivation } from "../hooks/useLogicActivation";

interface Props {
  /** Optional callback to jump back to the source scene/asset for a given stable ID. */
  onJumpToSource?: (stableId: string) => void;
}

/** In-memory ring buffer for hot-reload events (last 20). */
const MAX_EVENTS = 20;

/**
 * RuntimePreviewInspector — polls the live preview metrics + mapping list
 * every 500ms and renders them. StableId-only on the editor side; no Bevy
 * Entity IDs are exposed.
 *
 * v2 extensions:
 * - Last rebuild cause
 * - Hot-reload events timeline (subscribe to useHotReloadStatus)
 * - Logic activation summaries
 * - Runtime-facing warnings
 * - Jump-back-to-source affordances
 */
export default function RuntimePreviewInspector({ onJumpToSource }: Props) {
  const [metrics, setMetrics] = useState<PreviewMetrics | null>(null);
  const [mapping, setMapping] = useState<PreviewMappingEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [provenance, setProvenance] = useState<{
    stableId: string;
    data: PreviewProvenance;
  } | null>(null);
  // PR4 correction: use useLogicActivation hook instead of inline (window as any) cast
  const { snapshot: logicLog } = useLogicActivation({ pollIntervalMs: 500 });
  const [hotReloadEvents, setHotReloadEvents] = useState<HotReloadEvent[]>([]);
  const [warnings, setWarnings] = useState<string[]>([]);
  const [showTimeline, setShowTimeline] = useState(false);
  const [lastRebuildCause, setLastRebuildCause] = useState<string | null>(null);
  const eventBuffer = useRef<HotReloadEvent[]>([]);

  const { lastReloadedAt } = useHotReloadStatus();

  const refresh = useCallback(async () => {
    try {
      const [m, mp] = await Promise.all([
        getPreviewMetrics(),
        getPreviewMapping(),
      ]);
      setMetrics(m);
      setMapping(mp);
      setError(null);

      // Runtime-facing warnings from metrics (demoted from errors)
      if (m && m.warnings && m.warnings.length > 0) {
        setWarnings(m.warnings);
      }

      // Rebuild cause from metrics if available
      if (m && (m as any).last_rebuild_cause) {
        setLastRebuildCause((m as any).last_rebuild_cause);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, 500);
    return () => clearInterval(id);
  }, [refresh]);

  // Subscribe to hot-reload events for the timeline
  useEffect(() => {
    let sourceUnsub = () => {};
    let assetUnsub = () => {};
    (async () => {
      const { subscribe } = await import("../services/hot-reload");
      sourceUnsub = subscribe("hot-reload-source", (event) => {
        eventBuffer.current = [...eventBuffer.current.slice(-MAX_EVENTS + 1), event];
        setHotReloadEvents([...eventBuffer.current]);
      });
      assetUnsub = subscribe("hot-reload-asset", (event) => {
        eventBuffer.current = [...eventBuffer.current.slice(-MAX_EVENTS + 1), event];
        setHotReloadEvents([...eventBuffer.current]);
      });
    })();
    return () => {
      sourceUnsub();
      assetUnsub();
    };
  }, []);

  const handleShowProvenance = useCallback(async (stableId: string) => {
    try {
      const data = await getPreviewProvenance(stableId);
      if (data === null) {
        setProvenance(null);
        return;
      }
      setProvenance({ stableId, data });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  return (
    <div
      className="runtime-preview-inspector"
      data-testid="runtime-preview-inspector"
    >
      <header className="rpi-header">
        <h3>Runtime Preview</h3>
        {error && (
          <span className="rpi-error" data-testid="rpi-error" title={error}>
            error
          </span>
        )}
      </header>

      {metrics && (
        <dl className="rpi-metrics" data-testid="rpi-metrics">
          <div className="rpi-metric">
            <dt>FPS</dt>
            <dd data-testid="rpi-fps">{metrics.fps.toFixed(1)}</dd>
          </div>
          <div className="rpi-metric">
            <dt>Frame</dt>
            <dd data-testid="rpi-frame-time">
              {metrics.frame_time_ms.toFixed(2)} ms
            </dd>
          </div>
          <div className="rpi-metric">
            <dt>Rebuilds</dt>
            <dd data-testid="rpi-rebuilds">{metrics.rebuild_count}</dd>
          </div>
        </dl>
      )}

      {/* Last rebuild cause */}
      {lastRebuildCause && (
        <div className="rpi-rebuild-cause" data-testid="rpi-rebuild-cause">
          <span className="rpi-rebuild-cause-label">Last rebuild:</span>
          <span className="rpi-rebuild-cause-value">{lastRebuildCause}</span>
        </div>
      )}

      {/* Runtime-facing warnings */}
      {warnings.length > 0 && (
        <div className="rpi-warnings" data-testid="rpi-warnings">
          <h4 className="rpi-warnings-title">⚠ Warnings</h4>
          <ul className="rpi-warnings-list">
            {warnings.map((w, i) => (
              <li key={i} className="rpi-warning-item" data-testid={`rpi-warning-${i}`}>
                {w}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Logic activation summaries */}
      {logicLog && (
        <div className="rpi-logic-summary" data-testid="rpi-logic-summary">
          <h4>Logic State</h4>
          <dl className="rpi-logic-dl">
            <div className="rpi-logic-entry">
              <dt>Log size</dt>
              <dd data-testid="rpi-logic-size">{logicLog.size}</dd>
            </div>
            <div className="rpi-logic-entry">
              <dt>Cursor</dt>
              <dd data-testid="rpi-logic-cursor">{logicLog.cursor}</dd>
            </div>
            <div className="rpi-logic-entry">
              <dt>Undo</dt>
              <dd data-testid="rpi-logic-undo">{logicLog.can_undo ? "yes" : "no"}</dd>
            </div>
            <div className="rpi-logic-entry">
              <dt>Redo</dt>
              <dd data-testid="rpi-logic-redo">{logicLog.can_redo ? "yes" : "no"}</dd>
            </div>
          </dl>
        </div>
      )}

      {/* Hot-reload events timeline toggle */}
      <div className="rpi-timeline-toggle">
        <button
          type="button"
          className="rpi-timeline-toggle-btn"
          onClick={() => setShowTimeline((v) => !v)}
          data-testid="rpi-timeline-toggle"
        >
          {showTimeline ? "▼" : "▶"} Hot-reload Events
          {hotReloadEvents.length > 0 && (
            <span className="rpi-event-count" data-testid="rpi-event-count">
              {hotReloadEvents.length}
            </span>
          )}
        </button>
      </div>

      {showTimeline && (
        <div className="rpi-timeline" data-testid="rpi-timeline">
          {hotReloadEvents.length === 0 ? (
            <p className="rpi-timeline-empty">No events yet</p>
          ) : (
            <ul className="rpi-timeline-list">
              {hotReloadEvents.map((ev, i) => (
                <li key={i} className="rpi-timeline-item" data-testid={`rpi-timeline-item-${i}`}>
                  <span className="rpi-timeline-type">
                    {ev.type === "hot-reload-source" ? "src" : "asset"}
                  </span>
                  <span className="rpi-timeline-detail">
                    {ev.type === "hot-reload-source"
                      ? (ev as any).fileId
                      : (ev as any).assetId}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      <h4>Projected Instances</h4>
      {mapping.length === 0 ? (
        <p className="rpi-empty" data-testid="rpi-empty">
          No Scene Instances projected. Place a Scene Asset to see the mapping.
        </p>
      ) : (
        <ul className="rpi-list" data-testid="rpi-list">
          {mapping.map((m) => (
            <li
              key={m.stable_id}
              className="rpi-row"
              data-testid={`rpi-row-${m.stable_id}`}
            >
              <button
                className="rpi-row-btn"
                onClick={() => handleShowProvenance(m.stable_id)}
                data-testid={`rpi-row-btn-${m.stable_id}`}
              >
                <span className="rpi-row-name">{m.stable_id}</span>
                <span className="rpi-row-asset">{m.asset_ref}</span>
                <span className="rpi-row-count">{m.component_count} comp</span>
              </button>
              {onJumpToSource && (
                <button
                  type="button"
                  className="rpi-jump-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    onJumpToSource(m.stable_id);
                  }}
                  data-testid={`rpi-jump-btn-${m.stable_id}`}
                  title="Jump back to source"
                >
                  ↩
                </button>
              )}
            </li>
          ))}
        </ul>
      )}

      {provenance && (
        <section
          className="rpi-provenance"
          data-testid={`rpi-provenance-${provenance.stableId}`}
        >
          <header className="rpi-provenance-header">
            <h4>Provenance: {provenance.stableId}</h4>
            <button
              className="rpi-provenance-close"
              onClick={() => setProvenance(null)}
              data-testid="rpi-provenance-close"
            >
              ×
            </button>
          </header>
          <dl>
            <div>
              <dt>Local ID</dt>
              <dd>{provenance.data.local_id}</dd>
            </div>
            <div>
              <dt>Asset Ref</dt>
              <dd>{provenance.data.asset_ref}</dd>
            </div>
            <div>
              <dt>From Instance</dt>
              <dd>{provenance.data.is_from_instance ? "yes" : "no"}</dd>
            </div>
          </dl>
          <h5>Components</h5>
          <ul>
            {provenance.data.components.map((c) => (
              <li key={c}>{c}</li>
            ))}
          </ul>
          {onJumpToSource && (
            <button
              type="button"
              className="rpi-jump-source-btn"
              onClick={() => {
                onJumpToSource(provenance.stableId);
              }}
              data-testid={`rpi-jump-source-${provenance.stableId}`}
            >
              Jump back to source
            </button>
          )}
        </section>
      )}
    </div>
  );
}
