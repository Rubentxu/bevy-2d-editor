/**
 * useChangeWorkbench — state manager for the ChangeWorkbench panel.
 *
 * Manages:
 * - The list of pending ChangeSets awaiting approval
 * - Per-ChangeSet selection state (which op indices are selected)
 * - The queue of ChangeSets being reviewed
 * - Recent ChangeSet history summaries
 *
 * Exposes actions: approve, approveSelected, reject.
 *
 * This hook is NOT responsible for rendering — it only manages state and
 * delegates I/O to EditorGateway.
 */

import { useCallback, useReducer, type Reducer } from "react";
import {
  getEditorGateway,
  type PendingChangeSetSummary,
  type ChangeSetSummary,
  type ApproveSelectedOpsResult,
} from "../services/EditorGateway";

// ─── Types ────────────────────────────────────────────────────────────────────

export interface PendingChangeSet extends PendingChangeSetSummary {
  /** Indices of ops currently selected for approval. */
  selectedIndices: Set<number>;
  /** Expanded/collapsed state of the card. */
  expanded: boolean;
}

export interface ChangeSetQueueState {
  /** All pending ChangeSets retrieved from the WASM registry. */
  pending: PendingChangeSet[];
  /** ChangeSets that were recently approved/rejected (for the history panel). */
  history: ChangeSetSummary[];
  /** Whether the pending list is currently being loaded. */
  loading: boolean;
  /** Error message if the last fetch failed. */
  error: string | null;
  /** ID of the ChangeSet currently being reviewed (or null). */
  activeId: string | null;
}

// ─── Actions ─────────────────────────────────────────────────────────────────

type Action =
  | { type: "SET_PENDING"; pending: PendingChangeSet[] }
  | { type: "SET_HISTORY"; history: ChangeSetSummary[] }
  | { type: "SET_LOADING"; loading: boolean }
  | { type: "SET_ERROR"; error: string | null }
  | { type: "SET_ACTIVE"; id: string | null }
  | { type: "TOGGLE_OP"; id: string; index: number }
  | { type: "SELECT_ALL"; id: string }
  | { type: "DESELECT_ALL"; id: string }
  | { type: "REMOVE_PENDING"; id: string };

function reducer(
  state: ChangeSetQueueState,
  action: Action,
): ChangeSetQueueState {
  switch (action.type) {
    case "SET_PENDING":
      return { ...state, pending: action.pending, loading: false, error: null };

    case "SET_HISTORY":
      return { ...state, history: action.history };

    case "SET_LOADING":
      return { ...state, loading: action.loading };

    case "SET_ERROR":
      return { ...state, error: action.error, loading: false };

    case "SET_ACTIVE": {
      const pending = state.pending.map((cs) => ({
        ...cs,
        expanded: cs.id === action.id ? true : cs.expanded,
      }));
      return { ...state, activeId: action.id, pending };
    }

    case "TOGGLE_OP": {
      const pending = state.pending.map((cs) => {
        if (cs.id !== action.id) return cs;
        const selectedIndices = new Set(cs.selectedIndices);
        if (selectedIndices.has(action.index)) {
          selectedIndices.delete(action.index);
        } else {
          selectedIndices.add(action.index);
        }
        return { ...cs, selectedIndices };
      });
      return { ...state, pending };
    }

    case "SELECT_ALL": {
      const pending = state.pending.map((cs) => {
        if (cs.id !== action.id) return cs;
        return {
          ...cs,
          selectedIndices: new Set(
            Array.from({ length: cs.op_count }, (_, i) => i),
          ),
        };
      });
      return { ...state, pending };
    }

    case "DESELECT_ALL": {
      const pending = state.pending.map((cs) => {
        if (cs.id !== action.id) return cs;
        return { ...cs, selectedIndices: new Set<number>() };
      });
      return { ...state, pending };
    }

    case "REMOVE_PENDING":
      return {
        ...state,
        pending: state.pending.filter((cs) => cs.id !== action.id),
        activeId: state.activeId === action.id ? null : state.activeId,
      };

    default:
      return state;
  }
}

const initialState: ChangeSetQueueState = {
  pending: [],
  history: [],
  loading: false,
  error: null,
  activeId: null,
};

// ─── Hook ─────────────────────────────────────────────────────────────────────

export function useChangeWorkbench() {
  const [state, dispatch] = useReducer(reducer, initialState);

  /** Refresh the pending ChangeSets from the WASM registry. */
  const refreshPending = useCallback(async () => {
    dispatch({ type: "SET_LOADING", loading: true });
    const gateway = getEditorGateway();
    const result = await gateway.getPendingChangeSets();
    if (!result.ok) {
      dispatch({ type: "SET_ERROR", error: result.error ?? "Unknown error" });
      return;
    }
    const pending: PendingChangeSet[] = (result.value ?? []).map((cs) => ({
      ...cs,
      selectedIndices: new Set<number>(),
      expanded: false,
    }));
    dispatch({ type: "SET_PENDING", pending });
  }, []);

  /** Refresh recent history from the operation log. */
  const refreshHistory = useCallback(async () => {
    const gateway = getEditorGateway();
    const result = await gateway.getChangeSetSummaries();
    if (!result.ok) {
      // History errors are non-fatal — just log and keep current history.
      console.warn("[ChangeWorkbench] failed to load history:", result.error);
      return;
    }
    dispatch({ type: "SET_HISTORY", history: result.value ?? [] });
  }, []);

  /** Load both pending and history. */
  const load = useCallback(async () => {
    await Promise.all([refreshPending(), refreshHistory()]);
  }, [refreshPending, refreshHistory]);

  /** Approve all ops in a ChangeSet. */
  const approveChangeSet = useCallback(
    async (id: string) => {
      const gateway = getEditorGateway();
      const result = await gateway.approveChangeSet(id);
      if (!result.ok) {
        dispatch({
          type: "SET_ERROR",
          error: result.error ?? "Approve failed",
        });
        return false;
      }
      dispatch({ type: "REMOVE_PENDING", id });
      await refreshHistory();
      return true;
    },
    [refreshHistory],
  );

  /** Approve only the selected op indices in a ChangeSet. */
  const approveSelectedOps = useCallback(
    async (id: string, indices: number[]) => {
      if (indices.length === 0) return false;
      const gateway = getEditorGateway();
      const result = await gateway.approveSelectedOps(id, indices);
      if (!result.ok) {
        dispatch({
          type: "SET_ERROR",
          error: result.error ?? "Approve failed",
        });
        return false;
      }
      dispatch({ type: "REMOVE_PENDING", id });
      await refreshHistory();
      return true;
    },
    [refreshHistory],
  );

  /** Reject and discard a ChangeSet. */
  const rejectChangeSet = useCallback(async (id: string) => {
    const gateway = getEditorGateway();
    const result = await gateway.rejectChangeSet(id);
    if (!result.ok) {
      dispatch({ type: "SET_ERROR", error: result.error ?? "Reject failed" });
      return false;
    }
    dispatch({ type: "REMOVE_PENDING", id });
    return true;
  }, []);

  /** Toggle a single op selection in a ChangeSet. */
  const toggleOp = useCallback((id: string, index: number) => {
    dispatch({ type: "TOGGLE_OP", id, index });
  }, []);

  /** Select all ops in a ChangeSet. */
  const selectAll = useCallback((id: string) => {
    dispatch({ type: "SELECT_ALL", id });
  }, []);

  /** Deselect all ops in a ChangeSet. */
  const deselectAll = useCallback((id: string) => {
    dispatch({ type: "DESELECT_ALL", id });
  }, []);

  /** Set the actively-reviewed ChangeSet. */
  const setActive = useCallback((id: string | null) => {
    dispatch({ type: "SET_ACTIVE", id });
  }, []);

  return {
    state,
    load,
    refreshPending,
    refreshHistory,
    approveChangeSet,
    approveSelectedOps,
    rejectChangeSet,
    toggleOp,
    selectAll,
    deselectAll,
    setActive,
  };
}
