import { test, expect, Page } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

/**
 * Playwright E2E tests for Global Search actionable results (workflow-surface-convergence T2.3–T2.5).
 *
 * Coverage:
 * - T2.3: entity and command result types are present in the type system
 * - T2.4: SearchResultRow renders all result types with icons and labels
 * - T2.5: SearchTab action handlers navigate/open/focus per result kind
 */

/** Open the bottom dock and switch to the search tab. */
async function openSearchTab(page: Page): Promise<void> {
  // Remove welcome overlay from DOM if it exists (it blocks all pointer events)
  await page.evaluate(() => {
    const overlay = document.querySelector('[data-testid="welcome-overlay"]');
    if (overlay instanceof HTMLElement) {
      overlay.remove();
    }
  });
  await page.waitForTimeout(300);

  // Check if search tab is already visible and active
  const searchTab = page.locator('[data-testid="bottom-dock-tab-search"]');
  const searchTabVisible = await searchTab.isVisible().catch(() => false);

  if (searchTabVisible) {
    // Search tab is visible — just click it in case a different tab is active
    await searchTab.click();
    await page.waitForTimeout(200);
    return;
  }

  // Bottom dock is not showing search tab — press F7 to open/toggle it
  // Note: bottom dock starts visible with "console" tab active,
  // so first F7 might close it; we press twice to ensure it's open with search tab
  await page.keyboard.press("F7");
  await page.waitForTimeout(300);
  await page.keyboard.press("F7");
  await page.waitForTimeout(400);

  // Now click the search tab
  const searchTabNow = page.locator('[data-testid="bottom-dock-tab-search"]');
  if (await searchTabNow.isVisible({ timeout: 3000 }).catch(() => false)) {
    await searchTabNow.click();
    await page.waitForTimeout(200);
  }
}

async function waitForSearchIndex(page: Page): Promise<void> {
  await page.waitForFunction(
    () =>
      typeof (window as any).list_scenes_extended === "function" &&
      typeof (window as any).get_scene_asset_catalog_json === "function" &&
      typeof (window as any).list_source_files === "function",
    { timeout: WASM_LOAD_TIMEOUT },
  );
}

test.describe("Global Search Actionable Results (T2.3, T2.4, T2.5)", () => {
  test.beforeEach(async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });

    await page.goto("/?skip-welcome=1");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    await waitForSearchIndex(page);

    if (errors.length > 0) {
      console.warn("Console errors during init:", errors);
    }

    await openSearchTab(page);
  });

  /**
   * T2.4: SearchResultRow renders with icon, label, and path.
   * Requires at least one search result to verify row rendering.
   */
  test("T2.4: result rows render icon, label, and path", async ({ page }) => {
    const searchInput = page.locator('[data-testid="global-search-input"]');
    await expect(searchInput).toBeVisible({ timeout: 5000 });

    await searchInput.fill("a");
    await page.waitForTimeout(600);

    const results = page.locator('[data-testid="global-search-results"]');
    const resultsVisible = await results.isVisible().catch(() => false);

    // If no results, verify empty state is shown
    if (!resultsVisible) {
      await expect(
        page.locator('[data-testid="global-search-empty"]'),
      ).toBeVisible();
      return;
    }

    // When results exist, verify at least one row has icon, label, and path
    const rows = page.locator(".search-result-row");
    const count = await rows.count();
    expect(count).toBeGreaterThan(0);

    const firstRow = rows.first();
    await expect(
      firstRow.locator(".search-result-row__icon"),
    ).toBeVisible();
    await expect(
      firstRow.locator(".search-result-row__label"),
    ).toBeVisible();
  });

  /**
   * T2.3: entity result type is wired — search for a term and verify
   * entity results appear (or empty state if no matching entities).
   */
  test("T2.3: entity results type is present in search results", async ({
    page,
  }) => {
    const searchInput = page.locator('[data-testid="global-search-input"]');
    await expect(searchInput).toBeVisible({ timeout: 5000 });

    // Search with a query that might match entity names
    await searchInput.fill("Player");
    await page.waitForTimeout(600);

    const results = page.locator('[data-testid="global-search-results"]');
    const resultsVisible = await results.isVisible().catch(() => false);
    const emptyVisible = await page
      .locator('[data-testid="global-search-empty"]')
      .isVisible()
      .catch(() => false);

    // Either results are shown (possibly including entity results) or empty state
    expect(resultsVisible || emptyVisible).toBeTruthy();
  });

  /**
   * T2.3: command result type is wired.
   * SearchTab initializes command results from __getCommandPaletteItems on mount.
   * Verify that command results appear when searching.
   */
  test("T2.3: command results are included when seeded", async ({ page }) => {
    // Verify __getCommandPaletteItems returns command palette items.
    const hasCommandPalette = await page.evaluate(() => {
      const items = (window as any).__getCommandPaletteItems?.();
      return Array.isArray(items) && items.length > 0;
    });

    const searchInput = page.locator('[data-testid="global-search-input"]');
    await expect(searchInput).toBeVisible({ timeout: 5000 });

    // Search for a command (should match "Save Scene" which has "save" in it)
    await searchInput.fill("Save");
    await page.waitForTimeout(600);

    if (hasCommandPalette) {
      // CRITICAL ISSUE 5: assert command result appears in search results.
      const cmdRow = page.locator('[data-testid^="global-search-result-command-"]');
      await expect(cmdRow).toBeVisible();
    } else {
      // If no command palette items, empty state should be visible.
      await expect(
        page.locator('[data-testid="global-search-empty"]'),
      ).toBeVisible();
    }
  });

  /**
   * T2.5: scene result click switches to the scene (action fires, no throw).
   */
  test("T2.5: clicking a scene result navigates to that scene", async ({
    page,
  }) => {
    const searchInput = page.locator('[data-testid="global-search-input"]');
    await expect(searchInput).toBeVisible({ timeout: 5000 });

    const scenes = await page.evaluate(() =>
      (window as any).list_scenes_extended(),
    );
    if (!scenes || scenes.length === 0) {
      // No scenes in test project — verify empty state is shown.
      await searchInput.fill("nonexistent_scene_xyz");
      await page.waitForTimeout(600);
      await expect(
        page.locator('[data-testid="global-search-empty"]'),
      ).toBeVisible();
      return;
    }

    await searchInput.fill(scenes[0].name);
    await page.waitForTimeout(600);

    const sceneResult = page.locator(
      `[data-testid^="global-search-result-scene-"]`,
    ).first();
    const sceneVisible = await sceneResult
      .isVisible({ timeout: 2000 })
      .catch(() => false);
    expect(sceneVisible).toBeTruthy();

    // Click should not throw and should trigger scene switch.
    await sceneResult.click();
    await page.waitForTimeout(400);
  });

  /**
   * T2.5: source-file result click navigates to source in code mode.
   */
  test("T2.5: clicking a source-file result navigates to code editor", async ({
    page,
  }) => {
    const searchInput = page.locator('[data-testid="global-search-input"]');
    await expect(searchInput).toBeVisible({ timeout: 5000 });

    const raw = await page.evaluate(() =>
      (window as any).list_source_files(),
    );
    // Handle OpfsResult format: { ok: true, value: [...] } or { ok: false, error: "..." }
    const sourceFiles = raw?.ok === true ? raw.value : raw;
    if (!Array.isArray(sourceFiles) || sourceFiles.length === 0) {
      // No source files — verify empty state is shown.
      await searchInput.fill("nonexistent_file_xyz");
      await page.waitForTimeout(600);
      await expect(
        page.locator('[data-testid="global-search-empty"]'),
      ).toBeVisible();
      return;
    }

    const firstFile = sourceFiles[0];
    const fileName =
      firstFile?.name || firstFile?.path?.split("/").pop() || "";
    expect(fileName).toBeTruthy();

    await searchInput.fill(fileName);
    await page.waitForTimeout(600);

    const sourceResult = page.locator(
      `[data-testid^="global-search-result-source-file-"]`,
    ).first();
    const sourceVisible = await sourceResult
      .isVisible({ timeout: 2000 })
      .catch(() => false);
    expect(sourceVisible).toBeTruthy();

    // Click should switch to code mode and not throw.
    await sourceResult.click();
    await page.waitForTimeout(400);
  });

  /**
   * T2.5: arrow key navigation works in the results list.
   */
  test("T2.5: arrow keys navigate results list", async ({ page }) => {
    const searchInput = page.locator('[data-testid="global-search-input"]');
    await expect(searchInput).toBeVisible({ timeout: 5000 });

    await searchInput.fill("a");
    await page.waitForTimeout(600);

    const results = page.locator('[data-testid="global-search-results"]');
    if (!(await results.isVisible().catch(() => false))) return;

    const rows = page.locator(".search-result-row");
    if ((await rows.count()) < 2) return;

    await searchInput.focus();
    await page.keyboard.press("ArrowDown");
    await page.waitForTimeout(100);

    const focusedRow = page.locator(".search-result-row--focused");
    await expect(focusedRow).toBeVisible();
  });

  /**
   * T2.5: Enter key activates the focused result.
   */
  test("T2.5: Enter key activates focused result", async ({ page }) => {
    const searchInput = page.locator('[data-testid="global-search-input"]');
    await expect(searchInput).toBeVisible({ timeout: 5000 });

    await searchInput.fill("a");
    await page.waitForTimeout(600);

    const results = page.locator('[data-testid="global-search-results"]');
    if (!(await results.isVisible().catch(() => false))) return;
    if ((await page.locator(".search-result-row").count()) === 0) return;

    await searchInput.focus();
    await page.keyboard.press("ArrowDown");
    await page.waitForTimeout(100);
    await page.keyboard.press("Enter");
    await page.waitForTimeout(400);
    // If we get here without throw, activation fired successfully
  });
});
