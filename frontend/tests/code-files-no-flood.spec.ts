/**
 * T2.7 — No useCodeFiles error flood on empty sources/.
 *
 * Validates that when the OPFS `sources/` directory is empty or non-existent,
 * useCodeFiles does NOT flood the console with errors. The hook should:
 *   - Tolerate string-or-object shape from listSourceFiles (defensive parse).
 *   - Drop the refresh interval to 5s when no source files exist.
 *   - Pause polling on tab blur (visibilitychange).
 */

import {
  expect,
  type ConsoleMessage,
  type Page,
  test,
} from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await page.goto("/?skip-welcome=1");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
}

async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  await page.waitForTimeout(500);
  const overlay = page.locator('[data-testid="welcome-overlay"]');
  if ((await overlay.count()) === 0) return;
  try {
    await overlay
      .locator('[data-testid="welcome-skip-btn"]')
      .click({ force: true, timeout: 5_000 });
    await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  } catch {
    /* swallow */
  }
}

test.describe("useCodeFiles no error flood (T2.7)", () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });

    // Inject a counter and error list into the page BEFORE any JS runs.
    await page.addInitScript(() => {
      (window as any).__codeFilesErrors = [];
      const origError = console.error.bind(console);
      console.error = (...args: unknown[]) => {
        const text = args
          .map((a) => (typeof a === "string" ? a : JSON.stringify(a)))
          .join(" ");
        if (
          text.includes("useCodeFiles") ||
          text.includes("list_source_files") ||
          text.includes("source_files")
        ) {
          ((window as any).__codeFilesErrors as string[]).push(text);
        }
        origError(...args);
      };
    });

    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);
  });

  test("no console errors from useCodeFiles after 3s with empty sources", async ({
    page,
  }) => {
    // Wait 3 seconds — enough for multiple polls at the fast 500ms cadence.
    // If the interval clamp to 5s works for empty sources, there should be
    // at most 6 fast polls before the clamp kicks in.
    await page.waitForTimeout(3000);

    // Query the error list from the page context.
    const errors = await page.evaluate(
      () => (window as any).__codeFilesErrors ?? [],
    );
    expect(errors).toHaveLength(0);
  });

  test("no error flood after tab loses and regains focus", async ({ page }) => {
    // Wait for initial polls to settle.
    await page.waitForTimeout(1500);

    // Capture error count at the 1.5s mark as baseline.
    const baselineErrors = await page.evaluate(
      () => (window as any).__codeFilesErrors?.length ?? 0,
    );

    // Simulate tab blur (the hook should pause polling via visibilitychange).
    await page.evaluate(() => {
      Object.defineProperty(document, "visibilityState", {
        value: "hidden",
        writable: true,
      });
      document.dispatchEvent(new Event("visibilitychange"));
    });

    // Keep tab hidden for 2 seconds — no polls should fire during this window.
    await page.waitForTimeout(2000);

    // Check that error count did not grow during the hidden window.
    const hiddenErrors = await page.evaluate(
      () => (window as any).__codeFilesErrors?.length ?? 0,
    );
    expect(hiddenErrors).toBe(baselineErrors);

    // Simulate tab regaining focus.
    await page.evaluate(() => {
      Object.defineProperty(document, "visibilityState", {
        value: "visible",
        writable: true,
      });
      document.dispatchEvent(new Event("visibilitychange"));
    });

    // Wait another 2 seconds after restore.
    await page.waitForTimeout(2000);

    // Still zero errors after blur/restore cycle.
    const finalErrors = await page.evaluate(
      () => (window as any).__codeFilesErrors?.length ?? 0,
    );
    expect(finalErrors).toBe(0);
  });
});
