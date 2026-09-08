/**
 * Welcome-state test helpers.
 *
 * The WelcomeOverlay is gated by an OPFS-backed
 * `welcome-dismissed.json` flag. When a test needs to assert the
 * "first visit" UX (overlay visible, dismiss button, etc.) it must
 * clear that flag before navigating to `/`.
 *
 * `clearWelcomeDismissed()` is best-effort: it tries to delete the
 * OPFS file directly; if OPFS isn't exposed to the test runner, the
 * overlay simply defaults to visible (which is what we want anyway).
 *
 * Usage:
 *   test.beforeEach(async ({ page }) => {
 *     await page.goto("/?skip-welcome=1");
 *     await waitForEditorReady(page);
 *     await clearWelcomeDismissed(page);
 *     // page.reload() preserves ?skip-welcome=1 — use page.goto("/")
 *     await page.goto("/");
 *     await waitForEditorReady(page);
 *   });
 *
 * The 3-step pattern (warm-up + clear + re-navigate) is required
 * because the overlay's OPFS hydration races with the engine-bridge
 * bridge installation; clearing OPFS before the warm-up can leave
 * the welcome-overlay hidden if the engine starts in parallel.
 */

import type { Page } from "@playwright/test";

export async function clearWelcomeDismissed(page: Page): Promise<void> {
  await page.evaluate(async () => {
    try {
      const opfs = (navigator as unknown as {
        storage?: { getDirectory?: () => Promise<unknown> };
      }).storage?.getDirectory;
      if (typeof opfs !== "function") return;
      const root = await (navigator as unknown as {
        storage: { getDirectory: () => Promise<FileSystemDirectoryHandle> };
      }).storage.getDirectory();
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
