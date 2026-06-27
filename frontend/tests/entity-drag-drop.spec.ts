import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("Entity Drag-and-Drop Reparenting", () => {
  test("drag entity onto another entity changes its parent", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load scene: e1 is parent of e2
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "dnd-test",
          name: "DnD Test",
          entities: [
            { id: "e1", name: "Parent", parent: null, components: [] },
            { id: "e2", name: "Child", parent: "e1", components: [] },
          ],
        })
      )
    );

    // Verify e2 has parent e1
    const snapBefore = await page.evaluate(() => {
      const s = (window as any).get_scene_snapshot?.();
      return s ? JSON.parse(s) : null;
    });
    const e2Before = snapBefore.entities.find((e: any) => e.id === "e2");
    expect(e2Before.parent).toBe("e1");

    // Select e2
    await page.locator('[data-testid="hierarchy-entity-e2"]').click();
    await page.waitForTimeout(200);

    // Drag e2 and drop it onto the panel background (to reparent to root)
    const e2 = page.locator('[data-testid="hierarchy-entity-e2"]');
    const panel = page.locator('[data-testid="hierarchy-panel"]');

    await e2.dragTo(panel);

    await page.waitForTimeout(500);

    // Verify e2 is now at root (parent = null)
    const snapAfter = await page.evaluate(() => {
      const s = (window as any).get_scene_snapshot?.();
      return s ? JSON.parse(s) : null;
    });
    const e2After = snapAfter.entities.find((e: any) => e.id === "e2");
    expect(e2After.parent).toBeNull();
  });

  test("dropping entity onto itself is a no-op", async ({ page }) => {
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
          scene_id: "noop-dnd-test",
          name: "Noop DnD Test",
          entities: [
            { id: "self-e1", name: "Self", parent: null, components: [] },
          ],
        })
      )
    );

    const e1 = page.locator('[data-testid="hierarchy-entity-self-e1"]');
    await e1.dragTo(e1);
    await page.waitForTimeout(300);

    const snap = await page.evaluate(() => {
      const s = (window as any).get_scene_snapshot?.();
      return s ? JSON.parse(s) : null;
    });
    const e1After = snap.entities.find((e: any) => e.id === "self-e1");
    expect(e1After.parent).toBeNull();
  });
});
