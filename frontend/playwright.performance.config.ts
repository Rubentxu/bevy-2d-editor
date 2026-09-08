import { defineConfig, devices } from "@playwright/test";

/**
 * Performance cohort — runs the G6 performance corpus (v1.0-stabilization §5.2).
 *
 * `testMatch` is restricted to `perf-*.spec.ts` so the cohort is bounded.
 * Each spec carries the `@full` tag in addition to `@performance`, so the
 * cohort also runs in `playwright.full.config.ts` as part of the superset.
 *
 * Per spec §6.4:
 *   - Runs daily, NOT on every PR (PR-time CI uses smoke + domain + persistence + a11y).
 *   - Failure is non-blocking for merge (a budget regression is a perf bug,
 *     not a correctness bug; the cycle must be patched separately).
 *   - Wall time budget: 240 s per spec (each spec measures a single op
 *     that should fit in a few seconds; the long timeout is for CI worst-case).
 */
export default defineConfig({
  testDir: "./tests",
  testMatch: ["perf-*.spec.ts"],
  timeout: 240_000,
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
