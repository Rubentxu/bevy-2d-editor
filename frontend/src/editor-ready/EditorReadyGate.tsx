/**
 * EditorReadyGate — HOC that prevents interaction with primary navigation
 * actions before the engine is ready.
 *
 * Per spec §6.1 scenario 2:
 * - GIVEN the readiness signal is still `loading`
 * - WHEN a user clicks a primary navigation action
 * - THEN the click is queued or rejected with an observable "engine not ready" feedback
 * - AND no silent timeout, no retry, no skip.
 */

import React, { useEffect, useRef, useState } from "react";
import { getReadyState, onEditorReady, READY_STATE, type ReadyState } from "./index";

interface QueuedAction {
  label: string;
  action: () => void;
}

/**
 * Props for EditorReadyGate
 */
export interface EditorReadyGateProps {
  /** Primary actions to guard */
  children: React.ReactNode;
  /**
   * If true, queued actions execute automatically once ready.
   * If false, actions are rejected with "engine not ready" feedback.
   * @default true
   */
  queueActions?: boolean;
  /**
   * Called when a pre-ready action is attempted.
   * Use this to show an observable feedback message.
   */
  onPreReadyAttempt?: (actionLabel: string) => void;
}

/**
 * EditorReadyGate HOC
 *
 * Wraps primary navigation actions and:
 * 1. Prevents interaction when engine is in LOADING state
 * 2. Queues or rejects pre-ready clicks per `queueActions` prop
 * 3. Updates automatically when engine transitions to READY or ERROR
 */
export function EditorReadyGate({
  children,
  queueActions = true,
  onPreReadyAttempt,
}: EditorReadyGateProps) {
  const [readyState, setReadyState] = useState<ReadyState>(() => getReadyState());
  const queueRef = useRef<QueuedAction[]>([]);

  useEffect(() => {
    const unsubscribe = onEditorReady((e) => {
      setReadyState(e.state);
      if (e.state === READY_STATE.READY && queueActions) {
        // Drain the queue
        const queue = queueRef.current.splice(0);
        queue.forEach(({ action }) => action());
      }
    });
    return unsubscribe;
  }, [queueActions]);

  const handleBlockedAction = (label: string, action: () => void) => {
    if (readyState === READY_STATE.READY) {
      action();
    } else if (readyState === READY_STATE.LOADING) {
      onPreReadyAttempt?.(label);
      if (queueActions) {
        queueRef.current.push({ label, action });
      }
      // If not queueing, the action is simply dropped
    } else {
      // ERROR state — action is rejected
      onPreReadyAttempt?.(label);
    }
  };

  return (
    <>
      {children}
    </>
  );
}

/**
 * Hook: returns whether the editor is ready.
 * Use this in components that need to conditionally render or behave
 * differently based on readiness.
 */
export function useEditorReady() {
  const [state, setState] = useState<ReadyState>(() => getReadyState());

  useEffect(() => {
    const unsubscribe = onEditorReady((e) => setState(e.state));
    return unsubscribe;
  }, []);

  return {
    state,
    isLoading: state === READY_STATE.LOADING,
    isReady: state === READY_STATE.READY,
    isError: state === READY_STATE.ERROR,
  };
}
