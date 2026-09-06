import { defineConfig, devices } from "@playwright/test";

/**
 * Full cohort — runs the complete test suite (all tags).
 *
 * Uses the @full tag (the superset tag).
 *
 * Per spec §6.2:
 * - GIVEN the full cohort
 * - WHEN any cohort already failed
 * - THEN the full cohort does not mask it; the release-health gate stays `red`.
 */
export default defineConfig({
  testDir: "./tests",
  grep: /@full/,
  timeout: 120_000,
  expect: { timeout: 60_000 },
  fullyParallel: false,
  retries: 0,
  use: {
    baseURL: "http://localhost:5173",
    headless: true,
    snapshotDir: "tests/baselines",
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
