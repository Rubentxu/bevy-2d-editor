import { defineConfig, devices } from "@playwright/test";

/**
 * Persistence cohort — runs OPFS/persistence tests.
 *
 * Uses the @persistence tag.
 *
 * Per spec §6.2:
 * - GIVEN OPFS is unavailable in the headless runner
 * - WHEN the persistence cohort runs
 * - THEN each persistence test is tagged with a tracked unblock condition
 *   or explicitly marked skipped with rationale per §B2 rule 5
 * - AND the rationale is machine-checkable by docs-check (Wave C2).
 */
export default defineConfig({
  testDir: "./tests",
  grep: /@persistence/,
  timeout: 120_000,
  expect: { timeout: 60_000 },
  fullyParallel: false,
  retries: 0,
  use: {
    baseURL: "http://localhost:5173",
    headless: true,
    // OPFS may be unavailable in headless; tests handle this gracefully
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
