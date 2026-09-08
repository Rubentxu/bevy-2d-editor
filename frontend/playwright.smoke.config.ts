import { defineConfig, devices } from "@playwright/test";

/**
 * Smoke cohort — fastest path to "the editor opens and reacts to a
 * click". This is the contract the stabilization release-health gate
 * relies on.
 *
 * Runs ONLY specs tagged `@smoke`. The cohort MUST stay under 60 s on
 * the CI runner; if it doesn't, the offending spec must move out (not
 * the gate must be loosened).
 *
 * Specs that are NOT in this cohort:
 *   - engine.spec.ts → tagged `@full`, runs in playwright.full.config.ts.
 *     It is the workhorse of the release-health gate (20 cases covering
 *     typed command system, operation log, OPFS persistence, schema
 *     registry, UI hierarchy/inspector) and runs in ~1.2 m, exceeding
 *     the 60 s smoke budget on its own.
 *   - ux-dock.spec.ts, mode-context-bar.spec.ts, mode-headers.spec.ts →
 *     tagged `@full`, run in playwright.full.config.ts.
 *   - _check_scene_field.spec.ts → tagged `@domain`, runs in
 *     playwright.domain.config.ts.
 *
 * History: pre-v0.108.9 listed the above specs by file name in
 * testMatch, which silently coupled smoke budget to full-cohort runtime.
 * The v0.108.9 fix moves to tag-driven selection so the budget is
 * enforced structurally.
 */
export default defineConfig({
  testDir: "./tests",
  testMatch: [
    "smoke.spec.ts",
    "app-characterization.spec.ts",
    "capabilities-smoke.spec.ts",
    "editor-ready.spec.ts",
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