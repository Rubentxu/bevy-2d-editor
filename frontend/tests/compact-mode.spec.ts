/**
 * T1.7 — Compact mode regression test.
 *
 * Validates that the editor renders in compact mode (single column with tabs)
 * when the viewport is below 1280 px, and that no UI overflows or is
 * unreachable.
 *
 * Viewports covered:
 *   - 1024×768  (below threshold, landscape)
 *   - 768×1024  (below threshold, portrait)
 *
 * Compact mode is a single-column layout with tabs for panels:
 *   Assets · Scene · Outline · Properties · Tools
 *
 * The Scene tab is the default and fills the main area.
 */
import { expect, Page, test } from "@playwright/test";
/**
 * Dismiss the Welcome overlay if it appears.
 */
import { waitForEditorReady } from "./helpers/waitForEditorReady";

async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  const overlay = page.locator('[data-testid="welcome-overlay"]');
  await page.waitForTimeout(500);
  const count = await overlay.count();
  if (count === 0) return;
  const skipBtn = overlay.locator('[data-testid="welcome-skip-btn"]');
  try {
    await skipBtn.click({ force: true, timeout: 5_000 });
    await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  } catch {
    /* swallow */
  }
}

/** Check whether the dock layout is in compact mode (from data attribute). */
async function isCompactMode(page: Page): Promise<boolean> {
  return await page.evaluate(() => {
    const el = document.querySelector('[data-testid="dock-layout"]');
    return el?.getAttribute("data-compact") === "true";
  });
}

test.describe("Compact mode below 1280 px threshold", { tag: ["@full"] }, () => {
  for (const [label, width, height] of [
    ["1024×768 (landscape)", 1024, 768],
    ["768×1024 (portrait)", 768, 1024],
  ] as const) {
    test.describe(`at ${label}`, () => {
      test.beforeEach(async ({ page }) => {
        await page.goto("/?skip-welcome=1");
        await page.setViewportSize({ width, height });
        await waitForEditorReady(page);
        await dismissWelcomeIfPresent(page);
      });

      test("dock layout renders in compact mode", async ({ page }) => {
        const compact = await isCompactMode(page);
        expect(compact).toBe(true);
      });

      test("compact tab bar is visible with expected tabs", async ({
        page,
      }) => {
        const tabBar = page.locator('[data-testid="dock-compact-tabs"]');
        await expect(tabBar).toBeVisible();

        // Assets, Scene, Outline, Properties tabs must be present
        for (const tab of ["assets", "scene", "outline", "properties"]) {
          const tabBtn = page.locator(
            `[data-testid="dock-compact-tab-${tab}"]`,
          );
          await expect(tabBtn).toBeVisible();
        }
      });

      test("Scene tab is active by default", async ({ page }) => {
        const sceneTab = page.locator(
          '[data-testid="dock-compact-tab-scene"]',
        );
        await expect(sceneTab).toHaveAttribute("aria-selected", "true");
      });

      test("switching tabs changes the active panel", async ({ page }) => {
        // Click Assets tab
        const assetsTab = page.locator(
          '[data-testid="dock-compact-tab-assets"]',
        );
        await assetsTab.click();
        await expect(assetsTab).toHaveAttribute("aria-selected", "true");

        // Scene tab should no longer be selected
        const sceneTab = page.locator(
          '[data-testid="dock-compact-tab-scene"]',
        );
        await expect(sceneTab).toHaveAttribute("aria-selected", "false");
      });

      test("no UI element overflows the viewport (no horizontal scroll)", async ({
        page,
      }) => {
        // Check that the body does not have unexpected overflow
        const bodyOverflow = await page.evaluate(() => {
          return {
            overflowX: getComputedStyle(document.body).overflowX,
            clientWidth: document.body.clientWidth,
            scrollWidth: document.body.scrollWidth,
          };
        });

        // clientWidth should equal scrollWidth in compact mode (no overflow)
        expect(bodyOverflow.scrollWidth).toBeLessThanOrEqual(
          bodyOverflow.clientWidth + 1,
        );
      });

      test("menu bar is visible and menus are still functional in compact mode", async ({
        page,
      }) => {
        // Menu bar must be visible
        await expect(
          page.locator('[data-testid="menubar"]'),
        ).toBeVisible();

        // Open File menu — use chained locator for the portaled dropdown
        await page
          .locator('[data-testid="menu-file"] .menu-trigger')
          .click();

        const dropdown = page.locator('[data-testid="menu-dropdown"]');
        await expect(dropdown).toBeVisible();

        // Dropdown must not be clipped
        const dropdownBox = await dropdown.boundingBox();
        expect(dropdownBox).not.toBeNull();
        expect(dropdownBox!.height).toBeGreaterThan(20);

        // Close with Escape
        await page.keyboard.press("Escape");
        await expect(dropdown).not.toBeAttached();
      });

      test("status bar is visible in compact mode", async ({ page }) => {
        await expect(
          page.locator('[data-testid="dock-region-status"]'),
        ).toBeVisible();
      });

      test("switching to Assets tab shows the left dock content", async ({
        page,
      }) => {
        const assetsTab = page.locator(
          '[data-testid="dock-compact-tab-assets"]',
        );
        await assetsTab.click();

        // The assets panel should be visible (rendered as left dock)
        const assetsPanel = page.locator(
          '[data-testid="dock-compact-panel-assets"]',
        );
        await expect(assetsPanel).toBeVisible();
      });
    });
  }
});

/**
 * Regression guard: at 1280 px (exactly at threshold) the layout must still
 * use the desktop (non-compact) grid. This prevents compact mode from
 * accidentally triggering on valid 1280 px screens.
 */
test.describe("At exactly 1280 px (threshold boundary)", { tag: ["@full"] }, () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await page.setViewportSize({ width: 1280, height: 800 });
    await waitForEditorReady(page);
    await dismissWelcomeIfPresent(page);
  });

  test("uses desktop (non-compact) layout", async ({ page }) => {
    const compact = await isCompactMode(page);
    expect(compact).toBe(false);
  });

  test("3-column dock layout is visible", async ({ page }) => {
    await expect(
      page.locator('[data-testid="dock-region-left"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="dock-region-center"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="dock-region-right"]'),
    ).toBeVisible();
  });
});
