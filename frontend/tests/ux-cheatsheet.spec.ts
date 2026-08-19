import { test, expect, Page } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";

/**
 * Phase 3.3 — Cheat Sheet (`?` key) test.
 */



test.describe("UX Cheat Sheet — Phase 3.3", { tag: ["@full"] }, () => {
  test("`?` opens the cheat sheet", async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);

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