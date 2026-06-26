import { test, expect } from "@playwright/test";

test.describe("Spike — Smoke Tests", () => {
  test("page loads with correct title", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveTitle("Bevy 2D Editor — Spike");
  });

  test("canvas element exists with correct id", async ({ page }) => {
    await page.goto("/");
    const canvas = page.locator("#bevy-canvas");
    await expect(canvas).toBeVisible();
  });

  test("sidebar panel renders with controls", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h2", { hasText: "Spike" })).toBeVisible();
    await expect(page.locator('input[type="number"]')).toHaveCount(2);
    await expect(page.getByText("Move Sprite")).toBeVisible();
  });

  test("initial status shows Loading WASM", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("Loading WASM...")).toBeVisible({ timeout: 5_000 });
  });
});
