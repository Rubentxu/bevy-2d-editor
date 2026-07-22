import { useEffect, useState, useCallback } from "react";
import {
  PreviewMetrics,
  PreviewMappingEntry,
  PreviewProvenance,
  getPreviewMetrics,
  getPreviewMapping,
  getPreviewProvenance,
} from "../services/scene-assets";

/**
 * RuntimePreviewInspector — polls the live preview metrics + mapping list
 * every 500ms and renders them. StableId-only on the editor side; no Bevy
 * Entity IDs are exposed.
 */
export default function RuntimePreviewInspector() {
  const [metrics, setMetrics] = useState<PreviewMetrics | null>(null);
  const [mapping, setMapping] = useState<PreviewMappingEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [provenance, setProvenance] = useState<{
    stableId: string;
    data: PreviewProvenance;
  } | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [m, mp] = await Promise.all([
        getPreviewMetrics(),
        getPreviewMapping(),
      ]);
      setMetrics(m);
      setMapping(mp);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, 500);
    return () => clearInterval(id);
  }, [refresh]);

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
        </section>
      )}
    </div>
  );
}
