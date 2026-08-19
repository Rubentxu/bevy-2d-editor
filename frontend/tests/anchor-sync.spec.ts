import { test, expect } from "@playwright/test";

/**
 * Tests for the preview world's Anchor Component insertion (preview-anchor-sync cycle).
 * The preview world is rebuilt on scene change via spawn_entity, which now reads
 * `editor.Sprite2D.values.anchor` and inserts a Bevy `Anchor` Component after the
 * Sprite Component (overriding the `#[require(Anchor)]` default).
 *
 * We verify the round-trip: load a scene with various anchors, then read it back
 * via get_scene_snapshot to confirm the anchor string was preserved through
 * the rebuild.
 */

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("Preview Anchor Sync — Anchor Component insertion", { tag: ["@domain"] }, () => {
  test("TopLeft anchor sprite round-trips through scene rebuild", async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on("console", (msg) => consoleLogs.push(msg.text()));

    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    // Wait for the load_scene_json and get_scene_snapshot bridges.
    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).get_scene_snapshot === "function",
      { timeout: 30_000 }
    );

    const sceneWithTopLeft = JSON.stringify({
      version: "0.1",
      scene_id: "scene_anchor_topleft",
      name: "TopLeft Anchor Test",
      entities: [
        {
          id: "ent_topleft",
          name: "TL",
          components: [
            { type_id: "editor.Name", values: { name: "TL" } },
            {
              type_id: "editor.Transform2D",
              values: {
                translation: { x: 0, y: 0 },
                rotation: 0,
                scale: { x: 1, y: 1 },
              },
            },
            {
              type_id: "editor.Sprite2D",
              values: {
                asset: "assets/test.png",
                color: { r: 1, g: 0, b: 0, a: 1 },
                anchor: "TopLeft",
              },
            },
          ],
        },
      ],
    });

    const result = await page.evaluate(async (sceneJson) => {
      await (window as any).load_scene_json(sceneJson);
      // Allow rebuild to flush.
      await new Promise((r) => setTimeout(r, 500));
      const snap = await (window as any).get_scene_snapshot();
      const parsed = typeof snap === "string" ? JSON.parse(snap) : snap;
      const entity = parsed.entities[0];
      const sprite = entity.components.find(
        (c: any) => c.type_id === "editor.Sprite2D"
      );
      return { anchor: sprite?.values?.anchor ?? null };
    }, sceneWithTopLeft);

    expect(result.anchor).toBe("TopLeft");

    // Should not produce warnings about TopLeft being invalid.
    const anchorWarnings = consoleLogs.filter(
      (l) =>
        l.toLowerCase().includes("anchor") &&
        l.toLowerCase().includes("not recognized")
    );
    expect(anchorWarnings).toEqual([]);
  });

  test("Invalid anchor string still loads scene (with warning)", async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on("console", (msg) => consoleLogs.push(`[${msg.type()}] ${msg.text()}`));

    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    await page.waitForFunction(
      () => typeof (window as any).load_scene_json === "function",
      { timeout: 30_000 }
    );

    const sceneWithInvalidAnchor = JSON.stringify({
      version: "0.1",
      scene_id: "scene_invalid_anchor",
      name: "Invalid Anchor Test",
      entities: [
        {
          id: "ent_invalid",
          name: "Bad",
          components: [
            { type_id: "editor.Name", values: { name: "Bad" } },
            {
              type_id: "editor.Sprite2D",
              values: {
                asset: "assets/x.png",
                color: { r: 1, g: 1, b: 1, a: 1 },
                anchor: "MiddleSomewhere",
              },
            },
          ],
        },
      ],
    });

    // Scene should still load successfully — invalid anchor falls back to Center + warn.
    const ok = await page.evaluate(async (sceneJson) => {
      try {
        await (window as any).load_scene_json(sceneJson);
        return true;
      } catch (e) {
        return false;
      }
    }, sceneWithInvalidAnchor);

    expect(ok).toBe(true);

    // Give Bevy systems time to rebuild (3s should be plenty for many frames).
    await page.waitForTimeout(3000);

    // Dump all console messages for debugging if test fails again.
    const anchorWarnings = consoleLogs.filter(
      (l) =>
        l.toLowerCase().includes("anchor") &&
        l.toLowerCase().includes("not recognized")
    );

    if (anchorWarnings.length === 0) {
      console.log("DEBUG: All console messages during test:");
      consoleLogs.forEach((l) => console.log(`  ${l}`));
    }

    expect(anchorWarnings.length).toBeGreaterThan(0);
  });

  test("All 9 anchors round-trip correctly", async ({ page }) => {
    const anchors = [
      "Center",
      "TopLeft",
      "TopCenter",
      "TopRight",
      "CenterLeft",
      "CenterRight",
      "BottomLeft",
      "BottomCenter",
      "BottomRight",
    ];

    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).get_scene_snapshot === "function",
      { timeout: 30_000 }
    );

    for (const anchor of anchors) {
      const scene = JSON.stringify({
        version: "0.1",
        scene_id: `scene_${anchor}`,
        name: anchor,
        entities: [
          {
            id: `ent_${anchor}`,
            name: anchor,
            components: [
              {
                type_id: "editor.Sprite2D",
                values: {
                  asset: "assets/x.png",
                  color: { r: 1, g: 1, b: 1, a: 1 },
                  anchor,
                },
              },
            ],
          },
        ],
      });

      const result = await page.evaluate(async (sceneJson) => {
        await (window as any).load_scene_json(sceneJson);
        await new Promise((r) => setTimeout(r, 200));
        const snap = await (window as any).get_scene_snapshot();
        const parsed = typeof snap === "string" ? JSON.parse(snap) : snap;
        const sprite = parsed.entities[0].components.find(
          (c: any) => c.type_id === "editor.Sprite2D"
        );
        return sprite?.values?.anchor ?? null;
      }, scene);

      expect(result, `anchor ${anchor} should round-trip`).toBe(anchor);
    }
  });
});
