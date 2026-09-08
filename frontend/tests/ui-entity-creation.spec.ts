/**
 * ui-entity-creation.spec.ts — BJ-3 closure test.
 *
 * Proves that a user can create entities in the editor via the UI
 * surface (the `+ Add Entity` button) without any backend hand-edit
 * of editor data. This is the load-bearing evidence for the G1
 * sub-deliverable "create a small complete 2D game without
 * hand-editing editor data".
 *
 * The test loads a clean editor (no OPFS-mounted sample), clicks
 * `+ Add Entity`, and asserts that the entity count in the scene
 * snapshot increases and that the new entity is rendered in the
 * Hierarchy panel.
 *
 * @full — runs in the full cohort (not smoke; needs WASM engine).
 */

import { test, expect } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";

test.describe("BJ-3 — UI entity creation", { tag: ["@full"] }, () => {
  test("add_entity_button_creates_one_entity", async ({ page }) => {
    await page.goto("/");
    await waitForEditorReady(page);

    // Snapshot must start empty (no entities).
    const initialCount = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot?.();
      const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
      return doc?.entities?.length ?? 0;
    });
    expect(initialCount, "scene should start with 0 entities").toBe(0);

    // Click + Add Entity.
    const addBtn = page.locator('[data-testid="add-entity-btn"]');
    await addBtn.waitFor({ state: "visible", timeout: 5000 });
    await addBtn.click();

    // Wait until the snapshot reflects exactly one entity.
    await page.waitForFunction(
      () => {
        const snap = (window as any).get_scene_snapshot?.();
        const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
        return doc?.entities?.length === 1;
      },
      undefined,
      { timeout: 5000 },
    );

    // Confirm the entity is rendered in the Hierarchy panel.
    const entityId = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot();
      const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
      return doc?.entities?.[0]?.id;
    });
    expect(entityId, "created entity should have an id").toBeTruthy();

    const row = page.locator(`[data-testid="hierarchy-entity-${entityId}"]`);
    await expect(row, "entity should be rendered in Hierarchy panel").toBeVisible();
  });

  test("add_entity_button_can_be_clicked_multiple_times", async ({ page }) => {
    await page.goto("/");
    await waitForEditorReady(page);

    const addBtn = page.locator('[data-testid="add-entity-btn"]');
    await addBtn.waitFor({ state: "visible", timeout: 5000 });

    // Click + Add Entity three times.
    for (let i = 0; i < 3; i++) {
      await addBtn.click();
    }

    // Wait until the snapshot reflects exactly three entities.
    await page.waitForFunction(
      () => {
        const snap = (window as any).get_scene_snapshot?.();
        const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
        return doc?.entities?.length === 3;
      },
      undefined,
      { timeout: 5000 },
    );

    // Confirm three distinct entities are rendered in the Hierarchy
    // panel.
    const entityIds: string[] = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot();
      const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
      return (doc?.entities ?? []).map((e: any) => e.id).filter(Boolean);
    });
    expect(entityIds, "should have 3 distinct entity ids").toHaveLength(3);
    expect(new Set(entityIds).size, "entity ids must be distinct").toBe(3);

    for (const id of entityIds) {
      const row = page.locator(`[data-testid="hierarchy-entity-${id}"]`);
      await expect(row, `entity ${id} should be rendered`).toBeVisible();
    }
  });
});
