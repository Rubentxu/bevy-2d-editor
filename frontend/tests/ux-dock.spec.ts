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
  await page.goto("/");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
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
    const cols = getComputedStyle(layout).gridTemplateColumns
      .trim()
      .split(/\s+/);
    return cols.map((c) => (c.endsWith("px") ? parseFloat(c) : NaN));
  });
}

test.describe("Defold-inspired 3-region dock layout", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEngine(page);
    await expect(page.locator('[data-testid="dock-layout"]')).toBeVisible();
  });

  test("renders all three regions", async ({ page }) => {
    await expect(page.locator('[data-testid="dock-region-left"]')).toBeVisible();
    await expect(
      page.locator('[data-testid="dock-region-center"]'),
    ).toBeVisible();
    await expect(page.locator('[data-testid="dock-region-right"]')).toBeVisible();
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
      const cols = getComputedStyle(layout).gridTemplateColumns
        .trim()
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
