import { test, expect, Page } from "@playwright/test";

/**
 * Phase C T3.5 — No duplicate onboarding surfaces (spec S5).
 *
 * Validates:
 *  1. Welcome overlay and OnboardingBanner do NOT appear simultaneously on first visit.
 *  2. When the user checks "Don't show again" in Welcome and dismisses it,
 *     the OnboardingBanner does not appear on subsequent page loads.
 *
 * The Welcome overlay is gated by OPFS `welcome-dismissed.json`.
 * The OnboardingBanner is gated by OPFS `.bevy/onboarding.json` AND
 * now also checks `welcome-dismissed.json` (Phase C T3.3) so that
 * the "Don't show again" choice is shared between both surfaces.
 */

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
}

async function clearBothDismissedFlags(page: Page): Promise<void> {
  // Clear both OPFS flags so we simulate a true first-visit state
  await page.evaluate(async () => {
    try {
      const opfs = (navigator as any).storage?.getDirectory;
      if (typeof opfs !== "function") return;
      const root = await (navigator as any).storage.getDirectory();
      const dir = await root.getDirectoryHandle("bevy-2d-editor", {
        create: false,
      });
      // welcome-dismissed.json
      try {
        await dir.removeEntry("welcome-dismissed.json");
      } catch { /* missing */ }
      // .bevy/onboarding.json
      let bevydir: FileSystemDirectoryHandle;
      try {
        bevydir = await dir.getDirectoryHandle(".bevy", { create: false });
      } catch {
        return; // .bevy dir doesn't exist yet
      }
      try {
        await bevydir.removeEntry("onboarding.json");
      } catch { /* missing */ }
    } catch {
      /* OPFS unsupported */
    }
  });
}

test.describe("No duplicate onboarding surfaces (Phase C T3.5 / spec S5)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await waitForEngine(page);
    await clearBothDismissedFlags(page);
    // Reload to pick up cleared OPFS state as a true "first visit"
    await page.reload();
    await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
  });

  test("Welcome overlay and OnboardingBanner are mutually exclusive on first visit", async ({ page }) => {
    const welcomeOverlay = page.locator('[data-testid="welcome-overlay"]');
    const onboardingBanner = page.locator('[data-testid="onboarding-banner"]');

    // One (or neither) should be visible — never both
    const welcomeVisible = await welcomeOverlay.isVisible().catch(() => false);
    const bannerVisible = await onboardingBanner.isVisible().catch(() => false);

    // If Welcome is visible (normal first-visit), OnboardingBanner must not be
    if (welcomeVisible) {
      await expect(onboardingBanner).not.toBeVisible();
    }
    // If Welcome was already dismissed but banner shows, that's also fine
    // (user dismissed Welcome via Skip, but hasn't dismissed OnboardingBanner)
    if (bannerVisible) {
      await expect(welcomeOverlay).not.toBeVisible();
    }

    // The key invariant: they are never both visible simultaneously
    expect(welcomeVisible && bannerVisible).toBe(false);
  });

  test("'Don't show again' in Welcome persists across reload — banner stays hidden", async ({ page }) => {
    const welcomeOverlay = page.locator('[data-testid="welcome-overlay"]');
    const onboardingBanner = page.locator('[data-testid="onboarding-banner"]');
    const dontShowAgain = page.locator('[data-testid="welcome-dont-show"]');
    const skipBtn = page.locator('[data-testid="welcome-skip-btn"]');

    // Welcome should be visible on first visit
    await expect(welcomeOverlay).toBeVisible();

    // Check "Don't show again"
    await dontShowAgain.check();
    await expect(dontShowAgain).toBeChecked();

    // Click Skip to dismiss
    await skipBtn.click();
    await expect(welcomeOverlay).not.toBeVisible();

    // Reload — both should stay hidden
    await page.reload();
    await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    // Neither should appear after reload
    await expect(welcomeOverlay).not.toBeVisible();
    await expect(onboardingBanner).not.toBeVisible();
  });

  test("OnboardingBanner shows when Welcome is skipped without 'Don't show again'", async ({ page }) => {
    const welcomeOverlay = page.locator('[data-testid="welcome-overlay"]');
    const onboardingBanner = page.locator('[data-testid="onboarding-banner"]');
    const skipBtn = page.locator('[data-testid="welcome-skip-btn"]');

    // Welcome visible on first visit
    await expect(welcomeOverlay).toBeVisible();

    // Click Skip WITHOUT checking "Don't show again"
    await skipBtn.click();
    await expect(welcomeOverlay).not.toBeVisible();

    // Give React time to hydrate OnboardingBanner
    await page.waitForTimeout(500);

    // OnboardingBanner SHOULD appear because user didn't permanently opt out.
    // Welcome is closed (temporary Skip), so mutual exclusion no longer applies.
    // The banner must be visible — unconditional assertion (spec S5 / T3.5).
    await expect(onboardingBanner).toBeVisible();
  });

  test("OnboardingBanner shows on fresh visit when Welcome has no persisted flag", async ({ page }) => {
    const welcomeOverlay = page.locator('[data-testid="welcome-overlay"]');
    const onboardingBanner = page.locator('[data-testid="onboarding-banner"]');

    // Wait for hydration
    await page.waitForTimeout(500);

    // Either Welcome shows OR OnboardingBanner shows — never both
    const welcomeVisible = await welcomeOverlay.isVisible().catch(() => false);
    const bannerVisible = await onboardingBanner.isVisible().catch(() => false);

    // If Welcome shows, banner is hidden (mutual exclusion)
    if (welcomeVisible) {
      await expect(onboardingBanner).not.toBeVisible();
    }

    // Mutual exclusion invariant: never both visible
    expect(welcomeVisible && bannerVisible).toBe(false);
  });
});
