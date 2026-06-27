import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("Delete Key Shortcut", () => {
  test("Delete key removes selected entity from hierarchy (screenshot diff)", async ({ page }) => {
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
          scene_id: "delete-test",
          name: "Delete Test",
          entities: [],
        })
      )
    );

    // Create an entity
    await page.evaluate(() =>
      (window as any).dispatch_command(
        JSON.stringify({
          command: {
            type: "CreateEntity",
            id: "del-e1",
            name: "DeleteMe",
            components: [],
          },
          metadata: { authorship: "test", timestamp: 0 },
        })
      )
    );

    await expect(page.locator('[data-testid="hierarchy-entity-del-e1"]')).toBeVisible({ timeout: 10_000 });

    // Select the entity
    await page.locator('[data-testid="hierarchy-entity-del-e1"]').click();
    await page.waitForTimeout(300);

    const hierarchyPanel = page.locator('[data-testid="hierarchy-panel"]');
    const beforeScreenshot = await hierarchyPanel.screenshot();

    // Press Delete
    await page.keyboard.press("Delete");
    await page.waitForTimeout(500);

    await expect(page.locator('[data-testid="hierarchy-entity-del-e1"]')).not.toBeVisible();

    const afterScreenshot = await hierarchyPanel.screenshot();

    // Screenshots must differ (non-zero pixel diff)
    expect(beforeScreenshot).not.toEqual(afterScreenshot);
  });

  test("Delete key does nothing when no entity selected", async ({ page }) => {
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
          scene_id: "noop-delete-test",
          name: "Noop Delete Test",
          entities: [],
        })
      )
    );

    // No entity selected — press Delete
    await page.keyboard.press("Delete");
    await page.waitForTimeout(300);

    // Scene should still be empty
    const snapshot = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot?.();
      return snap ? JSON.parse(snap) : null;
    });
    expect(snapshot?.entities?.length ?? 0).toBe(0);
  });

  test("Delete key does not fire when typing in input", async ({ page }) => {
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
          scene_id: "input-delete-guard-test",
          name: "Input Guard Test",
          entities: [],
        })
      )
    );

    // Create and select an entity
    await page.evaluate(() =>
      (window as any).dispatch_command(
        JSON.stringify({
          command: {
            type: "CreateEntity",
            id: "input-del-e1",
            name: "GuardMe",
            components: [],
          },
          metadata: { authorship: "test", timestamp: 0 },
        })
      )
    );

    await expect(page.locator('[data-testid="hierarchy-entity-input-del-e1"]')).toBeVisible({ timeout: 10_000 });
    await page.locator('[data-testid="hierarchy-entity-input-del-e1"]').click();
    await page.waitForTimeout(200);

    // Focus an input in the inspector
    const nameInput = page.locator("input.entity-name");
    await nameInput.focus();
    await page.waitForTimeout(200);

    // Press Delete while typing — should NOT delete
    await page.keyboard.press("Delete");
    await page.waitForTimeout(500);

    // Entity should still be visible
    await expect(page.locator('[data-testid="hierarchy-entity-input-del-e1"]')).toBeVisible();
  });
});
