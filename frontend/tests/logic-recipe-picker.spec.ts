/**
 * Playwright E2E tests for Logic Workflow v2 — RecipePicker (PR4).
 *
 * Coverage:
 * - RecipePicker renders first when entering logic mode
 * - "Start from blank graph" is opt-in only (not auto-created)
 * - Recipe list shows built-in recipes
 * - Clicking a recipe opens it
 * - "Start from blank graph" creates a blank graph
 */

import { test, expect, Page } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("Logic RecipePicker (PR4 — Logic Workflow v2)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    // Wait for logic graph WASM functions to be available
    await page.waitForFunction(
      () =>
        typeof (window as any).create_logic_graph_asset === "function" &&
        typeof (window as any).list_logic_graph_assets === "function" &&
        typeof (window as any).open_logic_graph_asset === "function",
      { timeout: WASM_LOAD_TIMEOUT },
    );
  });

  /**
   * GIVEN the editor is loaded
   * WHEN the user switches to logic mode
   * THEN the RecipePicker is shown as the first surface (not the graph editor)
   */
  test("RecipePicker renders first in logic mode", async ({ page }) => {
    // Trigger logic mode via WASM — switch editor mode
    await page.evaluate(() => {
      (window as any).__setEditorMode?.("logic");
    });
    await page.waitForTimeout(500);

    // Find the recipe picker
    const picker = page.locator('[data-testid="recipe-picker"]');
    await expect(picker).toBeVisible({ timeout: 10000 });
  });

  /**
   * GIVEN the editor is in logic mode with RecipePicker visible
   * WHEN the user clicks "Start from blank graph"
   * THEN the blank graph editor is shown (not auto-created on mount)
   */
  test("blank graph is opt-in only via Start from blank graph button", async ({ page }) => {
    // Trigger logic mode
    await page.evaluate(() => {
      (window as any).__setEditorMode?.("logic");
    });
    await page.waitForTimeout(500);

    // Verify RecipePicker is shown first
    const picker = page.locator('[data-testid="recipe-picker"]');
    await expect(picker).toBeVisible({ timeout: 10000 });

    // Verify the blank graph button exists
    const blankBtn = page.locator('[data-testid="recipe-blank-btn"]');
    await expect(blankBtn).toBeVisible();

    // Click "Start from blank graph"
    await blankBtn.click();
    await page.waitForTimeout(1000);

    // After clicking, the graph editor should be visible
    const editor = page.locator('[data-testid="logic-graph-editor"]');
    await expect(editor).toBeVisible({ timeout: 10000 });
  });

  /**
   * GIVEN the RecipePicker is visible
   * THEN it shows the title "Choose a Recipe"
   * AND the subtitle about starting patterns or scratch
   */
  test("RecipePicker shows correct title and subtitle", async ({ page }) => {
    await page.evaluate(() => {
      (window as any).__setEditorMode?.("logic");
    });
    await page.waitForTimeout(500);

    const title = page.locator('[data-testid="recipe-picker-title"]');
    await expect(title).toBeVisible();
    await expect(title).toHaveText("Choose a Recipe");

    const subtitle = page.locator('[data-testid="recipe-picker-subtitle"]');
    await expect(subtitle).toBeVisible();
  });

  /**
   * GIVEN the RecipePicker is visible
   * WHEN the user clicks a built-in recipe
   * THEN the graph editor is shown with that recipe loaded
   */
  test("clicking a built-in recipe opens the graph editor", async ({ page }) => {
    await page.evaluate(() => {
      (window as any).__setEditorMode?.("logic");
    });
    await page.waitForTimeout(500);

    // Find the recipe-blank-btn to confirm picker is showing
    await expect(page.locator('[data-testid="recipe-blank-btn"]')).toBeVisible();

    // Find the first recipe button (built-in recipe)
    const recipeBtn = page.locator('[data-testid^="recipe-btn-"]').first();
    await expect(recipeBtn).toBeVisible();

    // Get the recipe ID from the testid
    const testId = await recipeBtn.getAttribute("data-testid");
    const recipeId = testId?.replace("recipe-btn-", "");
    expect(recipeId).toBeTruthy();

    await recipeBtn.click();
    await page.waitForTimeout(1000);

    // After selection, the graph editor should be visible
    const editor = page.locator('[data-testid="logic-graph-editor"]');
    await expect(editor).toBeVisible({ timeout: 10000 });
  });
});
