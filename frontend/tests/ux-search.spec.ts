import { expect, Page, test } from "@playwright/test";

/**
 * v0.81 Tier 1 — Global Search tab.
 *
 * Validates that the SearchTab (bottom dock) renders the search input,
 * helper text, and empty-state behavior. The hook is fully wired to the
 * scenes / scene-assets / source-files / asset-files hooks, but on a fresh
 * page load the index is empty for every category so we use an unmatched
 * query string to assert the empty state.
 *
 * Note: the bottom dock starts visible by default (F7 toggles it), so the
 * test doesn't need to toggle F7 — it just selects the Search tab.
 */

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () =>
      typeof (window as any).load_scene_json === "function" &&
      typeof (window as any).dispatch_command === "function",
    undefined,
    { timeout: 30_000 },
  );
}

/**
 * Dismiss the Phase E welcome overlay if it appears so the bottom dock
 * tabs remain clickable. The overlay renders after OPFS hydration and
 * can briefly intercept pointer events.
 */
async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  const overlay = page.locator('[data-testid="welcome-overlay"]');
  await page.waitForTimeout(300);
  if ((await overlay.count()) === 0) return;
  const skipBtn = overlay.locator('[data-testid="welcome-skip-btn"]');
  try {
    await skipBtn.waitFor({ state: "visible", timeout: 5_000 });
    await skipBtn.click({ force: true, timeout: 5_000 });
    await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  } catch {
    /* overlay may have unmounted itself; safe to continue */
  }
}

/**
 * Ensure the Search tab is the active tab in the bottom dock.
 */
async function activateSearchTab(page: Page): Promise<void> {
  const bottomDock = page.locator('[data-testid="dock-bottom"]');
  await expect(bottomDock).toBeVisible();
  const tab = page.locator('[data-testid="bottom-dock-tab-search"]');
  // Click if not already selected.
  if ((await tab.getAttribute("aria-selected")) !== "true") {
    await tab.click();
  }
  await expect(page.locator('[data-testid="bottom-tabpanel-search"]')).toBeVisible();
}

test.describe("Global Search (v0.81)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);
    await activateSearchTab(page);
  });

  test("shows the search input and placeholder text", async ({ page }) => {
    const input = page.locator('[data-testid="global-search-input"]');
    await expect(input).toBeVisible();
    await expect(input).toHaveAttribute("placeholder", /Search/i);
  });

  test("empty query shows helper text", async ({ page }) => {
    await expect(
      page.locator('[data-testid="global-search-helper"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="global-search-helper"]'),
    ).toContainText(/scenes/i);
  });

  test("query with no matches shows the empty state", async ({ page }) => {
    const input = page.locator('[data-testid="global-search-input"]');
    await input.fill("zzzzz_no_match_xyzzy");
    // Wait for the 150ms debounce + index scan to settle.
    await page.waitForTimeout(500);
    await expect(
      page.locator('[data-testid="global-search-empty"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="global-search-empty"]'),
    ).toContainText("zzzzz_no_match_xyzzy");
  });

  test("non-empty input clears helper text and shows either results or empty state", async ({
    page,
  }) => {
    const input = page.locator('[data-testid="global-search-input"]');
    await input.fill("a");
    await page.waitForTimeout(500);
    // Either we found matches or we didn't — both are valid; just ensure
    // the helper text is gone and one of the two states is visible.
    await expect(
      page.locator('[data-testid="global-search-helper"]'),
    ).toHaveCount(0);
    const resultsCount = await page
      .locator('[data-testid="global-search-results"]')
      .count();
    const emptyCount = await page
      .locator('[data-testid="global-search-empty"]')
      .count();
    expect(resultsCount + emptyCount).toBe(1);
  });
});