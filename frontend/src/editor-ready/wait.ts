/**
 * Promise-based editor readiness contract.
 *
 * This module provides the canonical async wait function for the editor
 * readiness signal. It is the single entry point for async readiness
 * waiting in the frontend.
 *
 * The contract:
 * - `waitForEditorReady()` resolves when `window.__bevyEngineStarted === true`
 * - Resolves immediately if already ready
 * - Rejects with Error after `timeoutMs` (default 20s)
 * - Poll interval is configurable (default 50ms)
 *
 * Usage:
 *   import { waitForEditorReady } from "../editor-ready/wait";
 *   await waitForEditorReady();
 */

const DEFAULT_TIMEOUT_MS = 20_000;
const DEFAULT_POLL_MS = 50;

/**
 * Check if the editor engine has reported readiness synchronously.
 */
export function isEditorReady(): boolean {
  return (
    typeof window !== "undefined" &&
    (window as any).__bevyEngineStarted === true
  );
}

/**
 * Waits until the WASM engine reports readiness through the single signal
 * `window.__bevyEngineStarted`. Returns immediately if the signal is
 * already true. Rejects after `timeoutMs` so callers can surface a
 * meaningful diagnostic instead of hanging.
 *
 * @param timeoutMs - Maximum time to wait in milliseconds (default: 20000)
 * @param pollMs   - Polling interval in milliseconds (default: 50)
 * @throws Error if the editor does not become ready within timeoutMs
 */
export async function waitForEditorReady(
  timeoutMs: number = DEFAULT_TIMEOUT_MS,
  pollMs: number = DEFAULT_POLL_MS,
): Promise<void> {
  if (isEditorReady()) {
    return;
  }

  const start = Date.now();

  while (!isEditorReady()) {
    if (Date.now() - start > timeoutMs) {
      throw new Error(
        `Editor engine did not become ready within ${timeoutMs}ms.`,
      );
    }
    await new Promise((resolve) => setTimeout(resolve, pollMs));
  }
}
