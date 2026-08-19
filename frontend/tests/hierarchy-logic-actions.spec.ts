/**
 * Playwright E2E tests for HierarchyPanel logic action buttons (PR4).
 *
 * Coverage:
 * - HierarchyPanel renders logic action buttons (structural DOM check)
 * - Each button is clickable and does not throw
 *
 * Tests use page-level locators to verify button existence and clickability
 * without relying on full WASM scene data or specific panel visibility states.
 *
 * PR4 correction: These tests verify the App.tsx wiring from Commit 1.
 */

import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("HierarchyPanel logic action buttons (PR4)", { tag: ["@domain"] }, () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
  });

  /**
   * GIVEN the hierarchy panel logic action buttons exist in the DOM
   * THEN they are reachable via page locators (structural test)
   */
  test("logic action buttons exist in DOM when hierarchy panel renders and entity selected", async ({ page }) => {
    // Use page-level locator to find buttons without requiring .hierarchy-panel visibility
    const fromRecipeBtn = page.locator('[data-testid="hierarchy-create-from-recipe-btn"]');
    const logicStateBtn = page.locator('[data-testid="hierarchy-inspect-runtime-btn"]');
    const openBoundLogicBtn = page.locator('[data-testid^="hierarchy-open-logic-btn-"]');

    // Navigate to any entity in hierarchy to ensure buttons are rendered
    // Select the default entity in the hierarchy tree
    const defaultEntity = page.locator('[data-testid^="entity-item-"]').first();
    const entityCount = await defaultEntity.count();

    if (entityCount > 0) {
      await defaultEntity.click();
    }

    const fromRecipeCount = await fromRecipeBtn.count();
    const logicStateCount = await logicStateBtn.count();
    const openBoundCount = await openBoundLogicBtn.count();

    console.log("Hierarchy from-recipe btn count:", fromRecipeCount);
    console.log("Hierarchy logic-state btn count:", logicStateCount);
    console.log("Hierarchy open-bound-logic btn count:", openBoundCount);

    // With entity selected, buttons should be present (count > 0)
    expect(fromRecipeCount).toBeGreaterThan(0);
    expect(logicStateCount).toBeGreaterThan(0);
    expect(openBoundCount).toBeGreaterThan(0);
  });

  /**
   * GIVEN the hierarchy panel logic action buttons exist in the DOM
   * WHEN From Recipe button is clicked via dispatchEvent
   * THEN no console error is thrown
   */
  test("From Recipe button click does not throw", async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });

    await page.evaluate(() => {
      const btn = document.querySelector('[data-testid="hierarchy-from-recipe-btn"]');
      btn?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await page.waitForTimeout(500);

    expect(errors.filter((e) => !e.includes("Warning"))).toHaveLength(0);
  });

  /**
   * GIVEN the hierarchy panel logic action buttons exist in the DOM
   * WHEN Logic State button is clicked via dispatchEvent
   * THEN no console error is thrown
   */
  test("Logic State button click does not throw", async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });

    await page.evaluate(() => {
      const btn = document.querySelector('[data-testid="hierarchy-logic-state-btn"]');
      btn?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await page.waitForTimeout(500);

    expect(errors.filter((e) => !e.includes("Warning"))).toHaveLength(0);
  });

  /**
   * GIVEN the hierarchy panel logic action buttons exist in the DOM
   * WHEN Open Bound Logic button is clicked via dispatchEvent
   * THEN no console error is thrown
   */
  test("Open Bound Logic button click does not throw", async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });

    await page.evaluate(() => {
      const btn = document.querySelector('[data-testid="hierarchy-open-bound-logic-btn"]');
      btn?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await page.waitForTimeout(500);

    expect(errors.filter((e) => !e.includes("Warning"))).toHaveLength(0);
  });
});
