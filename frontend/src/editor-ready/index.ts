/**
 * editor-ready contract module
 *
 * Single observable readiness signal visible to E2E cohorts (Playwright),
 * unit tests, and React hooks. Replaces fragmented polling across
 * components and hooks.
 *
 * Exports:
 * - READY_STATE: frozen enum of ready states
 * - EditorReadyEvent: event shape for listeners
 * - onEditorReady(): subscribe to readiness changes
 * - getReadyState(): get current state synchronously
 * - waitForEditorReady(): re-export from utils/waitForEditorReady
 */

import { waitForEditorReady as _waitForEditorReady, isEditorReady } from "../utils/waitForEditorReady";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

export const READY_STATE = {
  LOADING: "loading",
  READY: "ready",
  ERROR: "error",
} as const;

export type ReadyState = (typeof READY_STATE)[keyof typeof READY_STATE];

export interface EditorReadyEvent {
  state: ReadyState;
  reason?: string;        // populated when state === ERROR
  measuredMs?: number;    // engine init elapsed
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

type Listener = (e: EditorReadyEvent) => void;

let _currentState: ReadyState = READY_STATE.LOADING;
let _listeners: Set<Listener> = new Set();
let _initStart: number | null = null;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Subscribe to readiness events. Returns an unsubscribe function.
 */
export function onEditorReady(listener: Listener): () => void {
  _listeners.add(listener);
  // Emit current state immediately to new subscriber
  listener(_buildEvent(_currentState));
  return () => {
    _listeners.delete(listener);
  };
}

/**
 * Get the current readiness state synchronously.
 */
export function getReadyState(): ReadyState {
  return _currentState;
}

/**
 * Re-export of the underlying waitForEditorReady helper.
 * Use this in async contexts where you need to await readiness.
 */
export { _waitForEditorReady as waitForEditorReady };

/**
 * Alias for isEditorReady from the underlying helper.
 */
export { isEditorReady };

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function _buildEvent(state: ReadyState, reason?: string): EditorReadyEvent {
  return {
    state,
    reason,
    measuredMs: _initStart != null ? Date.now() - _initStart : undefined,
  };
}

function _emit(state: ReadyState, reason?: string) {
  _currentState = state;
  const event = _buildEvent(state, reason);
  _listeners.forEach((l) => l(event));
}

/**
 * Called by the engine bridge when the engine signals ready.
 * @internal
 */
export function __emitReady() {
  if (_initStart === null) {
    _initStart = Date.now();
  }
  _emit(READY_STATE.READY);
}

/**
 * Called by the engine bridge when the engine fails to initialize.
 * @internal
 */
export function __emitError(reason: string) {
  _emit(READY_STATE.ERROR, reason);
}

/**
 * Called to reset state before a new engine init cycle.
 * @internal
 */
export function __reset() {
  _initStart = null;
  _currentState = READY_STATE.LOADING;
}
