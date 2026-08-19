import { test, expect } from '@playwright/test';

test('create and list tileset', { tag: ["@domain"] }, async ({ page }) => {
  await page.goto('/');

  // Open tileset panel (assume there's a button or tab for it)
  await page.click('[data-testid="tileset-panel-btn"]');

  // Create a new tileset
  await page.click('button:has-text("+ New Tileset")');
  await page.fill('input[placeholder="Name"]', 'Grass Tileset');
  await page.fill('input[placeholder="Image path"]', 'assets/tilesets/grass.png');
  await page.fill('input[placeholder="Tile W"]', '16');
  await page.fill('input[placeholder="Tile H"]', '16');
  await page.fill('input[placeholder="Columns"]', '16');
  await page.click('button:has-text("Create")');

  // Verify tileset appears in list
  await expect(page.locator('li:has-text("Grass Tileset")')).toBeVisible();
});
