import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("Multi-Scene", { tag: ["@persistence"] }, () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    await page.waitForFunction(
      () =>
        typeof (window as any).scene_create === "function" &&
        typeof (window as any).scene_switch === "function" &&
        typeof (window as any).scene_delete === "function" &&
        typeof (window as any).list_scenes_extended === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Ensure clean state: delete all scenes except the default, then recreate
    const scenes = await page.evaluate(() => (window as any).list_scenes_extended());
    for (const s of scenes) {
      if (scenes.length === 1) break;
      await page.evaluate((id: string) => (window as any).scene_delete(id), s.id);
    }
  });

  test("create new scene via + button, verify appears in tabs", async ({ page }) => {
    // Load a fresh project first
    await page.evaluate(() => (window as any).load_project());

    const initialScenes = await page.evaluate(() => (window as any).list_scenes_extended());
    const initialCount = initialScenes.length;

    // Click the + button (intercept window.prompt)
    await page.route("**/*", (route) => route.fulfill({ body: "" }));

    page.on("dialog", async (dialog) => {
      expect(dialog.type()).toBe("prompt");
      await dialog.accept("Test Scene");
    });

    await page.locator('[data-testid="scene-tab-new-btn"]').click();

    // Wait for the new scene to appear
    await page.waitForFunction(
      (prev) => (window as any).list_scenes_extended().length === prev + 1,
      initialCount,
      { timeout: 5000 }
    );

    const scenes = await page.evaluate(() => (window as any).list_scenes_extended());
    expect(scenes).toHaveLength(initialCount + 1);
    const newScene = scenes.find((s: any) => s.name === "Test Scene");
    expect(newScene).toBeDefined();
  });

  test("switch scene, verify current scene changes", async ({ page }) => {
    await page.evaluate(() => (window as any).load_project());

    // Create a second scene
    const newId = await page.evaluate(() => (window as any).scene_create("Scene B"));
    const scenesBefore = await page.evaluate(() => (window as any).list_scenes_extended());
    const initialId = scenesBefore.find((s: any) => s.is_active).id;

    // Click the new scene tab
    await page.locator(`[data-testid="scene-tab-${newId}"]`).click();

    // Wait for switch to complete
    await page.waitForFunction(
      (expected: string) => (window as any).get_current_scene_id() === expected,
      newId,
      { timeout: 5000 }
    );

    const currentId = await page.evaluate(() => (window as any).get_current_scene_id());
    expect(currentId).toBe(newId);
    expect(currentId).not.toBe(initialId);
  });

  test("create entity in scene A, switch to B, switch back to A → entity still there", async ({ page }) => {
    await page.evaluate(() => (window as any).load_project());

    // Get scene A
    const sceneA = await page.evaluate(() => {
      const scenes = (window as any).list_scenes_extended();
      return scenes.find((s: any) => s.is_active);
    });

    // Create a second scene
    const sceneBId = await page.evaluate(() => (window as any).scene_create("Scene B"));

    // Go back to scene A
    await page.evaluate((id: string) => (window as any).scene_switch(id), sceneA.id);

    // Add an entity to scene A
    await page.evaluate(() => {
      (window as any).dispatch_command(JSON.stringify({
        command: {
          type: "AddEntity",
          parent: null,
          name: "Player",
        },
        metadata: { authorship: "user", timestamp: Date.now() },
      }));
    });

    // Verify entity exists in scene A
    let snapA = await page.evaluate(() => (window as any).get_scene_snapshot());
    const entityInA = snapA.entities.find((e: any) => e.name === "Player");
    expect(entityInA).toBeDefined();

    // Switch to scene B
    await page.evaluate((id: string) => (window as any).scene_switch(id), sceneBId);
    snapA = await page.evaluate(() => (window as any).get_scene_snapshot());
    // Scene B should be empty
    expect(snapA.entities).toHaveLength(0);

    // Switch back to scene A
    await page.evaluate((id: string) => (window as any).scene_switch(id), sceneA.id);

    // Verify entity is still there
    const snapBack = await page.evaluate(() => (window as any).get_scene_snapshot());
    const entityStillThere = snapBack.entities.find((e: any) => e.name === "Player");
    expect(entityStillThere).toBeDefined();
  });

  test("cannot delete last scene — delete button absent or no-op", async ({ page }) => {
    await page.evaluate(() => (window as any).load_project());

    // Get the single scene id
    const scenes = await page.evaluate(() => (window as any).list_scenes_extended());
    const lastId = scenes[0].id;

    // Attempt to delete — should either throw or leave scene intact
    await page.evaluate((id: string) => (window as any).scene_delete(id), lastId);

    const scenesAfter = await page.evaluate(() => (window as any).list_scenes_extended());
    expect(scenesAfter).toHaveLength(1);
    expect(scenesAfter[0].id).toBe(lastId);
  });
});
