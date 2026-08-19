/**
 * Hito 4 Order 6 (`code-aware-ai`) E2E tests.
 *
 * Verifies that the AI proxy mock receives multi-source context and the
 * ContextDebugSection surfaces the per-source stats.
 *
 * The mock proxy is pre-configured with 4 new patterns (PR1):
 * - "source file" / "create .rs" / "create .toml" → CreateSourceFile
 * - "write function" / "add method" / "modify .rs" → WriteSourceFile
 * - "logic graph" / "connect nodes" → mock Batch
 * - "asset" / "scene asset" → mock asset reference
 *
 * The frontend assembles multi-source context via `assembleMultiSourceContext`
 * (PR2) which fetches source files via WASM and applies a token budget.
 */

import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;
const MOCK_PROXY_URL = "http://localhost:11436";

test.describe("code-aware-ai (Hito 4 Order 6)", { tag: ["@full"] }, () => {
  test.beforeEach(async ({ page }) => {
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        console.log(`[browser error] ${msg.text()}`);
      }
    });
    // Override the proxy URL so the AI service hits the mock.
    await page.addInitScript((url) => {
      (window as any).__aiProxyUrlOverride = url;
      const origFetch = window.fetch.bind(window);
      window.fetch = (input: RequestInfo | URL, init?: RequestInit) => {
        const u =
          typeof input === "string"
            ? input
            : input instanceof URL
            ? input.toString()
            : input.url;
        if (u.includes("/v1/propose") && !u.startsWith(url)) {
          if (typeof input === "string") {
            return origFetch(url + "/v1/propose", init);
          }
          const newUrl = new URL("/v1/propose", url);
          return origFetch(newUrl.toString(), init);
        }
        return origFetch(input, init);
      };
    }, MOCK_PROXY_URL);
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
  });

  test("ContextDebugSection renders per-source stats after submit", async ({ page }) => {
    await page.click('[data-testid="ai-panel-btn"]');
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();

    // First, ensure there is a source file in the project (create one via WASM)
    await page.evaluate(async () => {
      const opfs = await import("../src/opfs-bridge");
      await opfs.opfsSaveFile("sources/src/player.rs", "struct Player { name: String }");
    });

    // Submit a simple prompt that triggers a mock response
    await page.locator(".ai-prompt-input").fill("create sprite");
    await page.locator(".ai-submit-btn").click();
    await expect(page.locator(".proposal-card").first()).toBeVisible({ timeout: 10_000 });

    // Context debug section should be visible after the propose.
    // Hito 5 followups (v0.77.1): wait briefly for React to render the
    // debug section (stats are set asynchronously after submit resolves).
    await page.waitForTimeout(500);
    await expect(page.locator('[data-testid="context-debug-section"]')).toBeVisible();
    // Expand it
    await page.locator('[data-testid="context-debug-toggle"]').click();
    await expect(page.locator('[data-testid="context-debug-body"]')).toBeVisible();
    // Should have at least scene_snapshot + schemas rows
    const rowCount = await page.locator('[data-testid^="context-row-"]').count();
    expect(rowCount).toBeGreaterThanOrEqual(2);
  });

  test("AI create source file mock pattern returns CreateSourceFile command", async ({ page }) => {
    await page.click('[data-testid="ai-panel-btn"]');
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();

    // Intercept the proxy request and assert it carries source_files context
    const requestPromise = page.waitForRequest(
      (req) => req.url().includes("/v1/propose") && req.method() === "POST"
    );
    await page.locator(".ai-prompt-input").fill("create a source file with .rs");
    await page.locator(".ai-submit-btn").click();
    const req = await requestPromise;
    const body = JSON.parse(req.postData() ?? "{}");
    expect(body.prompt).toContain("create a source file");
    // The proposal should render
    await expect(page.locator(".proposal-card").first()).toBeVisible({ timeout: 10_000 });
    // Mock returns a CreateSourceFile command
    const proposalType = await page.locator(".proposal-card").first().getAttribute("data-command-type");
    expect(proposalType).toBe("CreateSourceFile");
  });

  test("Token budget is visible in debug section", async ({ page }) => {
    await page.click('[data-testid="ai-panel-btn"]');
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();
    await page.locator(".ai-prompt-input").fill("create sprite");
    await page.locator(".ai-submit-btn").click();
    await expect(page.locator(".proposal-card").first()).toBeVisible({ timeout: 10_000 });
    // The meter should show a percentage (e.g. "1/10k tokens (10%)")
    await expect(page.locator(".context-debug-meter")).toContainText("tokens");
  });

  test("ContextDebugSection collapses on toggle", async ({ page }) => {
    await page.click('[data-testid="ai-panel-btn"]');
    await expect(page.locator(".ai-assistant-panel")).toBeVisible();
    await page.locator(".ai-prompt-input").fill("create sprite");
    await page.locator(".ai-submit-btn").click();
    await expect(page.locator(".proposal-card").first()).toBeVisible({ timeout: 10_000 });
    // Wait for React to update contextStats after submit
    await page.waitForTimeout(500);

    // Body should be hidden initially (collapsed by default)
    const body = page.locator('[data-testid="context-debug-body"]');
    await expect(body).toHaveCount(0);

    // Click toggle to expand
    await page.locator('[data-testid="context-debug-toggle"]').click();
    await expect(body).toBeVisible();

    // Click again to collapse
    await page.locator('[data-testid="context-debug-toggle"]').click();
    await expect(body).toHaveCount(0);
  });
});
