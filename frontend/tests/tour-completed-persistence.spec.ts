/**
 * Persists the "tutorial walkthrough completed" flag and verifies the
 * WelcomeOverlay's "Take the tour" button greys out after Finish.
 *
 * Scenario S1:
 *   1. Open the welcome overlay.
 *   2. Click "Take the tour" → stepper opens.
 *   3. Click Next 4 times to reach step 5.
 *   4. Click "Finish" → stepper closes.
 *   5. Re-open the welcome overlay (page.reload() without ?skip-welcome).
 *   6. Assert the "Take the tour" button is now rendered in the
 *      disabled "Tour already taken" state with aria-disabled=true and
 *      data-tour-completed="true".
 */
import { expect, test, type Page } from "@playwright/test";

const A11Y_TIMEOUT = 10_000;

/** Open the welcome overlay by clearing the persistent-dismissal flag
 *  so it appears on the next render. Mirrors the helpers used in the
 *  sibling `tutorial-walkthrough.spec.ts`. */
async function openWelcomeOverlay(page: Page): Promise<void> {
  // Make sure OPFS is clear of the permanent-dismissal flag from prior tests.
  await page.evaluate(async () => {
    try {
      if (typeof navigator !== "undefined" && navigator.storage?.getDirectory) {
        const root = await navigator.storage.getDirectory();
        const dir = await root.getDirectoryHandle("bevy-2d-editor", {
          create: false,
        });
        // Best effort: try to remove the welcome-dismissed.json file if present.
        try {
          await dir.removeEntry("welcome-dismissed.json");
        } catch {
          // file may not exist — ignore
        }
        try {
          await dir.removeEntry(".bevy/tour-flags.json");
        } catch {
          // file may not exist — ignore
        }
      }
    } catch {
      // OPFS unavailable in this runner — fall back to localStorage.
    }
    // Clear any tour-completed flag too so we start clean.
    try {
      localStorage.removeItem("bevy-2d-editor:tour-completed");
    } catch {
      // ignore
    }
  });
  // Visit the editor (no skip-welcome) so the welcome overlay shows.
  await page.goto("/");
  await page
    .locator('[data-testid="welcome-overlay"]')
    .waitFor({ state: "visible", timeout: A11Y_TIMEOUT });
}

/** Click "Take the tour" and wait for the stepper to appear. */
async function startTour(page: Page): Promise<void> {
  await page.locator('[data-testid="welcome-tour-btn"]').click();
  await page
    .locator('[data-testid="tour-stepper"]')
    .waitFor({ state: "visible", timeout: A11Y_TIMEOUT });
}

test.describe("Tour completed persistence — S1", {
  tag: ["@accessibility", "@full"],
}, () => {
  test("WelcomeOverlay greys out the tour button after Finish", async ({
    page,
  }) => {
    // 1+2: open the welcome overlay and start the tour.
    await openWelcomeOverlay(page);
    await startTour(page);

    // 3: advance through steps 1 → 2 → 3 → 4 → 5.
    for (let i = 0; i < 4; i++) {
      await page.locator('[data-testid="tour-next-btn"]').click();
    }
    await expect(
      page.locator('[data-testid="tour-step-counter"]'),
    ).toContainText("Step 5 of 5: Play & Test");

    // 4: click Finish. This writes the persistence flag.
    await page.locator('[data-testid="tour-finish-btn"]').click();

    // The stepper closes; the welcome overlay is also closed.
    await expect(
      page.locator('[data-testid="tour-stepper"]'),
    ).toBeHidden({ timeout: A11Y_TIMEOUT });

    // 5: reload so the welcome overlay re-appears (no ?skip-welcome).
    // Wait briefly for the OPFS write to land first; the helper inside
    // services/tour.ts is a no-await Promise, but the underlying OPFS
    // call completes within the same task tick.
    await page.waitForTimeout(150);
    await page.goto("/");

    // 6: the welcome overlay mounts. The Take-the-tour button is now
    // rendered in the "Tour already taken" disabled state.
    await page
      .locator('[data-testid="welcome-overlay"]')
      .waitFor({ state: "visible", timeout: A11Y_TIMEOUT });

    const tourBtn = page.locator('[data-testid="welcome-tour-btn"]');
    await expect(tourBtn).toBeVisible();
    await expect(tourBtn).toHaveAttribute("aria-disabled", "true");
    await expect(tourBtn).toHaveAttribute("data-tour-completed", "true");
    await expect(tourBtn).toBeDisabled();
    await expect(tourBtn).toContainText("Tour already taken");
  });
});
