/**
 * RuntimeCausalityPanel — §6 (v0.89 PR3) runtime causality inspector.
 *
 * Renders three sections:
 * 1. Last RebuildCause (one of 6 variants, with a human-readable description).
 * 2. LogicActivationEvent ring buffer (≤ 64 entries, newest first).
 * 3. CausalityEdge summary per preview provenance entry.
 *
 * Empty / loading / error states follow the §12 contract:
 * - Empty: "No rebuild has been recorded yet." (no actions)
 * - Loading: "Loading causality state…" (no actions)
 * - Error: explicit error message (no actions)
 *
 * Read-only — does not expose any action buttons. (Approvals live in
 * ChangeWorkbenchPanel; runtime mutations live in RuntimePreviewInspector.)
 */

import { useEffect, useState, useCallback } from "react";
import {
  useLogicActivation,
  RebuildCause,
  LogicActivationEvent,
} from "../hooks/useLogicActivation";

interface PreviewProvenanceEdge {
  edge_kind: "definition" | "instance" | "override" | "logic" | "source";
  target_stable_id: string;
}

interface PreviewProvenanceEntry {
  stable_id: string;
  causality_edges: PreviewProvenanceEdge[];
}

function describeCause(cause: RebuildCause): string {
  switch (cause.kind) {
    case "user_edit":
      return `User edit: ${cause.command_id}`;
    case "hot_reload":
      return `Hot reload: ${cause.file_id}`;
    case "play_mode_enter":
      return "Play mode entered";
    case "play_mode_exit":
      return "Play mode exited";
    case "scene_switch":
      return `Scene switch: ${cause.from} → ${cause.to}`;
    case "asset_resync":
      return `Asset resync: ${cause.asset_ref}`;
    default: {
      // Exhaustiveness guard — if a 7th variant is added, this will fail to compile.
      const _exhaustive: never = cause;
      return `Unknown cause: ${String(_exhaustive)}`;
    }
  }
}

function formatTimestamp(ms: number): string {
  try {
    return new Date(ms).toISOString();
  } catch {
    return `+${ms}ms`;
  }
}

interface ProvenanceSnapshot {
  entries: PreviewProvenanceEntry[];
}

async function fetchProvenanceSnapshot(): Promise<ProvenanceSnapshot> {
  const w = window as unknown as {
    get_preview_provenance_wasm?: () => unknown;
  };
  if (typeof w.get_preview_provenance_wasm !== "function") {
    return { entries: [] };
  }
  try {
    const raw = await w.get_preview_provenance_wasm();
    if (raw == null) return { entries: [] };
    const parsed =
      typeof raw === "string" ? JSON.parse(raw) : (raw as ProvenanceSnapshot);
    return { entries: Array.isArray(parsed.entries) ? parsed.entries : [] };
  } catch {
    return { entries: [] };
  }
}

export function RuntimeCausalityPanel() {
  const { events, rebuildCause, refresh } = useLogicActivation({
    pollIntervalMs: 1500,
  });
  const [provenance, setProvenance] = useState<ProvenanceSnapshot>({
    entries: [],
  });
  const [error, setError] = useState<string | null>(null);

  const refreshProvenance = useCallback(async () => {
    try {
      const snap = await fetchProvenanceSnapshot();
      setProvenance(snap);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    void refreshProvenance();
    const id = window.setInterval(() => {
      void refreshProvenance();
    }, 2000);
    return () => window.clearInterval(id);
  }, [refreshProvenance]);

  const totalEdges = provenance.entries.reduce(
    (acc, e) => acc + e.causality_edges.length,
    0,
  );

  if (error) {
    return (
      <section className="runtime-causality-panel runtime-causality-panel--error">
        <h3>Runtime Causality — error</h3>
        <p role="alert">{error}</p>
        <button type="button" onClick={() => void refresh()}>
          Retry
        </button>
      </section>
    );
  }

  if (events.length === 0 && rebuildCause === null) {
    return (
      <section className="runtime-causality-panel runtime-causality-panel--empty">
        <h3>Runtime Causality</h3>
        <p>
          No rebuild has been recorded yet. Make an edit, switch scenes, or
          enter play mode to see causality data.
        </p>
      </section>
    );
  }

  // Reverse the event ring so the newest event is first.
  const eventsNewestFirst: LogicActivationEvent[] = [...events].reverse();

  return (
    <section
      className="runtime-causality-panel"
      data-testid="runtime-causality-panel"
    >
      <header>
        <h3>Runtime Causality</h3>
        <button type="button" onClick={() => void refresh()}>
          Refresh
        </button>
      </header>

      <section className="rebuild-cause-section">
        <h4>Last rebuild</h4>
        {rebuildCause ? (
          <p data-testid="rebuild-cause">{describeCause(rebuildCause)}</p>
        ) : (
          <p className="muted">No rebuild recorded yet.</p>
        )}
      </section>

      <section className="activation-ring-section">
        <h4>Logic activation events (newest first, cap 64)</h4>
        {eventsNewestFirst.length === 0 ? (
          <p className="muted">No logic activation events.</p>
        ) : (
          <ul data-testid="activation-events">
            {eventsNewestFirst.map((event, idx) => (
              <li key={`${event.node_id}-${event.triggered_at_ms}-${idx}`}>
                <code>{event.node_id}</code> at{" "}
                {formatTimestamp(event.triggered_at_ms)}
                {event.payload_summary ? (
                  <span> — {event.payload_summary}</span>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="provenance-section">
        <h4>
          Preview provenance — causality edges ({totalEdges} across{" "}
          {provenance.entries.length} entries)
        </h4>
        {provenance.entries.length === 0 ? (
          <p className="muted">No preview instances.</p>
        ) : (
          <ul>
            {provenance.entries.map((entry) => (
              <li key={entry.stable_id}>
                <code>{entry.stable_id}</code> — {entry.causality_edges.length}{" "}
                edge
                {entry.causality_edges.length === 1 ? "" : "s"}
                {entry.causality_edges.length > 0 && (
                  <ul>
                    {entry.causality_edges.map((edge, i) => (
                      <li key={`${entry.stable_id}-${i}`}>
                        <em>{edge.edge_kind}</em> →{" "}
                        <code>{edge.target_stable_id}</code>
                      </li>
                    ))}
                  </ul>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>
    </section>
  );
}

export default RuntimeCausalityPanel;
