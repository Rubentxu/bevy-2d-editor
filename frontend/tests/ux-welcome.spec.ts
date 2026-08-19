import { expect, Page, test } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";

/**
 * Phase E — Welcome overlay (Defold-inspired redesign).
 *
 * Validates that the first-visit WelcomeOverlay appears with all 5 workflow
 * cards + Skip / Take the tour buttons + Don't show again checkbox.
 *
 * The overlay is gated by an OPFS-backed `welcome-dismissed.json` flag.
 * We forcibly clear that flag in beforeEach so the test sees a "first
 * visit" regardless of prior session state.
 */



async function clearWelcomeDismissed(page: Page): Promise<void> {
  // The WelcomeOverlay reads from OPFS at `welcome-dismissed.json`. The
  // simplest cross-browser way to "force" first visit in a test is to
  // delete the file via OPFS, but OPFS isn't exposed to test scripts.
  // Instead we expose the overlay by sending the same intent via the
  // DOM: set a sessionStorage flag we use as a re-show signal AND clear
  // the OPFS file via window.eval + the opfs-bridge. If OPFS is missing
  // (test runner) the welcome-overlays simply defaults to "first visit".
  await page.evaluate(async () => {
    try {
      // Best-effort: try to remove the OPFS file using the same module.
      const opfs = (navigator as any).storage?.getDirectory;
      if (typeof opfs !== "function") return;
      const root = await (navigator as any).storage.getDirectory();
      const dir = await root.getDirectoryHandle("bevy-2d-editor", {
        create: false,
      });
      try {
        await dir.removeEntry("welcome-dismissed.json");
      } catch {
        /* missing is fine */
      }
    } catch {
      /* OPFS unsupported — welcome defaults to visible */
    }
  });
}

test.describe("Defold-inspired welcome overlay (Phase E)", { tag: ["@full"] }, () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);
    await clearWelcomeDismissed(page);
    // Reload so the welcome-overlay re-reads OPFS fresh.
    await page.reload();
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
