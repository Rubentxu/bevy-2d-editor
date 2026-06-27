import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

function getParent(entityId: string, entities: any[]): string | undefined {
  const entity = entities.find((e) => e.id === entityId);
  return entity?.parent ?? undefined;
}

/**
 * Complete HTML5 DnD sequence using Playwright's locator.dispatchEvent().
 *
 * Key: locator.dispatchEvent() properly coordinates Playwright's internal
 * browser context so subsequent locators see the updated DOM after React re-renders.
 *
 * Sequence:
 * 1. dragstart on source → React sets draggedId
 * 2. waitForFunction polls until .dragging class appears (proves React flushed)
 * 3. dragover on target → React sets dragOverId
 * 4. drop on target → ReparentEntity dispatched
 * 5. dragend on source → cleanup
 */
async function fireDragAndDrop(page: any, sourceSelector: string, targetSelector: string) {
  // 1. Trigger dragstart
  await page.locator(sourceSelector).dispatchEvent("dragstart");

  // 2. Wait for React to flush setDraggedId → .dragging class visible
  await page.waitForFunction(
    (sel: string) => document.querySelector(sel)?.classList.contains("dragging"),
    sourceSelector,
    { timeout: 5000 }
  );

  // 3. dragover on target
  await page.locator(targetSelector).dispatchEvent("dragover");

  // 4. drop on target
  await page.locator(targetSelector).dispatchEvent("drop");

  // 5. dragend on source for cleanup
  await page.locator(sourceSelector).dispatchEvent("dragend");
}

test.describe("Entity Drag-and-Drop Reparenting", () => {
  test("drag entity onto panel background reparents to root", async ({ page }) => {
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
          scene_id: "dnd-test",
          name: "DnD Test",
          entities: [
            { id: "e1", name: "Parent", parent: null, components: [] },
            { id: "e2", name: "Child", parent: "e1", components: [] },
          ],
        })
      )
    );

    const snapBefore = await page.evaluate(() => {
      const s = (window as any).get_scene_snapshot?.();
      return s ? JSON.parse(s) : null;
    });
    expect(getParent("e2", snapBefore.entities)).toBe("e1");

    // Root zone: drop e2 onto the empty root area
    await fireDragAndDrop(
      page,
      '[data-testid="hierarchy-entity-e2"]',
      ".hierarchy-root-zone"
    );
    await page.waitForTimeout(500);

    const snapAfter = await page.evaluate(() => {
      const s = (window as any).get_scene_snapshot?.();
      return s ? JSON.parse(s) : null;
    });
    const e2After = snapAfter.entities.find((e: any) => e.id === "e2");
    expect(e2After.parent).toBeUndefined();
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
          entities: [{ id: "self-e1", name: "Self", parent: null, components: [] }],
        })
      )
    );

    await fireDragAndDrop(
      page,
      '[data-testid="hierarchy-entity-self-e1"]',
      '[data-testid="hierarchy-entity-self-e1"]'
    );
    await page.waitForTimeout(300);

    const snap = await page.evaluate(() => {
      const s = (window as any).get_scene_snapshot?.();
      return s ? JSON.parse(s) : null;
    });
    const e1After = snap.entities.find((e: any) => e.id === "self-e1");
    expect(e1After.parent).toBeUndefined();
  });

  test("drag sibling onto sibling reparents correctly", async ({ page }) => {
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
          scene_id: "sibling-dnd-test",
          name: "Sibling DnD Test",
          entities: [
            { id: "e1", name: "One", parent: null, components: [] },
            { id: "e2", name: "Two", parent: null, components: [] },
            { id: "e3", name: "Three", parent: "e1", components: [] },
          ],
        })
      )
    );

    const snapBefore = await page.evaluate(() => {
      const s = (window as any).get_scene_snapshot?.();
      return s ? JSON.parse(s) : null;
    });
    expect(getParent("e3", snapBefore.entities)).toBe("e1");

    await fireDragAndDrop(
      page,
      '[data-testid="hierarchy-entity-e3"]',
      '[data-testid="hierarchy-entity-e2"]'
    );
    await page.waitForTimeout(500);

    const snapAfter = await page.evaluate(() => {
      const s = (window as any).get_scene_snapshot?.();
      return s ? JSON.parse(s) : null;
    });
    expect(getParent("e3", snapAfter.entities)).toBe("e2");
  });
});
