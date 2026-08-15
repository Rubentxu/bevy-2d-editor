/**
 * Waits until the WASM engine reports readiness through the single signal
 * `window.__bevyEngineStarted`. Returns immediately if the signal is
 * already true. Rejects (via throw) after `timeoutMs` so callers can
 * surface a meaningful diagnostic instead of hanging.
 *
 * This helper replaces the ad-hoc polling loops previously duplicated in
 * `useAIAssistant.ts` and `useLogicGraph.ts`. The single readiness
 * signal is published by `engine-bridge.ts:initEngine` only after
 * `start_engine` returned without throwing, so waiting for it is the
 * correct contract: the WASM linear-memory views are valid, the Bevy
 * `App` has finished its first frame, and any bridge export is safe to
 * call.
 */
export async function waitForEditorReady(
  timeoutMs: number = 20_000,
  pollMs: number = 50,
): Promise<void> {
  const start = Date.now();
  while (!(
    typeof window !== "undefined" &&
    (window as any).__bevyEngineStarted === true
  )) {
    if (Date.now() - start > timeoutMs) {
      throw new Error(
        `Editor engine did not become ready within ${timeoutMs}ms.`,
      );
    }
    await new Promise((resolve) => setTimeout(resolve, pollMs));
  }
}

export function isEditorReady(): boolean {
  return (
    typeof window !== "undefined" &&
    (window as any).__bevyEngineStarted === true
  );
}
