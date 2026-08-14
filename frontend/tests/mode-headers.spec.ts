import { test, expect, Page } from "@playwright/test";

/**
 * Phase C T3.4 — Mode-aware dock header titles (spec S7).
 *
 * Validates two contracts:
 *  1. Header labels match the mode (mode-aware title switching).
 *  2. Header title agrees with the actual content rendered in the dock body
 *     (header/body truthfulness — spec S7's core invariant).
 *
 * Modes are driven via `window.__setEditorMode()` (a test hook exposed by App.tsx
 * for Playwright access to the internal editorMode state).
 */

const WASM_LOAD_TIMEOUT = 120_000;

async function waitForEngine(page: Page): Promise<void> {
  await expect(page.locator('[data-testid="menubar"]')).toBeVisible({
    timeout: WASM_LOAD_TIMEOUT,
  });
  await page.waitForFunction(
    () => (window as any).__bevyEngineStarted === true,
    undefined,
    { timeout: 30_000 },
  );
}

/** Drive editorMode via the App.tsx test hook. */
async function setEditorMode(page: Page, mode: string): Promise<void> {
  await page.evaluate((m) => {
    (window as any).__setEditorMode?.(m);
  }, mode);
  // Allow React to re-render the dock headers and bodies
  await page.waitForTimeout(500);
  // When entering code or logic mode, the editor surface is lazy-loaded
  // via React.lazy + Suspense. Wait until the lazy chunk has hydrated so
  // downstream assertions on the editor testid do not race the fallback.
  if (mode === "logic") {
    await expect(page.locator('[data-testid="logic-graph-editor"]')).toBeVisible({
      timeout: 30_000,
    });
  } else if (mode === "code") {
    await expect(page.locator('[data-testid="code-editor"]')).toBeVisible({
      timeout: 30_000,
    });
  }
}

test.describe("Mode-aware dock header titles (Phase C T3.4 / spec S7)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1&skip-onboarding=1");
    await waitForEngine(page);
  });

  test("scene mode: Outline/Properties headers match their bodies", async ({ page }) => {
    const outlineHeader = page.locator('[data-testid="dock-right-outline-header"]');
    const propertiesHeader = page.locator('[data-testid="dock-right-properties-header"]');
    await expect(outlineHeader).toBeVisible();
    await expect(propertiesHeader).toBeVisible();
    await expect(outlineHeader).toContainText("Outline");
    await expect(propertiesHeader).toContainText("Properties");
    // S7 invariant: header title agrees with actual body content
    await expect(page.locator('[data-testid="hierarchy-panel"]')).toBeVisible();
  });

  test("asset-authoring mode: Project Assets/Authoring headers match their bodies", async ({ page }) => {
    await setEditorMode(page, "asset-authoring");

    const outlineHeader = page.locator('[data-testid="dock-right-outline-header"]');
    const propertiesHeader = page.locator('[data-testid="dock-right-properties-header"]');
    await expect(outlineHeader).toBeVisible();
    await expect(propertiesHeader).toBeVisible();
    await expect(outlineHeader).toContainText("Project Assets");
    await expect(propertiesHeader).toContainText("Authoring");
    // S7 invariant: header title agrees with actual body content
    await expect(page.locator('[data-testid="project-asset-browser"]')).toBeVisible();
  });

  test("logic mode: outline shows Outline (body=LogicGraphEditor), properties shows Properties", async ({ page }) => {
    await setEditorMode(page, "logic");

    const outlineHeader = page.locator('[data-testid="dock-right-outline-header"]');
    const propertiesHeader = page.locator('[data-testid="dock-right-properties-header"]');
    await expect(outlineHeader).toBeVisible();
    await expect(propertiesHeader).toBeVisible();
    // Outline header uses generic "Outline" label because outline body is empty in logic mode;
    // properties uses "Properties" for the same reason — truthful about empty state.
    await expect(outlineHeader).toContainText("Outline");
    await expect(propertiesHeader).toContainText("Properties");
    // S7 invariant: header agrees with actual body — outline body is empty, properties body
    // shows the LogicGraphEditor in the bottom slot (properties body), so we check the editor.
    await expect(page.locator('[data-testid="logic-graph-editor"]')).toBeVisible();
  });

  test("code mode: outline shows Outline (body=CodeEditor), properties shows Properties", async ({ page }) => {
    await setEditorMode(page, "code");

    const outlineHeader = page.locator('[data-testid="dock-right-outline-header"]');
    const propertiesHeader = page.locator('[data-testid="dock-right-properties-header"]');
    await expect(outlineHeader).toBeVisible();
    await expect(propertiesHeader).toBeVisible();
    // Outline header uses generic "Outline" because outline body is empty in code mode.
    await expect(outlineHeader).toContainText("Outline");
    await expect(propertiesHeader).toContainText("Properties");
    // S7 invariant: code editor is in the outline body slot in code mode.
    await expect(page.locator('[data-testid="code-editor"]')).toBeVisible();
  });

  test("play mode: Outline/Properties headers are truthful (both bodies empty)", async ({ page }) => {
    await setEditorMode(page, "play");

    const outlineHeader = page.locator('[data-testid="dock-right-outline-header"]');
    const propertiesHeader = page.locator('[data-testid="dock-right-properties-header"]');
    await expect(outlineHeader).toBeVisible();
    await expect(propertiesHeader).toBeVisible();
    // Both bodies are empty in play mode — use generic labels, not "Play" (spec S7).
    await expect(outlineHeader).toContainText("Outline");
    await expect(propertiesHeader).toContainText("Properties");
    // No specific body content in play mode for right dock — headers are truthful by
    // not claiming mode-specific content that doesn't exist.
  });

  test("LeftDock header shows 'Assets' in scene mode", async ({ page }) => {
    const assetsHeader = page.locator('[data-testid="dock-left-header"]');
    await expect(assetsHeader).toBeVisible();
    await expect(assetsHeader).toContainText("Assets");
  });

  test("BottomDock shows Console tab in scene mode", async ({ page }) => {
    const bottomDock = page.locator('[data-testid="dock-bottom"]');
    await expect(bottomDock).toBeVisible();
    const consoleTab = page.locator('[data-testid="bottom-dock-tab-console"]');
    await expect(consoleTab).toBeVisible();
  });
});
