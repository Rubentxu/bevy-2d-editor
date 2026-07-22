import { test, expect, Page } from "@playwright/test";

/**
 * Phase 3.3 — Cheat Sheet (`?` key) test.
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

test.describe("UX Cheat Sheet — Phase 3.3", () => {
  test("`?` opens the cheat sheet", async ({ page }) => {
    await page.goto("/");
    await waitForEngine(page);

    // Ensure no input is focused so `?` is not absorbed
    await page.evaluate(() => {
      const active = document.activeElement as HTMLElement | null;
      if (active && typeof active.blur === "function") active.blur();
    });

    await expect(page.locator('[data-testid="cheat-sheet"]')).toHaveCount(0);

    await page.keyboard.press("?");

    await expect(page.locator('[data-testid="cheat-sheet"]')).toBeVisible({
      timeout: 5_000,
    });

    // At least one shortcut row should render
    const rows = page.locator('[data-testid^="cheat-sheet-row-"]');
    await expect(rows.first()).toBeVisible({ timeout: 2_000 });
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThan(5);

    // Escape closes
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="cheat-sheet"]')).toHaveCount(0, {
      timeout: 2_000,
    });
  });
});