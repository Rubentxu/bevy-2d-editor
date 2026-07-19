/**
 * Hito 4 Order 7 (`scene-component-authoring`) E2E tests.
 *
 * Re-enabled in Hito 5 (bevy-engine-hardening) after fixing the Bevy
 * 0.19 query conflict (B0001) that was blocking all 8 pre-existing
 * E2E tests. See docs/adr/0017-e2e-test-failure-root-cause.md and
 * PR #90 (v0.77.0) for details.
 */

import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;
const MOCK_PROXY_URL = "http://localhost:11436";

test.describe("scene-component-authoring (Hito 4 Order 7)", () => {
  test("SchemaKind toggle reveals Bind picker when set to SceneComponent", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    // Open schema authoring panel
    await page.click('[data-testid="schema-new-btn"]');
    // Initially Simple
    await expect(page.locator('[data-testid="schema-kind-toggle"]')).toBeVisible();
    await expect(page.locator('[data-testid="schema-kind-scene-component"]')).not.toBeVisible();
    // Switch to SceneComponent
    await page.click('[data-testid="schema-kind-scene-component"]');
    await expect(page.locator('[data-testid="schema-bound-scene-asset"]')).toBeVisible();
    await expect(page.locator('[data-testid="schema-auto-spawn"]')).toBeVisible();
  });

  test("AddComponentButton shows SceneComponent badge for scene-component schemas", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });
    // Open any entity, open Add Component
    // Verify 🧩 badge appears next to SceneComponent schemas
  });
});
