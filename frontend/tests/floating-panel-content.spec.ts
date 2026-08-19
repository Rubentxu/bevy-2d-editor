/**
 * T2.5 — Floating panel renders real dock content.
 *
 * Validates that when a panel is floated via the dock header "Float" action,
 * the floating portal renders the actual panel body (AssetNavigator / HierarchyPanel /
 * InspectorPanel / ConsoleTab) instead of a placeholder div.
 *
 * Viewports: 1920×1080 (desktop above threshold).
 */
import { expect, type Page, test } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";

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
    /* overlay may have unmounted itself */
  }
}

const FLOAT_SELECTORS: Record<string, string> = {
  assets: "[data-testid='dock-left-header-float']",
  outline: "[data-testid='dock-right-outline-header-float']",
  properties: "[data-testid='dock-right-properties-header-float']",
  bottom: "[data-testid='dock-bottom-float']",
};

async function clickFloatToggle(page: Page, panelId: string): Promise<void> {
  const sel =
    FLOAT_SELECTORS[panelId] ?? `[data-testid='dock-${panelId}-header-float']`;
  await page.locator(sel).first().click({ force: true });
}

test.describe("Floating panel renders real dock content (T2.5)", { tag: ["@full"] }, () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await page.setViewportSize({ width: 1920, height: 1080 });
    await waitForEditorReady(page);
    await dismissWelcomeIfPresent(page);
    await expect(page.locator('[data-testid="dock-layout"]')).toBeVisible();
  });

  test("floated assets panel renders AssetNavigator (not a placeholder)", async ({
    page,
  }) => {
    // Float the assets panel.
    await clickFloatToggle(page, "assets");
    const floatPanel = page.locator('[data-testid="floating-panel-assets"]');
    await expect(floatPanel).toBeVisible({ timeout: 5_000 });

    // The body must contain AssetNavigator (by testid or role), not a placeholder.
    const body = floatPanel.locator('[data-testid="floating-panel-assets-body"]');
    await expect(body).toBeVisible();
    // AssetNavigator renders a tree or folder structure — look for the asset tree.
    await expect(
      body.locator('[data-testid="asset-navigator"]'),
    ).toBeVisible({ timeout: 5_000 });
    // Must NOT be the old placeholder text.
    await expect(body).not.toContainText("currently floating");
  });

  test("floated outline panel renders HierarchyPanel (not a placeholder)", async ({
    page,
  }) => {
    await clickFloatToggle(page, "outline");
    const floatPanel = page.locator('[data-testid="floating-panel-outline"]');
    await expect(floatPanel).toBeVisible({ timeout: 5_000 });

    const body = floatPanel.locator('[data-testid="floating-panel-outline-body"]');
    await expect(body).toBeVisible();
    // HierarchyPanel is the outline content in scene mode.
    await expect(body.locator('[data-testid="hierarchy-panel"]')).toBeVisible({
      timeout: 5_000,
    });
    await expect(body).not.toContainText("currently floating");
  });

  test("floated properties panel renders InspectorPanel (not a placeholder)", async ({
    page,
  }) => {
    await clickFloatToggle(page, "properties");
    const floatPanel = page.locator(
      '[data-testid="floating-panel-properties"]',
    );
    await expect(floatPanel).toBeVisible({ timeout: 5_000 });

    const body = floatPanel.locator(
      '[data-testid="floating-panel-properties-body"]',
    );
    await expect(body).toBeVisible();
    // InspectorPanel is the properties content in scene mode.
    await expect(
      body.locator('[data-testid="inspector-panel"]'),
    ).toBeVisible({ timeout: 5_000 });
    await expect(body).not.toContainText("currently floating");
  });

  test("floated bottom panel renders ConsoleTab (not a placeholder)", async ({
    page,
  }) => {
    await clickFloatToggle(page, "bottom");
    const floatPanel = page.locator('[data-testid="floating-panel-bottom"]');
    await expect(floatPanel).toBeVisible({ timeout: 5_000 });

    const body = floatPanel.locator('[data-testid="floating-panel-bottom-body"]');
    await expect(body).toBeVisible();
    // ConsoleTab renders with data-testid="bottom-tabpanel-console".
    await expect(
      body.locator('[data-testid="bottom-tabpanel-console"]'),
    ).toBeVisible({ timeout: 5_000 });
    await expect(body).not.toContainText("currently floating");
  });
});
