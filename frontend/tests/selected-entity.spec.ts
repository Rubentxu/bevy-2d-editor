import { test, expect, Page } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";

/**
 * Selected-entity wiring tests (T1.2 correction).
 *
 * PR1 added selected_entity to the AI request body. These tests verify:
 * - When an entity is selected, selected_entity is populated in the AI request
 * - When no entity is selected, selected_entity is null in the AI request
 */



async function seedEmptyScene(page: Page, sceneId: string, sceneName: string): Promise<void> {
  await page.evaluate(
    ({ sceneId, sceneName }) =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "1",
          scene_id: sceneId,
          name: sceneName,
          entities: [],
        })
      ),
    { sceneId, sceneName }
  );
  await page.waitForFunction(
    () => {
      const snap = (window as any).get_scene_snapshot?.();
      const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
      return doc && Array.isArray(doc.entities) && doc.entities.length === 0;
    },
    undefined,
    { timeout: 5_000 }
  );
}

test.describe("S1 selected_entity wiring (T1.2 correction)", { tag: ["@domain"] }, () => {
  test.beforeEach(async ({ page }) => {
    // Use skip-welcome to bypass the welcome overlay
    await page.goto("/?skip-welcome=1", { waitUntil: "domcontentloaded" });
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);
    // The Welcome overlay may still render briefly — click Skip if visible
    const skipBtn = page.locator('[data-testid="welcome-skip-btn"]');
    try {
      await skipBtn.waitFor({ state: "visible", timeout: 2000 });
      await skipBtn.click();
    } catch {
      // Not visible — skip
    }
    // Seed an empty scene so the Add Entity button is available
    await seedEmptyScene(page, "selected-entity-test", "Selected Entity Test");
  });

  test("AI request carries populated selected_entity when entity is selected", async ({
    page,
  }) => {
    // ── 1. Create an entity via the Add Entity button ─────────────────────────
    const addBtn = page.locator('[data-testid="add-entity-btn"]');
    await addBtn.waitFor({ state: "visible", timeout: 5000 });
    await addBtn.click();
    await page.waitForFunction(
      () => {
        const snap = (window as any).get_scene_snapshot?.();
        const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
        return doc?.entities?.length === 1;
      },
      undefined,
      { timeout: 5000 }
    );

    // Get the entity info
    const entityInfo = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot();
      const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
      const entity = doc?.entities?.[0];
      return { id: entity?.id, name: entity?.name };
    });
    expect(entityInfo.id).toBeTruthy();

    // ── 2. Select the entity in the hierarchy ────────────────────────────────
    await page.locator(`[data-testid="hierarchy-entity-${entityInfo.id}"]`).click();
    await page.waitForTimeout(500);

    // ── 3. Open AI panel ─────────────────────────────────────────────────────
    const aiButton = page.locator('[data-testid="toolbar-group-tools"] button:last-child');
    await aiButton.waitFor({ state: "attached", timeout: 10000 });
    await page.evaluate(() => {
      const container = document.querySelector('[data-testid="toolbar-group-tools"]');
      const btn = container?.querySelector("button:last-child") as HTMLButtonElement;
      btn?.click();
    });
    await expect(page.locator(".ai-assistant-panel")).toBeVisible({ timeout: 10000 });

    // ── 4. Submit an AI prompt and intercept the request ───────────────────
    const requestPromise = page.waitForRequest(
      (req) => req.url().includes("/v1/propose") && req.method() === "POST"
    );
    await page.locator(".ai-prompt-input").fill("create a sprite for this entity");
    await page.locator(".ai-submit-btn").click();

    const req = await requestPromise;
    const body = JSON.parse(req.postData() ?? "{}");

    // ── 5. Assert selected_entity is populated ───────────────────────────────
    expect(body.selected_entity).not.toBeNull();
    expect(body.selected_entity?.stable_id).toBe(entityInfo.id);
    expect(Array.isArray(body.selected_entity?.components)).toBe(true);
  });

  test("AI request carries null selected_entity when no entity is selected", async ({
    page,
  }) => {
    // ── 1. Open AI panel ─────────────────────────────────────────────────────
    const aiButton = page.locator('[data-testid="toolbar-group-tools"] button:last-child');
    await aiButton.waitFor({ state: "attached", timeout: 10000 });
    await page.evaluate(() => {
      const container = document.querySelector('[data-testid="toolbar-group-tools"]');
      const btn = container?.querySelector("button:last-child") as HTMLButtonElement;
      btn?.click();
    });
    await expect(page.locator(".ai-assistant-panel")).toBeVisible({ timeout: 10000 });

    // ── 2. Submit an AI prompt with NO entity selected ─────────────────────
    const requestPromise = page.waitForRequest(
      (req) => req.url().includes("/v1/propose") && req.method() === "POST"
    );
    await page.locator(".ai-prompt-input").fill("create a background");
    await page.locator(".ai-submit-btn").click();

    const req = await requestPromise;
    const body = JSON.parse(req.postData() ?? "{}");

    // ── 3. Assert selected_entity is null ───────────────────────────────────
    expect(body.selected_entity).toBeNull();
  });
});
