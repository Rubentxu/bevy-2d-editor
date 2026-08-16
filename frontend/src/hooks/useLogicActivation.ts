/**
 * useLogicActivation — typed hook for the §6 Logic Activation ring buffer
 * and the legacy logic log state, polled from WASM.
 *
 * Provides:
 * - `events`: the last ≤ 64 `LogicActivationEvent` entries (ring buffer).
 * - `rebuildCause`: the most recent `RebuildCause` (one of 6 variants).
 * - `legacy`: the legacy `LogicLogState` (size/can_undo/can_redo/cursor)
 *   from `get_logic_log_state()`. Kept for backward compatibility with the
 *   RuntimePreviewInspector.
 *
 * The hook returns `{ events, rebuildCause, legacy, refresh }` and is null-safe
 * when WASM is not yet ready (matches useLogicGraph.ts:147-155 fallback pattern).
 *
 * PR3 (v0.89): rewritten to consume the new `get_rebuild_cause_wasm` and
 * `get_logic_activation_events_wasm` exports added in editor-application.
 */

import { useCallback, useEffect, useState } from "react";

// ─── Types mirroring editor-model::RebuildCause ──────────────────────────────

export type RebuildCause =
  | { kind: "user_edit"; command_id: string }
  | { kind: "hot_reload"; file_id: string }
  | { kind: "play_mode_enter" }
  | { kind: "play_mode_exit" }
  | { kind: "scene_switch"; from: string; to: string }
  | { kind: "asset_resync"; asset_ref: string };

// ─── Types mirroring editor-model::logic_activation::LogicActivationEvent ────

export interface LogicActivationEvent {
  node_id: string;
  triggered_at_ms: number;
  payload_summary?: string;
}

// ─── Legacy logic log state (kept for RuntimePreviewInspector compat) ──────

export interface LogicLogState {
  size: number;
  can_undo: boolean;
  can_redo: boolean;
  cursor: number;
}

interface UseLogicActivationOptions {
  /** Polling interval in milliseconds (default: 1000). */
  pollIntervalMs?: number;
}

interface UseLogicActivationResult {
  events: LogicActivationEvent[];
  rebuildCause: RebuildCause | null;
  legacy: LogicLogState | null;
  refresh: () => Promise<void>;
}

const EMPTY_EVENTS: LogicActivationEvent[] = [];

/**
 * Hook that polls the §6 logic activation ring + rebuild cause + legacy
 * logic log state, returning typed snapshots.
 *
 * @example
 * const { events, rebuildCause, legacy } = useLogicActivation({ pollIntervalMs: 2000 });
 * if (rebuildCause?.kind === "user_edit") {
 *   console.log("last edit:", rebuildCause.command_id);
 * }
 */
export function useLogicActivation(
  options: UseLogicActivationOptions = {},
): UseLogicActivationResult {
  const { pollIntervalMs = 1000 } = options;

  const [events, setEvents] = useState<LogicActivationEvent[]>(EMPTY_EVENTS);
  const [rebuildCause, setRebuildCause] = useState<RebuildCause | null>(null);
  const [legacy, setLegacy] = useState<LogicLogState | null>(null);

  const refresh = useCallback(async () => {
    const w = window as unknown as {
      get_rebuild_cause_wasm?: () => unknown;
      get_logic_activation_events_wasm?: () => unknown;
      get_logic_log_state?: () => unknown;
    };

    if (typeof w.get_rebuild_cause_wasm === "function") {
      try {
        const raw = await w.get_rebuild_cause_wasm();
        if (raw == null || raw === "null" || raw === "undefined") {
          setRebuildCause(null);
        } else {
          const parsed =
            typeof raw === "string" ? JSON.parse(raw) : (raw as RebuildCause);
          setRebuildCause(parsed);
        }
      } catch {
        // ignore — refresh will retry on next tick
      }
    }

    if (typeof w.get_logic_activation_events_wasm === "function") {
      try {
        const raw = await w.get_logic_activation_events_wasm();
        const parsed =
          typeof raw === "string"
            ? JSON.parse(raw)
            : (raw as LogicActivationEvent[]);
        setEvents(Array.isArray(parsed) ? parsed : EMPTY_EVENTS);
      } catch {
        setEvents(EMPTY_EVENTS);
      }
    }

    if (typeof w.get_logic_log_state === "function") {
      try {
        const raw = await w.get_logic_log_state();
        const parsed =
          typeof raw === "string" ? JSON.parse(raw) : (raw as LogicLogState);
        setLegacy(parsed);
      } catch {
        setLegacy(null);
      }
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => {
      void refresh();
    }, pollIntervalMs);
    return () => window.clearInterval(id);
  }, [pollIntervalMs, refresh]);

  return { events, rebuildCause, legacy, refresh };
}
