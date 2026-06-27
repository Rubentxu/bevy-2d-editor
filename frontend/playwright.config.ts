import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
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
      reuseExistingServer: false,
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
