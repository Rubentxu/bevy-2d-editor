/**
 * ApplyBackPanel — §7 (v0.89 PR4) Runtime Apply-Back panel.
 *
 * Renders the list of `RuntimeDelta` records from `EditorSession.runtime_delta_buffer`
 * and lets the user select which deltas to apply back to the authoring state.
 *
 * "Create ChangeSet" builds a single `PendingChangeSet` via
 * `create_apply_back_change_set_wasm` and submits it to the workbench via
 * `submit_pending_change_set`. The workbench then handles the approval flow
 * (per the §12 partial-apply semantics).
 *
 * Per NFR-1, this panel reads ONLY `apply_back_eligible`, `instance_id`,
 * `target_local_id`, `component_type_id`, `field_path`, `baseline_value`,
 * and `runtime_value` from the delta — never anything Bevy-Entity-related.
 *
 * Empty / loading / error states follow the §12 contract.
 */

import { useEffect, useState, useCallback } from "react";

interface RuntimeDelta {
  instance_id: string;
  target_local_id: string;
  component_type_id: string;
  field_path: string;
  baseline_value: unknown;
  runtime_value: unknown;
  captured_at_ms: number;
  apply_back_eligible: boolean;
}

interface Props {
  /** Optional callback after a ChangeSet is submitted to the workbench. */
  onChangeSetCreated?: (changeSetId: string) => void;
}

function deltaId(d: RuntimeDelta): string {
  return `${d.instance_id}|${d.component_type_id}|${d.field_path}`;
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return "null";
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

async function fetchRuntimeDeltas(): Promise<RuntimeDelta[]> {
  const w = window as unknown as {
    get_runtime_deltas_wasm?: () => Promise<unknown> | unknown;
  };
  if (typeof w.get_runtime_deltas_wasm !== "function") {
    return [];
  }
  try {
    const raw = await w.get_runtime_deltas_wasm();
    if (raw == null) return [];
    const parsed =
      typeof raw === "string" ? JSON.parse(raw) : (raw as RuntimeDelta[]);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

async function createChangeSet(deltaIds: string[]): Promise<string | null> {
  const w = window as unknown as {
    create_apply_back_change_set_wasm?: (
      idsJson: string,
    ) => Promise<unknown> | unknown;
  };
  if (typeof w.create_apply_back_change_set_wasm !== "function") {
    return null;
  }
  try {
    const raw = await w.create_apply_back_change_set_wasm(
      JSON.stringify(deltaIds),
    );
    if (raw == null) return null;
    return typeof raw === "string" ? raw : JSON.stringify(raw);
  } catch {
    return null;
  }
}

async function submitToWorkbench(
  changeSetJson: string,
): Promise<string | null> {
  const w = window as unknown as {
    submit_pending_change_set?: (json: string) => Promise<unknown> | unknown;
  };
  if (typeof w.submit_pending_change_set !== "function") {
    return null;
  }
  try {
    const raw = await w.submit_pending_change_set(changeSetJson);
    return raw == null ? null : String(raw);
  } catch {
    return null;
  }
}

export function ApplyBackPanel({ onChangeSetCreated }: Props) {
  const [deltas, setDeltas] = useState<RuntimeDelta[]>([]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [lastResult, setLastResult] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      const list = await fetchRuntimeDeltas();
      setDeltas(list);
      setError(null);
      // Pre-select only eligible deltas.
      const eligibleIds = list
        .filter((d) => d.apply_back_eligible)
        .map(deltaId);
      setSelected(new Set(eligibleIds));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => {
      void refresh();
    }, 2000);
    return () => window.clearInterval(id);
  }, [refresh]);

  const toggleSelected = useCallback((id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const onCreate = useCallback(async () => {
    const ids = Array.from(selected);
    if (ids.length === 0) return;
    setBusy(true);
    setError(null);
    try {
      const csJson = await createChangeSet(ids);
      if (csJson == null) {
        setError("WASM create_apply_back_change_set_wasm not available");
        return;
      }
      const csId = await submitToWorkbench(csJson);
      if (csId) {
        setLastResult(
          `Submitted ChangeSet ${csId} with ${ids.length} delta(s) to the workbench.`,
        );
        onChangeSetCreated?.(csId);
      } else {
        setLastResult(
          `Built ChangeSet (${ids.length} delta(s)) but workbench submission not available.`,
        );
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [selected, onChangeSetCreated]);

  if (loading && deltas.length === 0) {
    return (
      <section
        className="apply-back-panel apply-back-panel--loading"
        data-testid="apply-back-panel"
      >
        <h3>Apply-Back</h3>
        <p>Loading runtime deltas…</p>
      </section>
    );
  }

  if (error) {
    return (
      <section
        className="apply-back-panel apply-back-panel--error"
        data-testid="apply-back-panel"
      >
        <h3>Apply-Back — error</h3>
        <p role="alert">{error}</p>
        <button type="button" onClick={() => void refresh()}>
          Retry
        </button>
      </section>
    );
  }

  if (deltas.length === 0) {
    return (
      <section
        className="apply-back-panel apply-back-panel--empty"
        data-testid="apply-back-panel"
      >
        <h3>Apply-Back</h3>
        <p>
          No runtime deltas. Enter play mode, modify a Tunable field, and exit
          to record a delta.
        </p>
      </section>
    );
  }

  const eligibleCount = deltas.filter((d) => d.apply_back_eligible).length;
  const selectedEligibleCount = deltas.filter(
    (d) => d.apply_back_eligible && selected.has(deltaId(d)),
  ).length;

  return (
    <section className="apply-back-panel" data-testid="apply-back-panel">
      <header>
        <h3>Apply-Back</h3>
        <p className="muted">
          {deltas.length} runtime delta(s) — {eligibleCount} eligible,{" "}
          {selectedEligibleCount} selected.
        </p>
      </header>

      <ul className="delta-list">
        {deltas.map((d) => {
          const id = deltaId(d);
          const isSelected = selected.has(id);
          return (
            <li
              key={id}
              className={`delta-item ${d.apply_back_eligible ? "delta-item--eligible" : "delta-item--ineligible"}`}
              data-testid={`delta-item-${id}`}
            >
              <label>
                <input
                  type="checkbox"
                  checked={isSelected}
                  disabled={!d.apply_back_eligible || busy}
                  onChange={() => toggleSelected(id)}
                />
                <code>{d.component_type_id}</code>.<code>{d.field_path}</code>{" "}
                on instance <code>{d.instance_id}</code> — baseline=
                <code>{formatValue(d.baseline_value)}</code> → runtime=
                <code>{formatValue(d.runtime_value)}</code>
                {!d.apply_back_eligible && (
                  <em> (not eligible — Never policy)</em>
                )}
              </label>
            </li>
          );
        })}
      </ul>

      <footer>
        <button
          type="button"
          onClick={() => void onCreate()}
          disabled={busy || selectedEligibleCount === 0}
          data-testid="create-change-set-button"
        >
          {busy
            ? "Building…"
            : `Create ChangeSet (${selectedEligibleCount} delta(s))`}
        </button>
        {lastResult && (
          <p className="apply-back-result" data-testid="apply-back-result">
            {lastResult}
          </p>
        )}
      </footer>
    </section>
  );
}

export default ApplyBackPanel;
