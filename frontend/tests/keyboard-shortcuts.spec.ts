import { test, expect } from "@playwright/test";

const WASM_LOAD_TIMEOUT = 120_000;

test.describe("Keyboard Shortcuts — Undo/Redo", () => {
  test("Ctrl+Z undo removes an entity from hierarchy (screenshot diff)", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function" &&
        typeof (window as any).get_scene_snapshot === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load empty scene
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "shortcuts-test",
          name: "Shortcuts Test",
          entities: [],
        })
      )
    );

    // Dispatch CreateEntity to add an entity
    await page.evaluate(() =>
      (window as any).dispatch_command(
        JSON.stringify({
          command: {
            type: "CreateEntity",
            id: "e1",
            name: "Entity One",
            components: [{ type_id: "editor.Name", values: { name: "Entity One" } }],
          },
          metadata: { authorship: "test", timestamp: 0 },
        })
      )
    );

    // Wait for hierarchy to render the entity
    await expect(page.locator('[data-testid="hierarchy-entity-e1"]')).toBeVisible({ timeout: 10_000 });

    // Take baseline screenshot of hierarchy panel
    const hierarchyPanel = page.locator('[data-testid="hierarchy-panel"]');
    const beforeScreenshot = await hierarchyPanel.screenshot();

    // Press Ctrl+Z to undo (no canvas click to avoid focus issues)
    await page.keyboard.press("Control+z");
    await page.waitForTimeout(500);

    // Verify entity is gone
    await expect(page.locator('[data-testid="hierarchy-entity-e1"]')).not.toBeVisible();

    // Take screenshot after undo
    const afterScreenshot = await hierarchyPanel.screenshot();

    // Verify screenshots are different (non-zero pixel diff)
    expect(beforeScreenshot).not.toEqual(afterScreenshot);
  });

  test("Ctrl+Z undo then Ctrl+Y redo restores entity", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function" &&
        typeof (window as any).undo === "function" &&
        typeof (window as any).redo === "function" &&
        typeof (window as any).get_scene_snapshot === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load empty scene
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "undo-redo-test",
          name: "Undo Redo Test",
          entities: [],
        })
      )
    );

    // Dispatch CreateEntity
    await page.evaluate(() =>
      (window as any).dispatch_command(
        JSON.stringify({
          command: {
            type: "CreateEntity",
            id: "redo-e1",
            name: "RedoTest",
            components: [{ type_id: "editor.Name", values: { name: "RedoTest" } }],
          },
          metadata: { authorship: "test", timestamp: 0 },
        })
      )
    );

    // Wait for entity in hierarchy
    await expect(page.locator('[data-testid="hierarchy-entity-redo-e1"]')).toBeVisible({ timeout: 10_000 });

    // Take baseline screenshot before undo
    const hierarchyPanel = page.locator('[data-testid="hierarchy-panel"]');
    const baselineScreenshot = await hierarchyPanel.screenshot();

    // Undo
    await page.keyboard.press("Control+z");
    await page.waitForTimeout(500);
    await expect(page.locator('[data-testid="hierarchy-entity-redo-e1"]')).not.toBeVisible();

    // Redo
    await page.keyboard.press("Control+y");
    await page.waitForTimeout(500);
    await expect(page.locator('[data-testid="hierarchy-entity-redo-e1"]')).toBeVisible();

    // Take screenshot after undo+redo roundtrip
    const afterRoundtrip = await hierarchyPanel.screenshot();

    // Verify screenshot matches baseline (within tolerance)
    expect(baselineScreenshot).toEqual(afterRoundtrip);
  });

  test("Ctrl+Z does not trigger editor undo when focus is in input", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).dispatch_command === "function" &&
        typeof (window as any).get_log_state === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load empty scene
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "input-guard-test",
          name: "Input Guard Test",
          entities: [],
        })
      )
    );

    // Create an entity so we can check log state
    await page.evaluate(() =>
      (window as any).dispatch_command(
        JSON.stringify({
          command: {
            type: "CreateEntity",
            id: "input-guard-e1",
            name: "GuardTest",
            components: [{ type_id: "editor.Name", values: { name: "GuardTest" } }],
          },
          metadata: { authorship: "test", timestamp: 0 },
        })
      )
    );

    // Wait for entity to appear in hierarchy and select it
    await expect(page.locator('[data-testid="hierarchy-entity-input-guard-e1"]')).toBeVisible({ timeout: 10_000 });
    await page.locator('[data-testid="hierarchy-entity-input-guard-e1"]').click();
    await page.waitForTimeout(300);

    // Verify can_undo is true
    const stateBefore = JSON.parse(
      await page.evaluate(() => (window as any).get_log_state())
    );
    expect(stateBefore.can_undo).toBe(true);

    // Focus an input in the inspector panel (entity name input)
    const nameInput = page.locator('input.entity-name');
    await nameInput.focus();
    await page.waitForTimeout(200);

    // Press Ctrl+Z while in input — should NOT trigger editor undo
    await page.keyboard.press("Control+z");
    await page.waitForTimeout(500);

    // Entity should still be visible (editor undo was blocked by input guard)
    await expect(page.locator('[data-testid="hierarchy-entity-input-guard-e1"]')).toBeVisible();
  });

  test("Ctrl+Z with no entries does nothing (can_undo=false)", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({ timeout: WASM_LOAD_TIMEOUT });

    await page.waitForFunction(
      () =>
        typeof (window as any).load_scene_json === "function" &&
        typeof (window as any).get_log_state === "function",
      { timeout: WASM_LOAD_TIMEOUT }
    );

    // Load empty scene (no operations)
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "noop-test",
          name: "Noop Test",
          entities: [],
        })
      )
    );

    // Verify can_undo is false
    const state = JSON.parse(
      await page.evaluate(() => (window as any).get_log_state())
    );
    expect(state.can_undo).toBe(false);

    // Press Ctrl+Z — should not crash and log state should remain unchanged
    await page.keyboard.press("Control+z");
    await page.waitForTimeout(500);

    const stateAfter = JSON.parse(
      await page.evaluate(() => (window as any).get_log_state())
    );
    expect(stateAfter.can_undo).toBe(false);
    expect(stateAfter.size).toBe(0);
  });
});
