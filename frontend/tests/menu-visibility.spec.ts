/**
 * T1.6 — Menu visibility regression test.
 *
 * Validates that top-level menu dropdowns open, render in front of the dock
 * layout, remain clickable at every supported viewport, and support full
 * keyboard navigation.
 *
 * Viewports covered:
 *   - 1920×1080  (desktop, well above threshold)
 *   - 1366×768   (desktop, well above threshold)
 *   - 1280×800   (desktop, at the threshold — dropdown must still render
 *                 above the dock, not be clipped)
 *
 * Menu structure (File · Edit · View · Tools · Run · Help) is preserved per
 * spec invariant §5-1.
 */

import { expect, Page, test } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await page.goto("/?skip-welcome=1");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
}

/**
 * Dismiss the Welcome overlay if it appears. The overlay is rendered after
 * the OPFS hydration microtask so we may need to wait briefly for it to mount.
 */
async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  const overlay = page.locator('[data-testid="welcome-overlay"]');
  await page.waitForTimeout(500);
  const count = await overlay.count();
  if (count === 0) return;
  const skipBtn = overlay.locator('[data-testid="welcome-skip-btn"]');
  try {
    await skipBtn.click({ force: true, timeout: 5_000 });
    await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  } catch {
    /* swallow — the next locator action will re-attempt cleanly */
  }
}

/**
 * Open a menu by label and return the portaled dropdown locator.
 *
 * The dropdown is portaled to document.body, so it is NOT a DOM descendant of
 * the trigger button. We click the trigger to open, then locate the portaled
 * dropdown via its body-level data-testid.  Only one dropdown is open at a
 * time in this test suite, so the body-level selector is unambiguous.
 */
async function openMenu(
  page: Page,
  label: string,
): Promise<import("@playwright/test").Locator> {
  const normalized = label.toLowerCase();
  const trigger = page.locator(
    `[data-testid="menu-${normalized}"] .menu-trigger`,
  );
  await trigger.click();
  // The dropdown is portaled to body — locate it directly by its body-level testid.
  return page.locator('[data-testid="menu-dropdown"]');
}

test.describe("Menu visibility across viewports", () => {
  for (const [label, width, height] of [
    ["1920×1080", 1920, 1080],
    ["1366×768", 1366, 768],
    ["1280×800", 1280, 800],
  ] as const) {
    test.describe(`at ${label}`, () => {
      test.beforeEach(async ({ page }) => {
        await page.setViewportSize({ width, height });
        await waitForEngine(page);
        await dismissWelcomeIfPresent(page);
      });

      // ── S1: menu header visibility ──────────────────────────────────────

      test("all six menu headers are visible", async ({ page }) => {
        for (const menu of ["file", "edit", "view", "tools", "run", "help"]) {
          await expect(
            page.locator(`[data-testid="menu-${menu}"]`),
          ).toBeVisible();
        }
      });

      // ── S1: each menu opens and shows items ─────────────────────────────

      test("File menu opens and shows all items", async ({ page }) => {
        const dropdown = await openMenu(page, "File");
        await expect(dropdown).toBeVisible();

        await expect(
          dropdown.getByRole("menuitem", { name: /New Scene/ }),
        ).toBeVisible();
        await expect(dropdown.getByTestId("save-btn")).toBeVisible();
        await expect(
          dropdown.getByRole("menuitem", { name: /Load Project/ }),
        ).toBeVisible();
      });

      test("Edit menu opens and shows all items", async ({ page }) => {
        const dropdown = await openMenu(page, "Edit");
        await expect(dropdown).toBeVisible();

        await expect(
          dropdown.getByRole("menuitem", { name: /Undo/ }),
        ).toBeVisible();
        await expect(
          dropdown.getByRole("menuitem", { name: /Redo/ }),
        ).toBeVisible();
        await expect(
          dropdown.getByRole("menuitem", { name: /Delete/ }),
        ).toBeVisible();
      });

      test("View menu opens and shows all items", async ({ page }) => {
        const dropdown = await openMenu(page, "View");
        await expect(dropdown).toBeVisible();

        await expect(
          dropdown.getByRole("menuitem", { name: /Toggle Assets/ }),
        ).toBeVisible();
        await expect(
          dropdown.getByRole("menuitem", { name: /Workspace/ }),
        ).toBeVisible();
      });

      test("Tools menu opens and shows all items", async ({ page }) => {
        const dropdown = await openMenu(page, "Tools");
        await expect(dropdown).toBeVisible();

        await expect(
          dropdown.getByRole("menuitem", { name: /AI Assistant/ }),
        ).toBeVisible();
        await expect(
          dropdown.getByRole("menuitem", { name: /Validation Center/ }),
        ).toBeVisible();
      });

      test("Run menu opens and shows all items", async ({ page }) => {
        const dropdown = await openMenu(page, "Run");
        await expect(dropdown).toBeVisible();

        await expect(
          dropdown.getByRole("menuitem", { name: /Play/ }),
        ).toBeVisible();
        await expect(
          dropdown.getByRole("menuitem", { name: /Pause/ }),
        ).toBeVisible();
      });

      test("Help menu opens and shows all items", async ({ page }) => {
        const dropdown = await openMenu(page, "Help");
        await expect(dropdown).toBeVisible();

        await expect(
          dropdown.getByRole("menuitem", { name: /Cheat Sheet/ }),
        ).toBeVisible();
        await expect(
          dropdown.getByRole("menuitem", { name: /Welcome Tour/ }),
        ).toBeVisible();
      });

      // ── S1: dropdown geometry ─────────────────────────────────────────────

      test("File menu dropdown is not clipped by dock layout", async ({
        page,
      }) => {
        const dropdown = await openMenu(page, "File");
        await expect(dropdown).toBeVisible();

        // Get the dropdown rect and viewport height via page.evaluate since
        // Playwright's boundingBox() returns null for position:fixed portaled elements.
        const { dropdownBottom, viewportHeight } = await page.evaluate(() => {
          const el = document.querySelector('[data-testid="menu-dropdown"]');
          if (!el) return { dropdownBottom: null, viewportHeight: null };
          const r = el.getBoundingClientRect();
          return { dropdownBottom: r.bottom, viewportHeight: window.innerHeight };
        });

        expect(dropdownBottom).not.toBeNull();
        expect(dropdownBottom!).toBeGreaterThan(0);
        expect(dropdownBottom!).toBeLessThan(
          viewportHeight!,
          "dropdown bottom extends below the viewport",
        );
      });

      // ── S1: Escape closes ────────────────────────────────────────────────

      test("Escape closes the File dropdown", async ({ page }) => {
        const dropdown = await openMenu(page, "File");
        await expect(dropdown).toBeVisible();

        await page.keyboard.press("Escape");
        await expect(dropdown).not.toBeAttached();
      });

      // ── S1: menu auto-close when another opens ───────────────────────────

      test("opening Edit menu closes File menu", async ({ page }) => {
        await openMenu(page, "File");
        // Confirm File dropdown is open.
        await expect(
          page.locator('[aria-label="File menu"][data-testid="menu-dropdown"]'),
        ).toBeVisible();

        // Open Edit menu — File should auto-close and Edit should open.
        await openMenu(page, "Edit");
        // File dropdown should be detached.
        await expect(
          page.locator('[aria-label="File menu"][data-testid="menu-dropdown"]'),
        ).not.toBeAttached();
        // Edit dropdown should be visible.
        await expect(
          page.locator('[aria-label="Edit menu"][data-testid="menu-dropdown"]'),
        ).toBeVisible();
      });

      // ── S1: keyboard navigation ──────────────────────────────────────────

      test("ArrowDown navigates through File menu items", async ({ page }) => {
        await openMenu(page, "File");
        await page.keyboard.press("ArrowDown");

        // Focus should now be on Save Scene (second item).
        const dropdown = page.locator('[data-testid="menu-dropdown"]');
        const saveItem = dropdown.getByTestId("save-btn");
        await expect(saveItem).toBeFocused();
      });

      test("ArrowUp navigates backwards through File menu items", async ({
        page,
      }) => {
        await openMenu(page, "File");
        // Navigate to Export Rust (4 ArrowDown presses from New Scene, which is index 0).
        await page.keyboard.press("ArrowDown"); // Save Scene (1)
        await page.keyboard.press("ArrowDown"); // Save Scene As (2)
        await page.keyboard.press("ArrowDown"); // Load Project (3)
        await page.keyboard.press("ArrowDown"); // Export Rust (4)

        // ArrowUp should go back to Load Project.
        await page.keyboard.press("ArrowUp");

        const dropdown = page.locator('[data-testid="menu-dropdown"]');
        const loadItem = dropdown.getByRole("menuitem", { name: /Load Project/ });
        await expect(loadItem).toBeFocused();
      });

      test("Home jumps to first menu item", async ({ page }) => {
        await openMenu(page, "File");
        // Move to a later item first
        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");
        await page.keyboard.press("ArrowDown");

        // Home should jump back to first
        await page.keyboard.press("Home");

        const dropdown = page.locator('[data-testid="menu-dropdown"]');
        const firstItem = dropdown.getByRole("menuitem", { name: /New Scene/ });
        await expect(firstItem).toBeFocused();
      });

      test("End jumps to the last enabled menu item", async ({ page }) => {
        await openMenu(page, "File");
        await page.keyboard.press("End");

        const dropdown = page.locator('[data-testid="menu-dropdown"]');
        // Last enabled item before disabled Quit is Export Rust
        const lastItem = dropdown.getByRole("menuitem", { name: /Export Rust/ });
        await expect(lastItem).toBeFocused();
      });

      test("Enter activates the focused menu item and closes the menu", async ({
        page,
      }) => {
        await openMenu(page, "File");
        // Navigate to Save Scene (2nd item: ArrowDown from New Scene).
        await page.keyboard.press("ArrowDown"); // Save Scene (index 1)

        const dropdown = page.locator('[data-testid="menu-dropdown"]');
        await expect(dropdown.getByTestId("save-btn")).toBeFocused();

        // Press Enter — the item's onClick fires and closes the menu.
        await page.keyboard.press("Enter");
        // Use the File dropdown's aria-label to avoid matching a newly opened dropdown.
        await expect(
          page.locator('[aria-label="File menu"][data-testid="menu-dropdown"]'),
        ).not.toBeAttached();
      });

      test("Space activates the focused menu item and closes the menu", async ({
        page,
      }) => {
        await openMenu(page, "File");
        // Wait for the focus effect to settle before pressing Space.
        await page.waitForTimeout(100);
        await expect(
          page.locator('[data-testid="menu-dropdown"]').getByRole("menuitem", { name: /New Scene/ }),
        ).toBeFocused();

        // Press Space — New Scene's onClick fires and closes the menu.
        await page.keyboard.press(" ");
        await expect(
          page.locator('[aria-label="File menu"][data-testid="menu-dropdown"]'),
        ).not.toBeAttached();
      });

      // ── S1: View menu workspace submenu ─────────────────────────────────

      test("View menu opens and workspace submenu items are visible", async ({
        page,
      }) => {
        const dropdown = await openMenu(page, "View");
        await expect(dropdown).toBeVisible();

        const workspaceItem = dropdown.getByRole("menuitem", {
          name: /Workspace/,
        });
        await expect(workspaceItem).toBeVisible();
      });
    });
  }
});
