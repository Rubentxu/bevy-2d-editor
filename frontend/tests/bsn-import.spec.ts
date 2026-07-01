import { test, expect } from "@playwright/test";

test.describe("BSN File Import", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app and wait for it to be ready
    await page.goto("/");
    await page.waitForSelector('[data-testid="project-asset-browser"]', {
      timeout: 10000,
    });
  });

  test("Import .bsn button is visible in ProjectAssetBrowser toolbar", async ({
    page,
  }) => {
    const importBtn = page.getByTestId("import-bsn-btn");
    await expect(importBtn).toBeVisible();
    await expect(importBtn).toHaveText("Import .bsn");
  });

  test("clicking Import .bsn opens file picker (accepts .bsn)", async ({
    page,
  }) => {
    const importBtn = page.getByTestId("import-bsn-btn");
    const fileInput = page.getByTestId("bsn-file-input");

    // The file input should be hidden but attached
    await expect(fileInput).toBeHidden();

    // Clicking the button should trigger the hidden file input
    const fileInputPromise = page.waitForEvent("filechooser", {
      timeout: 5000,
    });
    await importBtn.click();
    const fileChooser = await fileInputPromise;
    expect(fileChooser.elem()).toBe(fileInput.element());
  });

  test("file input accepts only .bsn files", async ({ page }) => {
    const fileInput = page.getByTestId("bsn-file-input");
    await expect(fileInput).toHaveAttribute("accept", ".bsn");
  });
});
