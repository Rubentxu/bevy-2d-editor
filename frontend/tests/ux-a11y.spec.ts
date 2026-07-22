import { test, expect, Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

/**
 * Phase 4 — Accessibility smoke test using @axe-core/playwright.
 *
 * Loads the editor's landing screen (after WASM boot), runs the default
 * axe-core ruleset, and asserts that the page has zero critical or serious
 * WCAG 2.1 AA violations. Lower-severity findings (moderate/minor) are
 * logged for triage but do not fail the test.
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

test.describe("UX Accessibility — Phase 4", () => {
  test("landing screen has zero critical or serious axe violations", async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await waitForEngine(page);

    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
      .analyze();

    const critical = results.violations.filter(
      (v) => v.impact === "critical" || v.impact === "serious"
    );

    if (critical.length > 0) {
      // Print a concise digest for triage.
      // eslint-disable-next-line no-console
      console.error(
        "axe critical/serious violations:\n" +
          critical
            .map(
              (v) =>
                `  - [${v.impact}] ${v.id} (${v.nodes.length} nodes): ${v.help}`
            )
            .join("\n")
      );
    }

    expect(critical).toEqual([]);
  });
});
