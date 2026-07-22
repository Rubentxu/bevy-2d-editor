import { test, expect, Page } from "@playwright/test";

/**
 * Phase 1 — UX Overhaul: onboarding tests
 *
 * Validates the quick wins that make the editor no longer look broken to a
 * first-time visitor:
 *  - "+ Add Entity" button is visible in the Hierarchy panel
 *  - Clicking the button creates a visible entity (and bumps the snapshot)
 *  - Pressing `N` (no input focused) creates another entity
 *
 * No OPFS dependency — uses `load_scene_json` then in-memory `dispatch_command`
 * to seed initial empty scene state, mirroring `capabilities-smoke.spec.ts`.
 */

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () =>
      typeof (window as any).load_scene_json === "function" &&
      typeof (window as any).dispatch_command === "function" &&
      typeof (window as any).get_scene_snapshot === "function",
    undefined,
    { timeout: 30_000 }
  );
}

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

async function getEntityCount(page: Page): Promise<number> {
  const snap = await page.evaluate(() => (window as any).get_scene_snapshot());
  const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
  return doc?.entities?.length ?? 0;
}

test.describe("UX Onboarding — Add Entity button + N shortcut", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await waitForEngine(page);
    await seedEmptyScene(page, "ux-onboarding", "UX Onboarding");
  });

  test("'Add Entity' button is visible in the Hierarchy panel header", async ({ page }) => {
    const addBtn = page.locator('[data-testid="add-entity-btn"]');
    await expect(addBtn).toBeVisible();
    await expect(addBtn).toContainText(/Add Entity/);
    await expect(addBtn).toHaveAttribute("title", /Create new entity \(N\)/);
  });

  test("Clicking '+ Add Entity' creates a visible entity in the hierarchy", async ({ page }) => {
    // Seed scene has 0 entities
    expect(await getEntityCount(page)).toBe(0);

    // Click the add entity button in the hierarchy header
    await page.locator('[data-testid="add-entity-btn"]').click();

    // Wait for the entity count to bump to 1
    await page.waitForFunction(
      () => {
        const snap = (window as any).get_scene_snapshot?.();
        const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
        return doc?.entities?.length === 1;
      },
      undefined,
      { timeout: 5_000 }
    );

    // The new entity is named "Entity 1" (first of the "Entity N" series)
    const snap = await page.evaluate(() => (window as any).get_scene_snapshot());
    const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
    expect(doc.entities).toHaveLength(1);
    expect(doc.entities[0].name).toBe("Entity 1");
    expect(doc.entities[0].id).toBeTruthy();
  });

  test("Pressing N (no input focused) creates another entity", async ({ page }) => {
    // Create the first one via the button so we have something to compare against
    await page.locator('[data-testid="add-entity-btn"]').click();
    await page.waitForFunction(
      () => {
        const snap = (window as any).get_scene_snapshot?.();
        const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
        return doc?.entities?.length === 1;
      },
      undefined,
      { timeout: 5_000 }
    );

    // Press N — make sure no input is focused first
    await page.evaluate(() => {
      const active = document.activeElement as HTMLElement | null;
      if (active && typeof active.blur === "function") active.blur();
    });

    const before = await getEntityCount(page);
    expect(before).toBe(1);

    await page.keyboard.press("n");

    // Wait for entity count to bump to 2
    await page.waitForFunction(
      () => {
        const snap = (window as any).get_scene_snapshot?.();
        const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
        return doc?.entities?.length === 2;
      },
      undefined,
      { timeout: 5_000 }
    );

    const after = await getEntityCount(page);
    expect(after).toBe(2);

    // The second entity should be named "Entity 2" (collision-safe suffix)
    const snap = await page.evaluate(() => (window as any).get_scene_snapshot());
    const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
    const names = doc.entities.map((e: { name: string }) => e.name).sort();
    expect(names).toEqual(["Entity 1", "Entity 2"]);
  });
});
