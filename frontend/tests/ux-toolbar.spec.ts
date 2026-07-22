import { expect, Page, test } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () =>
      typeof (window as any).load_scene_json === "function" &&
      typeof (window as any).get_scene_snapshot === "function",
    undefined,
    { timeout: 30_000 }
  );
}

test.describe("UX Toolbar — Phase 2", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await waitForEngine(page);
  });

  test("toolbar groups exist and Run contains only Play", async ({ page }) => {
    for (const group of ["mode", "edit", "tools", "run"]) {
      await expect(page.locator(`[data-testid="toolbar-group-${group}"]`)).toBeVisible();
    }
    const runButtons = page.locator('[data-testid="toolbar-group-run"] button');
    await expect(runButtons).toHaveCount(1);
    await expect(runButtons.first()).toHaveAttribute("data-testid", "play-btn");
    await expect(page.locator('[data-testid="undo-btn"]')).toHaveAttribute("title", "Undo (Ctrl+Z)");
  });

  test("status bar is always visible", async ({ page }) => {
    await expect(page.locator('[data-testid="status-bar"]')).toBeVisible();
    await expect(page.locator('[data-testid="status-fps"]')).toContainText("FPS");
  });

  test("inspector search filters visible components", async ({ page }) => {
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "1",
          scene_id: "ux-toolbar",
          name: "UX Toolbar",
          entities: [
            {
              id: "search-target",
              name: "Search Target",
              components: [
                { type_id: "editor.Transform2D", values: {} },
                { type_id: "editor.Sprite2D", values: {} },
              ],
            },
          ],
        })
      )
    );

    const entity = page.locator('[data-testid="hierarchy-entity-search-target"]');
    await expect(entity).toBeVisible({ timeout: 5_000 });
    await entity.click();
    await expect(page.locator(".component-card")).toHaveCount(2);

    await page.locator('[data-testid="inspector-search"]').fill("sprite");
    await expect(page.locator(".component-card")).toHaveCount(1);
    await expect(page.locator(".component-card .type-id")).toContainText("editor.Sprite2D");
  });
});
