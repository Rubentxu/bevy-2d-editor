import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("Entity Inline Rename", () => {
  test("double-click entity name enters edit mode and Enter commits rename", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function" &&
        typeof (window as any).get_scene_snapshot === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load scene with one entity
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "rename-test",
          name: "Rename Test",
          entities: [
            {
              id: "rename-e1",
              name: "Original Name",
              components: [],
            },
          ],
        })
      )
    );

    // Verify entity shows original name
    await expect(page.locator('[data-testid="hierarchy-entity-rename-e1"]')).toBeVisible();
    const nameSpan = page.locator('[data-testid="hierarchy-entity-rename-e1"] .name');
    await expect(nameSpan).toHaveText("Original Name");

    // Double-click to enter edit mode
    await nameSpan.dblclick();

    // Input should appear and be focused
    const nameInput = page.locator('[data-testid="hierarchy-entity-rename-e1"] .name-input');
    await expect(nameInput).toBeVisible();

    // Clear and type new name
    await nameInput.fill("");
    await nameInput.fill("New Name");

    // Press Enter to commit
    await nameInput.press("Enter");

    // Verify name changed
    await expect(nameSpan).toHaveText("New Name");
  });

  test("Escape cancels rename without committing", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "escape-rename-test",
          name: "Escape Test",
          entities: [{ id: "escape-e1", name: "Keep Me", components: [] }],
        })
      )
    );

    const nameSpan = page.locator('[data-testid="hierarchy-entity-escape-e1"] .name');
    await nameSpan.dblclick();
    const nameInput = page.locator('[data-testid="hierarchy-entity-escape-e1"] .name-input');

    await nameInput.fill("Changed Name");
    await nameInput.press("Escape");

    // Name should remain unchanged
    await expect(nameSpan).toHaveText("Keep Me");
  });

  test("empty name is rejected (no-op)", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    await page.waitForFunction(
      () => typeof (window as any).load_scene_json === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "empty-rename-test",
          name: "Empty Test",
          entities: [{ id: "empty-e1", name: "Original", components: [] }],
        })
      )
    );

    const nameSpan = page.locator('[data-testid="hierarchy-entity-empty-e1"] .name');
    await nameSpan.dblclick();
    const nameInput = page.locator('[data-testid="hierarchy-entity-empty-e1"] .name-input');

    await nameInput.fill("   "); // whitespace only
    await nameInput.press("Enter");

    // Name unchanged
    await expect(nameSpan).toHaveText("Original");
  });
});