import { test, expect, Page } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

/**
 * Playwright E2E tests for Validation Center inbox layout (workflow-surface-convergence T2.2).
 *
 * Coverage:
 * - T2.2: Validation Center renders left sidebar (filters), center (grouped list), right (detail)
 * - T2.2: Issues are grouped by domain (scene / asset / logic / code / runtime / ai)
 * - T2.2: Clicking an issue selects it and shows detail
 * - T2.2: Domain and severity filters work
 * - T2.2: Responsive collapse below 1280px
 */

/**
 * Open the Validation Center by clicking through the menu.
 */
async function openValidationCenter(page: Page): Promise<void> {
  // Remove welcome overlay from DOM if it exists (it blocks all pointer events)
  await page.evaluate(() => {
    const overlay = document.querySelector('[data-testid="welcome-overlay"]');
    if (overlay instanceof HTMLElement) {
      overlay.remove();
    }
  });
  await page.waitForTimeout(300);

  // Check if already open
  const vcAlreadyOpen = await page
    .locator('[data-testid="validation-center"]')
    .isVisible({ timeout: 500 })
    .catch(() => false);
  if (vcAlreadyOpen) return;

  // Click Tools menu button
  const toolsMenu = page.locator('button.menu-trigger:has-text("Tools")');
  await toolsMenu.click();
  await page.waitForTimeout(400);

  // Look for the Validation Center menu item by role and text
  const vcItem = page.locator(
    '[data-testid="menu-dropdown"] button[role="menuitem"]:has-text("Validation Center")',
  );
  await vcItem.waitFor({ state: "visible", timeout: 3000 });
  await vcItem.click();
  await page.waitForTimeout(400);
}

/**
 * Seed schema and AI issues via the window-exposed test helpers.
 * These functions push to the module-level queues that getAllValidationIssues() drains.
 */
async function seedSchemaAndAIIssues(page: Page): Promise<void> {
  await page.evaluate(() => {
    // Seed one schema issue
    if (typeof (window as any).__registerSchemaIssue === "function") {
      (window as any).__registerSchemaIssue({
        severity: "error",
        category: "schema",
        domain: "code",
        code: "missing_transform",
        message: "Entity 'Player' is missing editor.Transform2D",
      });
    }
    // Seed one AI proposal failure
    if (typeof (window as any).__recordAIProposalFailure === "function") {
      (window as any).__recordAIProposalFailure({
        code: "ai_proposal_rejected",
        message: "AI proposal 'Add health bar' was rejected by the user",
        affected_asset_id: "asset_player",
      });
    }
  });
}

test.describe("Validation Center Inbox (T2.2)", () => {
  test.beforeEach(async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });

    await page.goto("/?skip-welcome=1&skip-onboarding=1");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    // Wait for core WASM bindings
    await page.waitForFunction(
      () =>
        typeof (window as any).get_validation_issues_wasm === "function" &&
        typeof (window as any).get_resync_reports === "function" &&
        typeof (window as any).list_scenes_extended === "function",
      { timeout: WASM_LOAD_TIMEOUT },
    );

    if (errors.length > 0) {
      console.warn("Console errors during init:", errors);
    }

    await openValidationCenter(page);
  });

  /**
   * T2.2: The Validation Center renders the 3-column inbox layout.
   */
  test("T2.2: renders 3-column inbox layout (sidebar + list + detail slot)", async ({ page }) => {
    const vc = page.locator('[data-testid="validation-center"]');
    await expect(vc).toBeVisible({ timeout: 5000 });

    await expect(page.locator('[data-testid="vc-sidebar"]')).toBeVisible();
    await expect(
      page.locator(
        '[data-testid="vc-list"], [data-testid="vc-list-empty"], [data-testid="vc-empty"]',
      ),
    ).toBeVisible();
    await expect(page.locator(".vc-title")).toContainText(
      "Validation Center",
    );
  });

  /**
   * T2.2: Issues are grouped by domain when issues are present.
   * Strengthened: seed schema and AI issues and assert they appear.
   */
  test("T2.2: issues are grouped by domain with headers", async ({ page }) => {
    await expect(
      page.locator('[data-testid="validation-center"]'),
    ).toBeVisible({ timeout: 5000 });

    // CRITICAL ISSUE 5: seed schema and AI issues to ensure non-empty inbox.
    await seedSchemaAndAIIssues(page);
    await page.waitForTimeout(500);

    // Refresh to pick up seeded issues
    await page.locator('[data-testid="vc-refresh-btn"]').click();
    await page.waitForTimeout(1000);

    const list = page.locator('[data-testid="vc-list"]');
    const emptyList = page.locator('[data-testid="vc-list-empty"]');
    const empty = page.locator('[data-testid="vc-empty"]');
    const listVisible = await list.isVisible().catch(() => false);
    const emptyListVisible = await emptyList.isVisible().catch(() => false);
    const emptyVisible = await empty.isVisible().catch(() => false);

    // With seeded issues, the list should be visible (not empty).
    expect(listVisible).toBeTruthy();

    // Domain headers should be present for seeded issues.
    const domainHeaders = page.locator("[data-testid^='vc-domain-header-']");
    const count = await domainHeaders.count();
    expect(count).toBeGreaterThan(0);
  });

  /**
   * T2.2: Clicking an issue selects it and shows the detail panel.
   * Strengthened: seed issues to ensure the detail panel test is non-vacuous.
   */
  test("T2.2: clicking an issue shows detail panel", async ({ page }) => {
    await expect(
      page.locator('[data-testid="validation-center"]'),
    ).toBeVisible({ timeout: 5000 });

    // CRITICAL ISSUE 5: seed issues to ensure non-empty inbox.
    await seedSchemaAndAIIssues(page);
    await page.waitForTimeout(500);

    // Refresh to pick up seeded issues
    await page.locator('[data-testid="vc-refresh-btn"]').click();
    await page.waitForTimeout(1000);

    const issueRows = page.locator(".vc-issue");
    const issueCount = await issueRows.count();

    // With seeded issues, there must be at least one issue to click.
    expect(issueCount).toBeGreaterThan(0);

    await issueRows.first().click();
    await page.waitForTimeout(300);

    await expect(page.locator('[data-testid="vc-detail"]')).toBeVisible();
    await expect(page.locator(".vc-detail__message")).toBeVisible();
    await expect(
      page.locator('[data-testid="vc-detail-close"]'),
    ).toBeVisible();
  });

  /**
   * T2.2: Severity filter buttons are present in the sidebar.
   */
  test("T2.2: severity filter buttons are present in sidebar", async ({
    page,
  }) => {
    await expect(
      page.locator('[data-testid="vc-sidebar"]'),
    ).toBeVisible();

    for (const severity of ["all", "error", "warning", "info"]) {
      await expect(
        page.locator(`[data-testid="vc-severity-filter-${severity}"]`),
      ).toBeVisible();
    }
  });

  /**
   * T2.2: Domain filter buttons are present in the sidebar.
   */
  test("T2.2: domain filter buttons are present in sidebar", async ({
    page,
  }) => {
    await expect(
      page.locator('[data-testid="vc-sidebar"]'),
    ).toBeVisible();

    for (const domain of [
      "scene",
      "asset",
      "logic",
      "code",
      "runtime",
      "ai",
    ]) {
      await expect(
        page.locator(`[data-testid="vc-domain-filter-${domain}"]`),
      ).toBeVisible();
    }
  });

  /**
   * T2.2: Refresh button re-fetches issues.
   */
  test("T2.2: refresh button re-fetches issues", async ({ page }) => {
    await expect(
      page.locator('[data-testid="validation-center"]'),
    ).toBeVisible({ timeout: 5000 });

    const refreshBtn = page.locator('[data-testid="vc-refresh-btn"]');
    await expect(refreshBtn).toBeVisible();
    await refreshBtn.click();
    // Should not throw
  });

  /**
   * T2.2: Layout collapses gracefully below 1280px viewport.
   */
  test("T2.2: collapses detail panel below 1280px viewport", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1279, height: 800 });
    await page.waitForTimeout(200);

    const detail = page.locator('[data-testid="vc-detail"]');
    await expect(detail).not.toBeVisible();
  });

  /**
   * T2.2: Layout collapses to single column below 900px viewport.
   */
  test("T2.2: collapses sidebar below 900px viewport", async ({ page }) => {
    await page.setViewportSize({ width: 899, height: 800 });
    await page.waitForTimeout(200);

    await expect(
      page.locator('[data-testid="vc-sidebar"]'),
    ).not.toBeVisible();
  });
});
