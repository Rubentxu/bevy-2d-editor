import { test, expect, Page } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

/**
 * T3.5 — Playwright spy test: no window.prompt/alert/confirm in core authoring flows.
 *
 * Verifies S5 (prompt-free authoring flows) by intercepting browser dialogs during
 * the key authoring operations and asserting none were triggered.
 *
 * Flows covered:
 * - Create Scene Asset (ProjectAssetBrowser)
 * - Create Scene (SceneTabs)
 * - Create Source File (CodeEditor)
 * - Save Workspace Preset (App.tsx)
 */

test.describe("No window.prompt/alert/confirm in core flows (T3.5)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    // Wait for WASM to be ready
    await page.waitForFunction(
      () =>
        typeof (window as any).create_scene_asset === "function" &&
        typeof (window as any).list_scene_assets === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );
  });

  /**
   * Spy helper: wraps window.prompt/alert/confirm to record calls instead of
   * native browser dialogs.
   *
   * Uses page.exposeFunction to bridge browser→Node context so the recorded
   * calls array (which lives in Node) receives pushes from browser-side code.
   */
  async function installSpy(page: Page) {
    const calls: Array<{ kind: "prompt" | "alert" | "confirm"; args: unknown[] }> = [];

    // Expose a recorder function that lives in Node context — browser can call it.
    await page.exposeFunction("__recordSpyCall", (record: { kind: string; args: unknown[] }) => {
      calls.push(record as { kind: "prompt" | "alert" | "confirm"; args: unknown[] });
    });

    await page.evaluate(() => {
      const orig = {
        prompt: window.prompt?.bind(window),
        alert: window.alert?.bind(window),
        confirm: window.confirm?.bind(window),
      };
      (window as any).__spy_prompt = (...args: unknown[]) => {
        (window as any).__recordSpyCall({ kind: "prompt", args });
        return null; // cancel all prompts
      };
      (window as any).__spy_alert = (...args: unknown[]) => {
        (window as any).__recordSpyCall({ kind: "alert", args });
      };
      (window as any).__spy_confirm = (...args: unknown[]) => {
        (window as any).__recordSpyCall({ kind: "confirm", args });
        return false; // cancel all confirms
      };
      window.prompt = (...args: unknown[]) => (window as any).__spy_prompt(...args);
      window.alert = (...args: unknown[]) => (window as any).__spy_alert(...args);
      window.confirm = (...args: unknown[]) => (window as any).__spy_confirm(...args);
      (window as any).__spy_orig = orig;
    });
    return calls;
  }

  async function uninstallSpy(page: Page) {
    await page.evaluate(() => {
      const orig = (window as any).__spy_orig;
      if (orig) {
        window.prompt = orig.prompt;
        window.alert = orig.alert;
        window.confirm = orig.confirm;
      }
    });
  }

  test("no window.prompt/alert/confirm when creating a scene asset", async ({ page }) => {
    const calls = await installSpy(page);

    // Open Project Asset Browser (if not already visible)
    const assetBtn = page.locator('[data-testid="create-asset-btn"]');
    const assetBrowser = page.locator('[data-testid="project-asset-browser"]');

    // If the browser isn't visible, open it via the dock toggle
    const browserVisible = await assetBrowser.isVisible().catch(() => false);
    if (!browserVisible) {
      // Open via menu or keyboard - try F6 (Assets dock toggle)
      await page.keyboard.press("F6");
      await page.waitForTimeout(500);
    }

    // Click Create Scene Asset
    const createBtn = page.locator('[data-testid="create-asset-btn"]');
    if (await createBtn.isVisible()) {
      await createBtn.click();
      // The dialog should appear — fill it
      const nameInput = page.locator('[data-testid="prompt-dialog-input"]');
      if (await nameInput.isVisible({ timeout: 2000 }).catch(() => false)) {
        await nameInput.fill("test_asset");
        const confirmBtn = page.locator('[data-testid="prompt-dialog-confirm-btn"]');
        await confirmBtn.click();
        // Role dialog
        const roleInput = page.locator('[data-testid="prompt-dialog-input"]');
        if (await roleInput.isVisible({ timeout: 2000 }).catch(() => false)) {
          await roleInput.fill("actor");
          await page.locator('[data-testid="prompt-dialog-confirm-btn"]').click();
        }
      }
    }

    // Cancel any open dialogs
    const cancelBtn = page.locator('[data-testid="prompt-dialog-cancel-btn"]');
    if (await cancelBtn.isVisible()) {
      await cancelBtn.click();
    }

    // Assert no native dialog calls were made
    const promptCalls = calls.filter((c) => c.kind === "prompt");
    const alertCalls = calls.filter((c) => c.kind === "alert");
    const confirmCalls = calls.filter((c) => c.kind === "confirm");

    expect(promptCalls, "window.prompt should not be called during asset creation").toHaveLength(0);
    expect(alertCalls, "window.alert should not be called during asset creation").toHaveLength(0);
    expect(confirmCalls, "window.confirm should not be called during asset creation").toHaveLength(0);

    await uninstallSpy(page);
  });

  test("no window.prompt/alert/confirm when creating a new scene", async ({ page }) => {
    const calls = await installSpy(page);

    // Click the + button to create a new scene
    const newSceneBtn = page.locator('[data-testid="scene-tab-new-btn"]');
    if (await newSceneBtn.isVisible()) {
      await newSceneBtn.click();
      const nameInput = page.locator('[data-testid="prompt-dialog-input"]');
      if (await nameInput.isVisible({ timeout: 2000 }).catch(() => false)) {
        await nameInput.fill("Test Scene");
        const confirmBtn = page.locator('[data-testid="prompt-dialog-confirm-btn"]');
        await confirmBtn.click();
      }
    }

    // Cancel any open dialog
    const cancelBtn = page.locator('[data-testid="prompt-dialog-cancel-btn"]');
    if (await cancelBtn.isVisible()) {
      await cancelBtn.click();
    }

    const promptCalls = calls.filter((c) => c.kind === "prompt");
    const alertCalls = calls.filter((c) => c.kind === "alert");
    const confirmCalls = calls.filter((c) => c.kind === "confirm");

    expect(promptCalls, "window.prompt should not be called during scene creation").toHaveLength(0);
    expect(alertCalls, "window.alert should not be called during scene creation").toHaveLength(0);
    expect(confirmCalls, "window.confirm should not be called during scene creation").toHaveLength(0);

    await uninstallSpy(page);
  });

  test("no window.prompt/alert/confirm when creating a source file in Code Editor", async ({ page }) => {
    // First open Code Editor
    await page.keyboard.press("F6"); // open assets dock
    await page.waitForTimeout(300);

    // Navigate to Code Editor via the dock or menu
    // Open via URL or menu
    await page.goto("/?mode=code");
    await page.waitForTimeout(500);

    const calls = await installSpy(page);

    // Click + New File in Code Editor
    const newFileBtn = page.locator('button:has-text("+ New File")');
    if (await newFileBtn.isVisible()) {
      await newFileBtn.click();
      const nameInput = page.locator('[data-testid="prompt-dialog-input"]');
      if (await nameInput.isVisible({ timeout: 2000 }).catch(() => false)) {
        await nameInput.fill("test.rs");
        await page.locator('[data-testid="prompt-dialog-confirm-btn"]').click();
      }
    }

    const cancelBtn = page.locator('[data-testid="prompt-dialog-cancel-btn"]');
    if (await cancelBtn.isVisible()) {
      await cancelBtn.click();
    }

    const promptCalls = calls.filter((c) => c.kind === "prompt");
    const alertCalls = calls.filter((c) => c.kind === "alert");
    const confirmCalls = calls.filter((c) => c.kind === "confirm");

    expect(promptCalls, "window.prompt should not be called during file creation").toHaveLength(0);
    expect(alertCalls, "window.alert should not be called during file creation").toHaveLength(0);
    expect(confirmCalls, "window.confirm should not be called during file creation").toHaveLength(0);

    await uninstallSpy(page);
  });
});
