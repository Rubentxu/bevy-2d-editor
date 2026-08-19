import { test, expect } from "@playwright/test";

test.describe("Auto Layer Panel", { tag: ["@domain"] }, () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app and wait for it to load
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: 10_000 });
  });

  test("Auto Layer button appears in topbar", async ({ page }) => {
    // The Auto Layer button should be visible in the topbar
    const autoLayerBtn = page.locator('[data-testid="auto-layer-panel-btn"]');
    await expect(autoLayerBtn).toBeVisible();
  });

  test("Auto Layer panel opens when button is clicked", async ({ page }) => {
    // Click the Auto Layer button
    await page.click('[data-testid="auto-layer-panel-btn"]');

    // The Auto Layer panel should be visible (may show empty state if no auto layers exist)
    // The panel uses the .auto-layer-panel class
    const panel = page.locator(".auto-layer-panel");
    await expect(panel).toBeVisible({ timeout: 5_000 });
  });

  test("Regenerate button is present in Auto Layer panel", async ({ page }) => {
    // Open the Auto Layer panel
    await page.click('[data-testid="auto-layer-panel-btn"]');

    // Wait for panel to be visible
    const panel = page.locator(".auto-layer-panel");
    await expect(panel).toBeVisible({ timeout: 5_000 });

    // The Regenerate button should be present within the panel
    const regenBtn = panel.locator("button.regen-btn, button:has-text('Regenerate')");
    await expect(regenBtn).toBeVisible();
  });

  test("Pattern grid is rendered when auto layer is selected", async ({ page }) => {
    // Open the Auto Layer panel
    await page.click('[data-testid="auto-layer-panel-btn"]');

    // Wait for panel to be visible
    const panel = page.locator(".auto-layer-panel");
    await expect(panel).toBeVisible({ timeout: 5_000 });

    // The 3x3 pattern grid should be rendered
    const patternGrid = panel.locator(".pattern-grid");
    await expect(patternGrid).toBeVisible();

    // Should have 9 cells (3x3)
    const cells = patternGrid.locator(".pattern-cell");
    await expect(cells).toHaveCount(9);
  });

  test("Add Rule button is present", async ({ page }) => {
    // Open the Auto Layer panel
    await page.click('[data-testid="auto-layer-panel-btn"]');

    // Wait for panel to be visible
    const panel = page.locator(".auto-layer-panel");
    await expect(panel).toBeVisible({ timeout: 5_000 });

    // The Add Rule button should be present
    const addRuleBtn = panel.locator("button:has-text('Add Rule')");
    await expect(addRuleBtn).toBeVisible();
  });

  test("no console errors when toggling Auto Layer panel", async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        errors.push(msg.text());
      }
    });

    // Open the Auto Layer panel
    await page.click('[data-testid="auto-layer-panel-btn"]');
    await page.waitForTimeout(500);

    // Close the panel
    await page.click('[data-testid="auto-layer-panel-btn"]');
    await page.waitForTimeout(500);

    // Filter out known non-critical errors (e.g., WASM loading warnings)
    const criticalErrors = errors.filter(
      (e) => !e.includes("WASM") && !e.includes("WebGL") && !e.includes("deprecated")
    );

    expect(criticalErrors).toHaveLength(0);
  });
});
