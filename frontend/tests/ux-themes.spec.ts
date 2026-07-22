import { test, expect, Page } from "@playwright/test";

/**
 * Phase 5 — Theme system (UX Overhaul).
 *
 * Validates the new dark/light theme contract:
 *   - The toggle button flips `data-theme` on <html>.
 *   - The body background color reflects the active theme's OKLCH tokens.
 *
 * The browser's `prefers-color-scheme` media query is non-deterministic
 * across CI environments, so we read the initial attribute and assert
 * that the toggle inverts it. We also explicitly seed `data-theme="dark"`
 * before the body-color test so the body assertion is deterministic.
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

test.describe("UX Themes — Phase 5", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await waitForEngine(page);
  });

  test("Clicking the theme toggle button flips data-theme on <html>", async ({
    page,
  }) => {
    const toggle = page.locator('[data-testid="theme-toggle-btn"]');
    await expect(toggle).toBeVisible();

    const before = await page
      .locator("html")
      .getAttribute("data-theme");
    const expected = before === "dark" ? "light" : "dark";

    await toggle.click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", expected);

    // Click again — should flip back.
    await toggle.click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", before);
  });

  test("Light theme applies the light OKLCH palette to body", async ({
    page,
  }) => {
    // Force light theme via the public attribute contract — same path the
    // toggle button uses internally.
    await page.evaluate(() => {
      document.documentElement.setAttribute("data-theme", "light");
    });
    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");

    // Give the browser a tick to apply the new CSS custom properties.
    await page.waitForFunction(
      () => {
        const bg = getComputedStyle(document.body).backgroundColor;
        // Accept either rgb()/rgba() (most browsers) or oklch() (Chromium
        // supports OKLCH in computed styles since 111).
        return /rgba?\(|oklch\(/i.test(bg);
      },
      undefined,
      { timeout: 2_000 },
    );

    const bg = await page.evaluate(() => {
      return getComputedStyle(document.body).backgroundColor;
    });

    // Chromium serializes oklch() in computed styles — accept either.
    let avg = 0;
    const rgbMatch = bg.match(
      /rgba?\(\s*(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)/i,
    );
    if (rgbMatch) {
      const [, r, g, b] = rgbMatch.map(Number);
      avg = (r + g + b) / 3;
    } else if (/oklch\(/i.test(bg)) {
      // oklch(0.98 0.005 260) — lightness 0.98 means very bright.
      const oklchMatch = bg.match(/oklch\(\s*(\d+(?:\.\d+)?)/i);
      if (oklchMatch) {
        // Convert lightness 0..1 to 0..255 average.
        avg = parseFloat(oklchMatch[1]) * 255;
      }
    }
    expect(avg).toBeGreaterThan(200);
  });
});
