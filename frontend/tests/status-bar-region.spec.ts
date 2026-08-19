/**
 * T2.6 — Status bar fills the grid region.
 *
 * Validates that the `.status-bar` visually occupies its allocated grid cell
 * at every supported viewport — the bar is not shrunk, not overflow-x,
 * and not `position: fixed` overriding the grid layout.
 *
 * Viewports:
 *   - 1920×1080  (desktop well above threshold)
 *   - 1366×768   (desktop above threshold)
 *   - 1280×800   (desktop at threshold — common laptop resolution)
 */
import { waitForEditorReady } from "./helpers/waitForEditorReady";


import { expect, type Page, test } from "@playwright/test";



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

test.describe("Status bar fills the grid region (T2.6)", { tag: ["@full"] }, () => {
  for (const [label, width, height] of [
    ["1920×1080", 1920, 1080],
    ["1366×768", 1366, 768],
    ["1280×800", 1280, 800],
  ] as const) {
    test.describe(`at ${label}`, () => {
      test.beforeEach(async ({ page }) => {
        await page.setViewportSize({ width, height });
        await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);
        await dismissWelcomeIfPresent(page);
      });

      test("status bar is visible and fills its grid cell", async ({ page }) => {
        const statusBar = page.locator('[data-testid="status-bar"]');
        await expect(statusBar).toBeVisible();

        // The status bar must not be position:fixed (which would lift it out of
        // the grid). It should be inside the dock-layout-status region.
        const layoutStatus = page.locator(
          '[data-testid="dock-region-status"]',
        );
        await expect(layoutStatus).toBeVisible();

        // Verify the status bar is a descendant of the layout status region via DOM.
        const isDescendant = await page.evaluate(() => {
          const statusBar = document.querySelector('[data-testid="status-bar"]');
          const layoutStatus = document.querySelector(
            '[data-testid="dock-region-status"]',
          );
          if (!statusBar || !layoutStatus) return false;
          return layoutStatus.contains(statusBar);
        });
        expect(isDescendant).toBe(true);

        // Check via bounding box that status bar width matches the viewport width.
        const bbox = await statusBar.boundingBox();
        expect(bbox).not.toBeNull();
        // Allow 1px tolerance for sub-pixel rendering.
        expect(bbox!.width).toBeGreaterThanOrEqual(width - 1);
        expect(bbox!.x).toBeLessThanOrEqual(1);
      });

      test("no horizontal overflow on the status bar", async ({ page }) => {
        const statusBar = page.locator('[data-testid="status-bar"]');
        await expect(statusBar).toBeVisible();

        // Evaluate overflow on the status bar element.
        const hasOverflow = await page.evaluate(() => {
          const bar = document.querySelector('[data-testid="status-bar"]');
          if (!bar) return false;
          return bar.scrollWidth > bar.clientWidth;
        });
        expect(hasOverflow).toBe(false);
      });

      test("all status segments are visible (not clipped)", async ({ page }) => {
        const statusBar = page.locator('[data-testid="status-bar"]');
        await expect(statusBar).toBeVisible();

        // Key segments must be visible: position, scene, zoom, fps, build.
        const segments = [
          '[data-testid="status-segment-position"]',
          '[data-testid="status-segment-scene"]',
          '[data-testid="status-segment-zoom"]',
          '[data-testid="status-segment-fps"]',
          '[data-testid="status-segment-build"]',
        ];
        for (const sel of segments) {
          const seg = page.locator(sel).first();
          await expect(seg).toBeVisible();
        }
      });
    });
  }
});
