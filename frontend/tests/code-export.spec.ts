import { test, expect } from "@playwright/test";

/**
 * E2E tests for code-export: Rust code generation via the ExportRustModal.
 */

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("Code Export — ExportRustModal", { tag: ["@full"] }, () => {
  test("modal opens and shows Rust source with ScenePlugin", async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on("console", (msg) => consoleLogs.push(msg.text()));

    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    // Wait for the wasm export_code function to be available.
    await page.waitForFunction(
      () => typeof (window as any).export_code === "function",
      { timeout: 30_000 }
    );

    // Click the Export .rs button.
    await page.click('[data-testid="export-rs-btn"]');

    // Modal should appear.
    await expect(page.locator('[data-testid="export-rs-modal"]')).toBeVisible({
      timeout: 10_000,
    });

    // Source textarea should contain Rust code with ScenePlugin.
    const sourceLocator = page.locator('[data-testid="export-rs-source"]');
    await expect(sourceLocator).toBeVisible();
    const source = await sourceLocator.inputValue();
    expect(source).toContain("pub struct ScenePlugin");

    // No console errors should have occurred.
    const errors = consoleLogs.filter((l) => l.toLowerCase().includes("error"));
    expect(errors).toEqual([]);
  });

  test("generated source contains bevy prelude header", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });

    await page.waitForFunction(
      () => typeof (window as any).export_code === "function",
      { timeout: 30_000 }
    );

    await page.click('[data-testid="export-rs-btn"]');

    const sourceLocator = page.locator('[data-testid="export-rs-source"]');
    await expect(sourceLocator).toBeVisible({ timeout: 10_000 });
    const source = await sourceLocator.inputValue();

    expect(source).toContain("use bevy::prelude::*;");
  });
});
