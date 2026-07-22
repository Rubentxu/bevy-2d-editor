import { test, expect, Page } from "@playwright/test";

/**
 * Phase 3.2 — Command Palette (Ctrl+K) tests.
 *
 *  - Ctrl+K opens the command palette
 *  - Typing filters the visible commands
 *  - Enter executes the focused command
 *  - Escape closes the palette
 */

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () =>
      typeof (window as any).load_scene_json === "function" &&
      typeof (window as any).get_scene_snapshot === "function",
    undefined,
    { timeout: 30_000 },
  );
}

test.describe("UX Command Palette — Phase 3.2", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await waitForEngine(page);
  });

  test("Ctrl+K opens the command palette", async ({ page }) => {
    // Ensure no input is focused
    await page.evaluate(() => {
      const active = document.activeElement as HTMLElement | null;
      if (active && typeof active.blur === "function") active.blur();
    });

    // Palette should not be visible initially
    await expect(page.locator('[data-testid="command-palette"]')).toHaveCount(0);

    // Open via Ctrl+K
    await page.keyboard.press("Control+k");

    await expect(page.locator('[data-testid="command-palette"]')).toBeVisible({
      timeout: 5_000,
    });
    await expect(page.locator('[data-testid="command-palette-input"]')).toBeFocused();

    // There must be at least 15 commands rendered (catalog has ~20; we cap
    // visible at 20).
    const items = page.locator('[data-testid^="command-palette-item-"]');
    const count = await items.count();
    expect(count).toBeGreaterThanOrEqual(15);
  });

  test("Typing filters the visible command list", async ({ page }) => {
    await page.evaluate(() => {
      const active = document.activeElement as HTMLElement | null;
      if (active && typeof active.blur === "function") active.blur();
    });
    await page.keyboard.press("Control+k");
    await expect(page.locator('[data-testid="command-palette"]')).toBeVisible();

    const input = page.locator('[data-testid="command-palette-input"]');
    await input.fill("undo");

    // "undo" should narrow the list — should match at least the Undo command
    await expect(
      page.locator('[data-testid="command-palette-item-edit.undo"]'),
    ).toHaveCount(1, { timeout: 2_000 });

    // And the result count should be smaller than the unfiltered list
    const filteredCount = await page
      .locator('[data-testid^="command-palette-item-"]')
      .count();
    expect(filteredCount).toBeLessThan(15);
  });

  test("Enter executes the focused command and closes the palette", async ({ page }) => {
    // Seed an empty scene so we can observe the Undo command's effect via
    // the entity counter (Undo on empty scene is a no-op, so use a
    // side-effect-free command: New Entity).
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "1",
          scene_id: "ux-commands",
          name: "UX Commands",
          entities: [],
        }),
      ),
    );

    await page.evaluate(() => {
      const active = document.activeElement as HTMLElement | null;
      if (active && typeof active.blur === "function") active.blur();
    });
    await page.keyboard.press("Control+k");
    await expect(page.locator('[data-testid="command-palette"]')).toBeVisible();

    // Type "new entity" to isolate the New Entity command (1 result).
    await page.locator('[data-testid="command-palette-input"]').fill("new entity");
    await expect(
      page.locator('[data-testid="command-palette-item-edit.new-entity"]'),
    ).toHaveCount(1);

    const before = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot?.();
      const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
      return doc?.entities?.length ?? 0;
    });

    // Press Enter — command runs and palette closes
    await page.keyboard.press("Enter");

    // Palette must be gone
    await expect(page.locator('[data-testid="command-palette"]')).toHaveCount(0, {
      timeout: 3_000,
    });

    // Entity count must have bumped
    await page.waitForFunction(
      (prev) => {
        const snap = (window as any).get_scene_snapshot?.();
        const doc = typeof snap === "string" ? JSON.parse(snap) : snap;
        return (doc?.entities?.length ?? 0) > prev;
      },
      before,
      { timeout: 5_000 },
    );
  });

  test("Escape closes the command palette", async ({ page }) => {
    await page.evaluate(() => {
      const active = document.activeElement as HTMLElement | null;
      if (active && typeof active.blur === "function") active.blur();
    });
    await page.keyboard.press("Control+k");
    await expect(page.locator('[data-testid="command-palette"]')).toBeVisible();

    await page.keyboard.press("Escape");
    // Allow the close animation to finish (200ms ease-out + RAF).
    await expect(page.locator('[data-testid="command-palette"]')).toBeHidden({
      timeout: 5_000,
    });
  });
});