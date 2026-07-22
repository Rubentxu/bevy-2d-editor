import { test, expect, Page } from "@playwright/test";

/**
 * Phase 5 — Toast system (UX Overhaul).
 *
 * Validates that:
 *   - Triggering an error dispatches a toast with severity="error".
 *   - The toast auto-dismisses after ~5s (we wait 5.5s to allow a small
 *     grace period).
 */

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () =>
      typeof (window as any).load_scene_json === "function" &&
      typeof (window as any).get_scene_snapshot === "function",
    undefined,
    { timeout: 30_000 },
  );
}

test.describe("UX Toasts — Phase 5", () => {
  test("Triggering an error surfaces a toast that auto-dismisses after 5s", async ({
    page,
  }) => {
    await page.goto("/");
    await waitForEngine(page);

    // Drive a known error path through App.tsx: handleRename with no entity
    // selection would normally short-circuit, so instead we hit handleLoad
    // by replacing the WASM function to throw — the catch in App.tsx calls
    // addToast("Load project failed: …", "error").
    await page.evaluate(() => {
      const original = (window as any).load_project;
      (window as any).load_project = () =>
        Promise.reject(new Error("synthetic-load-failure"));
      // Stash for cleanup.
      (window as any).__load_project_orig = original;
    });

    // The Load button lives in the Edit toolbar group.
    const loadBtn = page.locator('[data-testid="load-btn"]');
    await expect(loadBtn).toBeVisible();
    await loadBtn.click();

    // Toast appears.
    const toast = page.locator('[data-testid="toast-error"]');
    await expect(toast).toBeVisible({ timeout: 5_000 });
    await expect(toast).toContainText("Load project failed");

    // Auto-dismiss after ~5s. We wait a little longer to absorb scheduling
    // jitter from the 500ms prune interval in useToasts.
    await page.waitForFunction(
      () => document.querySelectorAll('[data-testid="toast-error"]').length === 0,
      undefined,
      { timeout: 5_500 },
    );

    // Confirm the toast is gone.
    await expect(toast).toHaveCount(0);
  });
});
