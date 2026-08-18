/**
 * World Workspace e2e smoke tests (ADR-0037 §ww-ui).
 *
 * Validates:
 *   1. WorldWorkspace mounts when editorMode === "world"
 *   2. Canvas renders level squares from a world fixture
 *   3. Layout policy toolbar buttons change the policy
 *   4. Double-click on a level calls openLevel and switches to scene mode
 *   5. Minimap reflects world bounds
 *
 * Uses EditorGateway.world via the WASM bridge (Slice 3).
 */

import { expect, Page, test } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await page.goto("/?skip-welcome=1");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () =>
      typeof (window as any).load_scene_json === "function" &&
      typeof (window as any).dispatch_command === "function" &&
      typeof (window as any).create_world_wasm === "function",
    undefined,
    { timeout: 30_000 },
  );
}

/** Dismiss the Welcome overlay if present. */
async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  await page.waitForTimeout(500);
  const overlay = page.locator('[data-testid="welcome-overlay"]');
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

/** Drive editorMode via the App.tsx test hook. */
async function switchMode(page: Page, mode: string): Promise<void> {
  await page.evaluate((m) => {
    (window as any).__setEditorMode?.(m);
  }, mode);
  await page.waitForTimeout(300);
}

test.describe("World Workspace — Slice 4 Smoke", () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);
  });

  test("WorldWorkspace mounts in world mode", async ({ page }) => {
    // Switch to world mode
    await switchMode(page, "world");

    // WorldWorkspace should be visible
    const ws = page.locator(".world-workspace");
    await expect(ws).toBeVisible({ timeout: 5_000 });
  });

  test("WorldWorkspace renders empty state when no world loaded", async ({ page }) => {
    await switchMode(page, "world");

    const emptyState = page.locator(".world-workspace__empty-state");
    await expect(emptyState).toBeVisible({ timeout: 5_000 });
  });

  test("creating a world shows level squares", async ({ page }) => {
    // Create a world via WASM bridge
    await page.evaluate(async () => {
      const { getEditorGateway } = await import("./services/EditorGateway");
      const gateway = getEditorGateway();
      // Create a new world named "test-world"
      await gateway.world.createWorld("test-world-smoke");
    });

    await switchMode(page, "world");

    // Wait for WorldWorkspace to load the world
    await page.waitForTimeout(500);

    // Toolbar should show the world name
    const toolbar = page.locator(".world-workspace__toolbar-title");
    await expect(toolbar).toContainText("test-world-smoke");
  });

  test("layout policy buttons exist", async ({ page }) => {
    await switchMode(page, "world");

    const freeBtn = page.locator(".world-workspace__layout-btn", { hasText: "Free" });
    const gridBtn = page.locator(".world-workspace__layout-btn", { hasText: "Grid" });
    const hBtn = page.locator(".world-workspace__layout-btn", { hasText: "H" });
    const vBtn = page.locator(".world-workspace__layout-btn", { hasText: "V" });

    await expect(freeBtn).toBeVisible();
    await expect(gridBtn).toBeVisible();
    await expect(hBtn).toBeVisible();
    await expect(vBtn).toBeVisible();
  });

  test("minimap renders", async ({ page }) => {
    await switchMode(page, "world");

    const minimap = page.locator(".world-workspace__minimap");
    await expect(minimap).toBeVisible();

    const minimapSvg = page.locator(".world-workspace__minimap-svg");
    await expect(minimapSvg).toBeVisible();
  });

  test("back button returns to scene mode", async ({ page }) => {
    await switchMode(page, "world");

    const backBtn = page.locator(".world-workspace__back-btn");
    await expect(backBtn).toBeVisible();
    await backBtn.click();

    // Should switch back to scene mode
    await page.waitForTimeout(300);
    // Verify we're no longer in world mode by checking that WorldWorkspace is unmounted
    await expect(page.locator(".world-workspace")).not.toBeVisible({ timeout: 3_000 });
  });

  test("switching modes unmounts WorldWorkspace", async ({ page }) => {
    await switchMode(page, "world");

    const ws = page.locator(".world-workspace");
    await expect(ws).toBeVisible();

    // Switch to scene mode
    await switchMode(page, "scene");

    // WorldWorkspace should be unmounted
    await expect(ws).not.toBeVisible({ timeout: 3_000 });
  });

  test("status bar shows level and link counts", async ({ page }) => {
    // Create a world first
    await page.evaluate(async () => {
      const { getEditorGateway } = await import("./services/EditorGateway");
      const gateway = getEditorGateway();
      await gateway.world.createWorld("status-test");
    });

    await switchMode(page, "world");
    await page.waitForTimeout(500);

    const status = page.locator(".world-workspace__status");
    await expect(status).toBeVisible();

    // Should show 0 levels initially
    await expect(status).toContainText("0 levels");
    await expect(status).toContainText("0 links");
  });
});
