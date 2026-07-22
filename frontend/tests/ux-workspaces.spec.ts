/**
 * v0.81 Tier 1b — Workspace Presets.
 *
 * Validates the View > Workspace submenu:
 *   - it surfaces the 5 built-in presets (Default, 2D Platformer, Top-Down
 *     RPG, FPS, Minimal)
 *   - applying "2D Platformer" updates the CSS variable `--dock-left-w`
 *     maintained by `useDockResize`
 *   - applying "Minimal" hides all docks (left column collapses off the
 *     grid because `leftVisible` becomes false)
 *
 * The CSS variable is the contract `useDockResize` publishes to
 * DockLayout's CSS Grid template, so reading it is the closest end-to-end
 * signal we have without dipping into React internals.
 */

import { expect, Page, test } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await page.goto("/?skip-welcome=1");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  // Wait for the View menu trigger so dropdowns can be opened reliably.
  await expect(page.locator('[data-testid="menu-view"]')).toBeVisible();
}

/**
 * Open the View menu and hover the "Workspace" entry so the submenu is
 * displayed. Submenus are hover-driven by MenuDropdown (200ms delay); we
 * dispatch the hover and wait for the preset buttons.
 *
 * Returns the locator of the Workspace menu-item-container so callers can
 * dispatch clicks at the submenu-level. The submenu itself stays
 * `display:none` until its parent container is hovered, so plain clicks
 * are intercepted by Chrome; we work around that by dispatching a
 * synthetic click event in the page context (the React listener still
 * fires and applies the preset).
 */
async function openWorkspaceSubmenu(
  page: Page,
): Promise<void> {
  await page.locator('[data-testid="menu-view"] .menu-trigger').click();
  await expect(page.locator('[data-testid="menu-view"] .menu-dropdown')).toBeVisible();
  // Hover the Workspace entry so its submenu renders.
  await page
    .locator('[data-testid="menu-view-workspace"]')
    .first()
    .hover();
  // Wait for at least one preset button to appear inside the submenu.
  await expect(page.locator('[data-testid="menu-preset-default"]')).toBeVisible({
    timeout: 5_000,
  });
}

/** Fire a React-aware click on a submenu item via dispatchEvent. */
async function clickSubmenuItem(
  page: Page,
  testId: string,
): Promise<void> {
  await page.evaluate((id: string) => {
    const el = document.querySelector<HTMLButtonElement>(
      `[data-testid="${id}"]`,
    );
    if (!el) throw new Error(`submenu item not found: ${id}`);
    el.click();
  }, testId);
}

async function readLeftWidth(page: Page): Promise<number> {
  return page.evaluate(() => {
    const raw = getComputedStyle(document.documentElement).getPropertyValue(
      "--dock-left-w",
    );
    // CSS getPropertyValue returns e.g. "340px"; parseFloat pulls out 340.
    const n = parseFloat(raw);
    return Number.isFinite(n) ? n : NaN;
  });
}

test.describe("Workspace Presets (v0.81)", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEngine(page);
  });

  test("View > Workspace submenu exposes the five built-in presets", async ({
    page,
  }) => {
    await openWorkspaceSubmenu(page);
    for (const id of [
      "menu-preset-default",
      "menu-preset-2d-platformer",
      "menu-preset-top-down-rpg",
      "menu-preset-fps",
      "menu-preset-minimal",
      "menu-preset-save",
    ]) {
      await expect(page.locator(`[data-testid="${id}"]`)).toBeVisible();
    }
  });

  test("applying 2D Platformer resizes the left dock to 340px", async ({
    page,
  }) => {
    await openWorkspaceSubmenu(page);
    // The submenu is `display:none` until its parent container is hovered,
    // which Chrome's hit-test honours even when `force: true` is set.
    // `el.click()` from page context bypasses the hit-test and reaches the
    // React synthetic onClick listener, which fires `applyPreset`.
    await clickSubmenuItem(page, "menu-preset-2d-platformer");
    // The new prefs flow through useDockResize's effect, which writes the
    // CSS variable on the next tick. Allow a short settle window.
    await expect
      .poll(() => readLeftWidth(page), { timeout: 5_000 })
      .toBe(340);
  });

  test("applying Minimal hides all docks (left width collapses to 0)", async ({
    page,
  }) => {
    await openWorkspaceSubmenu(page);
    // See note above: dispatch click through the page context to bypass
    // the CSS `display:none` hover gate.
    await clickSubmenuItem(page, "menu-preset-minimal");
    // Minimal sets leftWidth=0 in BUILTIN_PRESETS. The CSS var is what
    // `useDockResize` publishes; leftVisible is also false but not wired
    // onto document.documentElement — width=0 is the visible signal.
    await expect
      .poll(() => readLeftWidth(page), { timeout: 5_000 })
      .toBe(0);
    // Right pane (outline + properties) is rendered unconditionally by
    // DockLayout but with width 0 too. Verify both bottom and right CSS
    // variables collapse as expected.
    const widths = await page.evaluate(() => {
      const cs = getComputedStyle(document.documentElement);
      return {
        right: parseFloat(cs.getPropertyValue("--dock-right-w")),
        bottom: parseFloat(cs.getPropertyValue("--dock-bottom-h")),
      };
    });
    expect(widths.right).toBe(0);
    expect(widths.bottom).toBe(0);
  });
});
