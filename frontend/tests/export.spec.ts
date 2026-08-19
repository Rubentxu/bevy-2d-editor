import { test, expect } from "@playwright/test";

/**
 * Tests for the DynamicScene Export WASM binding (Hito 0 §9.5).
 * Exercises the editor → Bevy runtime mapping through the React frontend.
 */

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("DynamicScene Export — WASM binding", { tag: ["@domain"] }, () => {
  test("export_dynamic_scene_wasm on empty document", async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on("console", (msg) => consoleLogs.push(msg.text()));

    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    // Wait for the wasm function to be available.
    await page.waitForFunction(
      () => typeof (window as any).export_dynamic_scene_wasm === "function",
      { timeout: 30_000 }
    );

    const emptyDoc = JSON.stringify({
      version: "0.1",
      scene_id: "scene_empty",
      name: "Empty",
      entities: [],
    });

    const result = await page.evaluate(async (docJson) => {
      const raw = await (window as any).export_dynamic_scene_wasm(docJson);
      const response = JSON.parse(raw);
      const inner = JSON.parse(response.json);
      return { inner, warnings: response.warnings };
    }, emptyDoc);

    expect(result.inner.version).toBe("0.1.0");
    expect(result.inner.source_scene_id).toBe("scene_empty");
    expect(result.inner.entities).toEqual([]);
    expect(result.warnings).toEqual([]);

    // Ensure no errors were logged.
    const errors = consoleLogs.filter((l) => l.toLowerCase().includes("error"));
    expect(errors).toEqual([]);
  });

  test("export_dynamic_scene_wasm with all 3 components", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    await page.waitForFunction(
      () => typeof (window as any).export_dynamic_scene_wasm === "function",
      { timeout: 30_000 }
    );

    const doc = JSON.stringify({
      version: "0.1",
      scene_id: "scene_full",
      name: "Full",
      entities: [
        {
          id: "ent_01",
          name: "Player",
          components: [
            { type_id: "editor.Name", values: { name: "Player" } },
            {
              type_id: "editor.Transform2D",
              values: {
                translation: { x: 100, y: 200 },
                rotation: 0,
                scale: { x: 1, y: 1 },
              },
            },
            {
              type_id: "editor.Sprite2D",
              values: {
                asset: "assets/player.png",
                color: { r: 1, g: 0, b: 0, a: 1 },
                anchor: "Center",
              },
            },
          ],
        },
      ],
    });

    const result = await page.evaluate(async (docJson) => {
      const raw = await (window as any).export_dynamic_scene_wasm(docJson);
      const response = JSON.parse(raw);
      const inner = JSON.parse(response.json);
      return { inner, warnings: response.warnings };
    }, doc);

    expect(result.warnings).toEqual([]);
    expect(result.inner.entities.length).toBe(1);
    const e = result.inner.entities[0];
    expect(e.stable_id).toBe("ent_01");
    expect(e.name).toBe("Player");
    expect(e.parent_stable_id).toBeNull();
    expect(e.components["bevy.Name"]).toEqual({ name: "Player" });
    expect(e.components["bevy.Transform"]).toEqual({
      translation: [100, 200, 0],
      rotation: [0, 0, 0, 1],
      scale: [1, 1, 1],
    });
    expect(e.components["bevy.Sprite"]).toEqual({
      asset: "assets/player.png",
      color: [1, 0, 0, 1],
      anchor: "Center",
    });
  });

  test("export_dynamic_scene_wasm with empty asset emits warning", async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on("console", (msg) => consoleLogs.push(msg.text()));

    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    await page.waitForFunction(
      () => typeof (window as any).export_dynamic_scene_wasm === "function",
      { timeout: 30_000 }
    );

    const doc = JSON.stringify({
      version: "0.1",
      scene_id: "scene_warn",
      name: "Warning",
      entities: [
        {
          id: "ent_99",
          name: "Ghost",
          components: [
            { type_id: "editor.Name", values: { name: "Ghost" } },
            {
              type_id: "editor.Sprite2D",
              values: {
                asset: "",
                color: { r: 1, g: 1, b: 1, a: 1 },
                anchor: "Center",
              },
            },
          ],
        },
      ],
    });

    const result = await page.evaluate(async (docJson) => {
      const raw = await (window as any).export_dynamic_scene_wasm(docJson);
      const response = JSON.parse(raw);
      const inner = JSON.parse(response.json);
      return { inner, warnings: response.warnings };
    }, doc);

    // Sprite must be omitted.
    expect(result.inner.entities[0].components["bevy.Sprite"]).toBeUndefined();
    // Warning recorded.
    expect(result.warnings.length).toBe(1);
    expect(result.warnings[0].entity_stable_id).toBe("ent_99");
    expect(result.warnings[0].component_type_id).toBe("editor.Sprite2D");
    expect(result.warnings[0].message.toLowerCase()).toContain("empty asset");
  });
});
