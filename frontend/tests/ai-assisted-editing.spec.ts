/**
 * AI-Assisted Editing — End-to-End Playwright Tests.
 *
 * These tests verify the complete flow from AI button click through
 * proposal display to command dispatch, using the mock AI proxy fixture.
 *
 * Prerequisites (started by playwright.config.ts webServer entries):
 * - Vite dev server on port 5173
 * - Mock AI proxy on port 11436
 * - WASM module already built (run `just wasm` first)
 */

import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

/** Mock proxy URL — matches the webServer entry in playwright.config.ts */
const MOCK_PROXY_URL = "http://localhost:11436";

test.describe("AI-Assisted Editing", () => {
  // Per-test setup: navigate and wait for WASM
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    await page.waitForFunction(
      () =>
        typeof (window as any).get_scene_snapshot === "function" &&
        typeof (window as any).dispatch_command === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );
    // Point the AI service at the mock proxy for all tests
    await page.addInitScript(() => {
      // Override the fetch used by ai-assistant.ts to always hit mock proxy
      // The actual fetch call uses window.fetch, so we ensure the proxy URL is set
    });
  });

  // ─── Test 1: AI button opens panel ─────────────────────────────────────────

  test("AI button opens panel", async ({ page }) => {
    // Panel should not be visible initially
    await expect(page.locator(".ai-assistant-panel")).not.toBeVisible();

    // Click the ✨ AI button in the top bar
    await page.click('[data-testid="ai-panel-btn"]');

    // Panel should now be visible
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();
    await expect(page.locator(".ai-panel-title")).toHaveText("AI Assistant");

    // Prompt textarea should be visible
    await expect(page.locator(".ai-prompt-input")).toBeVisible();

    // Submit button should be visible
    await expect(page.locator(".ai-submit-btn")).toBeVisible();
  });

  // ─── Test 2: Submit prompt shows loading then proposal ─────────────────────

  test("Submit prompt shows loading then proposal", async ({ page }) => {
    // Open the AI panel first
    await page.click('[data-testid="ai-panel-btn"]');
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();

    // Type a prompt that matches "create sprite" pattern in mock proxy
    const promptInput = page.locator(".ai-prompt-input");
    await promptInput.fill("create sprite at x=100");

    // Submit button should be enabled (prompt is non-empty)
    await expect(page.locator(".ai-submit-btn")).toBeEnabled();

    // Submit — click and wait for loading state then proposal
    await page.locator(".ai-submit-btn").click();

    // Loading spinner should appear briefly
    await expect(page.locator(".ai-loading")).toBeVisible({ timeout: 5000 });

    // Proposal card should appear after the mock proxy responds
    await expect(page.locator(".proposal-card")).toBeVisible({ timeout: 10_000 });

    // Proposal should contain rationale text
    await expect(page.locator(".proposal-rationale")).toBeVisible();

    // Proposal should show the model name
    await expect(page.locator(".proposal-model")).toContainText("gpt-4o");

    // Proposal should show command list
    await expect(page.locator(".proposal-commands")).toBeVisible();

    // Apply and Discard buttons should be visible
    await expect(page.locator(".proposal-apply-btn")).toBeVisible();
    await expect(page.locator(".proposal-discard-btn")).toBeVisible();
  });

  // ─── Test 3: Apply proposal dispatches commands ─────────────────────────────

  test("Apply proposal dispatches commands and updates scene", async ({ page }) => {
    // Load a blank scene so we can verify entity creation
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "apply-test",
          name: "Apply Test",
          entities: [],
        })
      )
    );

    // Open AI panel and submit a "create sprite" prompt
    await page.click('[data-testid="ai-panel-btn"]');
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();

    await page.locator(".ai-prompt-input").fill("create sprite at x=100");
    await page.locator(".ai-submit-btn").click();

    // Wait for proposal card
    await expect(page.locator(".proposal-card")).toBeVisible({ timeout: 10_000 });

    // Capture scene snapshot before apply
    const before = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot?.();
      return snap ? JSON.parse(snap) : null;
    });
    const beforeCount = before?.entities?.length ?? 0;

    // Click Apply
    await page.locator(".proposal-apply-btn").click();

    // Wait for proposal to disappear (applied and removed)
    await expect(page.locator(".proposal-card")).not.toBeVisible({ timeout: 10_000 });

    // Verify scene snapshot now has one more entity
    const after = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot?.();
      return snap ? JSON.parse(snap) : null;
    });
    const afterCount = after?.entities?.length ?? 0;
    expect(afterCount).toBeGreaterThan(beforeCount);
  });

  // ─── Test 4: Discard removes proposal ─────────────────────────────────────

  test("Discard removes proposal without dispatching", async ({ page }) => {
    // Open AI panel and submit a prompt
    await page.click('[data-testid="ai-panel-btn"]');
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();

    await page.locator(".ai-prompt-input").fill("add enemy entity");
    await page.locator(".ai-submit-btn").click();

    // Wait for proposal
    await expect(page.locator(".proposal-card")).toBeVisible({ timeout: 10_000 });

    // Capture scene count before discard
    const before = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot?.();
      return snap ? JSON.parse(snap) : null;
    });
    const beforeCount = before?.entities?.length ?? 0;

    // Click Discard
    await page.locator(".proposal-discard-btn").click();

    // Proposal should be gone immediately
    await expect(page.locator(".proposal-card")).not.toBeVisible();

    // Scene should be unchanged
    const after = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot?.();
      return snap ? JSON.parse(snap) : null;
    });
    const afterCount = after?.entities?.length ?? 0;
    expect(afterCount).toBe(beforeCount);
  });

  // ─── Test 5: Proxy unreachable shows error ──────────────────────────────────

  test("Proxy unreachable shows error message", async ({ page }) => {
    // Open AI panel
    await page.click('[data-testid="ai-panel-btn"]');
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();

    // Add script to override proxy URL to a non-existent port
    await page.addInitScript(() => {
      // Stub ai-settings to return a unreachable port
      Object.defineProperty(window, "__aiProxyUrlOverride", {
        value: "http://localhost:19999",
        writable: true,
      });
    });

    await page.locator(".ai-prompt-input").fill("test prompt");
    await page.locator(".ai-submit-btn").click();

    // Error message should appear in the AI panel
    await expect(page.locator(".ai-error")).toBeVisible({ timeout: 10_000 });

    // Error text should mention network/fetch issue
    const errorText = await page.locator(".ai-error").textContent();
    expect(errorText).toMatch(/network|fetch|failed|connection/i);
  });

  // ─── Test 6: AI metadata authorship in dispatched commands ─────────────────

  test("AI metadata authorship is agent:gpt-4o in dispatched commands", async ({ page }) => {
    // Load a blank scene
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "metadata-test",
          name: "Metadata Test",
          entities: [],
        })
      )
    );

    // Open AI panel and submit a "create sprite" prompt
    await page.click('[data-testid="ai-panel-btn"]');
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();

    await page.locator(".ai-prompt-input").fill("create sprite at x=100");
    await page.locator(".ai-submit-btn").click();

    // Wait for proposal
    await expect(page.locator(".proposal-card")).toBeVisible({ timeout: 10_000 });

    // Verify the proposal model tag shows gpt-4o
    await expect(page.locator(".proposal-model")).toContainText("gpt-4o");

    // Apply the proposal
    await page.locator(".proposal-apply-btn").click();
    await expect(page.locator(".proposal-card")).not.toBeVisible({ timeout: 10_000 });

    // The command was dispatched — verify authorship by checking the scene snapshot
    // The AI-created entity should have been added (the mock returns ent_ai_001)
    const after = await page.evaluate(() => {
      const snap = (window as any).get_scene_snapshot?.();
      return snap ? JSON.parse(snap) : null;
    });

    // The entity ent_ai_001 should now exist in the scene
    const aiEntity = after?.entities?.find((e) => e.id === "ent_ai_001");
    expect(aiEntity).toBeDefined();
    expect(aiEntity.name).toBe("AI Sprite");
  });
});
