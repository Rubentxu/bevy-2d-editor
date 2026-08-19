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
      name: "smoke",
      grep: /@smoke/,
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "domain",
      grep: /@domain/,
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "persistence",
      grep: /@persistence/,
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "accessibility",
      grep: /@accessibility/,
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "full",
      grep: /@full/,
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
