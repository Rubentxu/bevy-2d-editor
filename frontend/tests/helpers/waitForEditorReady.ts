import { Page } from "@playwright/test";

/**
 * Single readiness contract for Playwright specs. Waits for the WASM
 * engine to publish `window.__bevyEngineStarted === true`, which
 * `frontend/src/engine-bridge.ts` only sets after `start_engine` returned
 * without throwing and the Bevy App has finished its first frame.
 *
 * Replaces ad-hoc waitForFunction patterns that poll individual bridge
 * exports (some of which were exposed before start_engine ran and so
 * could be `undefined` instead of functions).
 */
export async function waitForEditorReady(
  page: Page,
  timeoutMs: number = 30_000,
): Promise<void> {
  await page.waitForFunction(
    () => (window as any).__bevyEngineStarted === true,
    undefined,
    { timeout: timeoutMs },
  );
}