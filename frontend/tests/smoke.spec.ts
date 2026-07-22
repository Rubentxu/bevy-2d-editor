import { test, expect } from "@playwright/test";

test.describe("Spike — Smoke Tests", () => {
  test("page loads with correct title", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveTitle("Bevy 2D Editor");
  });

  test("canvas element exists with correct id", async ({ page }) => {
    await page.goto("/");
    const canvas = page.locator("#bevy-canvas");
    await expect(canvas).toBeVisible();
  });

  test("topbar renders with title and buttons", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator("h1", { hasText: "Bevy 2D Editor" })).toBeVisible();
    await expect(page.locator('[data-testid="undo-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="redo-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="save-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="load-btn"]')).toBeVisible();
  });

  test("hierarchy and inspector panels render", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="hierarchy-panel"]')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('[data-testid="inspector-panel"]')).toBeVisible({ timeout: 10_000 });
  });
});