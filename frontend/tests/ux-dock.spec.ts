import { expect, Page, test } from "@playwright/test";

/**
 * Phase B — 3-region dock layout (Defold-inspired redesign).
 *
 * Validates the DockLayout CSS Grid host:
 *   - All three regions render (left Assets / center viewport / right dock)
 *   - Default widths match the spec (280px left, 320px right)
 *   - The right divider is draggable and updates the dock width
 *
 * The widths are read from the computed `grid-template-columns` of the
 * dock-layout element rather than measuring DOM rectangles, because the
 * column widths are computed by the grid engine from the CSS variables
 * maintained by useDockResize.
 */

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await page.goto("/?skip-welcome=1&skip-onboarding=1");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () => (window as any).__bevyEngineStarted === true,
    undefined,
    { timeout: 30_000 },
  );
}

/**
 * Dismiss the Phase E welcome overlay if it appears. The overlay is rendered
 * after the OPFS hydration microtask so we may need to wait briefly for it
 * to mount, then click Skip with `force: true` to bypass any race where the
 * pointer is intercepted by a sibling backdrop element. Non-overlay tests
 * rely on this so the menubar / status bar / dock regions remain
 * interactive.
 */
async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  const overlay = page.locator('[data-testid="welcome-overlay"]');
  // Give the OPFS-hydrated WelcomeOverlay a chance to mount.
  await page.waitForTimeout(500);
  const count = await overlay.count();
  if (count === 0) return;
  const skipBtn = overlay.locator('[data-testid="welcome-skip-btn"]');
  try {
    await skipBtn.waitFor({ state: "visible", timeout: 5_000 });
    await skipBtn.click({ force: true, timeout: 5_000 });
    await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  } catch {
    // If Skip can't be clicked the overlay may have unmounted itself;
    // regardless, retry the click with force once.
    if ((await overlay.count()) > 0) {
      try {
        await skipBtn.click({ force: true, timeout: 2_000 });
      } catch {
        /* swallow — the next locator action will re-attempt cleanly */
      }
    }
  }
}

/**
 * Reads the dock-layout grid column widths. Non-px tokens (e.g. "1fr")
 * surface as NaN so the caller can pick the right column by index.
 */
async function readColumnWidths(page: Page): Promise<number[]> {
  return await page.evaluate(() => {
    const layout = document.querySelector(
      '[data-testid="dock-layout"]',
    ) as HTMLElement | null;
    if (!layout) return [];
    const cols = getComputedStyle(layout)
      .gridTemplateColumns.trim()
      .split(/\s+/);
    return cols.map((c) => (c.endsWith("px") ? parseFloat(c) : NaN));
  });
}

test.describe("Defold-inspired 3-region dock layout", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEngine(page);
    // Welcome overlay (Phase E) blocks pointer events for everything below
    // it. Dismiss it before exercising dock drag/click tests.
    await dismissWelcomeIfPresent(page);
    await expect(page.locator('[data-testid="dock-layout"]')).toBeVisible();
  });

  test("renders all three regions", async ({ page }) => {
    await expect(
      page.locator('[data-testid="dock-region-left"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="dock-region-center"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="dock-region-right"]'),
    ).toBeVisible();
  });

  test("left dock has 280px default width", async ({ page }) => {
    const widths = await readColumnWidths(page);
    expect(widths.length).toBe(3);
    // Allow ±1px slack for sub-pixel rounding.
    expect(widths[0]).toBeGreaterThanOrEqual(279);
    expect(widths[0]).toBeLessThanOrEqual(281);
  });

  test("right dock has 320px default width", async ({ page }) => {
    const widths = await readColumnWidths(page);
    expect(widths.length).toBe(3);
    expect(widths[2]).toBeGreaterThanOrEqual(319);
    expect(widths[2]).toBeLessThanOrEqual(321);
  });

  test("F7 toggles the bottom dock", async ({ page }) => {
    const bottomDock = page.locator('[data-testid="dock-bottom"]');
    await expect(bottomDock).toBeVisible();

    await page.keyboard.press("F7");
    await expect(bottomDock).toBeHidden();

    await page.keyboard.press("F7");
    await expect(bottomDock).toBeVisible();
  });

  test("bottom dock tabs are clickable", async ({ page }) => {
    for (const tab of ["console", "search", "output", "problems"]) {
      const tabButton = page.locator(`[data-testid="bottom-dock-tab-${tab}"]`);
      await tabButton.click();
      await expect(tabButton).toHaveAttribute("aria-selected", "true");
      await expect(
        page.locator(`[data-testid="bottom-tabpanel-${tab}"]`),
      ).toBeVisible();
    }
  });

  test("dragging the right divider updates the right dock width", async ({
    page,
  }) => {
    // The right divider lives on the left edge of the right dock region.
    // Compute its start X from the dock-layout rect minus the right column
    // width (the divider is 4px wide and straddles the boundary).
    const initial = await page.evaluate(() => {
      const layout = document.querySelector(
        '[data-testid="dock-layout"]',
      ) as HTMLElement | null;
      if (!layout) return null;
      const r = layout.getBoundingClientRect();
      const cols = getComputedStyle(layout)
        .gridTemplateColumns.trim()
        .split(/\s+/);
      const colWidths = cols.map((c) =>
        c.endsWith("px") ? parseFloat(c) : NaN,
      );
      // Right column starts at r.right - colWidths[2]
      const rightColStart = r.right - (colWidths[2] ?? 0);
      return {
        x: rightColStart + 2,
        y: r.top + r.height / 2,
        rightColWidth: colWidths[2] ?? 0,
      };
    });
    expect(initial).not.toBeNull();
    const { x, y } = initial as { x: number; y: number; rightColWidth: number };

    // Drag the right divider 60px to the LEFT — the right dock should grow
    // (the divider's `delta` is positive when moving left, per the hook
    // convention `setRightWidth(width - delta)`).
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x - 60, y, { steps: 10 });
    await page.mouse.up();

    // Allow the React state + CSS var propagation to settle.
    await page.waitForTimeout(150);

    const widths = await readColumnWidths(page);
    const newRightWidth = widths[2];
    // Initial 320 + ~60 growth (clamped by MAX_RIGHT=600) — accept a wide
    // band so the assertion stays stable across slow CI runners.
    expect(newRightWidth).toBeGreaterThan(340);
    expect(newRightWidth).toBeLessThanOrEqual(600);
  });
});

/**
 * Phase D — 7-segment status bar (Defold-inspired redesign).
 *
 * Validates that the StatusBar exposes the 7 expected segments (position /
 * selection / project / scene+dirty / zoom / fps / build) and that the
 * zoom segment opens a dropdown with the documented zoom options.
 */
test.describe("Defold-inspired status bar (Phase D)", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);
    await expect(page.locator('[data-testid="status-bar"]')).toBeVisible();
  });

  test("exposes all 7 status-bar segments", async ({ page }) => {
    for (const name of [
      "position",
      "selection",
      "project",
      "scene",
      "zoom",
      "fps",
      "build",
    ]) {
      await expect(
        page.locator(`[data-testid="status-segment-${name}"]`),
      ).toBeVisible();
    }
  });

  test("clicking the zoom segment opens a dropdown with zoom options", async ({
    page,
  }) => {
    // Click the zoom segment to open the dropdown.
    await page.locator('[data-testid="status-segment-zoom"]').click();
    const dropdown = page.locator('[data-testid="status-zoom-dropdown"]');
    await expect(dropdown).toBeVisible();

    // Verify every zoom preset from the spec (25/50/100/200 + Fit) is present.
    for (const preset of ["25", "50", "100", "200"]) {
      await expect(
        page.locator(`[data-testid="status-zoom-option-${preset}"]`),
      ).toBeAttached();
    }
    await expect(
      page.locator('[data-testid="status-zoom-fit"]'),
    ).toBeAttached();

    // Click outside closes the dropdown.
    await page.locator('[data-testid="menubar"]').click();
    await expect(dropdown).not.toBeVisible();
  });
});

/**
 * Phase E — F-keys + Reset Layout + fullscreen viewport.
 */
test.describe("Defold-inspired F-keys + Reset Layout (Phase E)", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEngine(page);
    await dismissWelcomeIfPresent(page);
    await expect(page.locator('[data-testid="dock-layout"]')).toBeVisible();
  });

  test("F6 toggles the left (Assets) dock", async ({ page }) => {
    const leftDock = page.locator('[data-testid="dock-left"]');
    await expect(leftDock).toBeVisible();

    await page.keyboard.press("F6");
    await expect(leftDock).not.toBeAttached();

    // The collapsed strip is shown instead.
    await expect(page.locator('[data-testid="dock-left-strip"]')).toBeVisible();

    await page.keyboard.press("F6");
    await expect(leftDock).toBeVisible();
  });

  test("F9 toggles fullscreen viewport via data-fullscreen", async ({
    page,
  }) => {
    await expect(page.locator('[data-testid="dock-left"]')).toBeVisible();

    await page.keyboard.press("F9");
    await expect.poll(async () =>
      page.evaluate(() => document.body.dataset.fullscreen ?? ""),
    ).toBe("true");

    // Exit fullscreen — the body attribute clears so docks come back.
    await page.keyboard.press("F9");
    await expect.poll(async () =>
      page.evaluate(() => document.body.dataset.fullscreen ?? ""),
    ).toBe("");
  });

  test("Reset Layout menu item restores default dock widths", async ({
    page,
  }) => {
    // Drag the right divider to grow the right column by ~80px so we can
    // confirm Reset Layout restores the 320px default.
    const startRect = await page.evaluate(() => {
      const layout = document.querySelector(
        '[data-testid="dock-layout"]',
      ) as HTMLElement | null;
      if (!layout) return null;
      const r = layout.getBoundingClientRect();
      const cols = getComputedStyle(layout)
        .gridTemplateColumns.trim()
        .split(/\s+/);
      const colWidths = cols.map((c) =>
        c.endsWith("px") ? parseFloat(c) : NaN,
      );
      return {
        x: r.right - (colWidths[2] ?? 0) + 2,
        y: r.top + r.height / 2,
        rightColWidth: colWidths[2] ?? 0,
      };
    });
    expect(startRect).not.toBeNull();
    const { x, y } = startRect as { x: number; y: number };

    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x - 80, y, { steps: 10 });
    await page.mouse.up();
    await page.waitForTimeout(150);

    // Confirm the right column has grown past the 320 default.
    const before = await readColumnWidths(page);
    expect(before[2]).toBeGreaterThan(350);

    // Open the View menu and click Reset Layout.
    await page.locator('[data-testid="menu-view"] .menu-trigger').click();
    const viewDropdown = page.locator(
      '[data-testid="menu-view"] .menu-dropdown',
    );
    // The dropdown is rendered through a React portal on document.body, so
    // wait for the stable data-testid the portal exposes.
    await page.waitForFunction(
      () =>
        document.querySelector('[data-testid="menu-reset-layout"]') !== null,
      undefined,
      { timeout: 10_000 },
    );
    await page.locator('[data-testid="menu-reset-layout"]').click();

    // After the click React state has reset and the OPFS save is debounced.
    await page.waitForTimeout(700);

    const after = await readColumnWidths(page);
    expect(after[2]).toBeGreaterThanOrEqual(319);
    expect(after[2]).toBeLessThanOrEqual(321);
  });
});
