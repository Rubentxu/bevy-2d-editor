import { defineConfig, devices } from "@playwright/test";

/**
 * Accessibility cohort — runs axe-core accessibility tests.
 *
 * Uses the @accessibility tag (existing in ux-a11y.spec.ts).
 * axe-core dependency already present in workspace.
 *
 * Per spec §6.2, the accessibility cohort:
 * - GIVEN the accessibility cohort
 * - WHEN run against the seeded state
 * - THEN axe violations are reported with the same set of selectors
 *   across three runs.
 */
export default defineConfig({
  testDir: "./tests",
  testMatch: ["ux-a11y.spec.ts", "a11y-critical-paths.spec.ts"],
  timeout: 120_000,
  expect: { timeout: 60_000 },
  fullyParallel: false,
  retries: 0,
  use: {
    baseURL: "http://localhost:5173",
    headless: true,
  },
  webServer: [
    {
      command: "node tests/fixtures/mock-ai-proxy.mjs",
      url: "http://localhost:11436/health",
      reuseExistingServer: true,
      timeout: 10_000,
      stdout: "ignore",
      stderr: "pipe",
    },
    {
      command: "npx vite",
      url: "http://localhost:5173",
      reuseExistingServer: true,
      timeout: 60_000,
    },
  ],
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
