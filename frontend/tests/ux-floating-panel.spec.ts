/**
 * v0.82 P2 — Floating panels (ADR-0025).
 *
 * End-to-end coverage for the dock-floats subsystem added in PR1:
 *
 *   1. Each dock header exposes a `Float` button (v0.82 P2 addition to
 *      DockHeader / BottomDock). Clicking it flips the dock out of the
 *      CSS-Grid layout into a `createPortal(…)` overlay.
 *   2. The floating panel renders into `document.body` (portal) and
 *      carries `data-testid="floating-panel-<panelId>"`.
 *   3. Clicking another floating panel promotes it to the focused
 *      variant (`--z-floating-panel-focused`).
 *   4. The `×` button on the floating panel header docks it back into
 *      its original grid cell.
 *   5. Reload restores floating rect from OPFS (schema v3).
 *   6. Schema v2 prefs (no `floats` key) upgrade to v3 losslessly.
 *
 * Test helpers:
 *   - `resetDockPrefs` clears both OPFS and localStorage snapshot.
 *   - `loadDockPrefs` waits for the OPFS bridge to bind before reading.
 *   - `dismissWelcomeIfPresent` keeps pointer events unblocked.
 */

import { expect, test, type Page } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;
const PREFERRED_FLOAT_PANEL_ID = "outline"; // outline has the cleanest header target

async function waitForEngine(page: Page): Promise<void> {
  await page.goto("/?skip-welcome=1");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
}

async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  await page.waitForTimeout(500);
  const overlay = page.locator('[data-testid="welcome-overlay"]');
  if ((await overlay.count()) === 0) return;
  try {
    await overlay
      .locator('[data-testid="welcome-skip-btn"]')
      .click({ force: true, timeout: 5_000 });
    await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  } catch {
    /* overlay may have unmounted itself */
  }
}

async function resetDockPrefs(page: Page): Promise<void> {
  // MUST be called *after* `waitForEngine` (a fresh page navigation).
  // The OPFS bridge is bound asynchronously once the WASM bundle has
  // mounted, so we wait for `opfs_exists` to be present on `window`
  // before issuing the first `page.evaluate` call (mirrors the
  // loadDockPrefs pattern in ux-drag-dock.spec.ts).
  await page.waitForFunction(
    () => typeof (window as any).opfs_exists === "function",
    undefined,
    { timeout: 10_000 },
  );
  await page.evaluate(async () => {
    const exists = await (window as any).opfs_exists("dock-prefs.json");
    if (exists) {
      await (window as any).opfs_delete_file("dock-prefs.json");
    }
    try {
      localStorage.removeItem("bevy-2d-editor:dock-panel-regions");
    } catch {
      /* localStorage may be disabled */
    }
  });
}

async function loadDockPrefs(page: Page): Promise<unknown | null> {
  await page.waitForFunction(
    () => typeof (window as any).opfs_exists === "function",
    undefined,
    { timeout: 10_000 },
  );
  return await page.evaluate(async () => {
    const exists = await (window as any).opfs_exists("dock-prefs.json");
    if (!exists) return null;
    const res = await (window as any).opfs_load_file("dock-prefs.json");
    if (!res || !res.ok || !res.value) return null;
    return JSON.parse(res.value);
  });
}

// Each docked panel emits a distinct testid for its Float button:
//   - left        → dock-left-header-float
//   - outline     → dock-right-outline-header-float
//   - properties  → dock-right-properties-header-float
//   - bottom      → dock-bottom-float  (no DockHeader; rendered inline)
// Already-floating panel → dock button uses `floating-panel-{id}-dock`.
const FLOAT_SELECTORS: Record<string, string> = {
  left: "[data-testid='dock-left-header-float']",
  outline: "[data-testid='dock-right-outline-header-float']",
  properties: "[data-testid='dock-right-properties-header-float']",
  assets: "[data-testid='dock-left-header-float']", // left dock hosts assets panel
  bottom: "[data-testid='dock-bottom-float']",
};

async function clickFloatToggle(page: Page, panelId: string): Promise<void> {
  const sel =
    FLOAT_SELECTORS[panelId] ?? `[data-testid='dock-${panelId}-header-float']`;
  const loc = page.locator(sel);
  if ((await loc.count()) === 0) {
    throw new Error(
      `No float toggle found for panelId=${panelId} (selector=${sel})`,
    );
  }
  await loc.first().click({ force: true });
}

test.describe("Floating panels (v0.82 P2, ADR-0025)", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);
    await expect(page.locator('[data-testid="dock-layout"]')).toBeVisible();
    // Reset state once the OPFS bridge is bound (post-WASM mount).
    await resetDockPrefs(page);
  });

  test("Float button appears in docked headers and toggles a portal overlay", async ({
    page,
  }) => {
    const floatBtn = page.locator(FLOAT_SELECTORS[PREFERRED_FLOAT_PANEL_ID]);
    await expect(floatBtn).toBeVisible();
    await floatBtn.click({ force: true });

    // Floating panel renders into document.body via portal.
    const floatOverlay = page.locator(
      `[data-testid="floating-panel-${PREFERRED_FLOAT_PANEL_ID}"]`,
    );
    await expect(floatOverlay).toBeVisible({ timeout: 5_000 });
    await expect(floatOverlay).toHaveAttribute(
      "data-panel-id",
      PREFERRED_FLOAT_PANEL_ID,
    );

    // Original grid slot no longer contains the panel — the body inside
    // the dock root for the right region should not contain the
    // outline-top section's hierarchy list anymore. We assert on
    // absence of the floating portal *inside* the dock-layout subtree.
    const inDock = page
      .locator('[data-testid="dock-layout"]')
      .locator(`[data-testid="floating-panel-${PREFERRED_FLOAT_PANEL_ID}"]`);
    await expect(inDock).toHaveCount(0);
  });

  test("Dock button on the floating overlay restores the panel to its grid cell", async ({
    page,
  }) => {
    const floatBtn = page.locator(FLOAT_SELECTORS[PREFERRED_FLOAT_PANEL_ID]);
    await floatBtn.click({ force: true });
    await expect(
      page.locator(
        `[data-testid="floating-panel-${PREFERRED_FLOAT_PANEL_ID}"]`,
      ),
    ).toBeVisible({ timeout: 5_000 });

    // Click the × Dock button — same panel id, on the floating overlay.
    await page
      .locator(
        `[data-testid="floating-panel-${PREFERRED_FLOAT_PANEL_ID}-dock"]`,
      )
      .click({ force: true });

    await expect(
      page.locator(
        `[data-testid="floating-panel-${PREFERRED_FLOAT_PANEL_ID}"]`,
      ),
    ).toHaveCount(0, { timeout: 5_000 });
    // The header is back inside the dock layout with the Float button
    // available again.
    await expect(
      page.locator(FLOAT_SELECTORS[PREFERRED_FLOAT_PANEL_ID]),
    ).toBeVisible();
  });

  test("Reload restores floating overlay from OPFS (schema v3 round-trip)", async ({
    page,
  }) => {
    const floatBtn = page.locator(FLOAT_SELECTORS[PREFERRED_FLOAT_PANEL_ID]);
    await floatBtn.click({ force: true });
    await expect(
      page.locator(
        `[data-testid="floating-panel-${PREFERRED_FLOAT_PANEL_ID}"]`,
      ),
    ).toBeVisible({ timeout: 5_000 });

    // Wait long enough for the debounced save (500ms) + a small safety
    // margin. The localStorage snapshot writes synchronously inside
    // save() so reload survival works even if the OPFS write is
    // interrupted.
    await page.waitForTimeout(1_500);

    const prefsBefore = (await loadDockPrefs(page)) as {
      schemaVersion: number;
      floats?: Record<string, unknown>;
    } | null;
    expect(prefsBefore).not.toBeNull();
    expect(prefsBefore?.schemaVersion).toBe(3);
    expect(prefsBefore?.floats?.[PREFERRED_FLOAT_PANEL_ID]).toBeTruthy();

    // Reload and confirm the overlay mounts again.
    await page.reload();
    await waitForEngine(page);
    await expect(
      page.locator(
        `[data-testid="floating-panel-${PREFERRED_FLOAT_PANEL_ID}"]`,
      ),
    ).toBeVisible({ timeout: 10_000 });
  });

  test("Schema v2 prefs (no `floats` key) upgrade to v3 losslessly", async ({
    page,
  }) => {
    // Write a v2 prefs file directly into OPFS to simulate an
    // upgrading user. The migration path in `migratePrefs` should
    // fill in `floats = {}` and bump the schemaVersion to 3.
    await page.waitForFunction(
      () => typeof (window as any).opfs_exists === "function",
      undefined,
      { timeout: 10_000 },
    );
    await page.evaluate(async () => {
      const v2Prefs = {
        schemaVersion: 2,
        panelRegions: {
          assets: "left",
          outline: "right",
          properties: "right",
          bottom: "bottom",
        },
        left: { width: 280, visible: true },
        right: {
          width: 320,
          visible: true,
          outlineVisible: true,
          outlineCollapsed: false,
          propertiesVisible: true,
          propertiesCollapsed: false,
          topHeight: 60,
        },
        bottom: { height: 240, visible: true },
        statusBar: { height: 24 },
      };
      await (window as any).opfs_save_file(
        "dock-prefs.json",
        JSON.stringify(v2Prefs),
      );
    });

    // Reload so the loader + migration run.
    await page.reload();
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);

    // After migration the prefs file on disk is at v3 with panelRegions
    // preserved verbatim from the v2 input and an empty floats map.
    await page.waitForTimeout(1_500);
    const prefs = (await loadDockPrefs(page)) as {
      schemaVersion: number;
      panelRegions: Record<string, string>;
      floats: Record<string, unknown>;
    } | null;
    expect(prefs).not.toBeNull();
    expect(prefs?.schemaVersion).toBe(3);
    expect(prefs?.panelRegions.assets).toBe("left");
    expect(prefs?.panelRegions.outline).toBe("right");
    expect(prefs?.panelRegions.properties).toBe("right");
    expect(prefs?.panelRegions.bottom).toBe("bottom");
    expect(prefs?.floats).toEqual({});

    // No floating overlay mounts because nothing was floating for the
    // upgrading user.
    await expect(page.locator('[data-testid^="floating-panel-"]')).toHaveCount(
      0,
    );
  });

  test("Clicking a second floating panel promotes it (focus stacking)", async ({
    page,
  }) => {
    // Lift two panels (outline and bottom).
    await page
      .locator(FLOAT_SELECTORS[PREFERRED_FLOAT_PANEL_ID])
      .click({ force: true });
    await page.locator(FLOAT_SELECTORS.bottom).click({ force: true });

    const outline = page.locator(
      `[data-testid="floating-panel-${PREFERRED_FLOAT_PANEL_ID}"]`,
    );
    const bottom = page.locator('[data-testid="floating-panel-bottom"]');
    await expect(outline).toBeVisible({ timeout: 5_000 });
    await expect(bottom).toBeVisible({ timeout: 5_000 });

    // Promote the bottom panel by clicking its header.
    await page
      .locator(`[data-testid="floating-panel-bottom-header"]`)
      .click({ force: true });

    // The bottom panel should now carry the `--z-floating-panel-focused`
    // class while the outline panel sits at the base z-index.
    await expect(bottom).toHaveClass(/_focused/);
    await expect(outline).not.toHaveClass(/_focused/);
  });
});
