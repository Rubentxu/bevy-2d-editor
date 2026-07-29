import { expect, Page, test } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await page.goto("/?skip-welcome=1");
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
}

test.describe("Defold-inspired menu bar", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEngine(page);
  });

  test("shows all six menu headers", async ({ page }) => {
    for (const menu of ["file", "edit", "view", "tools", "run", "help"]) {
      await expect(page.locator(`[data-testid="menu-${menu}"]`)).toBeVisible();
    }
  });

  test("opens the File menu with scene and project actions", async ({
    page,
  }) => {
    const menuTrigger = page.locator('[data-testid="menu-file"] .menu-trigger');
    await menuTrigger.click();

    // The dropdown is portaled to body — locate it by its body-level data-testid.
    const dropdown = page.locator('[data-testid="menu-dropdown"]');
    await expect(dropdown).toBeVisible();
    await expect(
      dropdown.getByRole("menuitem", { name: /New Scene/ }),
    ).toBeVisible();
    await expect(
      dropdown.getByRole("menuitem", {
        name: "Save Scene Ctrl+S",
        exact: true,
      }),
    ).toBeVisible();
    await expect(
      dropdown.getByRole("menuitem", { name: /Load Project/ }),
    ).toBeVisible();
  });

  test("Escape closes an open dropdown", async ({ page }) => {
    await page.locator('[data-testid="menu-file"] .menu-trigger').click();
    const dropdown = page.locator('[data-testid="menu-dropdown"]');
    await expect(dropdown).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(dropdown).not.toBeAttached();
  });
});
