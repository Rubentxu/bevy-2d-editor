import { defineConfig, devices } from "@playwright/test";

/**
 * Smoke cohort — fastest path to "the editor opens and reacts to a
 * click". This is the contract the stabilization release-health gate
 * relies on.
 *
 * Runs the smallest subset of specs that exercises: app boot, dock
 * layout, engine readiness, and a primary navigation action. The cohort
 * MUST stay under 60 s on the CI runner; if it doesn't, the offending
 * spec must move out (not the gate must be loosened).
 *
 * Excluded tests:
 *   - capabilities-smoke.spec.ts → "clicking Code button reveals code
 *     editor container": currently intercepted by WelcomeOverlay in the
 *     cohort context; the full suite runs it.
 */
export default defineConfig({
  testDir: "./tests",
  testMatch: [
    "smoke.spec.ts",
    "engine.spec.ts",
    "_check_scene_field.spec.ts",
    "mode-context-bar.spec.ts",
    "mode-headers.spec.ts",
    "ux-dock.spec.ts",
  ],
  testIgnore: [
    "**/e2e/**",
    "**/baselines/**",
  ],
  timeout: 120_000,
  expect: { timeout: 60_000 },
  fullyParallel: true,
  workers: 2,
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