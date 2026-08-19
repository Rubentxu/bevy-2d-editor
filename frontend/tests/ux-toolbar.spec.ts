import { expect, Page, test } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";



test.describe("UX Toolbar — Phase 2", { tag: ["@full"] }, () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);
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
    // Phase D — FPS is now a segment inside the 7-segment status bar.
    await expect(page.locator('[data-testid="status-segment-fps"]')).toContainText("FPS");
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
