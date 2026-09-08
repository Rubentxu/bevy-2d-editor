import { expect, test } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";
import { clearWelcomeDismissed } from "./helpers/welcome-state";

/**
 * Phase E — Welcome overlay (Defold-inspired redesign).
 *
 * Validates that the first-visit WelcomeOverlay appears with all 5 workflow
 * cards + Skip / Take the tour buttons + Don't show again checkbox.
 *
 * The overlay is gated by an OPFS-backed `welcome-dismissed.json` flag.
 * `clearWelcomeDismissed()` (in ./helpers/welcome-state.ts) wipes that
 * flag in beforeEach so the test sees a "first visit" regardless of
 * prior session state.
 */

test.describe("Defold-inspired welcome overlay (Phase E)", { tag: ["@full"] }, () => {
  test.beforeEach(async ({ page }) => {
    // Phase 1: navigate to /?skip-welcome=1 so the welcome overlay doesn't
    // block pointer events during WASM init (the overlay's OPFS hydration
    // can race with the engine-bridge bridge installation otherwise).
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);
    // Phase 2: clear OPFS so the overlay treats this as a "first visit".
    await clearWelcomeDismissed(page);
    // Phase 3: navigate to / (WITHOUT the skip-welcome query param) so the
    // WelcomeOverlay re-reads OPFS with the cleared flag and renders.
    // (Note: page.reload() preserves the ?skip-welcome=1 from Phase 1,
    //  which would keep urlSkip=true and the overlay would never show —
    //  use page.goto("/") instead.)
    await page.goto("/");
    await waitForEditorReady(page);
  });

  test("appears on first visit with all 5 workflow cards", async ({ page }) => {
    const overlay = page.locator('[data-testid="welcome-overlay"]');
    await expect(overlay).toBeVisible();
    for (const step of [
      "inspect-assets",
      "build-levels",
      "compose-logic",
      "wire-components",
      "play-&-test",
    ]) {
      await expect(
        page.locator(`[data-testid="welcome-card-${step}"]`),
      ).toBeVisible();
    }
    await expect(page.locator('[data-testid="welcome-skip-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="welcome-tour-btn"]')).toBeVisible();
    await expect(
      page.locator('[data-testid="welcome-dont-show"]'),
    ).toBeAttached();
  });

  test("clicking Skip closes the overlay", async ({ page }) => {
    const overlay = page.locator('[data-testid="welcome-overlay"]');
    await expect(overlay).toBeVisible();
    await page.locator('[data-testid="welcome-skip-btn"]').click();
    await expect(overlay).not.toBeVisible();
  });

  test("clicking Take the tour also closes the overlay", async ({ page }) => {
    const overlay = page.locator('[data-testid="welcome-overlay"]');
    await expect(overlay).toBeVisible();
    await page.locator('[data-testid="welcome-tour-btn"]').click();
    await expect(overlay).not.toBeVisible();
  });
});
