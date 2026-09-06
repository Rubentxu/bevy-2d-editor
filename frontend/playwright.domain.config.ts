import { defineConfig, devices } from "@playwright/test";

/**
 * Domain cohort — runs domain-specific tests (scene/asset/logic/code modules).
 *
 * Uses the @domain tag.
 *
 * Per spec §6.2, the domain cohort:
 * - GIVEN the domain cohort (scene/asset/logic/code modules)
 * - WHEN a test fails in one module
 * - THEN only that module's cases fail in this cohort; no cross-module cascade.
 */
export default defineConfig({
  testDir: "./tests",
  grep: /@domain/,
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
